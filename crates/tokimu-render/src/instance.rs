#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance2d {
    pub translation: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
}

impl Instance2d {
    pub fn new(translation: [f32; 2], scale: [f32; 2], rotation: f32) -> Self {
        Self {
            translation,
            scale,
            rotation,
        }
    }

    pub fn identity() -> Self {
        Self::default()
    }

    pub fn translated(x: f32, y: f32) -> Self {
        Self {
            translation: [x, y],
            ..Self::default()
        }
    }

    pub fn with_translation(mut self, translation: [f32; 2]) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_scale(mut self, scale: [f32; 2]) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn rotated(rotation: f32) -> Self {
        Self {
            rotation,
            ..Self::default()
        }
    }
}

impl Default for Instance2d {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }
}
