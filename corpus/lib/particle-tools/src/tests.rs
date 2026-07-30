use crate::{
    lower_particle_instances_2d, ParticleEmitter2d, ParticleError, ParticlePresentationRole,
    ParticleSpawn2d, ParticleSystem2d, ParticleSystemConfig, ParticleVec2, ParticleView2d,
    ScalarRange,
};

fn test_config(capacity: usize) -> ParticleSystemConfig {
    ParticleSystemConfig {
        capacity,
        maximum_burst: 16,
        maximum_lifetime: 4.0,
        maximum_step_seconds: 0.25,
    }
}

fn varied_burst(count: usize) -> ParticleSpawn2d {
    ParticleSpawn2d {
        count,
        origin: ParticleVec2::new(2.0, -1.0),
        inherited_velocity: ParticleVec2::new(0.5, 0.25),
        direction_radians: ScalarRange::new(-0.5, 0.5, "direction").unwrap(),
        speed: ScalarRange::new(2.0, 5.0, "speed").unwrap(),
        lifetime: ScalarRange::new(0.5, 1.5, "lifetime").unwrap(),
        initial_size: ScalarRange::new(0.2, 0.6, "initial_size").unwrap(),
        final_size: ScalarRange::constant(0.0),
        initial_rotation: ScalarRange::new(-1.0, 1.0, "rotation").unwrap(),
        angular_velocity: ScalarRange::new(-2.0, 2.0, "angular_velocity").unwrap(),
        acceleration: ParticleVec2::new(0.0, -1.0),
        drag: 0.4,
        presentation_role: ParticlePresentationRole(7),
    }
}

#[test]
fn equal_seed_and_steps_produce_byte_equivalent_state() {
    let mut first = ParticleSystem2d::new(test_config(32), 42).unwrap();
    let mut second = ParticleSystem2d::new(test_config(32), 42).unwrap();

    first.spawn(varied_burst(12)).unwrap();
    second.spawn(varied_burst(12)).unwrap();
    for _ in 0..5 {
        first.step(0.125).unwrap();
        second.step(0.125).unwrap();
    }

    assert_eq!(
        serde_json::to_vec(&first.snapshot()).unwrap(),
        serde_json::to_vec(&second.snapshot()).unwrap()
    );
}

#[test]
fn expiration_preserves_survivor_order() {
    let mut system = ParticleSystem2d::new(test_config(8), 9).unwrap();
    let mut short = varied_burst(1);
    short.lifetime = ScalarRange::constant(0.1);
    let mut long = varied_burst(2);
    long.lifetime = ScalarRange::constant(1.0);

    system.spawn(short).unwrap();
    system.spawn(long).unwrap();
    let report = system.step(0.25).unwrap();

    assert_eq!(report.expired, 1);
    assert_eq!(
        system
            .particles()
            .iter()
            .map(|particle| particle.id.0)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
}

#[test]
fn capacity_drops_newest_without_consuming_identity() {
    let mut system = ParticleSystem2d::new(test_config(3), 17).unwrap();
    let first = system.spawn(varied_burst(2)).unwrap();
    let second = system.spawn(varied_burst(3)).unwrap();

    assert_eq!((first.spawned, first.dropped), (2, 0));
    assert_eq!((second.spawned, second.dropped), (1, 2));
    assert_eq!(system.dropped_total(), 2);
    assert_eq!(
        system
            .particles()
            .iter()
            .map(|particle| particle.id.0)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    system.reset(17);
    assert_eq!(system.spawn(varied_burst(1)).unwrap().spawned, 1);
    assert_eq!(system.particles()[0].id.0, 1);
}

#[test]
fn invalid_request_leaves_state_unchanged() {
    let mut system = ParticleSystem2d::new(test_config(8), 21).unwrap();
    system.spawn(varied_burst(2)).unwrap();
    let mut reference = system.clone();
    let before = serde_json::to_vec(&system.snapshot()).unwrap();

    let mut invalid = varied_burst(1);
    invalid.drag = f32::NAN;
    assert_eq!(
        system.spawn(invalid),
        Err(ParticleError::NonFinite { field: "drag" })
    );

    assert_eq!(serde_json::to_vec(&system.snapshot()).unwrap(), before);
    system.spawn(varied_burst(1)).unwrap();
    reference.spawn(varied_burst(1)).unwrap();
    assert_eq!(
        serde_json::to_vec(&system.snapshot()).unwrap(),
        serde_json::to_vec(&reference.snapshot()).unwrap()
    );
}

#[test]
fn oversized_step_is_rejected_without_mutation() {
    let mut system = ParticleSystem2d::new(test_config(8), 3).unwrap();
    system.spawn(varied_burst(2)).unwrap();
    let before = serde_json::to_vec(&system.snapshot()).unwrap();

    assert!(matches!(
        system.step(0.5),
        Err(ParticleError::StepTooLarge { .. })
    ));
    assert_eq!(serde_json::to_vec(&system.snapshot()).unwrap(), before);
}

#[test]
fn fixed_rate_emission_is_stable_across_step_partitions() {
    let mut one_step_system = ParticleSystem2d::new(test_config(32), 14).unwrap();
    let mut split_system = ParticleSystem2d::new(test_config(32), 14).unwrap();
    let mut template = varied_burst(0);
    template.lifetime = ScalarRange::constant(2.0);
    let mut one_step = ParticleEmitter2d::new(template, 20.0).unwrap();
    let mut split = ParticleEmitter2d::new(template, 20.0).unwrap();

    one_step.emit(&mut one_step_system, 0.25).unwrap();
    for _ in 0..5 {
        split.emit(&mut split_system, 0.05).unwrap();
    }

    assert_eq!(
        serde_json::to_vec(one_step_system.particles()).unwrap(),
        serde_json::to_vec(split_system.particles()).unwrap()
    );
}

#[test]
fn disabled_emitter_does_not_consume_time_or_randomness() {
    let mut disabled_system = ParticleSystem2d::new(test_config(32), 14).unwrap();
    let mut reference_system = ParticleSystem2d::new(test_config(32), 14).unwrap();
    let template = varied_burst(0);
    let mut disabled = ParticleEmitter2d::new(template, 8.0).unwrap();
    let mut reference = ParticleEmitter2d::new(template, 8.0).unwrap();

    disabled.set_enabled(false);
    disabled.emit(&mut disabled_system, 0.25).unwrap();
    disabled.set_enabled(true);
    disabled.emit(&mut disabled_system, 0.25).unwrap();
    reference.emit(&mut reference_system, 0.25).unwrap();

    assert_eq!(
        serde_json::to_vec(disabled_system.particles()).unwrap(),
        serde_json::to_vec(reference_system.particles()).unwrap()
    );
}

#[test]
fn instance_lowering_is_ordered_bounded_and_provider_neutral() {
    let mut system = ParticleSystem2d::new(test_config(8), 31).unwrap();
    let mut request = varied_burst(3);
    request.origin = ParticleVec2::ZERO;
    request.speed = ScalarRange::constant(0.0);
    request.initial_size = ScalarRange::constant(0.25);
    request.final_size = ScalarRange::constant(0.125);
    system.spawn(request).unwrap();
    system.step(0.25).unwrap();

    let view =
        ParticleView2d::new(ParticleVec2::new(-1.0, -1.0), ParticleVec2::new(1.0, 1.0)).unwrap();
    let batch = lower_particle_instances_2d(system.particles(), view, 2);

    assert_eq!(batch.report.considered, 3);
    assert_eq!(batch.report.visible, 2);
    assert_eq!(batch.report.outside_view, 0);
    assert_eq!(batch.report.omitted_by_limit, 1);
    assert_eq!(
        batch
            .instances
            .iter()
            .map(|instance| instance.id.0)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(batch.instances.iter().all(|instance| {
        instance.presentation_role == ParticlePresentationRole(7)
            && instance.normalized_age > 0.0
            && instance.normalized_age < 1.0
    }));
}

#[test]
fn instance_lowering_reports_particles_outside_the_view() {
    let mut system = ParticleSystem2d::new(test_config(8), 7).unwrap();
    let mut request = varied_burst(2);
    request.origin = ParticleVec2::new(5.0, 5.0);
    request.speed = ScalarRange::constant(0.0);
    request.initial_size = ScalarRange::constant(0.1);
    system.spawn(request).unwrap();

    let view =
        ParticleView2d::new(ParticleVec2::new(-1.0, -1.0), ParticleVec2::new(1.0, 1.0)).unwrap();
    let batch = lower_particle_instances_2d(system.particles(), view, 8);

    assert!(batch.instances.is_empty());
    assert_eq!(batch.report.outside_view, 2);
}
