use tokimu_core::math::Mat4;

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
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
        }
    }
}
