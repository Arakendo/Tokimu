use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{FbxError, FbxLimits, FbxProperty, FbxRecord, FbxRecordDocument, FbxResult};

/// Bounded provider-local ASCII FBX source records.
///
/// ASCII syntax support intentionally stops at the same record-level evidence
/// as the binary decoder. It does not claim canonical scene semantics.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxAsciiDocument {
    pub version: u32,
    pub records: Vec<FbxRecord>,
    pub source_bytes: usize,
    pub source_fingerprint: String,
}

impl FbxRecordDocument for FbxAsciiDocument {
    fn records(&self) -> &[FbxRecord] {
        &self.records
    }

    fn source_fingerprint(&self) -> &str {
        &self.source_fingerprint
    }
}

pub fn decode_ascii_fbx_file(
    path: impl AsRef<Path>,
    limits: FbxLimits,
) -> FbxResult<FbxAsciiDocument> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| FbxError::Read {
        path: path.to_owned(),
        source,
    })?;
    decode_ascii_fbx(&bytes, limits)
}

/// Decodes the bounded ASCII record subset needed to compare source graphs.
///
/// Supported tokens are records, quoted strings, booleans, finite scalar
/// numbers, and numeric `a:` arrays. Unsupported syntax fails explicitly.
pub fn decode_ascii_fbx(bytes: &[u8], limits: FbxLimits) -> FbxResult<FbxAsciiDocument> {
    if bytes.len() > limits.max_input_bytes {
        return Err(FbxError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_input_bytes,
        });
    }
    let source = std::str::from_utf8(bytes).map_err(|error| FbxError::AsciiSyntax {
        offset: error.valid_up_to(),
        reason: "ASCII FBX input is not valid UTF-8".into(),
    })?;
    let mut parser = AsciiParser {
        source,
        cursor: 0,
        limits,
        records: 0,
        properties: 0,
    };
    let records = parser.parse_records(0, false)?;
    let version = records
        .iter()
        .find(|record| record.name == "FBXHeaderExtension")
        .and_then(|header| {
            header
                .children
                .iter()
                .find(|child| child.name == "FBXVersion")
        })
        .and_then(|record| match record.properties.first() {
            Some(FbxProperty::I64(value)) => u32::try_from(*value).ok(),
            _ => None,
        })
        .ok_or_else(|| parser.error("missing finite integer `FBXVersion` header record"))?;
    if !matches!(
        version,
        5800 | 6100 | 7100 | 7200 | 7300 | 7400 | 7500 | 7700
    ) {
        return Err(FbxError::UnsupportedVersion { version });
    }

    Ok(FbxAsciiDocument {
        version,
        records,
        source_bytes: bytes.len(),
        source_fingerprint: fingerprint(bytes),
    })
}

#[derive(Clone, Debug)]
enum AsciiValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

struct AsciiParser<'a> {
    source: &'a str,
    cursor: usize,
    limits: FbxLimits,
    records: usize,
    properties: usize,
}

impl AsciiParser<'_> {
    fn parse_records(
        &mut self,
        depth: usize,
        until_closing_brace: bool,
    ) -> FbxResult<Vec<FbxRecord>> {
        if depth >= self.limits.max_depth {
            return Err(FbxError::DepthLimit {
                limit: self.limits.max_depth,
            });
        }
        let mut records = Vec::new();
        loop {
            self.skip_trivia();
            if self.is_eof() {
                if until_closing_brace {
                    return Err(self.error("unterminated record block"));
                }
                return Ok(records);
            }
            if self.peek() == Some(b'}') {
                if !until_closing_brace {
                    return Err(self.error("unexpected closing brace"));
                }
                self.cursor += 1;
                return Ok(records);
            }
            records.push(self.parse_record(depth)?);
        }
    }

    fn parse_record(&mut self, depth: usize) -> FbxResult<FbxRecord> {
        if self.records >= self.limits.max_records {
            return Err(FbxError::RecordLimit {
                limit: self.limits.max_records,
            });
        }
        self.records += 1;
        let source_offset = self.cursor;
        let name = self.parse_name()?;
        self.skip_horizontal();
        self.expect(b':', "record name")?;
        self.skip_horizontal();

        let mut array_count = None;
        let mut values = Vec::new();
        if self.peek() == Some(b'*') {
            self.cursor += 1;
            let count_offset = self.cursor;
            let count = self.parse_unsigned("array length")?;
            if count > self.limits.max_array_elements {
                return Err(FbxError::ArrayLimit {
                    offset: count_offset,
                    actual: count,
                    limit: self.limits.max_array_elements,
                });
            }
            array_count = Some(count);
            self.skip_horizontal();
        } else if self.peek() == Some(b',') {
            // 3ds Max may retain an explicitly empty first CSV field before a
            // payload (`Content: , "...")`. Keep that field as source text so
            // positional record properties remain observable without giving
            // the syntax reader any material-specific behavior.
            values.push(AsciiValue::String(String::new()));
            self.increment_properties(1)?;
            loop {
                if !self.consume_value_separator() {
                    break;
                }
                self.skip_value_whitespace();
                values.push(self.parse_value()?);
                self.increment_properties(1)?;
            }
        } else if !self.at_record_end_or_block() || self.consume_line_wrapped_scalar_start() {
            values.push(self.parse_value()?);
            self.increment_properties(1)?;
            loop {
                if !self.consume_value_separator() {
                    break;
                }
                // Numeric arrays in real ASCII FBX exports are commonly
                // line-wrapped around a comma. Newlines remain record
                // boundaries only when no list separator was present.
                self.skip_value_whitespace();
                values.push(self.parse_value()?);
                self.increment_properties(1)?;
            }
        }

        self.skip_horizontal();
        let mut children = if self.peek() == Some(b'{') {
            self.cursor += 1;
            self.parse_records(depth + 1, true)?
        } else {
            Vec::new()
        };
        let properties = if let Some(expected_count) = array_count {
            let array_properties = children
                .iter()
                .find(|child| child.name == "a")
                .map(|child| child.properties.clone())
                .ok_or_else(|| self.error("array record is missing `a:` payload"))?;
            if array_properties.len() != expected_count {
                return Err(self.error(format!(
                    "array declares {expected_count} values but contains {}",
                    array_properties.len()
                )));
            }
            children.retain(|child| child.name != "a");
            vec![numeric_array(&name, &array_properties, source_offset)?]
        } else if is_unstarred_numeric_array_field(&name) {
            // Older ASCII FBX revisions write several array-shaped fields as
            // an unstarred comma-separated scalar list. Normalize only known
            // source fields so `Color: 1, 0, 0` and other ordinary scalar
            // tuples retain their record-level representation.
            let properties = values_to_properties(values);
            vec![numeric_array(&name, &properties, source_offset)?]
        } else {
            values_to_properties(values)
        };
        let end_offset = self.cursor;
        Ok(FbxRecord {
            name,
            source_offset,
            end_offset,
            property_byte_length: 0,
            properties,
            children,
        })
    }

    fn parse_name(&mut self) -> FbxResult<String> {
        let start = self.cursor;
        while let Some(byte) = self.peek() {
            if matches!(
                byte,
                b':' | b' ' | b'\t' | b'\r' | b'\n' | b'{' | b'}' | b','
            ) {
                break;
            }
            self.cursor += 1;
        }
        if start == self.cursor {
            return Err(self.error("expected record name"));
        }
        Ok(self.source[start..self.cursor].to_owned())
    }

    fn parse_value(&mut self) -> FbxResult<AsciiValue> {
        match self.peek() {
            Some(b'"') => self.parse_string().map(AsciiValue::String),
            Some(b'T') if self.token_boundary(self.cursor + 1) => {
                self.cursor += 1;
                Ok(AsciiValue::Bool(true))
            }
            Some(b'F') if self.token_boundary(self.cursor + 1) => {
                self.cursor += 1;
                Ok(AsciiValue::Bool(false))
            }
            // ASCII FBX exporters also use the legacy `Y`/`N` spelling for
            // yes/no properties (for example `Shading: Y`). Preserve it as
            // the same source boolean without opening the parser to general
            // unquoted string values.
            Some(b'Y') if self.token_boundary(self.cursor + 1) => {
                self.cursor += 1;
                Ok(AsciiValue::Bool(true))
            }
            Some(b'N') if self.token_boundary(self.cursor + 1) => {
                self.cursor += 1;
                Ok(AsciiValue::Bool(false))
            }
            // Some legacy properties use a one-letter source enum (for
            // example `Shading: W`). Keep the token as source text while
            // continuing to reject arbitrary bare identifiers such as
            // `infty` or `unsupported`.
            Some(byte) if byte.is_ascii_alphabetic() && self.token_boundary(self.cursor + 1) => {
                self.cursor += 1;
                Ok(AsciiValue::String((byte as char).to_string()))
            }
            Some(_) => self.parse_number(),
            None => Err(self.error("expected property value")),
        }
    }

    fn parse_string(&mut self) -> FbxResult<String> {
        self.expect(b'"', "string")?;
        let mut value = String::new();
        loop {
            let Some(character) = self.source[self.cursor..].chars().next() else {
                return Err(self.error("unterminated quoted string"));
            };
            self.cursor += character.len_utf8();
            match character {
                '"' => return Ok(value),
                '\\' => {
                    let Some(escaped) = self.peek() else {
                        return Err(self.error("unterminated string escape"));
                    };
                    self.cursor += 1;
                    value.push(match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        other => other as char,
                    });
                }
                // The document was validated as UTF-8 before parsing. Preserve
                // source identity here instead of treating non-ASCII names as
                // a provider-specific binary-only feature.
                character => value.push(character),
            }
        }
    }

    fn parse_number(&mut self) -> FbxResult<AsciiValue> {
        let start = self.cursor;
        while let Some(byte) = self.peek() {
            if matches!(
                byte,
                b',' | b' ' | b'\t' | b'\r' | b'\n' | b'{' | b'}' | b';'
            ) {
                break;
            }
            self.cursor += 1;
        }
        let token = &self.source[start..self.cursor];
        if token.is_empty() {
            return Err(self.error("expected numeric value"));
        }
        if let Ok(value) = token.parse::<i64>() {
            return Ok(AsciiValue::Integer(value));
        }
        let value = token.parse::<f64>().map_err(|_| FbxError::AsciiSyntax {
            offset: start,
            reason: format!("unsupported bare value `{token}`"),
        })?;
        if !value.is_finite() {
            return Err(FbxError::AsciiSyntax {
                offset: start,
                reason: "non-finite numeric value".into(),
            });
        }
        Ok(AsciiValue::Float(value))
    }

    fn parse_unsigned(&mut self, context: &'static str) -> FbxResult<usize> {
        let start = self.cursor;
        while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
            self.cursor += 1;
        }
        self.source[start..self.cursor]
            .parse()
            .map_err(|_| FbxError::AsciiSyntax {
                offset: start,
                reason: format!("expected unsigned {context}"),
            })
    }

    fn at_record_end_or_block(&self) -> bool {
        matches!(self.peek(), None | Some(b'\r' | b'\n' | b'{' | b'}' | b';'))
    }

    fn consume_line_wrapped_scalar_start(&mut self) -> bool {
        let mut cursor = self.cursor;
        let mut saw_line_break = false;
        while let Some(byte) = self.source.as_bytes().get(cursor).copied() {
            if !byte.is_ascii_whitespace() {
                break;
            }
            saw_line_break |= matches!(byte, b'\r' | b'\n');
            cursor += 1;
        }
        if !saw_line_break {
            return false;
        }
        if !matches!(
            self.source.as_bytes().get(cursor),
            Some(b'+' | b'-' | b'.' | b'0'..=b'9')
        ) {
            return false;
        }
        // Legacy ASCII animation records may put their first scalar on the
        // following line (`Key:\n 1924423250,...`). A numeric start is enough
        // to distinguish that continuation from the next named record.
        self.cursor = cursor;
        true
    }

    fn skip_trivia(&mut self) {
        loop {
            while matches!(self.peek(), Some(byte) if byte.is_ascii_whitespace()) {
                self.cursor += 1;
            }
            if self.peek() != Some(b';') {
                return;
            }
            while matches!(self.peek(), Some(byte) if byte != b'\n') {
                self.cursor += 1;
            }
        }
    }

    fn skip_horizontal(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r')) {
            self.cursor += 1;
        }
    }

    fn skip_value_whitespace(&mut self) {
        while matches!(self.peek(), Some(byte) if byte.is_ascii_whitespace()) {
            self.cursor += 1;
        }
    }

    fn consume_value_separator(&mut self) -> bool {
        self.skip_horizontal();
        if self.peek() == Some(b',') {
            self.cursor += 1;
            return true;
        }

        // Legacy ASCII files sometimes start a wrapped continuation line
        // with the separator instead of ending the prior line with one.
        // Only advance when that comma is actually present; otherwise the
        // newline remains a record boundary.
        let mut cursor = self.cursor;
        let mut saw_line_break = false;
        while let Some(byte) = self.source.as_bytes().get(cursor).copied() {
            if !byte.is_ascii_whitespace() {
                break;
            }
            saw_line_break |= matches!(byte, b'\r' | b'\n');
            cursor += 1;
        }
        if saw_line_break && self.source.as_bytes().get(cursor) == Some(&b',') {
            self.cursor = cursor + 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: u8, context: &'static str) -> FbxResult<()> {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            Ok(())
        } else {
            Err(self.error(format!("expected `{}` after {context}", expected as char)))
        }
    }

    fn increment_properties(&mut self, count: usize) -> FbxResult<()> {
        self.properties = self
            .properties
            .checked_add(count)
            .ok_or(FbxError::PropertyLimit {
                limit: self.limits.max_properties,
            })?;
        if self.properties > self.limits.max_properties {
            return Err(FbxError::PropertyLimit {
                limit: self.limits.max_properties,
            });
        }
        Ok(())
    }

    fn token_boundary(&self, offset: usize) -> bool {
        self.source.as_bytes().get(offset).is_none_or(|byte| {
            matches!(
                *byte,
                b',' | b' ' | b'\t' | b'\r' | b'\n' | b'{' | b'}' | b';'
            )
        })
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor).copied()
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.source.len()
    }

    fn error(&self, reason: impl Into<String>) -> FbxError {
        FbxError::AsciiSyntax {
            offset: self.cursor,
            reason: reason.into(),
        }
    }
}

fn values_to_properties(values: Vec<AsciiValue>) -> Vec<FbxProperty> {
    values
        .into_iter()
        .map(|value| match value {
            AsciiValue::Integer(value) => FbxProperty::I64(value),
            AsciiValue::Float(value) => FbxProperty::F64(value),
            AsciiValue::String(value) => FbxProperty::String(value),
            AsciiValue::Bool(value) => FbxProperty::Bool(value),
        })
        .collect()
}

fn numeric_array(name: &str, properties: &[FbxProperty], offset: usize) -> FbxResult<FbxProperty> {
    // ASCII FBX omits the binary property type tag. Keep the small explicit
    // mapping at the syntax adapter boundary rather than inferring types from
    // a particular fixture's values (for example, a normal array of 0/1).
    if name == "KeyTime" {
        return properties
            .iter()
            .map(|property| match property {
                FbxProperty::I64(value) => Ok(*value),
                _ => Err(FbxError::AsciiSyntax {
                    offset,
                    reason: format!("`{name}` array contains a non-integer value"),
                }),
            })
            .collect::<FbxResult<Vec<_>>>()
            .map(FbxProperty::I64Array);
    }
    if matches!(
        name,
        "PolygonVertexIndex"
            | "Edges"
            | "NormalsIndex"
            | "UVIndex"
            | "Materials"
            | "Indexes"
            | "KeyAttrFlags"
    ) {
        return properties
            .iter()
            .map(|property| match property {
                FbxProperty::I64(value) => {
                    i32::try_from(*value).map_err(|_| FbxError::AsciiSyntax {
                        offset,
                        reason: format!("`{name}` array value {value} is outside I32 range"),
                    })
                }
                _ => Err(FbxError::AsciiSyntax {
                    offset,
                    reason: format!("`{name}` array contains a non-integer value"),
                }),
            })
            .collect::<FbxResult<Vec<_>>>()
            .map(FbxProperty::I32Array);
    }

    properties
        .iter()
        .map(|property| match property {
            FbxProperty::I64(value) => Ok(*value as f64),
            FbxProperty::F64(value) => Ok(*value),
            _ => Err(FbxError::AsciiSyntax {
                offset,
                reason: "numeric array contains a non-numeric value".into(),
            }),
        })
        .collect::<FbxResult<Vec<_>>>()
        .map(FbxProperty::F64Array)
}

fn is_unstarred_numeric_array_field(name: &str) -> bool {
    matches!(
        name,
        "Vertices"
            | "PolygonVertexIndex"
            | "Edges"
            | "Normals"
            | "NormalsIndex"
            | "UV"
            | "UVIndex"
            | "Materials"
            | "Indexes"
            | "Weights"
            | "Transform"
            | "TransformLink"
            | "KeyTime"
            | "KeyValueFloat"
            | "KeyAttrFlags"
    )
}

fn fingerprint(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_bounded_ascii_records_and_arrays() {
        let document = decode_ascii_fbx(
            br#"FBXHeaderExtension: { FBXVersion: 7500 }
Objects: {
 Geometry: 1, "Geometry::Cube", "Mesh" {
  Vertices: *3 { a: 0, 1.5, 2 }
 }
}
Connections: { C: "OO", 1, 2 }
"#,
            FbxLimits::default(),
        )
        .unwrap();

        assert_eq!(document.version, 7500);
        let geometry = &document.records[1].children[0];
        assert!(matches!(
            geometry.children[0].properties.as_slice(),
            [FbxProperty::F64Array(values)] if values == &[0.0, 1.5, 2.0]
        ));
    }

    #[test]
    fn rejects_truncated_quoted_ascii_string() {
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { X: \"",
                FbxLimits::default()
            ),
            Err(FbxError::AsciiSyntax { .. })
        ));
    }

    #[test]
    fn preserves_utf8_quoted_ascii_strings() {
        let document = decode_ascii_fbx(
            "FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Model: 1, \"Model::aβカ😂\", \"Mesh\" }".as_bytes(),
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::I64(1), FbxProperty::String(value), FbxProperty::String(kind)]
                if value == "Model::aβカ😂" && kind == "Mesh"
        ));
    }

    #[test]
    fn preserves_explicit_empty_csv_fields() {
        let document = decode_ascii_fbx(
            br#"FBXHeaderExtension: { FBXVersion: 7500 }
Objects: { Content: , "payload" }"#,
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::String(empty), FbxProperty::String(value)]
                if empty.is_empty() && value == "payload"
        ));
    }

    #[test]
    fn rejects_declared_ascii_array_above_the_shared_limit() {
        let limits = FbxLimits {
            max_array_elements: 2,
            ..FbxLimits::default()
        };
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Vertices: *3 { a: 0, 1, 2 } }",
                limits
            ),
            Err(FbxError::ArrayLimit {
                actual: 3,
                limit: 2,
                ..
            })
        ));
    }

    #[test]
    fn rejects_declared_ascii_array_length_mismatch() {
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Vertices: *3 { a: 0, 1 } }",
                FbxLimits::default()
            ),
            Err(FbxError::AsciiSyntax { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_bare_ascii_tokens() {
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Value: unsupported }",
                FbxLimits::default()
            ),
            Err(FbxError::AsciiSyntax { .. })
        ));
    }

    #[test]
    fn decodes_legacy_yes_no_ascii_booleans() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Shading: Y Culling: N }",
            FbxLimits::default(),
        )
        .unwrap();

        let objects = &document.records[1];
        assert!(matches!(
            objects.children[0].properties.as_slice(),
            [FbxProperty::Bool(true)]
        ));
        assert!(matches!(
            objects.children[1].properties.as_slice(),
            [FbxProperty::Bool(false)]
        ));
    }

    #[test]
    fn preserves_legacy_single_letter_ascii_enums() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Shading: W }",
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::String(value)] if value == "W"
        ));
    }

    #[test]
    fn decodes_line_wrapped_ascii_array_values() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Vertices: *3 { a: 0,\n 1,\n 2 } }",
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::F64Array(values)] if values == &[0.0, 1.0, 2.0]
        ));
    }

    #[test]
    fn decodes_line_wrapped_ascii_values_with_a_leading_separator() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 6100 }\nObjects: { Vertices: 0, 1\n , 2, 3 }",
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::F64Array(values)] if values == &[0.0, 1.0, 2.0, 3.0]
        ));
    }

    #[test]
    fn decodes_line_wrapped_legacy_scalar_values() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 6100 }\nObjects: { Key: \n 1924423250, 0, L }",
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::I64(1_924_423_250), FbxProperty::I64(0), FbxProperty::String(value)]
                if value == "L"
        ));
    }

    #[test]
    fn normalizes_unstarred_legacy_uv_lists_to_source_arrays() {
        let document = decode_ascii_fbx(
            b"FBXHeaderExtension: { FBXVersion: 6100 }\nObjects: { UV: 0, 1, 0.5, 0.25 UVIndex: 0, 1 }",
            FbxLimits::default(),
        )
        .unwrap();

        assert!(matches!(
            document.records[1].children[0].properties.as_slice(),
            [FbxProperty::F64Array(values)] if values == &[0.0, 1.0, 0.5, 0.25]
        ));
        assert!(matches!(
            document.records[1].children[1].properties.as_slice(),
            [FbxProperty::I32Array(values)] if values == &[0, 1]
        ));
    }

    #[test]
    fn enforces_ascii_record_nesting_limit() {
        let limits = FbxLimits {
            max_depth: 2,
            ..FbxLimits::default()
        };
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: { Geometry: { Vertices: { a: 0 } } }",
                limits
            ),
            Err(FbxError::DepthLimit { limit: 2 })
        ));
    }

    #[test]
    fn enforces_ascii_record_limit() {
        let limits = FbxLimits {
            max_records: 1,
            ..FbxLimits::default()
        };
        assert!(matches!(
            decode_ascii_fbx(
                b"FBXHeaderExtension: { FBXVersion: 7500 }\nObjects: {}",
                limits
            ),
            Err(FbxError::RecordLimit { limit: 1 })
        ));
    }
}
