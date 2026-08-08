//! Candidate-to-renderer boundaries for the copied `hello-3d-mono` cases.
//!
//! These are intentionally corpus-local adapters. Candidate callers retain
//! their own vocabulary; only this boundary knows the renderer's current
//! provider-backed camera representation.

use tokimu::Camera;

/// Builds a renderer camera from Alternative B's candidate view matrix.
#[must_use]
pub fn alternative_b_camera(width: f32, height: f32, view: crate::alternative_b::Mat4) -> Camera {
    let mut camera = Camera::perspective_3d(width, height);
    camera.view = view.into_provider();
    camera
}

/// Builds a renderer camera from Alternative C's owned view matrix.
#[must_use]
pub fn alternative_c_camera(width: f32, height: f32, view: crate::alternative_c::Mat4) -> Camera {
    let mut camera = Camera::perspective_3d(width, height);
    camera.view = glam::Mat4::from_cols_array(&view.to_cols_array());
    camera
}
