use crate::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub clear_color: Color,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            clear_color: Color::BLACK,
        }
    }
}
