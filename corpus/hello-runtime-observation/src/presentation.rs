use std::collections::BTreeMap;

use presentation_control::{
    PresentationColor, PresentationControl, PresentationEmphasis, PresentationLayer,
    PresentationOverride, PresentationTargetDescriptor, PresentationTargetId,
    PresentationTargetKind, PresentationTint, ResolvedPresentation, SourcePresentation,
};
use serde::Serialize;

use crate::ObservationDiagnostic;

/// Imported-node identity is provider-neutral evidence, not an ECS entity ID
/// and not a renderer resource handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ImportedNodeId(pub usize);

/// One explicit relationship between application, source, and presentation
/// identities. No caller may substitute one field for another.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IdentityMappingObservation {
    pub entity: u64,
    pub imported_node: ImportedNodeId,
    pub presentation_target: PresentationTargetId,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationObservation {
    pub owner: &'static str,
    pub mappings: Vec<IdentityMappingObservation>,
    pub targets: Vec<ResolvedPresentationObservation>,
    pub diagnostics: Vec<ObservationDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResolvedPresentationObservation {
    pub target: PresentationTargetId,
    pub source: SourcePresentation,
    pub resolved: ResolvedPresentation,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum PresentationCommand {
    Select { target: PresentationTargetId },
    SetHotspot { target: PresentationTargetId },
    ClearSelection { target: PresentationTargetId },
    ClearHotspot { target: PresentationTargetId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationCommandDisposition {
    Accepted,
    RejectedUnknownTarget,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationCommandResult {
    pub disposition: PresentationCommandDisposition,
    pub target: PresentationTargetId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<ObservationDiagnostic>,
}

/// Corpus-local composition adapter. The existing presentation-control library
/// owns source-plus-override resolution; this type owns only the scenario's
/// explicit mapping evidence and semantic commands.
pub struct ScenarioPresentation {
    control: PresentationControl,
    mappings: BTreeMap<u64, IdentityMappingObservation>,
}

impl ScenarioPresentation {
    pub fn for_hole_punch(arm_entity: u64) -> Self {
        let mut control = PresentationControl::default();
        let target = PresentationTargetId::new(
            PresentationTargetKind::MeshPrimitive,
            "hole-punch/node/21/mesh-primitive",
        )
        .expect("static corpus target must be valid");
        control
            .register_target_with_descriptor(
                PresentationTargetDescriptor::new(target.clone())
                    .with_source_name("step2-arm")
                    .expect("static source name must be valid"),
                SourcePresentation::new(
                    PresentationColor::new(0.78, 0.82, 0.88)
                        .expect("static source color must be valid"),
                    1.0,
                    true,
                )
                .expect("static source presentation must be valid"),
            )
            .expect("static target must register once");
        let mapping = IdentityMappingObservation {
            entity: arm_entity,
            imported_node: ImportedNodeId(21),
            presentation_target: target,
        };
        Self {
            control,
            mappings: BTreeMap::from([(arm_entity, mapping)]),
        }
    }

    pub fn mapping_for_entity(&self, entity: u64) -> Option<&IdentityMappingObservation> {
        self.mappings.get(&entity)
    }

    pub fn apply(&mut self, command: PresentationCommand) -> PresentationCommandResult {
        let target = match &command {
            PresentationCommand::Select { target }
            | PresentationCommand::SetHotspot { target }
            | PresentationCommand::ClearSelection { target }
            | PresentationCommand::ClearHotspot { target } => target.clone(),
        };
        let result = match command {
            PresentationCommand::Select { .. } => self.control.set_override(
                &target,
                PresentationLayer::Selection,
                PresentationOverride::default()
                    .with_tint(PresentationTint::replace(
                        PresentationColor::new(0.35, 0.85, 0.95)
                            .expect("static selection tint must be valid"),
                    ))
                    .with_emphasis(PresentationEmphasis::Selected),
            ),
            PresentationCommand::SetHotspot { .. } => self.control.set_override(
                &target,
                PresentationLayer::Hotspot,
                PresentationOverride::default()
                    .with_tint(PresentationTint::replace(
                        PresentationColor::new(1.0, 0.65, 0.15)
                            .expect("static hotspot tint must be valid"),
                    ))
                    .with_emphasis(PresentationEmphasis::Hotspot),
            ),
            PresentationCommand::ClearSelection { .. } => self
                .control
                .clear_override(&target, PresentationLayer::Selection)
                .map(|_| ()),
            PresentationCommand::ClearHotspot { .. } => self
                .control
                .clear_override(&target, PresentationLayer::Hotspot)
                .map(|_| ()),
        };
        match result {
            Ok(()) => PresentationCommandResult {
                disposition: PresentationCommandDisposition::Accepted,
                target,
                diagnostic: None,
            },
            Err(_) => PresentationCommandResult {
                disposition: PresentationCommandDisposition::RejectedUnknownTarget,
                target: target.clone(),
                diagnostic: Some(presentation_diagnostic(
                    "presentation_target_unresolved",
                    format!("presentation target `{target}` is not registered by this scenario"),
                )),
            },
        }
    }

    pub fn observe(&self) -> PresentationObservation {
        let targets = self
            .control
            .targets()
            .map(|(target, _)| ResolvedPresentationObservation {
                target: target.clone(),
                source: self
                    .control
                    .target_state(target)
                    .expect("registered presentation target must retain source state")
                    .source(),
                resolved: self
                    .control
                    .resolve(target)
                    .expect("registered presentation target must resolve"),
            })
            .collect();
        PresentationObservation {
            owner: "application_presentation_adapter",
            mappings: self.mappings.values().cloned().collect(),
            targets,
            diagnostics: Vec::new(),
        }
    }
}

fn presentation_diagnostic(code: &'static str, message: String) -> ObservationDiagnostic {
    ObservationDiagnostic {
        code,
        owner: "application_presentation_adapter",
        message,
    }
}

#[cfg(test)]
mod tests {
    use presentation_control::{PresentationTargetId, PresentationTargetKind};

    use super::{PresentationCommand, PresentationCommandDisposition, ScenarioPresentation};

    #[test]
    fn selection_and_hotspot_compose_without_mutating_source_mapping() {
        let mut presentation = ScenarioPresentation::for_hole_punch(7);
        let mapping = presentation.mapping_for_entity(7).unwrap().clone();
        assert_eq!(mapping.imported_node.0, 21);

        assert_eq!(
            presentation
                .apply(PresentationCommand::Select {
                    target: mapping.presentation_target.clone(),
                })
                .disposition,
            PresentationCommandDisposition::Accepted
        );
        assert_eq!(
            presentation
                .apply(PresentationCommand::SetHotspot {
                    target: mapping.presentation_target.clone(),
                })
                .disposition,
            PresentationCommandDisposition::Accepted
        );
        let observation = presentation.observe();
        assert_eq!(observation.mappings, vec![mapping]);
        assert_eq!(
            observation.targets[0].resolved.emphasis,
            Some(presentation_control::PresentationEmphasis::Hotspot)
        );
        assert_eq!(
            observation.targets[0].source,
            presentation_control::SourcePresentation::new(
                presentation_control::PresentationColor::new(0.78, 0.82, 0.88).unwrap(),
                1.0,
                true,
            )
            .unwrap()
        );
    }

    #[test]
    fn unknown_presentation_target_is_explicit() {
        let mut presentation = ScenarioPresentation::for_hole_punch(7);
        let missing =
            PresentationTargetId::new(PresentationTargetKind::MeshPrimitive, "missing").unwrap();
        let result = presentation.apply(PresentationCommand::Select { target: missing });
        assert_eq!(
            result.disposition,
            PresentationCommandDisposition::RejectedUnknownTarget
        );
        assert_eq!(
            result.diagnostic.unwrap().code,
            "presentation_target_unresolved"
        );
    }
}
