pub trait AssetLoader {
    type Output;
    type Error: std::error::Error;

    fn load(&self, source: &[u8]) -> Result<Self::Output, Self::Error>;
}
