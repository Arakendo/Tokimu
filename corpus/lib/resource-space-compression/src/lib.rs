//! Explicit compression transformations over retained Resource Space bytes.
//!
//! This bridge composes two independent incubating contracts. Resource Space
//! continues to own logical identity and retained bytes; the compression
//! provider owns bounded byte transformations. Ordinary resource reads never
//! invoke this bridge implicitly.

use compression_provider::{
    CompressionCodec, CompressionError, CompressionGoal, CompressionObservation,
    CompressionProvider, DecodeLimits, DecodeRequest, EncodeRequest,
};
use resource_space::{
    ContentFingerprint, FolderId, InMemoryResourceSpace, ResourceEntry, ResourceKey,
    ResourceMetadata, ResourceName, ResourceSpaceError,
};
use thiserror::Error;

/// The caller-selected byte transformation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceCompressionTransform {
    Encode {
        codec: CompressionCodec,
        goal: CompressionGoal,
    },
    Decode {
        codec: CompressionCodec,
        limits: DecodeLimits,
    },
}

impl ResourceCompressionTransform {
    pub const fn codec(self) -> CompressionCodec {
        match self {
            Self::Encode { codec, .. } | Self::Decode { codec, .. } => codec,
        }
    }
}

/// Explicit destination behavior when a logical name is already retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceCollisionPolicy {
    Reject,
    Replace,
}

/// Caller-owned identity, mutation, metadata, and transformation decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCompressionRequest {
    pub source_folder: FolderId,
    pub source_name: ResourceName,
    pub destination_folder: FolderId,
    pub destination_name: ResourceName,
    pub transform: ResourceCompressionTransform,
    pub collision: ResourceCollisionPolicy,
    pub metadata: ResourceMetadata,
}

/// Records whether the bridge created or explicitly replaced the destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceTransformMutation {
    Inserted,
    Replaced,
}

/// Bounded provider-neutral evidence from one completed transformation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCompressionObservation {
    source: ResourceKey,
    source_fingerprint: ContentFingerprint,
    result: ResourceKey,
    result_fingerprint: ContentFingerprint,
    compression: CompressionObservation,
    mutation: ResourceTransformMutation,
}

impl ResourceCompressionObservation {
    pub fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub fn source_fingerprint(&self) -> &ContentFingerprint {
        &self.source_fingerprint
    }

    pub fn result(&self) -> &ResourceKey {
        &self.result
    }

    pub fn result_fingerprint(&self) -> &ContentFingerprint {
        &self.result_fingerprint
    }

    pub const fn compression(&self) -> &CompressionObservation {
        &self.compression
    }

    pub const fn mutation(&self) -> ResourceTransformMutation {
        self.mutation
    }
}

/// The retained result plus evidence linking it to its immutable source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCompressionResult {
    entry: ResourceEntry,
    observation: ResourceCompressionObservation,
}

impl ResourceCompressionResult {
    pub const fn entry(&self) -> &ResourceEntry {
        &self.entry
    }

    pub const fn observation(&self) -> &ResourceCompressionObservation {
        &self.observation
    }
}

#[derive(Debug, Error)]
pub enum ResourceCompressionBridgeError {
    #[error("resource-space lookup failed for source `{name}`: {error}")]
    Lookup {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("source resource `{name}` was not found in the selected logical folder")]
    MissingSource { name: ResourceName },
    #[error("destination resource `{name}` already exists and collision policy is reject")]
    DestinationExists { name: ResourceName },
    #[error("compression transformation failed: {0}")]
    Compression(#[from] CompressionError),
    #[error("resource-space mutation failed for destination `{name}`: {error}")]
    Store {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
}

/// Transforms one explicitly selected retained resource into one explicit
/// destination resource.
///
/// The byte transformation completes before any Resource Space mutation. A
/// failed transform therefore leaves both the source and destination state
/// unchanged. `Replace` is caller-selected and never inferred from a filename
/// or codec.
pub fn transform_resource<P: CompressionProvider>(
    space: &mut InMemoryResourceSpace,
    request: ResourceCompressionRequest,
    provider: &P,
) -> Result<ResourceCompressionResult, ResourceCompressionBridgeError> {
    let source = space
        .resource(request.source_folder, &request.source_name)
        .map_err(|error| ResourceCompressionBridgeError::Lookup {
            name: request.source_name.clone(),
            error: Box::new(error),
        })?
        .ok_or_else(|| ResourceCompressionBridgeError::MissingSource {
            name: request.source_name.clone(),
        })?;

    let transformed = match request.transform {
        ResourceCompressionTransform::Encode { codec, goal } => {
            provider.encode(EncodeRequest::new(codec, source.bytes()).with_goal(goal))?
        }
        ResourceCompressionTransform::Decode { codec, limits } => {
            provider.decode(DecodeRequest::new(codec, source.bytes(), limits))?
        }
    };

    let existing = space
        .resource(request.destination_folder, &request.destination_name)
        .map_err(|error| ResourceCompressionBridgeError::Lookup {
            name: request.destination_name.clone(),
            error: Box::new(error),
        })?;
    let (entry, mutation) = match (existing, request.collision) {
        (Some(_), ResourceCollisionPolicy::Reject) => {
            return Err(ResourceCompressionBridgeError::DestinationExists {
                name: request.destination_name,
            });
        }
        (Some(_), ResourceCollisionPolicy::Replace) => (
            space
                .replace_resource(
                    request.destination_folder,
                    &request.destination_name,
                    transformed.bytes,
                    request.metadata,
                )
                .map_err(|error| ResourceCompressionBridgeError::Store {
                    name: request.destination_name.clone(),
                    error: Box::new(error),
                })?,
            ResourceTransformMutation::Replaced,
        ),
        (None, _) => (
            space
                .insert_resource(
                    request.destination_folder,
                    request.destination_name.clone(),
                    transformed.bytes,
                    request.metadata,
                )
                .map_err(|error| ResourceCompressionBridgeError::Store {
                    name: request.destination_name,
                    error: Box::new(error),
                })?,
            ResourceTransformMutation::Inserted,
        ),
    };

    let observation = ResourceCompressionObservation {
        source: source.key().clone(),
        source_fingerprint: source.content_fingerprint(),
        result: entry.key().clone(),
        result_fingerprint: entry.content_fingerprint(),
        compression: transformed.observation,
        mutation,
    };
    Ok(ResourceCompressionResult { entry, observation })
}

#[cfg(test)]
mod tests;
