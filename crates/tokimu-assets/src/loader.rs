pub trait AssetLoader {
    type Output;

    fn load(&self, source: &[u8]) -> anyhow::Result<Self::Output>;
}
