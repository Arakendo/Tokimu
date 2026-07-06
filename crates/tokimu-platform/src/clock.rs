#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clock {
    pub delta_seconds: f64,
    pub elapsed_seconds: f64,
}
