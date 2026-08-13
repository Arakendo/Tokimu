//! Isolated Narrow-B semantic-construction candidate.
//!
//! This preserves the existing Tokimu value vocabulary while hiding provider
//! camera-module organization and provider failures behind a bounded contract.

#[cfg(all(feature = "provider-029", feature = "provider-033"))]
compile_error!("select exactly one Narrow-B provider revision");
#[cfg(not(any(feature = "provider-029", feature = "provider-033")))]
compile_error!("select one Narrow-B provider revision");

#[cfg(feature = "provider-029")]
use glam_029 as provider;
#[cfg(feature = "provider-033")]
use glam_033 as provider;

pub use provider::{Mat4, Vec3};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstructionOperation {
    ViewLookAtRh,
    PerspectiveRhGl,
    OrthographicRhGl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstructionFailure {
    NonFiniteInput,
    DegenerateView,
    InvalidFrustum,
    NonFiniteResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstructionError {
    pub operation: ConstructionOperation,
    pub failure: ConstructionFailure,
}

impl ConstructionError {
    const fn new(operation: ConstructionOperation, failure: ConstructionFailure) -> Self {
        Self { operation, failure }
    }
}

pub type ConstructionResult = Result<Mat4, ConstructionError>;

pub fn view_look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> ConstructionResult {
    const OPERATION: ConstructionOperation = ConstructionOperation::ViewLookAtRh;
    if !vector_is_finite(eye) || !vector_is_finite(target) || !vector_is_finite(up) {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::NonFiniteInput,
        ));
    }

    let forward = target - eye;
    if forward.length_squared() == 0.0
        || up.length_squared() == 0.0
        || forward.cross(up).length_squared() == 0.0
    {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::DegenerateView,
        ));
    }

    finite_result(OPERATION, provider_view(eye, target, up))
}

pub fn projection_perspective_rh_gl(
    vertical_fov_radians: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> ConstructionResult {
    const OPERATION: ConstructionOperation = ConstructionOperation::PerspectiveRhGl;
    if ![vertical_fov_radians, aspect_ratio, near, far]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::NonFiniteInput,
        ));
    }
    if !(0.0 < vertical_fov_radians
        && vertical_fov_radians < core::f32::consts::PI
        && aspect_ratio > 0.0
        && 0.0 < near
        && near < far)
    {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::InvalidFrustum,
        ));
    }

    finite_result(
        OPERATION,
        provider_perspective(vertical_fov_radians, aspect_ratio, near, far),
    )
}

pub fn projection_orthographic_rh_gl(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> ConstructionResult {
    const OPERATION: ConstructionOperation = ConstructionOperation::OrthographicRhGl;
    if ![left, right, bottom, top, near, far]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::NonFiniteInput,
        ));
    }
    if !(left < right && bottom < top && near < far) {
        return Err(ConstructionError::new(
            OPERATION,
            ConstructionFailure::InvalidFrustum,
        ));
    }

    finite_result(
        OPERATION,
        provider_orthographic(left, right, bottom, top, near, far),
    )
}

fn vector_is_finite(vector: Vec3) -> bool {
    vector.to_array().into_iter().all(f32::is_finite)
}

fn finite_result(operation: ConstructionOperation, matrix: Mat4) -> ConstructionResult {
    matrix
        .to_cols_array()
        .into_iter()
        .all(f32::is_finite)
        .then_some(matrix)
        .ok_or_else(|| ConstructionError::new(operation, ConstructionFailure::NonFiniteResult))
}

#[cfg(feature = "provider-029")]
fn provider_view(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

#[cfg(feature = "provider-033")]
fn provider_view(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    provider::camera::rh::view::look_at_mat4(eye, target, up)
}

#[cfg(feature = "provider-029")]
fn provider_perspective(vertical_fov_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    Mat4::perspective_rh_gl(vertical_fov_radians, aspect, near, far)
}

#[cfg(feature = "provider-033")]
fn provider_perspective(vertical_fov_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    provider::camera::rh::proj::opengl::perspective(vertical_fov_radians, aspect, near, far)
}

#[cfg(feature = "provider-029")]
fn provider_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
}

#[cfg(feature = "provider-033")]
fn provider_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    provider::camera::rh::proj::opengl::orthographic(left, right, bottom, top, near, far)
}
