//! Incubating adapter from logical resource bytes to Tokimu asset loading.
//!
//! `resource-space` retains immutable bytes and logical identity.
//! `tokimu-assets` owns handle allocation and lifecycle. This adapter makes
//! those contracts compose without making either one own the other.

use gltf_corpus::{decode_gltf_with_buffers, inspect_gltf, CorpusError, DecodedModel};
use resource_space::{
    FolderId, InMemoryResourceSpace, ResourceEntry, ResourceKey, ResourceName, ResourceSpaceError,
};
use thiserror::Error;
use tokimu_assets::{
    AssetHandle, AssetLifecycleObservation, AssetLoader, AssetStore, AssetStoreError,
};

/// An external glTF image resolved to an immutable logical resource.
///
/// This is source dependency evidence only. Image decoding, color
/// interpretation, texture preparation, and renderer upload remain separate
/// provider responsibilities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGltfImage {
    image_index: usize,
    source: ResourceEntry,
}

impl ResolvedGltfImage {
    pub const fn image_index(&self) -> usize {
        self.image_index
    }

    pub fn source(&self) -> &ResourceEntry {
        &self.source
    }
}

/// A decoded asset paired with its independent logical source identity.
#[derive(Debug)]
pub struct LoadedResourceAsset<T> {
    handle: AssetHandle<T>,
    value: T,
    source: ResourceKey,
    allocated: AssetLifecycleObservation,
    prepared: AssetLifecycleObservation,
}

impl<T> LoadedResourceAsset<T> {
    pub fn handle(&self) -> AssetHandle<T> {
        self.handle
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub fn allocated(&self) -> &AssetLifecycleObservation {
        &self.allocated
    }

    pub fn prepared(&self) -> &AssetLifecycleObservation {
        &self.prepared
    }
}

/// Reports loader and lifecycle failures without exposing provider internals.
#[derive(Debug, Error)]
pub enum ResourceAssetBridgeError {
    #[error("asset loader rejected resource `{resource:?}`: {message}")]
    LoaderRejected {
        resource: ResourceKey,
        message: String,
    },
    #[error("asset lifecycle failed for resource `{resource:?}`: {error}")]
    AssetStore {
        resource: ResourceKey,
        #[source]
        error: AssetStoreError,
    },
}

/// Reports an external glTF buffer resolution failure at the logical-resource
/// boundary, before the decoder receives any source bytes.
#[derive(Debug, Error)]
pub enum ResourceGltfBridgeError {
    #[error("glTF document `{document:?}` could not be inspected: {error}")]
    Inspection {
        document: ResourceKey,
        #[source]
        error: CorpusError,
    },
    #[error("glTF document `{document:?}` buffer {index} has no external URI")]
    MissingExternalUri { document: ResourceKey, index: usize },
    #[error(
        "glTF document `{document:?}` buffer URI `{uri}` is not a logical relative resource name"
    )]
    InvalidExternalUri { document: ResourceKey, uri: String },
    #[error("glTF document `{document:?}` image {index} has no external URI")]
    ImageMissingExternalUri { document: ResourceKey, index: usize },
    #[error(
        "glTF document `{document:?}` image URI `{uri}` is not a logical relative resource name"
    )]
    InvalidImageUri { document: ResourceKey, uri: String },
    #[error("resource-space lookup failed while resolving `{name}` for `{document:?}`: {error}")]
    Lookup {
        document: ResourceKey,
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("glTF document `{document:?}` references missing sibling resource `{name}`")]
    MissingResource {
        document: ResourceKey,
        name: ResourceName,
    },
    #[error("glTF document `{document:?}` failed to decode after resource resolution: {error}")]
    Decode {
        document: ResourceKey,
        #[source]
        error: CorpusError,
    },
}

/// Decodes a JSON glTF document whose external buffer URIs resolve inside one
/// explicit resource-space folder.
///
/// The caller chooses the folder boundary. URI syntax remains glTF-specific,
/// resource-space retains the source bytes, and the glTF corpus decoder owns
/// document and accessor validation. This is intentionally not a generic
/// resource resolver or a filesystem replacement.
pub fn decode_gltf_from_resource_space(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    document: &ResourceEntry,
) -> Result<DecodedModel, ResourceGltfBridgeError> {
    let document_key = document.key().clone();
    let inspection =
        inspect_gltf(document.bytes()).map_err(|error| ResourceGltfBridgeError::Inspection {
            document: document_key.clone(),
            error,
        })?;

    let mut buffers = Vec::with_capacity(inspection.buffers.len());
    for buffer in inspection.buffers {
        let uri = buffer
            .uri
            .ok_or_else(|| ResourceGltfBridgeError::MissingExternalUri {
                document: document_key.clone(),
                index: buffer.index,
            })?;
        let name = ResourceName::parse(&uri, space.case_policy()).map_err(|_| {
            ResourceGltfBridgeError::InvalidExternalUri {
                document: document_key.clone(),
                uri: uri.clone(),
            }
        })?;
        let entry = space
            .resource(folder, &name)
            .map_err(|error| ResourceGltfBridgeError::Lookup {
                document: document_key.clone(),
                name: name.clone(),
                error: Box::new(error),
            })?
            .ok_or_else(|| ResourceGltfBridgeError::MissingResource {
                document: document_key.clone(),
                name: name.clone(),
            })?;
        buffers.push(entry.bytes().to_vec());
    }

    decode_gltf_with_buffers(document.bytes(), buffers).map_err(|error| {
        ResourceGltfBridgeError::Decode {
            document: document_key,
            error,
        }
    })
}

/// Resolves every explicitly URI-backed glTF image inside one selected logical
/// resource-space folder.
///
/// Images embedded through `bufferView` are not external dependencies and are
/// intentionally omitted. The returned entries retain their logical identity
/// and immutable bytes; this bridge does not parse image formats or allocate a
/// renderer texture.
pub fn resolve_gltf_external_images_from_resource_space(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    document: &ResourceEntry,
) -> Result<Vec<ResolvedGltfImage>, ResourceGltfBridgeError> {
    let document_key = document.key().clone();
    let inspection =
        inspect_gltf(document.bytes()).map_err(|error| ResourceGltfBridgeError::Inspection {
            document: document_key.clone(),
            error,
        })?;

    let mut resolved = Vec::new();
    for image in inspection.images {
        if image.buffer_view.is_some() {
            continue;
        }
        let uri = image
            .uri
            .ok_or_else(|| ResourceGltfBridgeError::ImageMissingExternalUri {
                document: document_key.clone(),
                index: image.index,
            })?;
        let name = ResourceName::parse(&uri, space.case_policy()).map_err(|_| {
            ResourceGltfBridgeError::InvalidImageUri {
                document: document_key.clone(),
                uri: uri.clone(),
            }
        })?;
        let entry = space
            .resource(folder, &name)
            .map_err(|error| ResourceGltfBridgeError::Lookup {
                document: document_key.clone(),
                name: name.clone(),
                error: Box::new(error),
            })?
            .ok_or_else(|| ResourceGltfBridgeError::MissingResource {
                document: document_key.clone(),
                name,
            })?;
        resolved.push(ResolvedGltfImage {
            image_index: image.index,
            source: entry,
        });
    }

    Ok(resolved)
}

/// Loads immutable resource bytes through an existing provider-neutral loader.
///
/// The asset store receives only a stable diagnostic source label and lifecycle
/// transitions. It never receives a resource-space reference or assumes how
/// bytes were obtained. If decoding fails, no asset handle is allocated and the
/// original resource entry remains unchanged and available to callers.
pub fn load_resource_asset<T, L>(
    assets: &mut AssetStore,
    entry: &ResourceEntry,
    loader: &L,
) -> Result<LoadedResourceAsset<T>, ResourceAssetBridgeError>
where
    L: AssetLoader<Output = T>,
{
    let source = entry.key().clone();
    let value =
        loader
            .load(entry.bytes())
            .map_err(|error| ResourceAssetBridgeError::LoaderRejected {
                resource: source.clone(),
                message: error.to_string(),
            })?;
    let (handle, allocated) =
        assets.allocate_with_source_observed::<T, _>(source.address().to_string());
    let prepared =
        assets
            .mark_prepared(handle)
            .map_err(|error| ResourceAssetBridgeError::AssetStore {
                resource: source.clone(),
                error,
            })?;

    Ok(LoadedResourceAsset {
        handle,
        value,
        source,
        allocated,
        prepared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use resource_space::{
        AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
        ResourceRootDescriptor, ResourceRootId, StoreId,
    };

    struct ByteCountLoader;

    impl AssetLoader for ByteCountLoader {
        type Output = usize;

        fn load(&self, source: &[u8]) -> anyhow::Result<Self::Output> {
            Ok(source.len())
        }
    }

    struct RejectingLoader;

    impl AssetLoader for RejectingLoader {
        type Output = ();

        fn load(&self, _source: &[u8]) -> anyhow::Result<Self::Output> {
            Err(anyhow::anyhow!("fixture decode failure"))
        }
    }

    fn fixture_entry() -> ResourceEntry {
        let (space, folder) = fixture_space();
        let name = ResourceName::parse("models/box.glb", AddressCasePolicy::Sensitive)
            .expect("logical name");
        let mut space = space;
        space
            .insert_resource(
                folder,
                name.clone(),
                [1, 2, 3, 4],
                ResourceMetadata::default(),
            )
            .expect("resource");
        space
            .resource(folder, &name)
            .expect("lookup")
            .expect("entry")
    }

    fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
        let mut space =
            InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
        let root = ResourceRootId::from_u128(2);
        let folder = FolderId::from_u128(3);
        space
            .create_root(
                ResourceRootDescriptor::new(root, "fixtures"),
                folder,
                ResourceMetadata::default(),
            )
            .expect("root");
        (space, folder)
    }

    fn triangle_document() -> &'static [u8] {
        br#"{
          "asset":{"version":"2.0"},
          "buffers":[{"uri":"triangle.bin","byteLength":42}],
          "bufferViews":[
            {"buffer":0,"byteOffset":0,"byteLength":36},
            {"buffer":0,"byteOffset":36,"byteLength":6}
          ],
          "accessors":[
            {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},
            {"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}
          ],
          "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}]
        }"#
    }

    fn triangle_buffer() -> Vec<u8> {
        let mut bytes = Vec::with_capacity(42);
        for value in [
            0.0_f32, 0.0, 0.0, // first vertex
            1.0, 0.0, 0.0, // second vertex
            0.0, 1.0, 0.0, // third vertex
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for index in [0_u16, 1, 2] {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        bytes
    }

    fn image_document() -> &'static [u8] {
        br#"{
          "asset":{"version":"2.0"},
          "images":[{"uri":"swatch.png","mimeType":"image/png"}],
          "textures":[{"source":0}]
        }"#
    }

    #[test]
    fn bridge_loads_bytes_without_transferring_resource_ownership() {
        let entry = fixture_entry();
        let mut assets = AssetStore::default();

        let loaded = load_resource_asset(&mut assets, &entry, &ByteCountLoader).expect("load");

        assert_eq!(*loaded.value(), 4);
        assert_eq!(loaded.source(), entry.key());
        assert_eq!(loaded.allocated().source.as_deref(), Some("models/box.glb"));
        assert!(loaded.prepared().sequence > loaded.allocated().sequence);
        assert_eq!(assets.inventory().entries.len(), 1);
        assert_eq!(entry.bytes().as_ref(), &[1, 2, 3, 4]);
    }

    #[test]
    fn failed_decode_does_not_allocate_an_asset_or_consume_source_bytes() {
        let entry = fixture_entry();
        let mut assets = AssetStore::default();

        let error = load_resource_asset(&mut assets, &entry, &RejectingLoader)
            .expect_err("rejected fixture");

        assert!(matches!(
            error,
            ResourceAssetBridgeError::LoaderRejected { .. }
        ));
        assert!(assets.inventory().entries.is_empty());
        assert_eq!(entry.bytes().as_ref(), &[1, 2, 3, 4]);
    }

    #[test]
    fn gltf_external_buffer_resolves_from_an_explicit_resource_folder() {
        let (mut space, folder) = fixture_space();
        let document_name = ResourceName::parse("triangle.gltf", AddressCasePolicy::Sensitive)
            .expect("document name");
        let buffer_name =
            ResourceName::parse("triangle.bin", AddressCasePolicy::Sensitive).expect("buffer name");
        let document = space
            .insert_resource(
                folder,
                document_name,
                triangle_document(),
                ResourceMetadata::default(),
            )
            .expect("document");
        space
            .insert_resource(
                folder,
                buffer_name,
                triangle_buffer(),
                ResourceMetadata::default(),
            )
            .expect("buffer");

        let model = decode_gltf_from_resource_space(&space, folder, &document)
            .expect("logical resource decode");

        assert_eq!(model.primitives.len(), 1);
        assert_eq!(model.primitives[0].positions.len(), 3);
        assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        assert_eq!(document.bytes().as_ref(), triangle_document());
    }

    #[test]
    fn gltf_missing_external_buffer_is_reported_without_mutating_the_document() {
        let (mut space, folder) = fixture_space();
        let document_name = ResourceName::parse("triangle.gltf", AddressCasePolicy::Sensitive)
            .expect("document name");
        let document = space
            .insert_resource(
                folder,
                document_name,
                triangle_document(),
                ResourceMetadata::default(),
            )
            .expect("document");

        let error = decode_gltf_from_resource_space(&space, folder, &document)
            .expect_err("missing buffer must be explicit");

        assert!(matches!(
            error,
            ResourceGltfBridgeError::MissingResource { .. }
        ));
        assert_eq!(document.bytes().as_ref(), triangle_document());
    }

    #[test]
    fn gltf_external_image_resolves_without_invoking_an_image_provider() {
        let (mut space, folder) = fixture_space();
        let document_name = ResourceName::parse("material.gltf", AddressCasePolicy::Sensitive)
            .expect("document name");
        let image_name =
            ResourceName::parse("swatch.png", AddressCasePolicy::Sensitive).expect("image name");
        let document = space
            .insert_resource(
                folder,
                document_name,
                image_document(),
                ResourceMetadata::default(),
            )
            .expect("document");
        space
            .insert_resource(
                folder,
                image_name,
                [137, 80, 78, 71],
                ResourceMetadata::default(),
            )
            .expect("image");

        let resolved = resolve_gltf_external_images_from_resource_space(&space, folder, &document)
            .expect("logical resource resolution");

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].image_index(), 0);
        assert_eq!(resolved[0].source().name().as_str(), "swatch.png");
        assert_eq!(resolved[0].source().bytes().as_ref(), &[137, 80, 78, 71]);
        assert_eq!(document.bytes().as_ref(), image_document());
    }

    #[test]
    fn gltf_missing_external_image_is_reported_without_mutating_the_document() {
        let (mut space, folder) = fixture_space();
        let document_name = ResourceName::parse("material.gltf", AddressCasePolicy::Sensitive)
            .expect("document name");
        let document = space
            .insert_resource(
                folder,
                document_name,
                image_document(),
                ResourceMetadata::default(),
            )
            .expect("document");

        let error = resolve_gltf_external_images_from_resource_space(&space, folder, &document)
            .expect_err("missing image must be explicit");

        assert!(matches!(
            error,
            ResourceGltfBridgeError::MissingResource { .. }
        ));
        assert_eq!(document.bytes().as_ref(), image_document());
    }
}
