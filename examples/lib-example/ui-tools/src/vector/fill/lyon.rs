use lyon_path::{math::point, Path};
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, FillVertex};

use super::VectorFillRule;

pub(super) fn tessellate_lyon_contours(
    contours: &[Vec<[f32; 2]>],
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    let mut builder = Path::builder();
    for points in contours {
        builder.begin(point(points[0][0], points[0][1]));
        for vertex in &points[1..] {
            builder.line_to(point(vertex[0], vertex[1]));
        }
        builder.close();
    }
    let lyon_path = builder.build();
    let mut buffers = lyon_tessellation::VertexBuffers::<[f32; 2], u32>::new();
    let mut output = lyon_tessellation::BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
        vertex.position().to_array()
    });
    let lyon_fill_rule = match fill_rule {
        VectorFillRule::NonZero => FillRule::NonZero,
        VectorFillRule::EvenOdd => FillRule::EvenOdd,
    };
    FillTessellator::new()
        .tessellate_path(
            &lyon_path,
            &FillOptions::default().with_fill_rule(lyon_fill_rule),
            &mut output,
        )
        .map_err(|error| format!("general fill tessellation failed: {error:?}"))?;
    Ok(buffers
        .indices
        .chunks_exact(3)
        .flat_map(|triangle| {
            [
                buffers.vertices[triangle[0] as usize],
                buffers.vertices[triangle[1] as usize],
                buffers.vertices[triangle[2] as usize],
            ]
        })
        .collect())
}

pub(super) fn mesh_preserves_contour_bounds(
    contours: &[Vec<[f32; 2]>],
    triangles: &[[f32; 2]],
) -> bool {
    if triangles.is_empty() {
        return false;
    }
    let source = contours.iter().flatten().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |bounds, point| {
            [
                bounds[0].min(point[0]),
                bounds[1].min(point[1]),
                bounds[2].max(point[0]),
                bounds[3].max(point[1]),
            ]
        },
    );
    let mesh = triangles.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |bounds, point| {
            [
                bounds[0].min(point[0]),
                bounds[1].min(point[1]),
                bounds[2].max(point[0]),
                bounds[3].max(point[1]),
            ]
        },
    );
    // Lyon may move a boundary vertex by a few floating-point units while
    // tessellating a valid contour. Do not route an otherwise valid glyph
    // through the conservative self-intersection repair path for that noise;
    // reserve repair for meaningful missing extents.
    let extent = (source[2] - source[0])
        .abs()
        .max((source[3] - source[1]).abs());
    let tolerance = (extent * 1.0e-4).max(1.0e-6);
    source
        .iter()
        .zip(mesh.iter())
        .all(|(source, mesh)| (source - mesh).abs() <= tolerance)
}
