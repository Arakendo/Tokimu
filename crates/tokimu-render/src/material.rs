use crate::Color;
use crate::TextureHandle;

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub label: String,
    pub base_color: Color,
    pub texture: Option<TextureHandle>,
}

impl Material {
    pub fn new(label: impl Into<String>, base_color: Color) -> Self {
        Self {
            label: label.into(),
            base_color,
            texture: None,
        }
    }

    pub fn with_texture(mut self, texture: TextureHandle) -> Self {
        self.texture = Some(texture);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new("default-material", Color::rgb(0.96, 0.72, 0.28))
    }
}
