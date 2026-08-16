//! Corpus-local panorama enclosure lowering.

use super::super::*;

/// Builds the first Doom-owned sky presentation fixture. `SKY1` is a
/// horizontal panorama in the source package, so this consumer places it on
/// an enclosure rather than pretending the source marker is an ordinary flat
/// texture. The enclosure is intentionally static and corpus-local; it is not
/// a generic renderer sky contract or a claim of original Doom sky projection.
pub(crate) fn build_doom_sky_cylinder(
    center: Vec3,
    scene_radius: f32,
) -> Result<Mesh, tokimu::MeshValidationError> {
    const SEGMENTS: usize = 64;
    let radius = scene_radius * 1.5;
    let bottom = center.y - scene_radius * 3.0;
    let top = center.y + scene_radius * 3.0;
    let mut positions = Vec::with_capacity(SEGMENTS * 6);
    let mut normals = Vec::with_capacity(SEGMENTS * 6);
    let mut texture_coordinates = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..SEGMENTS {
        let u0 = segment as f32 / SEGMENTS as f32;
        let u1 = (segment + 1) as f32 / SEGMENTS as f32;
        let angle0 = u0 * std::f32::consts::TAU;
        let angle1 = u1 * std::f32::consts::TAU;
        let radial0 = Vec3::new(angle0.cos(), 0.0, angle0.sin());
        let radial1 = Vec3::new(angle1.cos(), 0.0, angle1.sin());
        let p0_bottom = center + radial0 * radius + Vec3::Y * (bottom - center.y);
        let p0_top = center + radial0 * radius + Vec3::Y * (top - center.y);
        let p1_bottom = center + radial1 * radius + Vec3::Y * (bottom - center.y);
        let p1_top = center + radial1 * radius + Vec3::Y * (top - center.y);

        for (position, normal, uv) in [
            (p0_bottom, -radial0, [u0, 1.0]),
            (p1_top, -radial1, [u1, 0.0]),
            (p1_bottom, -radial1, [u1, 1.0]),
            (p0_bottom, -radial0, [u0, 1.0]),
            (p0_top, -radial0, [u0, 0.0]),
            (p1_top, -radial1, [u1, 0.0]),
        ] {
            positions.push(position.to_array());
            normals.push(normal.to_array());
            texture_coordinates.push(uv);
        }
    }

    Mesh::new(positions, normals).with_texture_coordinates(texture_coordinates)
}
