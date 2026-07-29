use std::f32::consts::TAU;

use super::{VectorContour, VectorPath};

#[derive(Clone, Debug, Default)]
pub struct PathBuilder {
    contours: Vec<VectorContour>,
    current: Vec<[f32; 2]>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(mut self, point: [f32; 2]) -> Self {
        self.finish_open_contour();
        self.current.push(point);
        self
    }

    pub fn line_to(mut self, point: [f32; 2]) -> Self {
        self.current.push(point);
        self
    }

    pub fn close(mut self) -> Self {
        if !self.current.is_empty() {
            self.contours
                .push(VectorContour::new(std::mem::take(&mut self.current), true));
        }
        self
    }

    pub fn rect(mut self, min: [f32; 2], size: [f32; 2]) -> Self {
        self.finish_open_contour();
        let max = [min[0] + size[0], min[1] + size[1]];
        self.contours.push(VectorContour::new(
            vec![min, [max[0], min[1]], max, [min[0], max[1]]],
            true,
        ));
        self
    }

    pub fn rounded_rect(mut self, min: [f32; 2], size: [f32; 2], radius: f32) -> Self {
        self.finish_open_contour();
        let max = [min[0] + size[0], min[1] + size[1]];
        let radius = radius
            .max(0.0)
            .min(size[0].abs() * 0.5)
            .min(size[1].abs() * 0.5);
        let segments = 8;
        let centers = [
            [max[0] - radius, min[1] + radius],
            [max[0] - radius, max[1] - radius],
            [min[0] + radius, max[1] - radius],
            [min[0] + radius, min[1] + radius],
        ];
        let start_angles = [-TAU / 4.0, 0.0, TAU / 4.0, TAU / 2.0];
        let mut points = Vec::with_capacity(segments * 4);

        for (center, start) in centers.into_iter().zip(start_angles) {
            for step in 0..segments {
                let angle = start + (step as f32 / segments as f32) * TAU / 4.0;
                points.push([
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                ]);
            }
        }

        self.contours.push(VectorContour::new(points, true));
        self
    }

    pub fn build(mut self) -> VectorPath {
        self.finish_open_contour();
        VectorPath::new(self.contours)
    }

    fn finish_open_contour(&mut self) {
        if !self.current.is_empty() {
            self.contours
                .push(VectorContour::new(std::mem::take(&mut self.current), false));
        }
    }
}
