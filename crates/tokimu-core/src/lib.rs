pub mod component;
pub mod diagnostics;
pub mod entity;
pub mod event;
pub mod math;
pub mod relation;
pub mod resource;
pub mod scene;
pub mod schedule;
pub mod signal_log;
pub mod system;
pub mod time;
pub mod world;

pub use component::Component;
pub use diagnostics::Diagnostics;
pub use entity::EntityId;
pub use event::Event;
pub use relation::Relation;
pub use resource::Resource;
pub use scene::{
    compile_scene, SceneChange, SceneDoc, SceneEntityDoc, SceneHistoryRecord, SceneParent,
    ScenePosition,
};
pub use schedule::{Phase, Schedule};
pub use signal_log::SignalLog;
pub use system::{RegisteredSystem, System, SystemRegistry};
pub use time::{FixedTimeStep, TimeStepPolicy};
pub use world::World;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameOutcome {
    Continue,
    Exit,
}

#[cfg(test)]
mod tests {
    use crate::World;

    #[test]
    fn world_spawns_distinct_entities() {
        let mut world = World::default();
        let first = world.spawn();
        let second = world.spawn();

        assert_ne!(first, second);
    }
}
