//! Alternative D's bounded upstream-derived `Vec3` experiment.
//!
//! Derived from the audited scalar `glam` vector source at
//! `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7`; see
//! `alternative-d-bounded-fork/UPSTREAM-NOTICE.md`. This narrow adaptation has
//! no provider dependency, generated code, or SIMD implementation.

use core::ops::{Add, Div, Mul, Neg, Sub};

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
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
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
    fn bounded_fork_slice_stays_narrow_and_scalar() {
        assert_eq!((Vec3::ONE + Vec3::Y).to_array(), [1.0, 2.0, 1.0]);
        assert_eq!(Vec3::splat(2.0).to_array(), [2.0, 2.0, 2.0]);
        assert_eq!(
            Vec3::from_array([1.0, 2.0, 3.0]).to_array(),
            [1.0, 2.0, 3.0]
        );
        assert_eq!(
            (Vec3::new(2.0, 3.0, 4.0) * Vec3::new(5.0, 6.0, 7.0)).to_array(),
            [10.0, 18.0, 28.0]
        );
        assert_eq!(Vec3::ZERO.normalize_or_zero(), Vec3::ZERO);
    }
}
