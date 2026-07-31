use std::{
    fs,
    path::{Path, PathBuf},
};

use gltf_corpus::inspect_gltf_file;
use raster_image_corpus::{
    resolve_bmp_image_asset, resolve_jpeg_image_asset, resolve_png_image_asset, DecodeLimits,
    ImageOrientation, PixelFormat,
};
use screenshot::{write_bmp, Rgba8Image};
use tokimu_assets::{AssetId, AssetLifecycleKind, AssetStore};
use tokimu_render::{Color, Material, TextureHandle};

#[test]
fn box_textured_external_png_resolves_to_a_typed_asset_without_renderer_paths() {
    let gltf_path = fixture_root().join("upstream/Models/BoxTextured/glTF/BoxTextured.gltf");
    let inspection = inspect_gltf_file(&gltf_path).expect("BoxTextured should inspect");
    let image_uri = inspection.images[0]
        .uri
        .as_deref()
        .expect("BoxTextured should name its external image");
    assert_eq!(image_uri, "CesiumLogoFlat.png");

    let image_path = gltf_path
        .parent()
        .expect("glTF fixture must have a parent")
        .join(image_uri);
    let bytes = std::fs::read(&image_path).expect("BoxTextured PNG fixture should exist");

    let mut assets = AssetStore::default();
    let resolved = resolve_png_image_asset(
        &mut assets,
        "khronos/BoxTextured/CesiumLogoFlat.png",
        &bytes,
        DecodeLimits::default(),
    )
    .expect("selected external PNG should resolve");

    assert_eq!(resolved.handle.id(), AssetId(0));
    assert_eq!(resolved.allocation.kind, AssetLifecycleKind::Allocated);
    assert_eq!(resolved.prepared.kind, AssetLifecycleKind::Prepared);
    assert_eq!(resolved.image.pixel_format, PixelFormat::Rgba8);
    assert_eq!(resolved.image.output_orientation, ImageOrientation::TopDown);
    assert!(resolved.image.width > 0);
    assert!(resolved.image.height > 0);

    let inventory = assets.inventory();
    assert_eq!(inventory.entries.len(), 1);
    assert!(inventory.entries[0].prepared);
    assert_eq!(
        inventory.entries[0].source.as_deref(),
        Some("khronos/BoxTextured/CesiumLogoFlat.png")
    );

    let upload = raster_image_corpus::prepare_renderer_texture(
        &resolved.image,
        raster_image_corpus::TextureUse::ColorSrgb,
    )
    .expect("the application may explicitly treat BoxTextured pixels as color");
    assert_eq!(upload.texture.width, resolved.image.width);
    assert_eq!(upload.texture.height, resolved.image.height);
    assert_eq!(upload.texture.rgba8, resolved.image.pixels);
    assert_eq!(
        upload.texture_use,
        raster_image_corpus::TextureUse::ColorSrgb
    );
    assert_eq!(
        upload.source_color_space,
        raster_image_corpus::ColorSpace::Unspecified
    );
    assert_eq!(upload.target_gpu_format, "Rgba8UnormSrgb");

    let artifact = upload.artifact();
    assert_eq!(artifact.artifact_kind, "texture-upload-preparation");
    assert_eq!(artifact.source_stage, "decoded-image");
    assert!(!artifact.gpu_upload_performed);
    assert_eq!(artifact.width, resolved.image.width);
    assert_eq!(artifact.height, resolved.image.height);
    assert_eq!(
        artifact.pixel_fingerprint,
        resolved.image.pixel_fingerprint()
    );
    let json = upload
        .artifact_json()
        .expect("upload evidence should serialize");
    assert!(json.contains("\"gpu_upload_performed\": false"));
    assert!(!json.contains("\"rgba8\""));
}

#[test]
fn missing_image_bytes_fail_at_asset_resolution_before_asset_or_renderer_state_exists() {
    let mut assets = AssetStore::default();
    let error = raster_image_corpus::resolve_optional_png_image_asset(
        &mut assets,
        "khronos/BoxTextured/CesiumLogoFlat.png",
        None,
        DecodeLimits::default(),
    )
    .expect_err("absent dependency bytes must not become a renderer concern");

    assert!(error.to_string().contains("bytes are unavailable"));
    assert!(assets.inventory().entries.is_empty());
}

#[test]
fn jpeg_and_bmp_providers_converge_on_the_same_decoded_asset_contract() {
    let jpeg = fs::read(raster_fixture("testimgint.jpg"))
        .expect("admitted baseline JPEG fixture should exist");
    let bmp =
        fs::read(raster_fixture("shira_bird8.bmp")).expect("admitted BMP fixture should exist");
    let mut assets = AssetStore::default();

    let jpeg = resolve_jpeg_image_asset(
        &mut assets,
        "libjpeg-turbo/testimgint.jpg",
        &jpeg,
        DecodeLimits::default(),
    )
    .expect("baseline JPEG should resolve through asset identity");
    let bmp = resolve_bmp_image_asset(
        &mut assets,
        "libjpeg-turbo/shira_bird8.bmp",
        &bmp,
        DecodeLimits::default(),
    )
    .expect("bounded BMP should resolve through asset identity");

    assert_eq!(jpeg.handle.id(), AssetId(0));
    assert_eq!(bmp.handle.id(), AssetId(1));
    assert_eq!(jpeg.image.output_orientation, ImageOrientation::TopDown);
    assert_eq!(bmp.image.output_orientation, ImageOrientation::TopDown);
    assert!(assets
        .inventory()
        .entries
        .iter()
        .all(|entry| entry.prepared));
    assert_eq!(assets.inventory().entries.len(), 2);
}

#[test]
fn png_jpeg_and_bmp_prepare_the_same_provider_neutral_material_texture_shape() {
    let png =
        fs::read(png_suite_fixture("basn0g08.png")).expect("admitted PNG fixture should exist");
    let jpeg =
        fs::read(raster_fixture("testimgint.jpg")).expect("admitted JPEG fixture should exist");
    let bmp =
        fs::read(raster_fixture("shira_bird8.bmp")).expect("admitted BMP fixture should exist");
    let limits = DecodeLimits::default();
    let images = [
        raster_image_corpus::decode_png(&png, limits).expect("PNG should decode"),
        raster_image_corpus::decode_jpeg(&jpeg, limits).expect("JPEG should decode"),
        raster_image_corpus::decode_bmp(&bmp, limits).expect("BMP should decode"),
    ];

    for (index, image) in images.iter().enumerate() {
        let prepared = raster_image_corpus::prepare_renderer_texture(
            image,
            raster_image_corpus::TextureUse::ColorSrgb,
        )
        .expect("all admitted formats should prepare as normalized color textures");
        let material = Material::new("raster-material", Color::rgb(1.0, 1.0, 1.0))
            .with_texture(TextureHandle(index as u64 + 1));

        assert_eq!(material.texture, Some(TextureHandle(index as u64 + 1)));
        assert_eq!(
            prepared.texture_use,
            raster_image_corpus::TextureUse::ColorSrgb
        );
        assert_eq!(prepared.target_gpu_format, "Rgba8UnormSrgb");
        assert_eq!(prepared.source_orientation, ImageOrientation::TopDown);
        assert_eq!(prepared.texture.rgba8.len(), image.pixels.len());
    }
}

#[test]
fn repeated_image_requests_remain_explicit_until_a_dependency_resolver_owns_identity() {
    let bytes = minimal_rgba_png();
    let mut assets = AssetStore::default();

    let first = resolve_png_image_asset(
        &mut assets,
        "synthetic/repeated.png",
        &bytes,
        DecodeLimits::default(),
    )
    .expect("first image request should resolve");
    let second = resolve_png_image_asset(
        &mut assets,
        "synthetic/repeated.png",
        &bytes,
        DecodeLimits::default(),
    )
    .expect("second image request should resolve explicitly");

    assert_ne!(first.handle.id(), second.handle.id());
    assert_eq!(assets.inventory().entries.len(), 2);
    assert!(assets
        .inventory()
        .entries
        .iter()
        .all(|entry| entry.source.as_deref() == Some("synthetic/repeated.png")));
}

#[test]
fn texture_preparation_rejects_data_intent_until_the_renderer_has_a_data_contract() {
    let mut assets = AssetStore::default();
    let image = resolve_png_image_asset(
        &mut assets,
        "synthetic/empty.png",
        &minimal_rgba_png(),
        DecodeLimits::default(),
    )
    .expect("synthetic PNG should resolve");

    let error = raster_image_corpus::prepare_renderer_texture(
        &image.image,
        raster_image_corpus::TextureUse::LinearData,
    )
    .expect_err("the current renderer has no data-texture upload contract");
    assert!(error.to_string().contains("linear/data"));
}

#[test]
fn palette_transparency_survives_preparation_and_deterministic_cpu_review_export() {
    let bytes = fs::read(png_suite_fixture("tp1n3p08.png"))
        .expect("admitted PNG Suite transparency fixture should exist");
    let mut assets = AssetStore::default();
    let resolved = resolve_png_image_asset(
        &mut assets,
        "png-suite/tp1n3p08.png",
        &bytes,
        DecodeLimits::default(),
    )
    .expect("palette transparency fixture should resolve");
    assert!(
        resolved
            .image
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[3] < 255),
        "the selected fixture must exercise non-opaque pixels"
    );

    let prepared = raster_image_corpus::prepare_renderer_texture(
        &resolved.image,
        raster_image_corpus::TextureUse::ColorSrgb,
    )
    .expect("RGBA8 color texture preparation should preserve alpha bytes");

    let path = std::env::temp_dir().join(format!(
        "tokimu-raster-image-corpus-alpha-{}.bmp",
        std::process::id()
    ));
    write_bmp(
        &path,
        Rgba8Image {
            width: prepared.texture.width,
            height: prepared.texture.height,
            pixels: &prepared.texture.rgba8,
        },
    )
    .expect("deterministic CPU review export should succeed");

    let exported = raster_image_corpus::decode_bmp(
        &fs::read(&path).expect("review export should be readable"),
        DecodeLimits::default(),
    )
    .expect("CPU review export should decode through the BMP boundary");
    assert_eq!(exported.pixels, prepared.texture.rgba8);
    assert_eq!(
        exported.pixel_fingerprint(),
        resolved.image.pixel_fingerprint()
    );
    let _ = fs::remove_file(path);
}

#[test]
fn fbx_texture_reference_remains_source_evidence_until_a_separate_asset_resolver_supplies_bytes() {
    let document = fbx_corpus::decode_binary_fbx_file(
        fbx_fixture("max_gltf_material_7700_binary.fbx"),
        fbx_corpus::FbxLimits::default(),
    )
    .expect("selected FBX fixture should decode");
    let scene = fbx_corpus::resolve_source_scene(&document)
        .expect("selected FBX fixture should expose a source scene");
    let materials =
        fbx_corpus::resolve_materials(&document, &scene).expect("material evidence should resolve");
    let texture = materials
        .textures
        .iter()
        .find(|texture| {
            texture
                .file_name
                .as_deref()
                .or(texture.relative_file_name.as_deref())
                .is_some_and(|path| path.contains("checkerboard"))
        })
        .expect("selected FBX fixture should retain a checkerboard texture reference");

    let source_reference = texture
        .file_name
        .as_deref()
        .or(texture.relative_file_name.as_deref())
        .expect("FBX texture evidence should name a source reference");
    assert!(source_reference.contains("checkerboard"));
    assert!(materials.bindings.iter().any(|binding| {
        binding.child_id == texture.source_id || binding.parent_id == texture.source_id
    }));

    let mut assets = AssetStore::default();
    let error = raster_image_corpus::resolve_optional_png_image_asset(
        &mut assets,
        "fbx/max-gltf-material/checkerboard",
        None,
        DecodeLimits::default(),
    )
    .expect_err("FBX source references must not silently become decoded images");
    assert!(error.to_string().contains("bytes are unavailable"));
    assert!(assets.inventory().entries.is_empty());
}

fn minimal_rgba_png() -> Vec<u8> {
    // A 1x1 opaque red RGBA PNG. Keeping this local makes the upload-boundary
    // failure test independent of third-party fixture availability.
    vec![
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 240,
        31, 0, 5, 0, 1, 255, 137, 153, 61, 29, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ]
}

fn fixture_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/khronos-gltf-sample-assets");
    assert!(
        root.is_dir(),
        "missing Khronos fixtures at {}; run prepare-khronos-gltf-corpus.ps1",
        root.display()
    );
    root
}

fn fbx_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/fbx-corpus/upstream/data")
        .join(name)
}

fn png_suite_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite")
        .join(name)
}

fn raster_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages")
        .join(name)
}
