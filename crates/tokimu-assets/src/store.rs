use crate::{AssetHandle, AssetId};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetLifecycleKind {
    Allocated,
    Prepared,
    Replaced,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetLifecycleObservation {
    pub sequence: u64,
    pub asset_id: AssetId,
    pub generation: u64,
    pub kind: AssetLifecycleKind,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetStoreError {
    UnknownAsset(AssetId),
}

impl fmt::Display for AssetStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownAsset(id) => write!(f, "asset {} is not registered", id.0),
        }
    }
}

impl std::error::Error for AssetStoreError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetRecord {
    pub id: AssetId,
    pub source: Option<String>,
    pub generation: u64,
    pub prepared: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetInventory {
    pub entries: Vec<AssetRecord>,
}

impl fmt::Display for AssetInventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "asset browser")?;
        for entry in &self.entries {
            match &entry.source {
                Some(source) => writeln!(
                    f,
                    "  - {}@{} [{}] <- {}",
                    entry.id.0,
                    entry.generation,
                    if entry.prepared {
                        "prepared"
                    } else {
                        "allocated"
                    },
                    source
                )?,
                None => writeln!(
                    f,
                    "  - {}@{} [{}] <- <unknown>",
                    entry.id.0,
                    entry.generation,
                    if entry.prepared {
                        "prepared"
                    } else {
                        "allocated"
                    }
                )?,
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AssetStore {
    next_id: u64,
    next_observation_sequence: u64,
    entries: Vec<AssetRecord>,
}

impl AssetStore {
    pub fn allocate<T>(&mut self) -> AssetHandle<T> {
        self.allocate_observed().0
    }

    pub fn allocate_observed<T>(&mut self) -> (AssetHandle<T>, AssetLifecycleObservation) {
        self.allocate_internal(None)
    }

    pub fn allocate_with_source<T, S>(&mut self, source: S) -> AssetHandle<T>
    where
        S: Into<String>,
    {
        self.allocate_with_source_observed(source).0
    }

    pub fn allocate_with_source_observed<T, S>(
        &mut self,
        source: S,
    ) -> (AssetHandle<T>, AssetLifecycleObservation)
    where
        S: Into<String>,
    {
        self.allocate_internal(Some(source.into()))
    }

    pub fn mark_prepared<T>(
        &mut self,
        handle: AssetHandle<T>,
    ) -> Result<AssetLifecycleObservation, AssetStoreError> {
        let record = self.record_mut(handle.id())?;
        record.prepared = true;
        let record = record.clone();
        Ok(self.observe(&record, AssetLifecycleKind::Prepared))
    }

    /// Records that the logical asset behind a stable handle was replaced.
    ///
    /// The caller owns the replacement mechanism. The store only advances the
    /// provider-neutral generation and returns the resulting observation.
    pub fn mark_replaced<T>(
        &mut self,
        handle: AssetHandle<T>,
    ) -> Result<AssetLifecycleObservation, AssetStoreError> {
        let record = self.record_mut(handle.id())?;
        record.generation = record.generation.saturating_add(1);
        record.prepared = false;
        let record = record.clone();
        Ok(self.observe(&record, AssetLifecycleKind::Replaced))
    }

    pub fn release<T>(
        &mut self,
        handle: AssetHandle<T>,
    ) -> Result<AssetLifecycleObservation, AssetStoreError> {
        let index = self
            .entries
            .iter()
            .position(|record| record.id == handle.id())
            .ok_or(AssetStoreError::UnknownAsset(handle.id()))?;
        let record = self.entries.remove(index);
        Ok(self.observe(&record, AssetLifecycleKind::Released))
    }

    pub fn inventory(&self) -> AssetInventory {
        AssetInventory {
            entries: self.entries.clone(),
        }
    }

    fn allocate_internal<T>(
        &mut self,
        source: Option<String>,
    ) -> (AssetHandle<T>, AssetLifecycleObservation) {
        let id = AssetId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        let record = AssetRecord {
            id,
            source,
            generation: 0,
            prepared: false,
        };
        let observation = self.observe(&record, AssetLifecycleKind::Allocated);
        self.entries.push(record);
        (AssetHandle::new(id), observation)
    }

    fn record_mut(&mut self, id: AssetId) -> Result<&mut AssetRecord, AssetStoreError> {
        self.entries
            .iter_mut()
            .find(|record| record.id == id)
            .ok_or(AssetStoreError::UnknownAsset(id))
    }

    fn observe(
        &mut self,
        record: &AssetRecord,
        kind: AssetLifecycleKind,
    ) -> AssetLifecycleObservation {
        let observation = AssetLifecycleObservation {
            sequence: self.next_observation_sequence,
            asset_id: record.id,
            generation: record.generation,
            kind,
            source: record.source.clone(),
        };
        self.next_observation_sequence = self.next_observation_sequence.saturating_add(1);
        observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventories_allocated_assets_with_sources() {
        let mut store = AssetStore::default();
        let first = store.allocate_with_source::<u32, _>("models/cube.glb");
        let second = store.allocate::<u32>();

        let inventory = store.inventory();

        assert_eq!(first.id(), AssetId(0));
        assert_eq!(second.id(), AssetId(1));
        assert_eq!(inventory.entries.len(), 2);
        assert_eq!(
            inventory.entries[0],
            AssetRecord {
                id: AssetId(0),
                source: Some("models/cube.glb".into()),
                generation: 0,
                prepared: false,
            }
        );
        assert_eq!(
            inventory.entries[1],
            AssetRecord {
                id: AssetId(1),
                source: None,
                generation: 0,
                prepared: false,
            }
        );
        assert_eq!(
            format!("{inventory}"),
            "asset browser\n  - 0@0 [allocated] <- models/cube.glb\n  - 1@0 [allocated] <- <unknown>\n"
        );
    }

    #[test]
    fn lifecycle_observations_preserve_identity_generation_and_order() {
        let mut store = AssetStore::default();
        let (handle, allocated) = store.allocate_with_source_observed::<u32, _>("models/cube.glb");
        let prepared = store.mark_prepared(handle).unwrap();
        let replaced = store.mark_replaced(handle).unwrap();
        let prepared_again = store.mark_prepared(handle).unwrap();
        let released = store.release(handle).unwrap();

        assert_eq!(
            [
                allocated.kind,
                prepared.kind,
                replaced.kind,
                prepared_again.kind,
                released.kind,
            ],
            [
                AssetLifecycleKind::Allocated,
                AssetLifecycleKind::Prepared,
                AssetLifecycleKind::Replaced,
                AssetLifecycleKind::Prepared,
                AssetLifecycleKind::Released,
            ]
        );
        assert_eq!(
            [
                allocated.sequence,
                prepared.sequence,
                replaced.sequence,
                prepared_again.sequence,
                released.sequence,
            ],
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            [
                allocated.asset_id,
                prepared.asset_id,
                replaced.asset_id,
                prepared_again.asset_id,
                released.asset_id,
            ],
            [handle.id(); 5]
        );
        assert_eq!(
            [
                allocated.generation,
                prepared.generation,
                replaced.generation,
                prepared_again.generation,
                released.generation,
            ],
            [0, 0, 1, 1, 1]
        );
        assert!(store.inventory().entries.is_empty());
    }

    #[test]
    fn unknown_asset_transitions_fail_explicitly() {
        let mut store = AssetStore::default();
        let missing = AssetHandle::<u32>::new(AssetId(42));

        assert_eq!(
            store.mark_prepared(missing),
            Err(AssetStoreError::UnknownAsset(AssetId(42)))
        );
    }
}
