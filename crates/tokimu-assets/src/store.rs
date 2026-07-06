use crate::{AssetHandle, AssetId};

#[derive(Debug, Default)]
pub struct AssetStore {
    next_id: u64,
}

impl AssetStore {
    pub fn allocate<T>(&mut self) -> AssetHandle<T> {
        let id = AssetId(self.next_id);
        self.next_id += 1;
        AssetHandle::new(id)
    }
}
