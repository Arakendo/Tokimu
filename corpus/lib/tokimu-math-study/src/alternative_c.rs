//! Alternative C's original, narrow `Vec3` implementation.
//!
//! This module intentionally contains no provider reference. It covers only
//! the frozen `Vec3` manifest and shared conformance pressure; matrices,
//! quaternions, and other vector types have not yet earned an owned
//! implementation.

use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

/// Corpus-local owned three-dimensional vector candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);

    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }

    #[must_use]
    pub const fn from_array(values: [f32; 3]) -> Self {
        Self::new(values[0], values[1], values[2])
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).dot(self - other).sqrt()
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        self * self.dot(self).sqrt().recip()
    }

    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        let reciprocal_length = self.dot(self).sqrt().recip();
        if reciprocal_length.is_finite() && reciprocal_length > 0.0 {
            self * reciprocal_length
        } else {
            Self::ZERO
        }
    }

    #[must_use]
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    #[must_use]
    pub fn lerp(self, other: Self, scalar: f32) -> Self {
        self + (other - self) * scalar
    }

    #[must_use]
    pub const fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

/// Corpus-local owned four-dimensional vector candidate.
///
/// This type covers the current `Vec3::extend` / `Vec4::truncate` and bounded
/// CAD homogeneous matrix-vector pressure. Wider vector operations remain
/// unimplemented until their callers and conformance cases are admitted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[must_use]
    pub const fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Add for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
            self.w * scalar,
        )
    }
}

/// Corpus-local owned column-major transform candidate.
///
/// It follows the currently observed right-handed view and OpenGL-depth
/// projection conventions. Those conventions remain case-study evidence, not
/// a stable Tokimu contract. Inversion is intentionally absent pending a
/// dedicated singular-matrix conformance decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    columns: [Vec4; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self::from_columns(
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );

    #[must_use]
    pub const fn from_columns(x_axis: Vec4, y_axis: Vec4, z_axis: Vec4, w_axis: Vec4) -> Self {
        Self {
            columns: [x_axis, y_axis, z_axis, w_axis],
        }
    }

    #[must_use]
    pub const fn from_cols_array(values: &[f32; 16]) -> Self {
        Self::from_columns(
            Vec4::new(values[0], values[1], values[2], values[3]),
            Vec4::new(values[4], values[5], values[6], values[7]),
            Vec4::new(values[8], values[9], values[10], values[11]),
            Vec4::new(values[12], values[13], values[14], values[15]),
        )
    }

    #[must_use]
    pub const fn to_cols_array(self) -> [f32; 16] {
        [
            self.columns[0].x,
            self.columns[0].y,
            self.columns[0].z,
            self.columns[0].w,
            self.columns[1].x,
            self.columns[1].y,
            self.columns[1].z,
            self.columns[1].w,
            self.columns[2].x,
            self.columns[2].y,
            self.columns[2].z,
            self.columns[2].w,
            self.columns[3].x,
            self.columns[3].y,
            self.columns[3].z,
            self.columns[3].w,
        ]
    }

    #[must_use]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self::from_columns(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(translation.x, translation.y, translation.z, 1.0),
        )
    }

    #[must_use]
    pub const fn from_scale(scale: Vec3) -> Self {
        Self::from_columns(
            Vec4::new(scale.x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, scale.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, scale.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn from_rotation_x(angle_radians: f32) -> Self {
        let (sine, cosine) = angle_radians.sin_cos();
        Self::from_columns(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, cosine, sine, 0.0),
            Vec4::new(0.0, -sine, cosine, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn from_rotation_y(angle_radians: f32) -> Self {
        let (sine, cosine) = angle_radians.sin_cos();
        Self::from_columns(
            Vec4::new(cosine, 0.0, -sine, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(sine, 0.0, cosine, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn from_rotation_z(angle_radians: f32) -> Self {
        let (sine, cosine) = angle_radians.sin_cos();
        Self::from_columns(
            Vec4::new(cosine, sine, 0.0, 0.0),
            Vec4::new(-sine, cosine, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let forward = (center - eye).normalize();
        let side = forward.cross(up).normalize();
        let adjusted_up = side.cross(forward);
        Self::from_columns(
            Vec4::new(side.x, adjusted_up.x, -forward.x, 0.0),
            Vec4::new(side.y, adjusted_up.y, -forward.y, 0.0),
            Vec4::new(side.z, adjusted_up.z, -forward.z, 0.0),
            Vec4::new(-eye.dot(side), -eye.dot(adjusted_up), eye.dot(forward), 1.0),
        )
    }

    #[must_use]
    pub fn perspective_rh_gl(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let inverse_depth = 1.0 / (near - far);
        let focal_length = 1.0 / (0.5 * fov_y_radians).tan();
        Self::from_columns(
            Vec4::new(focal_length / aspect_ratio, 0.0, 0.0, 0.0),
            Vec4::new(0.0, focal_length, 0.0, 0.0),
            Vec4::new(0.0, 0.0, (near + far) * inverse_depth, -1.0),
            Vec4::new(0.0, 0.0, (2.0 * near * far) * inverse_depth, 0.0),
        )
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
        Self::from_columns(
            Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
            Vec4::new(0.0, 0.0, -2.0 / (far - near), 0.0),
            Vec4::new(
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -(far + near) / (far - near),
                1.0,
            ),
        )
    }

    #[must_use]
    pub const fn transpose(self) -> Self {
        Self::from_columns(
            Vec4::new(
                self.columns[0].x,
                self.columns[1].x,
                self.columns[2].x,
                self.columns[3].x,
            ),
            Vec4::new(
                self.columns[0].y,
                self.columns[1].y,
                self.columns[2].y,
                self.columns[3].y,
            ),
            Vec4::new(
                self.columns[0].z,
                self.columns[1].z,
                self.columns[2].z,
                self.columns[3].z,
            ),
            Vec4::new(
                self.columns[0].w,
                self.columns[1].w,
                self.columns[2].w,
                self.columns[3].w,
            ),
        )
    }

    /// Returns the inverse using pivoted Gauss-Jordan elimination.
    ///
    /// This candidate's provisional singular behavior is an all-NaN matrix.
    /// It is explicit experiment behavior, not a stable Tokimu guarantee.
    #[must_use]
    pub fn inverse(self) -> Self {
        let values = self.to_cols_array();
        let mut augmented = [[0.0_f32; 8]; 4];

        for row in 0..4 {
            for column in 0..4 {
                augmented[row][column] = values[column * 4 + row];
            }
            augmented[row][4 + row] = 1.0;
        }

        for pivot_column in 0..4 {
            let mut pivot_row = pivot_column;
            for row in (pivot_column + 1)..4 {
                if augmented[row][pivot_column].abs() > augmented[pivot_row][pivot_column].abs() {
                    pivot_row = row;
                }
            }

            let pivot = augmented[pivot_row][pivot_column];
            if !pivot.is_finite() || pivot == 0.0 {
                return Self::nan();
            }
            augmented.swap(pivot_column, pivot_row);

            for column in 0..8 {
                augmented[pivot_column][column] /= pivot;
            }

            for row in 0..4 {
                if row == pivot_column {
                    continue;
                }
                let factor = augmented[row][pivot_column];
                for column in 0..8 {
                    augmented[row][column] -= factor * augmented[pivot_column][column];
                }
            }
        }

        let mut inverse = [0.0_f32; 16];
        for row in 0..4 {
            for column in 0..4 {
                inverse[column * 4 + row] = augmented[row][4 + column];
            }
        }
        Self::from_cols_array(&inverse)
    }

    #[must_use]
    pub fn transform_point3(self, point: Vec3) -> Vec3 {
        (self.columns[0] * point.x
            + self.columns[1] * point.y
            + self.columns[2] * point.z
            + self.columns[3])
            .truncate()
    }

    #[must_use]
    pub fn transform_vector3(self, vector: Vec3) -> Vec3 {
        (self.columns[0] * vector.x + self.columns[1] * vector.y + self.columns[2] * vector.z)
            .truncate()
    }

    #[must_use]
    pub const fn w_axis(self) -> Vec4 {
        self.columns[3]
    }

    pub fn set_w_axis(&mut self, value: Vec4) {
        self.columns[3] = value;
    }

    const fn nan() -> Self {
        Self::from_columns(
            Vec4::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            Vec4::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            Vec4::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            Vec4::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN),
        )
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::from_columns(
            self.transform_vec4(other.columns[0]),
            self.transform_vec4(other.columns[1]),
            self.transform_vec4(other.columns[2]),
            self.transform_vec4(other.columns[3]),
        )
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, value: Vec4) -> Vec4 {
        self.transform_vec4(value)
    }
}

impl Mat4 {
    fn transform_vec4(self, value: Vec4) -> Vec4 {
        self.columns[0] * value.x
            + self.columns[1] * value.y
            + self.columns[2] * value.z
            + self.columns[3] * value.w
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_vec3_has_no_hidden_provider_boundary() {
        let value = (Vec3::new(1.0, 2.0, 3.0) + Vec3::Y) * 2.0;

        assert_eq!(value.to_array(), [2.0, 6.0, 6.0]);
        assert_eq!(
            value.cross(Vec3::new(0.0, 0.0, 1.0)).to_array(),
            [6.0, -2.0, 0.0]
        );
        assert_eq!(Vec3::ZERO.normalize_or_zero(), Vec3::ZERO);
        assert!(Vec3::ZERO
            .normalize()
            .to_array()
            .into_iter()
            .all(f32::is_nan));
    }

    #[test]
    fn owned_vec3_and_vec4_cover_the_current_boundary() {
        let extended = Vec3::new(1.0, 2.0, 3.0).extend(4.0);

        assert_eq!(extended.to_array(), [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(extended.truncate().to_array(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn owned_matrix_inverts_affine_transforms_and_makes_singular_behavior_visible() {
        let translation = Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(
            translation
                .inverse()
                .transform_point3(Vec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0]
        );

        assert!(Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0))
            .inverse()
            .to_cols_array()
            .into_iter()
            .all(f32::is_nan));
    }
}
