use tokimu::{App, World};

#[test]
fn facade_reexports_basic_engine_types() {
    let app = App::new();
    let mut world = World::default();

    assert_eq!(app.schedule.phases().len(), 9);
    assert_eq!(world.spawn().0, 0);
}
