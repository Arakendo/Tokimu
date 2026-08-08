//! Alternative B's first ownership-boundary probe.
//!
//! The five retained type names are represented as Tokimu candidate types.
//! Their mechanics presently delegate to the pinned `glam` provider, but that
//! provider is deliberately private to this module. The probe remains
//! incomplete: it establishes a measurable boundary before any stable API or
//! migration decision.

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

/// Corpus-local candidate for Tokimu's two-dimensional vector vocabulary.
///
/// There are no current direct callers for this type. Its intentionally small
/// surface records public-name and representation pressure without importing
/// unrelated provider operations into the experiment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    inner: glam::Vec2,
}

impl Vec2 {
    pub const ZERO: Self = Self {
        inner: glam::Vec2::ZERO,
    };

    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            inner: glam::Vec2::new(x, y),
        }
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 2] {
        self.inner.to_array()
    }
}

/// Corpus-local candidate for Tokimu's rotation vocabulary.
///
/// No current direct caller requires quaternion construction or composition.
/// The candidate therefore retains only identity and array observation until a
/// caller or explicitly reviewed compatibility requirement earns more API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    inner: glam::Quat,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        inner: glam::Quat::IDENTITY,
    };

    #[must_use]
    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            inner: glam::Quat::from_xyzw(x, y, z, w),
        }
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 4] {
        self.inner.to_array()
    }
}

/// Corpus-local candidate for Tokimu's three-dimensional vector vocabulary.
///
/// This is experimental evidence only. It must not be imported by stable
/// Tokimu crates or treated as a public engine contract.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    inner: glam::Vec3,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        inner: glam::Vec3::ZERO,
    };
    pub const ONE: Self = Self {
        inner: glam::Vec3::ONE,
    };
    pub const Y: Self = Self {
        inner: glam::Vec3::Y,
    };

    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: glam::Vec3::new(x, y, z),
        }
    }

    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self {
            inner: glam::Vec3::splat(value),
        }
    }

    #[must_use]
    pub const fn from_array(values: [f32; 3]) -> Self {
        Self {
            inner: glam::Vec3::from_array(values),
        }
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 3] {
        self.inner.to_array()
    }

    #[must_use]
    pub fn x(self) -> f32 {
        self.inner.x
    }

    #[must_use]
    pub fn y(self) -> f32 {
        self.inner.y
    }

    #[must_use]
    pub fn z(self) -> f32 {
        self.inner.z
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        Self::from_provider(self.inner.normalize())
    }

    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        Self::from_provider(self.inner.normalize_or_zero())
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::from_provider(self.inner.cross(other.inner))
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.inner.dot(other.inner)
    }

    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.inner.length_squared()
    }

    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        self.inner.distance(other.inner)
    }

    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self::from_provider(self.inner.min(other.inner))
    }

    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self::from_provider(self.inner.max(other.inner))
    }

    #[must_use]
    pub fn lerp(self, other: Self, scalar: f32) -> Self {
        Self::from_provider(self.inner.lerp(other.inner, scalar))
    }

    #[must_use]
    pub fn extend(self, w: f32) -> Vec4 {
        Vec4::from_provider(self.inner.extend(w))
    }

    #[must_use]
    pub(crate) const fn from_provider(value: glam::Vec3) -> Self {
        Self { inner: value }
    }

    #[must_use]
    pub(crate) const fn into_provider(self) -> glam::Vec3 {
        self.inner
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from_provider(self.inner + other.inner)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.inner += other.inner;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::from_provider(self.inner - other.inner)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from_provider(-self.inner)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::from_provider(self.inner * scalar)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::from_provider(self.inner * other.inner)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self::from_provider(self.inner / scalar)
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self::from_provider(self.inner / other.inner)
    }
}

/// Corpus-local candidate for Tokimu's four-dimensional vector vocabulary.
///
/// It covers the `Vec3::extend` / `Vec4::truncate` and homogeneous
/// matrix-vector pressure from the bounded CAD cursor-ray path. As with
/// [`Vec3`], the provider remains private.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    inner: glam::Vec4,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            inner: glam::Vec4::new(x, y, z, w),
        }
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        self.inner.to_array()
    }

    #[must_use]
    pub fn x(self) -> f32 {
        self.inner.x
    }

    #[must_use]
    pub fn y(self) -> f32 {
        self.inner.y
    }

    #[must_use]
    pub fn z(self) -> f32 {
        self.inner.z
    }

    #[must_use]
    pub fn w(self) -> f32 {
        self.inner.w
    }

    #[must_use]
    pub fn truncate(self) -> Vec3 {
        Vec3::from_provider(self.inner.truncate())
    }

    #[must_use]
    pub(crate) const fn from_provider(value: glam::Vec4) -> Self {
        Self { inner: value }
    }
}

/// Corpus-local candidate for Tokimu's transform and projection vocabulary.
///
/// The candidate deliberately exposes only candidate vectors and scalar data.
/// Its writable final column is represented by [`Self::set_w_axis`] rather
/// than a public foreign field; this is a documented ergonomics difference to
/// measure, not a stable replacement decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    inner: glam::Mat4,
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        inner: glam::Mat4::IDENTITY,
    };

    #[must_use]
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Self::from_provider(glam::Mat4::look_at_rh(
            eye.into_provider(),
            center.into_provider(),
            up.into_provider(),
        ))
    }

    #[must_use]
    pub fn perspective_rh_gl(
        vertical_fov_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::from_provider(glam::Mat4::perspective_rh_gl(
            vertical_fov_radians,
            aspect_ratio,
            near,
            far,
        ))
    }

    #[must_use]
    pub fn orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::from_provider(glam::Mat4::orthographic_rh_gl(
            left, right, bottom, top, near, far,
        ))
    }

    #[must_use]
    pub fn from_translation(translation: Vec3) -> Self {
        Self::from_provider(glam::Mat4::from_translation(translation.into_provider()))
    }

    #[must_use]
    pub fn from_scale(scale: Vec3) -> Self {
        Self::from_provider(glam::Mat4::from_scale(scale.into_provider()))
    }

    #[must_use]
    pub fn from_rotation_x(angle_radians: f32) -> Self {
        Self::from_provider(glam::Mat4::from_rotation_x(angle_radians))
    }

    #[must_use]
    pub fn from_rotation_y(angle_radians: f32) -> Self {
        Self::from_provider(glam::Mat4::from_rotation_y(angle_radians))
    }

    #[must_use]
    pub fn from_rotation_z(angle_radians: f32) -> Self {
        Self::from_provider(glam::Mat4::from_rotation_z(angle_radians))
    }

    #[must_use]
    pub fn from_cols_array(columns: &[f32; 16]) -> Self {
        Self::from_provider(glam::Mat4::from_cols_array(columns))
    }

    #[must_use]
    pub fn to_cols_array(self) -> [f32; 16] {
        self.inner.to_cols_array()
    }

    #[must_use]
    pub fn inverse(self) -> Self {
        Self::from_provider(self.inner.inverse())
    }

    #[must_use]
    pub fn transpose(self) -> Self {
        Self::from_provider(self.inner.transpose())
    }

    #[must_use]
    pub fn transform_point3(self, point: Vec3) -> Vec3 {
        Vec3::from_provider(self.inner.transform_point3(point.into_provider()))
    }

    #[must_use]
    pub fn transform_vector3(self, vector: Vec3) -> Vec3 {
        Vec3::from_provider(self.inner.transform_vector3(vector.into_provider()))
    }

    #[must_use]
    pub fn w_axis(self) -> Vec4 {
        Vec4::from_provider(self.inner.w_axis)
    }

    pub fn set_w_axis(&mut self, value: Vec4) {
        self.inner.w_axis = value.inner;
    }

    #[must_use]
    pub(crate) const fn from_provider(value: glam::Mat4) -> Self {
        Self { inner: value }
    }

    #[must_use]
    pub(crate) const fn into_provider(self) -> glam::Mat4 {
        self.inner
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::from_provider(self.inner * other.inner)
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, value: Vec4) -> Vec4 {
        Vec4::from_provider(self.inner * value.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_candidate_values_do_not_require_glam_at_the_call_site() {
        let value = (Vec3::new(1.0, 2.0, 3.0) + Vec3::Y) * 2.0;

        assert_eq!(value.to_array(), [2.0, 6.0, 6.0]);
        assert_eq!(value.x(), 2.0);
        assert_eq!(value.y(), 6.0);
        assert_eq!(value.z(), 6.0);
    }

    #[test]
    fn provider_crossing_is_explicit_and_crate_private() {
        let provider_value = glam::Vec3::new(1.0, 0.0, 0.0);
        let candidate_value = Vec3::from_provider(provider_value);

        assert_eq!(candidate_value.into_provider(), provider_value);
    }

    #[test]
    fn candidate_preserves_the_baseline_layout_for_this_probe() {
        assert_eq!(
            core::mem::size_of::<Vec3>(),
            core::mem::size_of::<glam::Vec3>()
        );
        assert_eq!(
            core::mem::align_of::<Vec3>(),
            core::mem::align_of::<glam::Vec3>()
        );
    }

    #[test]
    fn dot_is_available_for_the_shared_conformance_case() {
        assert_eq!(Vec3::new(1.0, 2.0, 3.0).dot(Vec3::new(4.0, 5.0, 6.0)), 32.0);
    }

    #[test]
    fn component_multiply_and_divide_cover_the_frozen_manifest() {
        let left = Vec3::new(10.0, 18.0, 28.0);
        let right = Vec3::new(2.0, 3.0, 4.0);

        assert_eq!((left / right).to_array(), [5.0, 6.0, 7.0]);
        assert_eq!((left / right * right).to_array(), left.to_array());
    }

    #[test]
    fn extend_and_truncate_cover_the_current_vec3_vec4_boundary() {
        let extended = Vec3::new(1.0, 2.0, 3.0).extend(4.0);

        assert_eq!(extended.to_array(), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(extended.w(), 4.0);
        assert_eq!(extended.truncate().to_array(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn matrix_boundary_uses_candidate_vectors_only() {
        let mut transform = Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(
            transform
                .transform_point3(Vec3::new(1.0, 2.0, 3.0))
                .to_array(),
            [5.0, 7.0, 9.0]
        );

        transform.set_w_axis(Vec4::new(7.0, 8.0, 9.0, 1.0));
        assert_eq!(transform.w_axis().to_array(), [7.0, 8.0, 9.0, 1.0]);
    }

    #[test]
    fn currently_unpressured_types_remain_minimal_candidate_probes() {
        assert_eq!(Vec2::new(1.0, 2.0).to_array(), [1.0, 2.0]);
        assert_eq!(Vec2::ZERO.to_array(), [0.0, 0.0]);
        assert_eq!(Quat::IDENTITY.to_array(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            Quat::from_xyzw(1.0, 2.0, 3.0, 4.0).to_array(),
            [1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn candidate_layouts_match_the_private_provider_for_all_initial_shapes() {
        macro_rules! assert_same_layout {
            ($candidate:ty, $provider:ty) => {
                assert_eq!(
                    core::mem::size_of::<$candidate>(),
                    core::mem::size_of::<$provider>()
                );
                assert_eq!(
                    core::mem::align_of::<$candidate>(),
                    core::mem::align_of::<$provider>()
                );
            };
        }

        assert_same_layout!(Vec2, glam::Vec2);
        assert_same_layout!(Vec3, glam::Vec3);
        assert_same_layout!(Vec4, glam::Vec4);
        assert_same_layout!(Quat, glam::Quat);
        assert_same_layout!(Mat4, glam::Mat4);
    }
}
