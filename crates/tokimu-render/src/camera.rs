use tokimu_core::math::{Mat4, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub view: Mat4,
    pub projection: Mat4,
}

impl Camera {
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        Self { view, projection }
    }

    pub fn orthographic_2d(width: f32, height: f32) -> Self {
        Self::orthographic_2d_with_height(width, height, 2.0)
    }

    pub fn orthographic_2d_with_height(width: f32, height: f32, world_height: f32) -> Self {
        let mut camera = Self::default();
        camera.set_orthographic_2d_with_height(width, height, world_height);
        camera
    }

    pub fn set_orthographic_2d(&mut self, width: f32, height: f32) {
        self.set_orthographic_2d_with_height(width, height, 2.0);
    }

    pub fn set_orthographic_2d_with_height(
        &mut self,
        width: f32,
        height: f32,
        world_height: f32,
    ) {
        let aspect_ratio = if height > 0.0 { width / height } else { 1.0 };
        let half_height = world_height * 0.5;
        let half_width = half_height * aspect_ratio;

        self.view = Mat4::IDENTITY;
        self.projection = Mat4::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -1.0,
            1.0,
        );
    }

    pub fn perspective_3d(width: f32, height: f32) -> Self {
        let mut camera = Self::default();
        camera.set_perspective_3d(width, height);
        camera
    }

    pub fn set_perspective_3d(&mut self, width: f32, height: f32) {
        let aspect_ratio = if height > 0.0 { width / height } else { 1.0 };
        let eye = Vec3::new(0.0, 0.0, 3.0);
        let target = Vec3::ZERO;
        let up = Vec3::Y;

        self.view = Mat4::look_at_rh(eye, target, up);
        self.projection = Mat4::perspective_rh_gl(60.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_perspective_camera() {
        let camera = Camera::perspective_3d(1280.0, 720.0);

        assert_ne!(camera.view, Mat4::IDENTITY);
        assert_ne!(camera.projection, Mat4::IDENTITY);
    }
}
