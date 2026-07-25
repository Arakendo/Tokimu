//! Shared SVG record analysis used by provider-specific corpus runners.

use ui_tools::{
    tessellate_general_fill_with_rule, SvgFillRule, SvgVectorRecord, VectorFillRule, VectorPath,
};

#[derive(Debug)]
pub(crate) struct SvgFillMeshes {
    pub triangles: Vec<[f32; 2]>,
    pub fill_paths: usize,
    pub diagnostics: Vec<String>,
}

pub(crate) fn summarize_paths(description: &str, paths: &[VectorPath]) -> String {
    let contour_count = paths.iter().map(|path| path.contours.len()).sum::<usize>();
    let point_count = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .map(|contour| contour.points.len())
        .sum::<usize>();
    let closed_contours = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .filter(|contour| contour.closed)
        .count();
    format!(
        "{description} paths={} contours={} points={} closed_contours={closed_contours}",
        paths.len(),
        contour_count,
        point_count,
    )
}

pub(crate) fn tessellate_closed_fills(
    records: &[SvgVectorRecord],
    diagnostic_context: &str,
) -> SvgFillMeshes {
    let mut result = SvgFillMeshes {
        triangles: Vec::new(),
        fill_paths: 0,
        diagnostics: Vec::new(),
    };

    for record in records {
        if !record.fill || !record.path.contours.iter().all(|contour| contour.closed) {
            continue;
        }
        let fill_rule = match record.fill_rule {
            SvgFillRule::NonZero => VectorFillRule::NonZero,
            SvgFillRule::EvenOdd => VectorFillRule::EvenOdd,
        };
        match tessellate_general_fill_with_rule(&record.path, fill_rule) {
            Ok(mut triangles) => {
                result.triangles.append(&mut triangles);
                result.fill_paths += 1;
            }
            Err(error) => result.diagnostics.push(format!(
                "closed {diagnostic_context} path fill tessellation failed: {error}"
            )),
        }
    }
    result
}
