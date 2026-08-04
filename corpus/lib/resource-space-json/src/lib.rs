//! Typed JSON bridge for logical Resource Space entries.
//!
//! `resource-space` owns qualified logical identity and immutable bytes. This
//! adapter owns the optional serde/JSON conversion only; it does not make JSON
//! a resource-store requirement, filesystem concern, or canonical byte format.

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use resource_space::{
    FolderId, InMemoryResourceSpace, ResourceEntry, ResourceMetadata, ResourceName,
    ResourceSpaceError,
};

/// Reports typed JSON serialization, parsing, and logical-store lookup errors.
#[derive(Debug, Error)]
pub enum ResourceJsonBridgeError {
    #[error("could not serialize resource JSON: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("could not decode JSON resource `{resource:?}`: {message}")]
    Decode {
        resource: resource_space::ResourceKey,
        message: String,
    },
    #[error("resource-space insertion failed for JSON resource `{name}`: {error}")]
    Store {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("resource-space lookup failed for JSON resource `{name}`: {error}")]
    Lookup {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("JSON resource `{name}` was not found in the selected logical folder")]
    MissingResource { name: ResourceName },
}

/// Serializes a value as compact JSON and inserts it into one explicit logical
/// folder. A caller-selected media type wins; otherwise this bridge records
/// `application/json` as useful format metadata.
pub fn store_json_resource<T: Serialize>(
    space: &mut InMemoryResourceSpace,
    folder: FolderId,
    name: ResourceName,
    value: &T,
    mut metadata: ResourceMetadata,
) -> Result<ResourceEntry, ResourceJsonBridgeError> {
    let bytes = serde_json::to_vec(value)?;
    let requested_name = name.clone();
    metadata
        .media_type
        .get_or_insert_with(|| "application/json".to_owned());
    space
        .insert_resource(folder, name, bytes, metadata)
        .map_err(|error| ResourceJsonBridgeError::Store {
            name: requested_name,
            error: Box::new(error),
        })
}

/// Decodes one already-resolved immutable resource entry as typed JSON.
/// Decode failure deliberately leaves the source entry unchanged and available
/// for callers to inspect or retry with another format adapter.
pub fn read_json_resource<T: DeserializeOwned>(
    resource: &ResourceEntry,
) -> Result<T, ResourceJsonBridgeError> {
    serde_json::from_slice(resource.bytes()).map_err(|error| ResourceJsonBridgeError::Decode {
        resource: resource.key().clone(),
        message: error.to_string(),
    })
}

/// Resolves and decodes a JSON sibling using the caller-selected folder.
pub fn resolve_json_resource<T: DeserializeOwned>(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    name: &ResourceName,
) -> Result<T, ResourceJsonBridgeError> {
    let resource = space
        .resource(folder, name)
        .map_err(|error| ResourceJsonBridgeError::Lookup {
            name: name.clone(),
            error: Box::new(error),
        })?
        .ok_or_else(|| ResourceJsonBridgeError::MissingResource { name: name.clone() })?;
    read_json_resource(&resource)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use resource_space::{AddressCasePolicy, ResourceRootDescriptor, ResourceRootId, StoreId};

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Manifest {
        title: String,
        revision: u32,
    }

    fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
        let mut space =
            InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
        let folder = FolderId::from_u128(2);
        space
            .create_root(
                ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "fixtures"),
                folder,
                ResourceMetadata::default(),
            )
            .expect("root");
        (space, folder)
    }

    #[test]
    fn round_trips_typed_json_without_changing_resource_space_identity() {
        let (mut space, folder) = fixture_space();
        let entry = store_json_resource(
            &mut space,
            folder,
            ResourceName::parse("project.json", AddressCasePolicy::Sensitive).expect("name"),
            &Manifest {
                title: "Corpus".to_owned(),
                revision: 7,
            },
            ResourceMetadata::default(),
        )
        .expect("store JSON");

        let decoded: Manifest = read_json_resource(&entry).expect("decode JSON");
        assert_eq!(decoded.title, "Corpus");
        assert_eq!(decoded.revision, 7);
        assert_eq!(
            entry.metadata().media_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(space.summary().resources(), 1);
    }

    #[test]
    fn caller_media_type_is_preserved() {
        let (mut space, folder) = fixture_space();
        let entry = store_json_resource(
            &mut space,
            folder,
            ResourceName::parse("manifest.data", AddressCasePolicy::Sensitive).expect("name"),
            &Manifest {
                title: "Corpus".to_owned(),
                revision: 1,
            },
            ResourceMetadata {
                media_type: Some("application/vnd.tokimu.manifest+json".to_owned()),
                ..Default::default()
            },
        )
        .expect("store JSON");

        assert_eq!(
            entry.metadata().media_type.as_deref(),
            Some("application/vnd.tokimu.manifest+json")
        );
    }

    #[test]
    fn invalid_json_is_diagnostic_and_leaves_source_available() {
        let (mut space, folder) = fixture_space();
        let entry = space
            .insert_resource(
                folder,
                ResourceName::parse("broken.json", AddressCasePolicy::Sensitive).expect("name"),
                b"{ invalid }".as_slice(),
                ResourceMetadata::default(),
            )
            .expect("resource");

        let error = read_json_resource::<Manifest>(&entry).expect_err("invalid JSON");
        assert!(matches!(error, ResourceJsonBridgeError::Decode { .. }));
        assert_eq!(entry.bytes().as_ref(), b"{ invalid }");
    }

    #[test]
    fn missing_logical_name_is_explicit() {
        let (space, folder) = fixture_space();
        let name = ResourceName::parse("missing.json", AddressCasePolicy::Sensitive).expect("name");

        let error = resolve_json_resource::<Manifest>(&space, folder, &name).expect_err("missing");
        assert!(matches!(
            error,
            ResourceJsonBridgeError::MissingResource { .. }
        ));
    }
}
