use serde::{Deserialize, Serialize};

use crate::PresentationControlError;

/// Provider-neutral linear RGB presentation color.
///
/// Opacity is intentionally separate so tint and transparency remain
/// independently composable.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl PresentationColor {
    pub const WHITE: Self = Self::new_unchecked(1.0, 1.0, 1.0);

    pub fn new(red: f32, green: f32, blue: f32) -> Result<Self, PresentationControlError> {
        validate_unit("red", red)?;
        validate_unit("green", green)?;
        validate_unit("blue", blue)?;
        Ok(Self::new_unchecked(red, green, blue))
    }

    pub const fn new_unchecked(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }

    pub(crate) fn multiplied(self, other: Self) -> Self {
        Self::new_unchecked(
            self.red * other.red,
            self.green * other.green,
            self.blue * other.blue,
        )
    }

    pub fn components(self) -> [f32; 3] {
        [self.red, self.green, self.blue]
    }
}

/// Presentation decoded from an importer or otherwise owned by the source.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourcePresentation {
    pub color: PresentationColor,
    pub opacity: f32,
    pub visible: bool,
}

impl SourcePresentation {
    pub fn new(
        color: PresentationColor,
        opacity: f32,
        visible: bool,
    ) -> Result<Self, PresentationControlError> {
        validate_unit("opacity", opacity)?;
        Ok(Self {
            color,
            opacity,
            visible,
        })
    }
}

impl Default for SourcePresentation {
    fn default() -> Self {
        Self {
            color: PresentationColor::WHITE,
            opacity: 1.0,
            visible: true,
        }
    }
}

pub(crate) fn validate_unit(
    field: &'static str,
    value: f32,
) -> Result<(), PresentationControlError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(PresentationControlError::InvalidUnitValue { field })
    }
}
