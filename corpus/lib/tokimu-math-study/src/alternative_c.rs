//! Alternative C's narrow, original owned-math candidate.
//!
//! This module intentionally contains no provider reference. It covers only
//! the caller-traced `Vec3`, `Vec4`, and `Mat4` manifest plus shared
//! conformance pressure. Quaternions and other vector types have not earned an
//! owned implementation.

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
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);
    pub const NEG_X: Self = Self::new(-1.0, 0.0, 0.0);
    pub const NEG_Y: Self = Self::new(0.0, -1.0, 0.0);
    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0);

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
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
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
        self * self.length().recip()
    }

    /// Returns a finite unit vector when this value can be normalized.
    ///
    /// Zero, non-finite, overflowed, and otherwise unrepresentable inputs are
    /// rejected instead of being converted into a plausible fallback value.
    #[must_use]
    pub fn try_normalize(self) -> Option<Self> {
        if !self.is_finite() {
            return None;
        }

        let reciprocal_length = self.length().recip();
        if !reciprocal_length.is_finite() || reciprocal_length <= 0.0 {
            return None;
        }

        let normalized = self * reciprocal_length;
        normalized.is_finite().then_some(normalized)
    }

    /// Normalizes finite nonzero input and accepts exactly zero as zero.
    ///
    /// This differs deliberately from the retained provider-observation
    /// method below: NaN and infinity remain explicit rejection.
    #[must_use]
    pub fn try_normalize_or_zero(self) -> Option<Self> {
        if self == Self::ZERO {
            return Some(Self::ZERO);
        }
        self.try_normalize()
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
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Accumulates values in caller-provided iteration order.
    ///
    /// This intentionally does not select the provider's `Sum` trait surface
    /// or permit reassociation/parallel reduction.
    pub fn sum_ordered(values: impl IntoIterator<Item = Self>) -> Self {
        let mut sum = Self::ZERO;
        for value in values {
            sum += value;
        }
        sum
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

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
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
/// a stable Tokimu contract. Checked constructors and queries implement the
/// experimental C0 numerical contract without selecting a stable API shape.
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
    pub const fn from_cols_array_2d(values: &[[f32; 4]; 4]) -> Self {
        Self::from_columns(
            Vec4::new(values[0][0], values[0][1], values[0][2], values[0][3]),
            Vec4::new(values[1][0], values[1][1], values[1][2], values[1][3]),
            Vec4::new(values[2][0], values[2][1], values[2][2], values[2][3]),
            Vec4::new(values[3][0], values[3][1], values[3][2], values[3][3]),
        )
    }

    #[must_use]
    pub const fn to_cols_array_2d(self) -> [[f32; 4]; 4] {
        [
            self.columns[0].to_array(),
            self.columns[1].to_array(),
            self.columns[2].to_array(),
            self.columns[3].to_array(),
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

    /// Constructs a finite right-handed view matrix or rejects a degenerate
    /// camera basis.
    #[must_use]
    pub fn try_look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Option<Self> {
        if !eye.is_finite() || !center.is_finite() {
            return None;
        }

        let forward = (center - eye).try_normalize()?;
        let normalized_up = up.try_normalize()?;
        let side = forward.cross(normalized_up).try_normalize()?;
        let adjusted_up = side.cross(forward);
        let matrix = Self::from_columns(
            Vec4::new(side.x, adjusted_up.x, -forward.x, 0.0),
            Vec4::new(side.y, adjusted_up.y, -forward.y, 0.0),
            Vec4::new(side.z, adjusted_up.z, -forward.z, 0.0),
            Vec4::new(-eye.dot(side), -eye.dot(adjusted_up), eye.dot(forward), 1.0),
        );
        matrix.is_finite().then_some(matrix)
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
    pub fn try_perspective_rh_gl(
        fov_y_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Option<Self> {
        let valid = fov_y_radians.is_finite()
            && aspect_ratio.is_finite()
            && near.is_finite()
            && far.is_finite()
            && 0.0 < fov_y_radians
            && fov_y_radians < core::f32::consts::PI
            && aspect_ratio > 0.0
            && near > 0.0
            && near < far;
        if !valid {
            return None;
        }

        let matrix = Self::perspective_rh_gl(fov_y_radians, aspect_ratio, near, far);
        matrix.is_finite().then_some(matrix)
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
    pub fn try_orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Option<Self> {
        if !left.is_finite()
            || !right.is_finite()
            || !bottom.is_finite()
            || !top.is_finite()
            || !near.is_finite()
            || !far.is_finite()
            || left >= right
            || bottom >= top
            || near >= far
        {
            return None;
        }

        let matrix = Self::orthographic_rh_gl(left, right, bottom, top, near, far);
        matrix.is_finite().then_some(matrix)
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

    /// Returns the inverse using the retained unchecked comparison behavior.
    ///
    /// Invalid input produces an all-NaN matrix for parity with the historical
    /// A/C observation. This C1 corpus-only path recognizes the ordinary
    /// affine form used by the retained CAD and GLB callers, then otherwise
    /// falls back to the C0 Gauss--Jordan reference. External or recoverable
    /// input must use `try_inverse`.
    #[must_use]
    pub fn inverse(self) -> Self {
        if let Some(inverse) = self.inverse_affine() {
            return inverse;
        }
        self.inverse_gauss_jordan().unwrap_or_else(Self::nan)
    }

    /// Returns a finite inverse whose two multiplication orders satisfy the
    /// current corpus residual bound.
    #[must_use]
    pub fn try_inverse(self) -> Option<Self> {
        if !self.is_finite() {
            return None;
        }

        let inverse = self.inverse_gauss_jordan()?;
        (inverse.is_finite()
            && Self::is_identity_within_residual(self * inverse)
            && Self::is_identity_within_residual(inverse * self))
        .then_some(inverse)
    }

    fn inverse_gauss_jordan(self) -> Option<Self> {
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
                return None;
            }
            augmented.swap(pivot_column, pivot_row);

            for value in &mut augmented[pivot_column] {
                *value /= pivot;
                if !value.is_finite() {
                    return None;
                }
            }

            let pivot_values = augmented[pivot_column];
            for (row, values) in augmented.iter_mut().enumerate() {
                if row == pivot_column {
                    continue;
                }
                let factor = values[pivot_column];
                for (value, pivot_value) in values.iter_mut().zip(pivot_values) {
                    *value -= factor * pivot_value;
                    if !value.is_finite() {
                        return None;
                    }
                }
            }
        }

        let mut inverse = [0.0_f32; 16];
        for row in 0..4 {
            for column in 0..4 {
                inverse[column * 4 + row] = augmented[row][4 + column];
            }
        }
        Some(Self::from_cols_array(&inverse))
    }

    /// Returns the inverse for the exact affine matrix form without invoking
    /// the general C0 reference algorithm.
    ///
    /// This is intentionally private C1 machinery: it owns neither a new
    /// public matrix category nor a caller-visible promise that every matrix
    /// will use this route. The bottom row test is exact because it classifies
    /// a representation produced by the existing constructors, not arbitrary
    /// externally supplied near-affine data.
    fn inverse_affine(self) -> Option<Self> {
        let [x_axis, y_axis, z_axis, translation] = self.columns;
        if x_axis.w != 0.0 || y_axis.w != 0.0 || z_axis.w != 0.0 || translation.w != 1.0 {
            return None;
        }

        let a00 = x_axis.x;
        let a01 = y_axis.x;
        let a02 = z_axis.x;
        let a10 = x_axis.y;
        let a11 = y_axis.y;
        let a12 = z_axis.y;
        let a20 = x_axis.z;
        let a21 = y_axis.z;
        let a22 = z_axis.z;
        let determinant = a00 * (a11 * a22 - a12 * a21) - a01 * (a10 * a22 - a12 * a20)
            + a02 * (a10 * a21 - a11 * a20);
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let reciprocal = determinant.recip();
        let inverse = Self::from_columns(
            Vec4::new(
                (a11 * a22 - a12 * a21) * reciprocal,
                (a12 * a20 - a10 * a22) * reciprocal,
                (a10 * a21 - a11 * a20) * reciprocal,
                0.0,
            ),
            Vec4::new(
                (a02 * a21 - a01 * a22) * reciprocal,
                (a00 * a22 - a02 * a20) * reciprocal,
                (a01 * a20 - a00 * a21) * reciprocal,
                0.0,
            ),
            Vec4::new(
                (a01 * a12 - a02 * a11) * reciprocal,
                (a02 * a10 - a00 * a12) * reciprocal,
                (a00 * a11 - a01 * a10) * reciprocal,
                0.0,
            ),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );
        let translation = -(inverse.transform_vector3(translation.truncate()));
        let inverse = Self::from_columns(
            inverse.columns[0],
            inverse.columns[1],
            inverse.columns[2],
            translation.extend(1.0),
        );
        inverse.is_finite().then_some(inverse)
    }

    #[must_use]
    pub fn transform_point3(self, point: Vec3) -> Vec3 {
        (self.columns[0] * point.x
            + self.columns[1] * point.y
            + self.columns[2] * point.z
            + self.columns[3])
            .truncate()
    }

    /// Projects a point through homogeneous coordinates and performs a
    /// checked perspective divide.
    #[must_use]
    pub fn try_project_point3(self, point: Vec3) -> Option<Vec3> {
        if !point.is_finite() {
            return None;
        }

        let homogeneous = self.transform_vec4(point.extend(1.0));
        if !homogeneous.is_finite() || homogeneous.w == 0.0 {
            return None;
        }

        let projected = homogeneous.truncate() / homogeneous.w;
        projected.is_finite().then_some(projected)
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

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.columns.into_iter().all(Vec4::is_finite)
    }

    fn is_identity_within_residual(matrix: Self) -> bool {
        const RESIDUAL_LIMIT: f32 = 1.0e-3;

        matrix
            .to_cols_array()
            .into_iter()
            .zip(Self::IDENTITY.to_cols_array())
            .all(|(actual, expected)| {
                actual.is_finite() && (actual - expected).abs() <= RESIDUAL_LIMIT
            })
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

    fn scalar_near(actual: f32, expected: f32, absolute: f32, relative: f32) -> bool {
        (actual - expected).abs() <= absolute.max(relative * actual.abs().max(expected.abs()))
    }

    fn vec3_near(actual: Vec3, expected: Vec3, absolute: f32, relative: f32) -> bool {
        actual
            .to_array()
            .into_iter()
            .zip(expected.to_array())
            .all(|(actual, expected)| scalar_near(actual, expected, absolute, relative))
    }

    fn next_range(seed: &mut u32, minimum: f32, maximum: f32) -> f32 {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let unit = (*seed >> 8) as f32 / ((u32::MAX >> 8) as f32);
        minimum + (maximum - minimum) * unit
    }

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
    fn checked_normalization_distinguishes_zero_nonfinite_and_valid_input() {
        assert_eq!(Vec3::new(3.0, 4.0, 0.0).length(), 5.0);
        assert_eq!(
            Vec3::new(3.0, 4.0, 0.0)
                .try_normalize()
                .expect("finite nonzero vector")
                .to_array(),
            [0.6, 0.8, 0.0]
        );
        assert_eq!(Vec3::ZERO.try_normalize(), None);
        assert_eq!(Vec3::ZERO.try_normalize_or_zero(), Some(Vec3::ZERO));
        assert_eq!(Vec3::new(f32::NAN, 0.0, 0.0).try_normalize_or_zero(), None);
        assert_eq!(Vec3::new(f32::INFINITY, 0.0, 0.0).try_normalize(), None);
        assert_eq!(Vec3::new(f32::MAX, f32::MAX, 0.0).try_normalize(), None);
    }

    #[test]
    fn post_doom_axis_values_preserve_raw_basis_mechanics() {
        assert_eq!(Vec3::X.cross(Vec3::Y), Vec3::Z);
        assert_eq!(-Vec3::X, Vec3::NEG_X);
        assert_eq!(-Vec3::Y, Vec3::NEG_Y);
        assert_eq!(-Vec3::Z, Vec3::NEG_Z);
    }

    #[test]
    fn raw_ieee_arithmetic_and_ordered_accumulation_remain_explicit() {
        assert!((Vec3::ONE / 0.0)
            .to_array()
            .into_iter()
            .all(f32::is_infinite));
        assert!((Vec3::ZERO / 0.0).to_array().into_iter().all(f32::is_nan));

        let first_order = Vec3::sum_ordered([
            Vec3::new(1.0e20, 0.0, 0.0),
            Vec3::new(-1.0e20, 0.0, 0.0),
            Vec3::X,
        ]);
        let second_order = Vec3::sum_ordered([
            Vec3::new(1.0e20, 0.0, 0.0),
            Vec3::X,
            Vec3::new(-1.0e20, 0.0, 0.0),
        ]);
        assert_eq!(first_order, Vec3::X);
        assert_eq!(second_order, Vec3::ZERO);
        assert_eq!(Vec3::sum_ordered([]), Vec3::ZERO);
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

    #[test]
    fn checked_matrix_paths_reject_invalid_external_values() {
        let eye = Vec3::new(0.0, 0.0, 5.0);
        assert!(Mat4::try_look_at_rh(eye, Vec3::ZERO, Vec3::Y).is_some());
        assert_eq!(Mat4::try_look_at_rh(Vec3::ZERO, Vec3::ZERO, Vec3::Y), None);
        assert_eq!(Mat4::try_look_at_rh(eye, Vec3::ZERO, Vec3::Z), None);
        assert_eq!(
            Mat4::try_look_at_rh(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::ZERO, Vec3::Y),
            None
        );

        assert!(Mat4::try_perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0).is_some());
        assert_eq!(Mat4::try_perspective_rh_gl(0.0, 1.0, 0.1, 100.0), None);
        assert_eq!(Mat4::try_perspective_rh_gl(1.0, 0.0, 0.1, 100.0), None);
        assert_eq!(Mat4::try_perspective_rh_gl(1.0, 1.0, 1.0, 1.0), None);
        assert_eq!(Mat4::try_perspective_rh_gl(f32::NAN, 1.0, 0.1, 100.0), None);

        assert!(Mat4::try_orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0).is_some());
        assert_eq!(
            Mat4::try_orthographic_rh_gl(1.0, -1.0, -1.0, 1.0, 0.1, 100.0),
            None
        );
        assert_eq!(
            Mat4::try_orthographic_rh_gl(-1.0, 1.0, -1.0, 1.0, 1.0, 1.0),
            None
        );

        assert!(Mat4::from_translation(Vec3::new(4.0, 5.0, 6.0))
            .try_inverse()
            .is_some());
        assert_eq!(
            Mat4::from_scale(Vec3::new(1.0, 0.0, 1.0)).try_inverse(),
            None
        );
        assert_eq!(Mat4::from_cols_array(&[f32::NAN; 16]).try_inverse(), None);
    }

    #[test]
    fn explicit_column_and_projection_boundaries_are_checked() {
        let columns = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        assert_eq!(
            Mat4::from_cols_array_2d(&columns).to_cols_array_2d(),
            columns
        );

        let projection =
            Mat4::try_perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0).expect("valid projection");
        let projected = projection
            .try_project_point3(Vec3::new(0.0, 0.0, -1.0))
            .expect("point in front of the camera");
        assert!(projected.is_finite());
        assert_eq!(projection.try_project_point3(Vec3::ZERO), None);
        assert_eq!(
            projection.try_project_point3(Vec3::new(f32::INFINITY, 0.0, -1.0)),
            None
        );
    }

    #[test]
    fn fixed_seed_vector_and_affine_properties_hold_with_selected_tolerances() {
        let mut seed = 0xC001_C0DE;
        for _ in 0..128 {
            let vector = Vec3::new(
                next_range(&mut seed, -1000.0, 1000.0),
                next_range(&mut seed, -1000.0, 1000.0),
                next_range(&mut seed, -1000.0, 1000.0),
            );
            let other = Vec3::new(
                next_range(&mut seed, -1000.0, 1000.0),
                next_range(&mut seed, -1000.0, 1000.0),
                next_range(&mut seed, -1000.0, 1000.0),
            );

            let normalized = vector.try_normalize().expect("generated nonzero vector");
            assert!(scalar_near(normalized.length(), 1.0, 1.0e-6, 2.0e-6));
            assert!(scalar_near(
                vector.dot(other),
                other.dot(vector),
                1.0e-6,
                2.0e-6
            ));
            let normalized_other = other.try_normalize().expect("generated nonzero vector");
            if let Some(normalized_cross) = vector.cross(other).try_normalize() {
                assert!(normalized_cross.dot(normalized).abs() <= 2.0e-6);
                assert!(normalized_cross.dot(normalized_other).abs() <= 2.0e-6);
            }

            let translation = Vec3::new(
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
            );
            let scale = Vec3::new(
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
            );
            let transform = Mat4::from_translation(translation)
                * Mat4::from_rotation_y(next_range(&mut seed, -3.0, 3.0))
                * Mat4::from_scale(scale);
            let inverse = transform
                .try_inverse()
                .expect("conditioned affine transform");
            let transformed = transform.transform_point3(vector);
            assert!(vec3_near(
                inverse.transform_point3(transformed),
                vector,
                1.0e-3,
                1.0e-5
            ));
        }
    }

    #[test]
    fn checked_inverse_retains_a_bounded_conditioning_observation() {
        // This does not claim a general condition-number policy. It records a
        // small, reproducible set of finite affine transforms around the
        // current corpus scale range, including an intentionally severe case.
        let cases = [
            (Vec3::new(0.25, 1.0, 1.0), true),
            (Vec3::new(1.0e-2, 1.0, 1.0), true),
            (Vec3::new(1.0e-3, 1.0, 1.0), false),
            (Vec3::new(1.0e-4, 1.0, 1.0), false),
            (Vec3::new(1.0e-6, 2.0, 0.5), false),
            (Vec3::new(1.0e-8, 3.0, 0.25), false),
            (Vec3::new(1.0e-10, 10.0, 0.1), false),
        ];

        for (scale, expected_acceptance) in cases {
            let transform = Mat4::from_translation(Vec3::new(10.0, -20.0, 30.0))
                * Mat4::from_rotation_y(0.73)
                * Mat4::from_rotation_x(-0.41)
                * Mat4::from_scale(scale);
            assert_eq!(
                transform.try_inverse().is_some(),
                expected_acceptance,
                "the current 1e-3 residual policy changed for scale={:?}",
                scale.to_array(),
            );
        }
    }

    #[test]
    fn c1_affine_inverse_matches_the_retained_scalar_reference() {
        let mut seed = 0xC1A0_0001_u32;
        for _ in 0..128 {
            let translation = Vec3::new(
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
            );
            let scale = Vec3::new(
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
            );
            let affine = Mat4::from_translation(translation)
                * Mat4::from_rotation_z(next_range(&mut seed, -3.0, 3.0))
                * Mat4::from_rotation_y(next_range(&mut seed, -3.0, 3.0))
                * Mat4::from_scale(scale);
            let c1 = affine.inverse();
            let reference = affine
                .inverse_gauss_jordan()
                .expect("conditioned affine inverse");
            assert!(
                c1.to_cols_array()
                    .into_iter()
                    .zip(reference.to_cols_array())
                    .all(|(actual, expected)| (actual - expected).abs() <= 1.0e-4),
                "C1 affine inverse diverged from scalar reference"
            );
            assert!(Mat4::is_identity_within_residual(affine * c1));
            assert!(Mat4::is_identity_within_residual(c1 * affine));
        }
    }

    #[test]
    fn c1_inverse_keeps_non_affine_matrices_on_the_scalar_reference_path() {
        let projection = Mat4::perspective_rh_gl(1.1, 1.6, 0.1, 100.0);
        let c1 = projection.inverse();
        let reference = projection
            .inverse_gauss_jordan()
            .expect("bounded projection inverse");
        assert_eq!(c1.to_cols_array(), reference.to_cols_array());
    }
}
