use crate::{plugin::Plugin, RuntimeConfig};
use tokimu_core::{Diagnostics, Schedule, World};

#[derive(Debug, Default)]
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub diagnostics: Diagnostics,
    pub config: RuntimeConfig,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P>(&mut self, plugin: &P) -> &mut Self
    where
        P: Plugin,
    {
        plugin.build(self);
        self
    }
}
