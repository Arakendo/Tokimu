use compression_provider::{
    BrotliCompressionProvider, CompressionCodec, CompressionGoal, DecodeLimits,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};

use super::*;

fn name(value: &str) -> ResourceName {
    ResourceName::parse(value, AddressCasePolicy::Sensitive).expect("valid fixture name")
}

fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
    let mut space = InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(2);
    space
        .create_root(
            ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "fixtures"),
            folder,
            ResourceMetadata::default(),
        )
        .expect("fixture root");
    (space, folder)
}

#[test]
fn encode_and_decode_preserve_source_and_explicit_result_identity() {
    let (mut space, folder) = fixture_space();
    let source_name = name("message.txt");
    let encoded_name = name("message.br");
    let decoded_name = name("message-copy.txt");
    let source_bytes = b"resource space compression evidence".repeat(32);
    let source = space
        .insert_resource(
            folder,
            source_name.clone(),
            source_bytes.clone(),
            ResourceMetadata::default(),
        )
        .expect("source");

    let encoded = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name: source_name.clone(),
            destination_folder: folder,
            destination_name: encoded_name.clone(),
            transform: ResourceCompressionTransform::Encode {
                codec: CompressionCodec::Brotli,
                goal: CompressionGoal::Balanced,
            },
            collision: ResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &BrotliCompressionProvider,
    )
    .expect("encode");
    let decoded = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name: encoded_name,
            destination_folder: folder,
            destination_name: decoded_name,
            transform: ResourceCompressionTransform::Decode {
                codec: CompressionCodec::Brotli,
                limits: DecodeLimits::new(1024, 4096).with_expansion_ratio(100),
            },
            collision: ResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &BrotliCompressionProvider,
    )
    .expect("decode");

    assert_eq!(decoded.entry().bytes().as_ref(), source_bytes);
    assert_eq!(encoded.observation().source(), source.key());
    assert_eq!(encoded.observation().result(), encoded.entry().key());
    assert_eq!(
        encoded.observation().mutation(),
        ResourceTransformMutation::Inserted
    );
    assert_eq!(space.resource(folder, &source_name).unwrap(), Some(source));
}

#[test]
fn reject_collision_leaves_existing_destination_unchanged() {
    let (mut space, folder) = fixture_space();
    let source_name = name("source.txt");
    let destination_name = name("existing.br");
    space
        .insert_resource(
            folder,
            source_name.clone(),
            b"source".as_slice(),
            ResourceMetadata::default(),
        )
        .expect("source");
    let existing = space
        .insert_resource(
            folder,
            destination_name.clone(),
            b"existing".as_slice(),
            ResourceMetadata::default(),
        )
        .expect("existing destination");

    let error = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name,
            destination_folder: folder,
            destination_name: destination_name.clone(),
            transform: ResourceCompressionTransform::Encode {
                codec: CompressionCodec::Brotli,
                goal: CompressionGoal::Fast,
            },
            collision: ResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &BrotliCompressionProvider,
    )
    .expect_err("collision must reject");

    assert!(matches!(
        error,
        ResourceCompressionBridgeError::DestinationExists { .. }
    ));
    assert_eq!(
        space.resource(folder, &destination_name).unwrap(),
        Some(existing)
    );
}

#[test]
fn replace_collision_is_explicit_and_observed() {
    let (mut space, folder) = fixture_space();
    let source_name = name("source.txt");
    let destination_name = name("existing.br");
    space
        .insert_resource(
            folder,
            source_name.clone(),
            b"compress me compress me compress me".as_slice(),
            ResourceMetadata::default(),
        )
        .expect("source");
    space
        .insert_resource(
            folder,
            destination_name.clone(),
            b"old".as_slice(),
            ResourceMetadata::default(),
        )
        .expect("existing destination");

    let result = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name,
            destination_folder: folder,
            destination_name,
            transform: ResourceCompressionTransform::Encode {
                codec: CompressionCodec::Brotli,
                goal: CompressionGoal::Small,
            },
            collision: ResourceCollisionPolicy::Replace,
            metadata: ResourceMetadata::default(),
        },
        &BrotliCompressionProvider,
    )
    .expect("explicit replace");

    assert_eq!(
        result.observation().mutation(),
        ResourceTransformMutation::Replaced
    );
    assert_ne!(result.entry().bytes().as_ref(), b"old");
}

#[test]
fn failed_decode_does_not_create_destination() {
    let (mut space, folder) = fixture_space();
    let source_name = name("broken.br");
    let destination_name = name("result.txt");
    space
        .insert_resource(
            folder,
            source_name.clone(),
            b"not brotli".as_slice(),
            ResourceMetadata::default(),
        )
        .expect("source");

    let error = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name: source_name.clone(),
            destination_folder: folder,
            destination_name: destination_name.clone(),
            transform: ResourceCompressionTransform::Decode {
                codec: CompressionCodec::Brotli,
                limits: DecodeLimits::new(1024, 1024),
            },
            collision: ResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata::default(),
        },
        &BrotliCompressionProvider,
    )
    .expect_err("malformed input");

    assert!(matches!(
        error,
        ResourceCompressionBridgeError::Compression(_)
    ));
    assert!(space.resource(folder, &destination_name).unwrap().is_none());
    assert_eq!(
        space
            .resource(folder, &source_name)
            .unwrap()
            .expect("source remains")
            .bytes()
            .as_ref(),
        b"not brotli"
    );
}
