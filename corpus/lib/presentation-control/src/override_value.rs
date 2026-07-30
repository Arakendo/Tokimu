use serde::{Deserialize, Serialize};

use crate::{color::validate_unit, PresentationColor, PresentationControlError};

/// How an override tint combines with the presentation beneath it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TintMode {
    Multiply,
    Replace,
}

/// Validated color treatment applied by one override layer.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationTint {
    pub color: PresentationColor,
    pub mode: TintMode,
}

impl PresentationTint {
    pub const fn multiply(color: PresentationColor) -> Self {
        Self {
            color,
            mode: TintMode::Multiply,
        }
    }

    pub const fn replace(color: PresentationColor) -> Self {
        Self {
            color,
            mode: TintMode::Replace,
        }
    }
}

/// Semantic emphasis requested by an application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresentationEmphasis {
    Selected,
    Hovered,
    Warning,
    Hotspot,
}

/// Deterministic override order from broad styling to transient emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PresentationLayer {
    Theme,
    Application,
    Selection,
    Hover,
    Warning,
    Hotspot,
}

/// A transient presentation request that does not mutate source asset truth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PresentationOverride {
    pub tint: Option<PresentationTint>,
    pub opacity_multiplier: Option<f32>,
    pub visible: Option<bool>,
    pub emphasis: Option<PresentationEmphasis>,
}

impl PresentationOverride {
    pub fn with_tint(mut self, tint: PresentationTint) -> Self {
        self.tint = Some(tint);
        self
    }

    pub fn with_opacity_multiplier(
        mut self,
        opacity_multiplier: f32,
    ) -> Result<Self, PresentationControlError> {
        validate_unit("opacity_multiplier", opacity_multiplier)?;
        self.opacity_multiplier = Some(opacity_multiplier);
        Ok(self)
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn with_emphasis(mut self, emphasis: PresentationEmphasis) -> Self {
        self.emphasis = Some(emphasis);
        self
    }

    pub(crate) fn validate(self) -> Result<Self, PresentationControlError> {
        if let Some(opacity_multiplier) = self.opacity_multiplier {
            validate_unit("opacity_multiplier", opacity_multiplier)?;
        }
        Ok(self)
    }
}
