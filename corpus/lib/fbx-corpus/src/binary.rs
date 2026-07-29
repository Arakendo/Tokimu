use std::{fs, io::Read, path::Path};

use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};

use crate::{FbxError, FbxResult};

const BINARY_PREFIX: &[u8; 22] = b"Kaydara FBX Binary  \0\x1a";
const HEADER_BYTES: usize = 27;

/// Byte order declared by a binary FBX header.
///
/// This is retained as source-format evidence. It does not affect any
/// Tokimu-owned model contract.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FbxByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FbxLimits {
    pub max_input_bytes: usize,
    pub max_records: usize,
    pub max_depth: usize,
    pub max_properties: usize,
    pub max_array_elements: usize,
    pub max_blob_bytes: usize,
}

impl Default for FbxLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024 * 1024,
            max_records: 250_000,
            max_depth: 256,
            max_properties: 2_000_000,
            max_array_elements: 32 * 1024 * 1024,
            max_blob_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxBinaryDocument {
    pub version: u32,
    pub byte_order: FbxByteOrder,
    pub records: Vec<FbxRecord>,
    pub footer_offset: usize,
    pub source_bytes: usize,
    pub source_fingerprint: String,
}

/// Common record access shared by bounded FBX syntax decoders.
///
/// This is provider-local corpus plumbing. It deliberately exposes source
/// records rather than defining a Tokimu asset or scene contract.
pub trait FbxRecordDocument {
    fn records(&self) -> &[FbxRecord];
    fn source_fingerprint(&self) -> &str;
}

impl FbxRecordDocument for FbxBinaryDocument {
    fn records(&self) -> &[FbxRecord] {
        &self.records
    }

    fn source_fingerprint(&self) -> &str {
        &self.source_fingerprint
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxRecord {
    pub name: String,
    pub source_offset: usize,
    pub end_offset: usize,
    pub property_byte_length: usize,
    pub properties: Vec<FbxProperty>,
    pub children: Vec<FbxRecord>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum FbxProperty {
    I16(i16),
    Bool(bool),
    I32(i32),
    F32(f32),
    F64(f64),
    I64(i64),
    Bytes(Vec<u8>),
    String(String),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
    I64Array(Vec<i64>),
    I32Array(Vec<i32>),
    BoolArray(Vec<bool>),
    ByteArray(Vec<u8>),
}

pub fn decode_binary_fbx_file(
    path: impl AsRef<Path>,
    limits: FbxLimits,
) -> FbxResult<FbxBinaryDocument> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| FbxError::Read {
        path: path.to_owned(),
        source,
    })?;
    decode_binary_fbx(&bytes, limits)
}

pub fn decode_binary_fbx(bytes: &[u8], limits: FbxLimits) -> FbxResult<FbxBinaryDocument> {
    if bytes.len() > limits.max_input_bytes {
        return Err(FbxError::InputTooLarge {
            actual: bytes.len(),
            limit: limits.max_input_bytes,
        });
    }
    if bytes.get(..BINARY_PREFIX.len()) != Some(BINARY_PREFIX) {
        return Err(FbxError::InvalidSignature);
    }
    let byte_order = match read_u8(bytes, BINARY_PREFIX.len(), "FBX byte-order marker")? {
        0 => FbxByteOrder::LittleEndian,
        1 => FbxByteOrder::BigEndian,
        marker => return Err(FbxError::UnsupportedByteOrder { marker }),
    };

    let version = read_u32(bytes, BINARY_PREFIX.len() + 1, byte_order, "FBX version")?;
    if !matches!(
        version,
        5800 | 6100 | 7100 | 7200 | 7300 | 7400 | 7500 | 7700
    ) {
        return Err(FbxError::UnsupportedVersion { version });
    }

    let wide_offsets = version >= 7500;
    let sentinel_bytes = if wide_offsets { 25 } else { 13 };
    let mut state = DecodeState {
        bytes,
        limits,
        records: 0,
        properties: 0,
        wide_offsets,
        sentinel_bytes,
        byte_order,
    };
    let mut cursor = HEADER_BYTES;
    let mut records = Vec::new();

    loop {
        if state.is_null_record(cursor)? {
            cursor = checked_add(cursor, sentinel_bytes, cursor, "top-level null record")?;
            break;
        }
        records.push(state.decode_record(&mut cursor, 0)?);
    }

    Ok(FbxBinaryDocument {
        version,
        byte_order,
        records,
        footer_offset: cursor,
        source_bytes: bytes.len(),
        source_fingerprint: fingerprint(bytes),
    })
}

/// Serializes provider-local source records as deterministic inspection evidence.
///
/// The artifact deliberately excludes decoder-specific metadata such as binary
/// byte order and footer position so every bounded source encoding can emit the
/// same record-level diagnostic shape.
pub fn source_records_json(document: &impl FbxRecordDocument) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(document.records())?)
}

struct DecodeState<'a> {
    bytes: &'a [u8],
    limits: FbxLimits,
    records: usize,
    properties: usize,
    wide_offsets: bool,
    sentinel_bytes: usize,
    byte_order: FbxByteOrder,
}

impl DecodeState<'_> {
    fn decode_record(&mut self, cursor: &mut usize, depth: usize) -> FbxResult<FbxRecord> {
        if depth >= self.limits.max_depth {
            return Err(FbxError::DepthLimit {
                limit: self.limits.max_depth,
            });
        }
        if self.records >= self.limits.max_records {
            return Err(FbxError::RecordLimit {
                limit: self.limits.max_records,
            });
        }
        self.records += 1;

        let source_offset = *cursor;
        let (end_offset, property_count, property_byte_length, name_length, header_bytes) =
            if self.wide_offsets {
                (
                    usize_from_u64(
                        read_u64(self.bytes, *cursor, self.byte_order, "record end offset")?,
                        *cursor,
                    )?,
                    usize_from_u64(
                        read_u64(
                            self.bytes,
                            *cursor + 8,
                            self.byte_order,
                            "record property count",
                        )?,
                        *cursor + 8,
                    )?,
                    usize_from_u64(
                        read_u64(
                            self.bytes,
                            *cursor + 16,
                            self.byte_order,
                            "record property length",
                        )?,
                        *cursor + 16,
                    )?,
                    read_u8(self.bytes, *cursor + 24, "record name length")? as usize,
                    25,
                )
            } else {
                (
                    read_u32(self.bytes, *cursor, self.byte_order, "record end offset")? as usize,
                    read_u32(
                        self.bytes,
                        *cursor + 4,
                        self.byte_order,
                        "record property count",
                    )? as usize,
                    read_u32(
                        self.bytes,
                        *cursor + 8,
                        self.byte_order,
                        "record property length",
                    )? as usize,
                    read_u8(self.bytes, *cursor + 12, "record name length")? as usize,
                    13,
                )
            };

        if end_offset <= source_offset || end_offset > self.bytes.len() {
            return Err(invalid_record(
                source_offset,
                format!(
                    "end offset {end_offset} is outside ({source_offset}, {}]",
                    self.bytes.len()
                ),
            ));
        }
        if property_count > self.limits.max_properties - self.properties {
            return Err(FbxError::PropertyLimit {
                limit: self.limits.max_properties,
            });
        }
        self.properties += property_count;

        let name_offset = checked_add(source_offset, header_bytes, source_offset, "record header")?;
        let name_end = checked_add(name_offset, name_length, source_offset, "record name")?;
        if name_end > end_offset {
            return Err(invalid_record(source_offset, "name extends beyond record"));
        }
        let name_bytes = slice(self.bytes, name_offset, name_end, "record name")?;
        let name = String::from_utf8_lossy(name_bytes).into_owned();

        let properties_end = checked_add(
            name_end,
            property_byte_length,
            source_offset,
            "property list",
        )?;
        if properties_end > end_offset {
            return Err(invalid_record(
                source_offset,
                "property list extends beyond record",
            ));
        }

        let mut property_cursor = name_end;
        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            properties.push(self.decode_property(&mut property_cursor, properties_end)?);
        }
        if property_cursor != properties_end {
            return Err(invalid_record(
                source_offset,
                format!(
                    "decoded properties end at {property_cursor}, declared end is {properties_end}"
                ),
            ));
        }

        let mut children = Vec::new();
        let mut child_cursor = properties_end;
        while child_cursor < end_offset {
            if self.is_null_record(child_cursor)? {
                child_cursor = checked_add(
                    child_cursor,
                    self.sentinel_bytes,
                    child_cursor,
                    "child null record",
                )?;
                break;
            }
            children.push(self.decode_record(&mut child_cursor, depth + 1)?);
        }
        if child_cursor != end_offset {
            return Err(invalid_record(
                source_offset,
                format!("decoded record ends at {child_cursor}, declared end is {end_offset}"),
            ));
        }

        *cursor = end_offset;
        Ok(FbxRecord {
            name,
            source_offset,
            end_offset,
            property_byte_length,
            properties,
            children,
        })
    }

    fn decode_property(&self, cursor: &mut usize, property_end: usize) -> FbxResult<FbxProperty> {
        let offset = *cursor;
        let code = read_u8(self.bytes, offset, "property type")?;
        *cursor = checked_add(*cursor, 1, offset, "property type")?;

        let property = match code {
            b'Y' => FbxProperty::I16(self.take_i16(cursor, property_end)?),
            b'C' => FbxProperty::Bool(self.take_u8(cursor, property_end)? != 0),
            b'I' => FbxProperty::I32(self.take_i32(cursor, property_end)?),
            b'F' => FbxProperty::F32(self.take_f32(cursor, property_end)?),
            b'D' => FbxProperty::F64(self.take_f64(cursor, property_end)?),
            b'L' => FbxProperty::I64(self.take_i64(cursor, property_end)?),
            b'R' => FbxProperty::Bytes(self.take_blob(cursor, property_end, offset)?),
            b'S' => {
                let bytes = self.take_blob(cursor, property_end, offset)?;
                FbxProperty::String(String::from_utf8_lossy(&bytes).into_owned())
            }
            b'f' => FbxProperty::F32Array(self.take_array(cursor, property_end, offset)?),
            b'd' => FbxProperty::F64Array(self.take_array(cursor, property_end, offset)?),
            b'l' => FbxProperty::I64Array(self.take_array(cursor, property_end, offset)?),
            b'i' => FbxProperty::I32Array(self.take_array(cursor, property_end, offset)?),
            b'b' => FbxProperty::BoolArray(self.take_array(cursor, property_end, offset)?),
            b'c' => FbxProperty::ByteArray(self.take_array(cursor, property_end, offset)?),
            code => return Err(FbxError::UnsupportedProperty { offset, code }),
        };
        Ok(property)
    }

    fn take_blob(
        &self,
        cursor: &mut usize,
        property_end: usize,
        offset: usize,
    ) -> FbxResult<Vec<u8>> {
        let length = self.take_u32(cursor, property_end)? as usize;
        if length > self.limits.max_blob_bytes {
            return Err(FbxError::BlobLimit {
                offset,
                actual: length,
                limit: self.limits.max_blob_bytes,
            });
        }
        Ok(self.take_bytes(cursor, property_end, length)?.to_vec())
    }

    fn take_array<T: FbxArrayElement>(
        &self,
        cursor: &mut usize,
        property_end: usize,
        offset: usize,
    ) -> FbxResult<Vec<T>> {
        let count = self.take_u32(cursor, property_end)? as usize;
        let encoding = self.take_u32(cursor, property_end)?;
        let encoded_bytes = self.take_u32(cursor, property_end)? as usize;
        if count > self.limits.max_array_elements {
            return Err(FbxError::ArrayLimit {
                offset,
                actual: count,
                limit: self.limits.max_array_elements,
            });
        }
        if encoded_bytes > self.limits.max_blob_bytes {
            return Err(FbxError::BlobLimit {
                offset,
                actual: encoded_bytes,
                limit: self.limits.max_blob_bytes,
            });
        }

        let encoded = self.take_bytes(cursor, property_end, encoded_bytes)?;
        let expected = count
            .checked_mul(T::BYTE_WIDTH)
            .ok_or_else(|| invalid_record(offset, "array byte length overflow"))?;
        let decoded = match encoding {
            0 => encoded.to_vec(),
            1 => {
                let mut decoder = ZlibDecoder::new(encoded);
                let mut output = Vec::with_capacity(expected);
                decoder
                    .read_to_end(&mut output)
                    .map_err(|source| FbxError::ArrayDecompression { offset, source })?;
                output
            }
            encoding => {
                return Err(FbxError::UnsupportedArrayEncoding { offset, encoding });
            }
        };
        if decoded.len() != expected {
            return Err(FbxError::InvalidArrayLength {
                offset,
                expected,
                actual: decoded.len(),
            });
        }

        Ok(decoded
            .chunks_exact(T::BYTE_WIDTH)
            .map(|bytes| T::from_bytes(bytes, self.byte_order))
            .collect())
    }

    fn is_null_record(&self, offset: usize) -> FbxResult<bool> {
        let end = checked_add(offset, self.sentinel_bytes, offset, "null record")?;
        Ok(slice(self.bytes, offset, end, "null record")?
            .iter()
            .all(|byte| *byte == 0))
    }

    fn take_bytes<'a>(
        &self,
        cursor: &mut usize,
        property_end: usize,
        length: usize,
    ) -> FbxResult<&'a [u8]>
    where
        Self: 'a,
    {
        let start = *cursor;
        let end = checked_add(start, length, start, "property value")?;
        if end > property_end {
            return Err(FbxError::Truncated {
                offset: start,
                context: "property value",
            });
        }
        *cursor = end;
        slice(self.bytes, start, end, "property value")
    }

    fn take_u8(&self, cursor: &mut usize, end: usize) -> FbxResult<u8> {
        Ok(self.take_bytes(cursor, end, 1)?[0])
    }

    fn take_i16(&self, cursor: &mut usize, end: usize) -> FbxResult<i16> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                i16::from_le_bytes(self.take_bytes(cursor, end, 2)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                i16::from_be_bytes(self.take_bytes(cursor, end, 2)?.try_into().unwrap())
            }
        })
    }

    fn take_u32(&self, cursor: &mut usize, end: usize) -> FbxResult<u32> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                u32::from_le_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                u32::from_be_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
        })
    }

    fn take_i32(&self, cursor: &mut usize, end: usize) -> FbxResult<i32> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                i32::from_le_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                i32::from_be_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
        })
    }

    fn take_i64(&self, cursor: &mut usize, end: usize) -> FbxResult<i64> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                i64::from_le_bytes(self.take_bytes(cursor, end, 8)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                i64::from_be_bytes(self.take_bytes(cursor, end, 8)?.try_into().unwrap())
            }
        })
    }

    fn take_f32(&self, cursor: &mut usize, end: usize) -> FbxResult<f32> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                f32::from_le_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                f32::from_be_bytes(self.take_bytes(cursor, end, 4)?.try_into().unwrap())
            }
        })
    }

    fn take_f64(&self, cursor: &mut usize, end: usize) -> FbxResult<f64> {
        Ok(match self.byte_order {
            FbxByteOrder::LittleEndian => {
                f64::from_le_bytes(self.take_bytes(cursor, end, 8)?.try_into().unwrap())
            }
            FbxByteOrder::BigEndian => {
                f64::from_be_bytes(self.take_bytes(cursor, end, 8)?.try_into().unwrap())
            }
        })
    }
}

trait FbxArrayElement: Sized {
    const BYTE_WIDTH: usize;
    fn from_bytes(bytes: &[u8], byte_order: FbxByteOrder) -> Self;
}

macro_rules! array_element {
    ($type:ty, $width:expr) => {
        impl FbxArrayElement for $type {
            const BYTE_WIDTH: usize = $width;

            fn from_bytes(bytes: &[u8], byte_order: FbxByteOrder) -> Self {
                let bytes = bytes.try_into().unwrap();
                match byte_order {
                    FbxByteOrder::LittleEndian => <$type>::from_le_bytes(bytes),
                    FbxByteOrder::BigEndian => <$type>::from_be_bytes(bytes),
                }
            }
        }
    };
}

array_element!(f32, 4);
array_element!(f64, 8);
array_element!(i32, 4);
array_element!(i64, 8);

impl FbxArrayElement for bool {
    const BYTE_WIDTH: usize = 1;

    fn from_bytes(bytes: &[u8], _byte_order: FbxByteOrder) -> Self {
        bytes[0] != 0
    }
}

impl FbxArrayElement for u8 {
    const BYTE_WIDTH: usize = 1;

    fn from_bytes(bytes: &[u8], _byte_order: FbxByteOrder) -> Self {
        bytes[0]
    }
}

fn read_u8(bytes: &[u8], offset: usize, context: &'static str) -> FbxResult<u8> {
    bytes
        .get(offset)
        .copied()
        .ok_or(FbxError::Truncated { offset, context })
}

fn read_u32(
    bytes: &[u8],
    offset: usize,
    byte_order: FbxByteOrder,
    context: &'static str,
) -> FbxResult<u32> {
    let end = checked_add(offset, 4, offset, context)?;
    let bytes = slice(bytes, offset, end, context)?.try_into().unwrap();
    Ok(match byte_order {
        FbxByteOrder::LittleEndian => u32::from_le_bytes(bytes),
        FbxByteOrder::BigEndian => u32::from_be_bytes(bytes),
    })
}

fn read_u64(
    bytes: &[u8],
    offset: usize,
    byte_order: FbxByteOrder,
    context: &'static str,
) -> FbxResult<u64> {
    let end = checked_add(offset, 8, offset, context)?;
    let bytes = slice(bytes, offset, end, context)?.try_into().unwrap();
    Ok(match byte_order {
        FbxByteOrder::LittleEndian => u64::from_le_bytes(bytes),
        FbxByteOrder::BigEndian => u64::from_be_bytes(bytes),
    })
}

fn slice<'a>(
    bytes: &'a [u8],
    start: usize,
    end: usize,
    context: &'static str,
) -> FbxResult<&'a [u8]> {
    bytes.get(start..end).ok_or(FbxError::Truncated {
        offset: start,
        context,
    })
}

fn checked_add(
    left: usize,
    right: usize,
    offset: usize,
    context: &'static str,
) -> FbxResult<usize> {
    left.checked_add(right)
        .ok_or(FbxError::Truncated { offset, context })
}

fn usize_from_u64(value: u64, offset: usize) -> FbxResult<usize> {
    usize::try_from(value)
        .map_err(|_| invalid_record(offset, format!("64-bit value {value} exceeds usize")))
}

fn invalid_record(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::InvalidRecord {
        offset,
        reason: reason.into(),
    }
}

fn fingerprint(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{write::ZlibEncoder, Compression};

    use super::*;

    const LITTLE_ENDIAN_HEADER_PREFIX: &[u8; 23] = b"Kaydara FBX Binary  \0\x1a\0";

    #[test]
    fn rejects_invalid_signature() {
        assert!(matches!(
            decode_binary_fbx(&[0; HEADER_BYTES], FbxLimits::default()),
            Err(FbxError::InvalidSignature)
        ));
    }

    #[test]
    fn rejects_unsupported_version() {
        let bytes = document_with_version(9999);
        assert!(matches!(
            decode_binary_fbx(&bytes, FbxLimits::default()),
            Err(FbxError::UnsupportedVersion { version: 9999 })
        ));
    }

    #[test]
    fn rejects_inputs_over_limit() {
        let bytes = document_with_version(7400);
        let limits = FbxLimits {
            max_input_bytes: bytes.len() - 1,
            ..FbxLimits::default()
        };
        assert!(matches!(
            decode_binary_fbx(&bytes, limits),
            Err(FbxError::InputTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_truncated_record_header() {
        let mut bytes = Vec::from(*LITTLE_ENDIAN_HEADER_PREFIX);
        bytes.extend_from_slice(&7400_u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3]);
        assert!(matches!(
            decode_binary_fbx(&bytes, FbxLimits::default()),
            Err(FbxError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_record_end_outside_input() {
        let mut bytes = Vec::from(*LITTLE_ENDIAN_HEADER_PREFIX);
        bytes.extend_from_slice(&7400_u32.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.push(0);
        assert!(matches!(
            decode_binary_fbx(&bytes, FbxLimits::default()),
            Err(FbxError::InvalidRecord { .. })
        ));
    }

    #[test]
    fn decodes_empty_document_deterministically() {
        let bytes = document_with_version(7400);
        let first = decode_binary_fbx(&bytes, FbxLimits::default()).unwrap();
        let second = decode_binary_fbx(&bytes, FbxLimits::default()).unwrap();
        assert_eq!(first, second);
        assert!(first.records.is_empty());
        assert_eq!(first.footer_offset, HEADER_BYTES + 13);
    }

    #[test]
    fn decodes_compressed_property_array() {
        let values = [3_i32, 5, 8];
        let raw = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut property = vec![b'i'];
        property.extend_from_slice(&(values.len() as u32).to_le_bytes());
        property.extend_from_slice(&1_u32.to_le_bytes());
        property.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        property.extend_from_slice(&compressed);

        let document = decode_binary_fbx(
            &document_with_record(7400, "Array", 1, &property),
            FbxLimits::default(),
        )
        .unwrap();
        assert_eq!(
            document.records[0].properties,
            vec![FbxProperty::I32Array(values.to_vec())]
        );
    }

    #[test]
    fn decodes_big_endian_header_and_property_array() {
        let values = [3_i32, 5, 8];
        let mut property = vec![b'i'];
        property.extend_from_slice(&(values.len() as u32).to_be_bytes());
        property.extend_from_slice(&0_u32.to_be_bytes());
        property.extend_from_slice(&((values.len() * 4) as u32).to_be_bytes());
        property.extend(values.iter().flat_map(|value| value.to_be_bytes()));

        let document = decode_binary_fbx(
            &big_endian_document_with_record(7400, "Array", 1, &property),
            FbxLimits::default(),
        )
        .unwrap();

        assert_eq!(document.byte_order, FbxByteOrder::BigEndian);
        assert_eq!(
            document.records[0].properties,
            vec![FbxProperty::I32Array(values.to_vec())]
        );
    }

    #[test]
    fn decodes_compressed_big_endian_property_array() {
        let values = [3_i32, 5, 8];
        let raw = values
            .iter()
            .flat_map(|value| value.to_be_bytes())
            .collect::<Vec<_>>();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut property = vec![b'i'];
        property.extend_from_slice(&(values.len() as u32).to_be_bytes());
        property.extend_from_slice(&1_u32.to_be_bytes());
        property.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
        property.extend_from_slice(&compressed);

        let document = decode_binary_fbx(
            &big_endian_document_with_record(7400, "Array", 1, &property),
            FbxLimits::default(),
        )
        .unwrap();

        assert_eq!(
            document.records[0].properties,
            vec![FbxProperty::I32Array(values.to_vec())]
        );
    }

    #[test]
    fn rejects_oversized_property_array() {
        let mut property = vec![b'i'];
        property.extend_from_slice(&2_u32.to_le_bytes());
        property.extend_from_slice(&0_u32.to_le_bytes());
        property.extend_from_slice(&8_u32.to_le_bytes());
        property.extend_from_slice(&[0; 8]);
        let limits = FbxLimits {
            max_array_elements: 1,
            ..FbxLimits::default()
        };

        assert!(matches!(
            decode_binary_fbx(&document_with_record(7400, "Array", 1, &property), limits),
            Err(FbxError::ArrayLimit {
                actual: 2,
                limit: 1,
                ..
            })
        ));
    }

    fn document_with_version(version: u32) -> Vec<u8> {
        let mut bytes = Vec::from(*LITTLE_ENDIAN_HEADER_PREFIX);
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.resize(HEADER_BYTES + if version >= 7500 { 25 } else { 13 }, 0);
        bytes
    }

    fn document_with_record(
        version: u32,
        name: &str,
        property_count: u32,
        properties: &[u8],
    ) -> Vec<u8> {
        assert!(version < 7500, "test helper uses the 32-bit record profile");
        let mut bytes = Vec::from(*LITTLE_ENDIAN_HEADER_PREFIX);
        bytes.extend_from_slice(&version.to_le_bytes());

        let end_offset = HEADER_BYTES + 13 + name.len() + properties.len();
        bytes.extend_from_slice(&(end_offset as u32).to_le_bytes());
        bytes.extend_from_slice(&property_count.to_le_bytes());
        bytes.extend_from_slice(&(properties.len() as u32).to_le_bytes());
        bytes.push(name.len() as u8);
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(properties);
        bytes.extend_from_slice(&[0; 13]);
        bytes
    }

    fn big_endian_document_with_record(
        version: u32,
        name: &str,
        property_count: u32,
        properties: &[u8],
    ) -> Vec<u8> {
        assert!(version < 7500, "test helper uses the 32-bit record profile");
        let mut bytes = BINARY_PREFIX.to_vec();
        bytes.push(1);
        bytes.extend_from_slice(&version.to_be_bytes());

        let end_offset = HEADER_BYTES + 13 + name.len() + properties.len();
        bytes.extend_from_slice(&(end_offset as u32).to_be_bytes());
        bytes.extend_from_slice(&property_count.to_be_bytes());
        bytes.extend_from_slice(&(properties.len() as u32).to_be_bytes());
        bytes.push(name.len() as u8);
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(properties);
        bytes.extend_from_slice(&[0; 13]);
        bytes
    }
}
