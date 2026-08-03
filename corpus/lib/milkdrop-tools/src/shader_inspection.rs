//! Bounded inspection of shader-bearing MilkDrop entries.
//!
//! This module records source facts that a future compatibility provider would
//! need before attempting any HLSL-to-WGSL lowering. It does not parse,
//! translate, compile, or execute shader code.

use serde::{Deserialize, Serialize};

use crate::{MilkDropConstruct, MilkDropPresetDocument, MilkDropSourceLocation};

/// The MilkDrop pass named by a shader-bearing preset entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropShaderStage {
    Warp,
    Composite,
}

/// A deliberately small source feature inventory for compatibility evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropShaderFeature {
    FunctionDeclaration,
    HlslScalarType,
    HlslVectorType,
    TextureSampling,
    ControlFlow,
    PreprocessorDirective,
}

/// A source-preserving translation disposition.
///
/// `Deferred` means the entry was retained and inspected, but no source is
/// passed to a renderer or silently approximated as WGSL.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropShaderTranslationDisposition {
    Deferred,
}

/// The explicit boundary that prevents a source feature from being translated.
///
/// These are diagnostic facts, not fallback behavior. In particular, a source
/// texture sample cannot select an asset, sampler, or backend binding by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilkDropShaderTranslationBlocker {
    HlslTranslationNotAdmitted,
    ControlFlowTranslationNotAdmitted,
    PreprocessorTranslationNotAdmitted,
    TextureRequirementsUnderReview,
}

/// One renderer-neutral inspection record for a MilkDrop shader entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilkDropShaderInspection {
    pub stage: MilkDropShaderStage,
    pub key: String,
    pub location: MilkDropSourceLocation,
    pub source_bytes: usize,
    pub features: Vec<MilkDropShaderFeature>,
    pub blockers: Vec<MilkDropShaderTranslationBlocker>,
    pub translation: MilkDropShaderTranslationDisposition,
}

/// Inspects shader-bearing entries without broadening the selected provider
/// subset into a shader compiler.
pub fn inspect_shader_entries(document: &MilkDropPresetDocument) -> Vec<MilkDropShaderInspection> {
    document
        .sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter_map(|entry| {
            let stage = match entry.construct {
                MilkDropConstruct::UnsupportedWarpShader => MilkDropShaderStage::Warp,
                MilkDropConstruct::UnsupportedCompositeShader => MilkDropShaderStage::Composite,
                _ => return None,
            };
            Some(MilkDropShaderInspection {
                stage,
                key: entry.key.clone(),
                location: entry.location,
                source_bytes: entry.value.len(),
                features: inspect_source_features(&entry.value),
                blockers: inspect_translation_blockers(&entry.value),
                translation: MilkDropShaderTranslationDisposition::Deferred,
            })
        })
        .collect()
}

fn inspect_translation_blockers(source: &str) -> Vec<MilkDropShaderTranslationBlocker> {
    let features = inspect_source_features(source);
    let mut blockers = vec![MilkDropShaderTranslationBlocker::HlslTranslationNotAdmitted];
    if features.contains(&MilkDropShaderFeature::ControlFlow) {
        blockers.push(MilkDropShaderTranslationBlocker::ControlFlowTranslationNotAdmitted);
    }
    if features.contains(&MilkDropShaderFeature::PreprocessorDirective) {
        blockers.push(MilkDropShaderTranslationBlocker::PreprocessorTranslationNotAdmitted);
    }
    if features.contains(&MilkDropShaderFeature::TextureSampling) {
        blockers.push(MilkDropShaderTranslationBlocker::TextureRequirementsUnderReview);
    }
    blockers
}

fn inspect_source_features(source: &str) -> Vec<MilkDropShaderFeature> {
    let lower = source.to_ascii_lowercase();
    let mut features = Vec::new();
    let mut add = |feature| {
        if !features.contains(&feature) {
            features.push(feature);
        }
    };

    if lower.contains('(') && lower.contains(')') && lower.contains('{') {
        add(MilkDropShaderFeature::FunctionDeclaration);
    }
    if ["float", "half", "double", "int", "bool"]
        .iter()
        .any(|token| lower.contains(token))
    {
        add(MilkDropShaderFeature::HlslScalarType);
    }
    if ["float2", "float3", "float4", "half2", "half3", "half4"]
        .iter()
        .any(|token| lower.contains(token))
    {
        add(MilkDropShaderFeature::HlslVectorType);
    }
    if ["tex2d", "texture", "sampler"]
        .iter()
        .any(|token| lower.contains(token))
    {
        add(MilkDropShaderFeature::TextureSampling);
    }
    if ["if", "for", "while", "switch"]
        .iter()
        .any(|token| contains_word(&lower, token))
    {
        add(MilkDropShaderFeature::ControlFlow);
    }
    if source
        .lines()
        .any(|line| line.trim_start().starts_with('#'))
    {
        add(MilkDropShaderFeature::PreprocessorDirective);
    }
    features
}

fn contains_word(source: &str, word: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|token| token == word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MilkDropPresetDocument;

    #[test]
    fn retains_shader_stage_and_hlsl_features_without_translation() {
        let document = MilkDropPresetDocument::parse(
            "[preset00]\nwarp_shader_0=float4 main(float2 uv) { if (uv.x > 0) return tex2D(s, uv); return 0; }\ncomp_shader_0=#define X 1 // float4 main() { return X; }",
        )
        .unwrap();

        let inspections = inspect_shader_entries(&document);
        assert_eq!(inspections.len(), 2);
        assert_eq!(inspections[0].stage, MilkDropShaderStage::Warp);
        assert!(inspections[0]
            .features
            .contains(&MilkDropShaderFeature::TextureSampling));
        assert!(inspections[0]
            .features
            .contains(&MilkDropShaderFeature::ControlFlow));
        assert!(inspections[0]
            .blockers
            .contains(&MilkDropShaderTranslationBlocker::TextureRequirementsUnderReview));
        assert_eq!(inspections[1].stage, MilkDropShaderStage::Composite);
        assert!(inspections[1]
            .features
            .contains(&MilkDropShaderFeature::PreprocessorDirective));
        assert!(inspections[1]
            .blockers
            .contains(&MilkDropShaderTranslationBlocker::PreprocessorTranslationNotAdmitted));
        assert_eq!(
            inspections[0].translation,
            MilkDropShaderTranslationDisposition::Deferred
        );
    }
}
