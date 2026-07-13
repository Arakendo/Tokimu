#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<[f32; 2]>,
}

impl Mesh {
    pub fn new(positions: Vec<[f32; 2]>) -> Self {
        Self { positions }
    }

    pub fn triangle() -> Self {
        Self::new(vec![[0.0, 0.6], [-0.6, -0.6], [0.6, -0.6]])
    }

    pub fn quad() -> Self {
        Self::new(vec![
            [-0.5, 0.5],
            [-0.5, -0.5],
            [0.5, -0.5],
            [-0.5, 0.5],
            [0.5, -0.5],
            [0.5, 0.5],
        ])
    }

    pub fn diamond() -> Self {
        Self::new(vec![
            [0.0, 0.6],
            [-0.55, 0.0],
            [0.0, -0.6],
            [0.0, 0.6],
            [0.0, -0.6],
            [0.55, 0.0],
        ])
    }

    pub fn vertex_count(&self) -> u32 {
        self.positions.len() as u32
    }
}
