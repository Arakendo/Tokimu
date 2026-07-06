use crate::AssetId;
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetHandle<T> {
    id: AssetId,
    marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
}
