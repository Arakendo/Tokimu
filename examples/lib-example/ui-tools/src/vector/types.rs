#[derive(Clone, Debug, PartialEq)]
pub struct VectorPath {
    pub contours: Vec<VectorContour>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorContour {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

impl VectorPath {
    pub fn new(contours: Vec<VectorContour>) -> Self {
        Self { contours }
    }

    pub fn is_finite(&self) -> bool {
        self.contours.iter().all(VectorContour::is_finite)
    }

    pub fn bounds(&self) -> Option<([f32; 2], [f32; 2])> {
        let mut points = self
            .contours
            .iter()
            .flat_map(|contour| contour.points.iter());
        let first = *points.next()?;
        let mut min = first;
        let mut max = first;

        for point in points {
            min[0] = min[0].min(point[0]);
            min[1] = min[1].min(point[1]);
            max[0] = max[0].max(point[0]);
            max[1] = max[1].max(point[1]);
        }

        Some((min, max))
    }
}

impl VectorContour {
    pub fn new(points: Vec<[f32; 2]>, closed: bool) -> Self {
        Self { points, closed }
    }

    pub fn is_finite(&self) -> bool {
        self.points
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite())
    }
}
