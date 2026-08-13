use std::{fs, path::PathBuf};

use compression_provider::{
    BrotliCompressionProvider, CompressionCodec, CompressionError, CompressionGoal,
    CompressionProvider, DecodeLimits, DecodeRequest, EncodeRequest, FlateCompressionProvider,
};
use resource_space::{
    AddressCasePolicy, ContentFingerprint, FolderId, InMemoryResourceSpace, ResourceMetadata,
    ResourceName, ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_compression::{
    transform_resource, ResourceCollisionPolicy, ResourceCompressionRequest,
    ResourceCompressionTransform,
};
use serde::Serialize;

const PAYLOAD: &[u8] = b"Tokimu compression corpus: semantic bytes, provider mechanisms.\n";

#[derive(Serialize)]
struct Report {
    schema: u32,
    artifact: ArtifactProvenance,
    claim: &'static str,
    cases: Vec<CaseObservation>,
    bounded_decode: BoundedDecodeObservation,
    resource_space: ResourceSpaceObservation,
}

#[derive(Serialize)]
struct ArtifactProvenance {
    generator: &'static str,
    selection: &'static str,
    input_fingerprint: String,
    flate_provider: &'static str,
    brotli_provider: &'static str,
    round_trip_limits: DecodeLimits,
    bounded_decode_limits: DecodeLimits,
}

#[derive(Serialize)]
struct CaseObservation {
    codec: CompressionCodec,
    goal: CompressionGoal,
    input_bytes: u64,
    encoded_bytes: u64,
    decoded_bytes: u64,
    byte_identical: bool,
}

#[derive(Serialize)]
struct BoundedDecodeObservation {
    codec: CompressionCodec,
    category: &'static str,
    rejected: bool,
}

#[derive(Serialize)]
struct ResourceSpaceObservation {
    source: String,
    encoded: String,
    decoded: String,
    retained_resources: usize,
    source_unchanged: bool,
    decoded_byte_identical: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flate = FlateCompressionProvider;
    let brotli = BrotliCompressionProvider;
    let mut cases = Vec::new();

    for codec in [
        CompressionCodec::Gzip,
        CompressionCodec::Deflate,
        CompressionCodec::Brotli,
    ] {
        let provider: &dyn CompressionProvider = match codec {
            CompressionCodec::Gzip | CompressionCodec::Deflate => &flate,
            CompressionCodec::Brotli => &brotli,
        };

        for goal in [
            CompressionGoal::Fast,
            CompressionGoal::Balanced,
            CompressionGoal::Small,
        ] {
            let encoded = provider.encode(EncodeRequest::new(codec, PAYLOAD).with_goal(goal))?;
            let decoded = provider.decode(DecodeRequest::new(
                codec,
                &encoded.bytes,
                DecodeLimits::new(16 * 1024, 16 * 1024),
            ))?;
            let byte_identical = decoded.bytes == PAYLOAD;
            if !byte_identical {
                return Err(format!("{codec:?}/{goal:?} did not round trip byte-for-byte").into());
            }

            cases.push(CaseObservation {
                codec,
                goal,
                input_bytes: encoded.observation.input_bytes,
                encoded_bytes: encoded.observation.output_bytes,
                decoded_bytes: decoded.observation.output_bytes,
                byte_identical,
            });
        }
    }

    let expansion_source = vec![b'A'; 4096];
    let encoded = brotli.encode(EncodeRequest::new(
        CompressionCodec::Brotli,
        &expansion_source,
    ))?;
    let bounded_error = brotli
        .decode(DecodeRequest::new(
            CompressionCodec::Brotli,
            &encoded.bytes,
            DecodeLimits::new(1024, 64),
        ))
        .expect_err("the corpus limit must reject expanded output");
    let category = match bounded_error {
        CompressionError::OutputLimitExceeded { .. } => "output-limit-exceeded",
        CompressionError::ExpansionLimitExceeded { .. } => "expansion-limit-exceeded",
        other => return Err(format!("unexpected bounded-decode failure: {other}").into()),
    };
    let resource_space = resource_space_evidence(&brotli)?;

    let report = Report {
        schema: 1,
        artifact: ArtifactProvenance {
            generator: "hello-compression",
            selection: "first-party-compression-fixture-v1",
            input_fingerprint: fingerprint(PAYLOAD),
            flate_provider: "flate2-1.1.9",
            brotli_provider: "brotli-8.0.2",
            round_trip_limits: DecodeLimits::new(16 * 1024, 16 * 1024),
            bounded_decode_limits: DecodeLimits::new(1024, 64),
        },
        claim: "bounded provider-neutral byte compression",
        cases,
        bounded_decode: BoundedDecodeObservation {
            codec: CompressionCodec::Brotli,
            category,
            rejected: true,
        },
        resource_space,
    };
    let path = write_report(&report)?;

    println!(
        "hello-compression: cases={}, bounded_decode={}, resource_space={}, artifact={}",
        report.cases.len(),
        report.bounded_decode.category,
        report.resource_space.decoded_byte_identical,
        path.display()
    );
    Ok(())
}

fn fingerprint(bytes: &[u8]) -> String {
    let fingerprint = ContentFingerprint::blake3(bytes);
    let digest = fingerprint
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{:?}:{digest}", fingerprint.algorithm()).to_ascii_lowercase()
}

fn resource_space_evidence(
    provider: &BrotliCompressionProvider,
) -> Result<ResourceSpaceObservation, Box<dyn std::error::Error>> {
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(100), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(101);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(102), "compression-corpus"),
        folder,
        ResourceMetadata::default(),
    )?;
    let source_name = logical_name("source.txt")?;
    let encoded_name = logical_name("source.br")?;
    let decoded_name = logical_name("decoded.txt")?;
    let source = space.insert_resource(
        folder,
        source_name.clone(),
        PAYLOAD,
        ResourceMetadata {
            media_type: Some("text/plain".to_owned()),
            ..Default::default()
        },
    )?;

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
            metadata: ResourceMetadata {
                media_type: Some("application/brotli".to_owned()),
                ..Default::default()
            },
        },
        provider,
    )?;
    let decoded = transform_resource(
        &mut space,
        ResourceCompressionRequest {
            source_folder: folder,
            source_name: encoded_name,
            destination_folder: folder,
            destination_name: decoded_name,
            transform: ResourceCompressionTransform::Decode {
                codec: CompressionCodec::Brotli,
                limits: DecodeLimits::new(16 * 1024, 16 * 1024),
            },
            collision: ResourceCollisionPolicy::Reject,
            metadata: ResourceMetadata {
                media_type: Some("text/plain".to_owned()),
                ..Default::default()
            },
        },
        provider,
    )?;
    let retained_source = space
        .resource(folder, &source_name)?
        .ok_or("source disappeared after transformation")?;

    let observation = ResourceSpaceObservation {
        source: source.key().address().to_string(),
        encoded: encoded.entry().key().address().to_string(),
        decoded: decoded.entry().key().address().to_string(),
        retained_resources: space.summary().resources(),
        source_unchanged: retained_source.bytes().as_ref() == PAYLOAD,
        decoded_byte_identical: decoded.entry().bytes().as_ref() == PAYLOAD,
    };
    if !observation.source_unchanged
        || !observation.decoded_byte_identical
        || observation.retained_resources != 3
    {
        return Err("Resource Space compression evidence violated its corpus claim".into());
    }
    Ok(observation)
}

fn logical_name(value: &str) -> Result<ResourceName, Box<dyn std::error::Error>> {
    Ok(ResourceName::parse(value, AddressCasePolicy::Sensitive)?)
}

fn write_report(report: &Report) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let directory = workspace_root().join("target/hello-compression");
    fs::create_dir_all(&directory)?;
    let path = directory.join("report.json");
    fs::write(&path, serde_json::to_vec_pretty(report)?)?;
    Ok(path)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("hello-compression must remain beneath corpus/")
        .to_path_buf()
}
