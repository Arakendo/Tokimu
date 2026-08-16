//! Fixed classic-presentation viewport used by the Doom corpus experiments.
//!
//! This is a private reconstruction convention shared by the ordered
//! presentation controls. It is not a Tokimu renderer viewport contract.

pub(crate) const CLASSIC_PRESENTATION_COLUMNS: usize = 320;
pub(crate) const CLASSIC_PRESENTATION_ROWS: usize = 200;
pub(crate) const CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV: f64 = std::f64::consts::FRAC_PI_4;

pub(crate) fn classic_presentation_half_vertical_fov() -> f64 {
    ((CLASSIC_PRESENTATION_ROWS as f64 / CLASSIC_PRESENTATION_COLUMNS as f64)
        * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan())
    .atan()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_fov_is_derived_from_the_classic_viewport_aspect() {
        let expected = ((200.0_f64 / 320.0) * std::f64::consts::FRAC_PI_4.tan()).atan();

        assert_eq!(classic_presentation_half_vertical_fov(), expected);
    }
}
