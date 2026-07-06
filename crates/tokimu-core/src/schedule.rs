#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Startup,
    PreUpdate,
    FixedUpdate,
    Update,
    PostUpdate,
    RenderPrepare,
    Render,
    PostRender,
    Shutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schedule {
    phases: Vec<Phase>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            phases: vec![
                Phase::Startup,
                Phase::PreUpdate,
                Phase::FixedUpdate,
                Phase::Update,
                Phase::PostUpdate,
                Phase::RenderPrepare,
                Phase::Render,
                Phase::PostRender,
                Phase::Shutdown,
            ],
        }
    }
}

impl Schedule {
    pub fn phases(&self) -> &[Phase] {
        &self.phases
    }
}
