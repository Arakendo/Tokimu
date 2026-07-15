use crate::{AssetHandle, AssetId};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetRecord {
    pub id: AssetId,
    pub source: Option<String>,
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
                Some(source) => writeln!(f, "  - {} <- {}", entry.id.0, source)?,
                None => writeln!(f, "  - {} <- <unknown>", entry.id.0)?,
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AssetStore {
    next_id: u64,
    entries: Vec<AssetRecord>,
}

impl AssetStore {
    pub fn allocate<T>(&mut self) -> AssetHandle<T> {
        let id = AssetId(self.next_id);
        self.next_id += 1;
        self.entries.push(AssetRecord { id, source: None });
        AssetHandle::new(id)
    }

    pub fn allocate_with_source<T, S>(&mut self, source: S) -> AssetHandle<T>
    where
        S: Into<String>,
    {
        let id = AssetId(self.next_id);
        self.next_id += 1;
        self.entries.push(AssetRecord {
            id,
            source: Some(source.into()),
        });
        AssetHandle::new(id)
    }

    pub fn inventory(&self) -> AssetInventory {
        AssetInventory {
            entries: self.entries.clone(),
        }
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
            }
        );
        assert_eq!(
            inventory.entries[1],
            AssetRecord {
                id: AssetId(1),
                source: None,
            }
        );
        assert_eq!(
            format!("{inventory}"),
            "asset browser\n  - 0 <- models/cube.glb\n  - 1 <- <unknown>\n"
        );
    }
}
