#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Tokimu".to_string(),
            width: 1280,
            height: 720,
        }
    }
}
