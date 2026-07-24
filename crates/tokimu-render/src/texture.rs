#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

impl Texture {
    pub fn rgba8(width: u32, height: u32, rgba8: Vec<u8>) -> Self {
        assert_eq!(rgba8.len(), (width * height * 4) as usize);
        Self {
            width,
            height,
            rgba8,
        }
    }
}
