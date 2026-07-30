use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    PresentationColor, PresentationControlError, PresentationEmphasis, PresentationLayer,
    PresentationOverride, PresentationTargetDescriptor, PresentationTargetId, SourcePresentation,
    TintMode,
};

/// Fully resolved presentation, ready for a renderer-facing lowering adapter.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedPresentation {
    pub color: PresentationColor,
    pub opacity: f32,
    pub visible: bool,
    pub emphasis: Option<PresentationEmphasis>,
}

impl From<SourcePresentation> for ResolvedPresentation {
    fn from(source: SourcePresentation) -> Self {
        Self {
            color: source.color,
            opacity: source.opacity,
            visible: source.visible,
            emphasis: None,
        }
    }
}

/// Source presentation and ordered transient overrides for one target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationTargetState {
    descriptor: PresentationTargetDescriptor,
    source: SourcePresentation,
    overrides: BTreeMap<PresentationLayer, PresentationOverride>,
}

impl PresentationTargetState {
    pub fn descriptor(&self) -> &PresentationTargetDescriptor {
        &self.descriptor
    }

    pub fn source(&self) -> SourcePresentation {
        self.source
    }

    pub fn overrides(&self) -> &BTreeMap<PresentationLayer, PresentationOverride> {
        &self.overrides
    }

    pub fn resolve(&self) -> ResolvedPresentation {
        self.overrides
            .values()
            .fold(self.source.into(), apply_override)
    }
}

/// Registry and deterministic resolver for transient presentation intent.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PresentationControl {
    #[serde(with = "target_map")]
    targets: BTreeMap<PresentationTargetId, PresentationTargetState>,
}

impl PresentationControl {
    pub fn register_target(
        &mut self,
        target: PresentationTargetId,
        source: SourcePresentation,
    ) -> Result<(), PresentationControlError> {
        self.register_target_with_descriptor(PresentationTargetDescriptor::new(target), source)
    }

    pub fn register_target_with_descriptor(
        &mut self,
        descriptor: PresentationTargetDescriptor,
        source: SourcePresentation,
    ) -> Result<(), PresentationControlError> {
        let target = descriptor.id().clone();
        if self.targets.contains_key(&target) {
            return Err(PresentationControlError::DuplicateTarget { target });
        }
        self.targets.insert(
            target,
            PresentationTargetState {
                descriptor,
                source,
                overrides: BTreeMap::new(),
            },
        );
        Ok(())
    }

    pub fn set_override(
        &mut self,
        target: &PresentationTargetId,
        layer: PresentationLayer,
        override_value: PresentationOverride,
    ) -> Result<(), PresentationControlError> {
        let override_value = override_value.validate()?;
        let state = self.targets.get_mut(target).ok_or_else(|| {
            PresentationControlError::UnknownTarget {
                target: target.clone(),
            }
        })?;
        state.overrides.insert(layer, override_value);
        Ok(())
    }

    pub fn clear_override(
        &mut self,
        target: &PresentationTargetId,
        layer: PresentationLayer,
    ) -> Result<Option<PresentationOverride>, PresentationControlError> {
        let state = self.targets.get_mut(target).ok_or_else(|| {
            PresentationControlError::UnknownTarget {
                target: target.clone(),
            }
        })?;
        Ok(state.overrides.remove(&layer))
    }

    pub fn clear_target_overrides(
        &mut self,
        target: &PresentationTargetId,
    ) -> Result<(), PresentationControlError> {
        let state = self.targets.get_mut(target).ok_or_else(|| {
            PresentationControlError::UnknownTarget {
                target: target.clone(),
            }
        })?;
        state.overrides.clear();
        Ok(())
    }

    pub fn resolve(
        &self,
        target: &PresentationTargetId,
    ) -> Result<ResolvedPresentation, PresentationControlError> {
        self.targets
            .get(target)
            .map(PresentationTargetState::resolve)
            .ok_or_else(|| PresentationControlError::UnknownTarget {
                target: target.clone(),
            })
    }

    pub fn target_state(&self, target: &PresentationTargetId) -> Option<&PresentationTargetState> {
        self.targets.get(target)
    }

    pub fn targets(
        &self,
    ) -> impl ExactSizeIterator<Item = (&PresentationTargetId, &PresentationTargetState)> {
        self.targets.iter()
    }
}

fn apply_override(
    mut resolved: ResolvedPresentation,
    override_value: &PresentationOverride,
) -> ResolvedPresentation {
    if let Some(tint) = override_value.tint {
        resolved.color = match tint.mode {
            TintMode::Multiply => resolved.color.multiplied(tint.color),
            TintMode::Replace => tint.color,
        };
    }
    if let Some(opacity_multiplier) = override_value.opacity_multiplier {
        resolved.opacity *= opacity_multiplier;
    }
    if let Some(visible) = override_value.visible {
        resolved.visible = visible;
    }
    if let Some(emphasis) = override_value.emphasis {
        resolved.emphasis = Some(emphasis);
    }
    resolved
}

mod target_map {
    use std::collections::{btree_map::Entry, BTreeMap};

    use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

    use super::{PresentationTargetId, PresentationTargetState};

    pub fn serialize<S>(
        targets: &BTreeMap<PresentationTargetId, PresentationTargetState>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        targets.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<BTreeMap<PresentationTargetId, PresentationTargetState>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries =
            Vec::<(PresentationTargetId, PresentationTargetState)>::deserialize(deserializer)?;
        let mut targets = BTreeMap::new();
        for (target, state) in entries {
            match targets.entry(target) {
                Entry::Vacant(entry) => {
                    entry.insert(state);
                }
                Entry::Occupied(entry) => {
                    return Err(D::Error::custom(format!(
                        "duplicate serialized presentation target `{}`",
                        entry.key()
                    )));
                }
            }
        }
        Ok(targets)
    }
}
