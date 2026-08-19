//! Headless application-lifecycle replay evidence for moving Doom geometry.

use super::super::*;

pub(crate) fn report_gameplay_snapshot_replay(app: &mut App) -> PlatformResult<()> {
    let source_before = app
        .thing_sprites
        .iter()
        .map(|thing| {
            (
                thing.source,
                thing.kind,
                thing.source_position,
                thing.source_angle,
                thing.floor_height,
                thing.source_sector,
            )
        })
        .collect::<Vec<_>>();
    let baseline = app.capture_gameplay_snapshot();
    let (first_damage, first_target) = apply_bounded_gameplay_script(app)?;
    let expected = app.capture_gameplay_snapshot();
    if expected == baseline {
        return Err("bounded gameplay script did not change snapshot state".into());
    }
    app.restore_gameplay_snapshot(&baseline);
    if app.capture_gameplay_snapshot() != baseline {
        return Err("gameplay snapshot did not restore the baseline exactly".into());
    }
    let (replay_damage, replay_target) = apply_bounded_gameplay_script(app)?;
    let replay = app.capture_gameplay_snapshot();
    if replay != expected || replay_damage != first_damage || replay_target != first_target {
        return Err("gameplay snapshot replay diverged after baseline restoration".into());
    }
    let source_after = app
        .thing_sprites
        .iter()
        .map(|thing| {
            (
                thing.source,
                thing.kind,
                thing.source_position,
                thing.source_angle,
                thing.floor_height,
                thing.source_sector,
            )
        })
        .collect::<Vec<_>>();
    if source_after != source_before {
        return Err("gameplay snapshot replay mutated imported Thing data".into());
    }
    let awake = replay
        .monster_runtime_states
        .iter()
        .flatten()
        .filter(|monster| monster.awake)
        .count();
    let inactive = replay
        .thing_sprite_active
        .iter()
        .filter(|active| !**active)
        .count();
    app.restore_gameplay_snapshot(&baseline);
    println!(
        "E1M1 Slice 9 gameplay snapshot replay: mutable-payload=[player-inventory,thing-active,thing-state-clocks,combat-health,play-random,monster-runtime-poses]; first-damage={first_damage}; target-thing={first_target}; replay-identical=true; restored-baseline=true; replay-awake-monsters={awake}; replay-inactive-things={inactive}; imported-things-mutated=false; wad-bytes-owned=false; renderer-resources-owned=false; persistence-format=none; renderer-initialized=false"
    );
    Ok(())
}

fn apply_bounded_gameplay_script(app: &mut App) -> PlatformResult<(i32, u32)> {
    app.player_inventory.ammo[0] = app.player_inventory.ammo[0].saturating_sub(1);
    let damage = app.play_random.pistol_damage();
    let target = app
        .thing_combat_states
        .iter_mut()
        .flatten()
        .find(|state| state.kind == 3004)
        .ok_or("E1M1 snapshot replay requires a source zombieman")?;
    target.apply_damage(damage);
    let target_source = target.source_thing;
    app.player_inventory.apply_damage(7);
    for (state, thing) in app.thing_sprite_states.iter_mut().zip(&app.thing_sprites) {
        state.advance(thing.initial_frame, 17);
    }
    app.thing_sprite_total_ticks = app.thing_sprite_total_ticks.saturating_add(17);
    if let Some((index, _)) = app
        .thing_combat_states
        .iter()
        .enumerate()
        .find(|(_, state)| state.is_none())
    {
        app.thing_sprite_active[index] = false;
    }
    let monster = app
        .monster_runtime_states
        .iter_mut()
        .flatten()
        .next()
        .ok_or("E1M1 snapshot replay requires a monster runtime record")?;
    monster.awake = true;
    monster.source_position[0] += 8.0;
    monster.source_angle_degrees = 0.0;
    monster.chase_state_index = 1;
    Ok((damage, target_source))
}

/// Replays E1M1's two moving-floor lifetimes through the application-owned
/// presentation seam without initializing a renderer.
pub(crate) fn report_moving_floor_resource_replay(app: &mut App) -> PlatformResult<()> {
    let (turbo_line_source, turbo_tag) = app
        .activation_source
        .linedefs
        .iter()
        .find(|line| line.special == 36)
        .map(|line| (line.source, line.tag))
        .ok_or("E1M1 contains no code-36 turbo floor")?;
    let mut turbo = DoomTurboLowerFloorRuntime::start_tagged(
        &app.activation_source,
        turbo_tag,
        DoomTurboLowerFloorPolicy::CLASSIC,
    )
    .map_err(|error| io::Error::other(format!("turbo-floor replay start failed: {error:?}")))?
    .into_iter()
    .next()
    .ok_or("E1M1 code-36 tag selected no sector")?;
    let turbo_start = turbo.current_floor_height;
    let turbo_destination = turbo.destination_floor_height;
    if let Some(observer) = app.spawn_observer.as_mut() {
        observer.sector = turbo.target_sector.record_index;
        observer.floor = turbo_start;
        observer.position.y = f32::from(turbo_start) + 36.0;
    }
    app.active_turbo_floors.push(turbo);
    let mut turbo_ticks = 0_u32;
    while app.active_turbo_floors[0].phase != DoomTurboLowerFloorPhase::Complete
        && turbo_ticks < 4_096
    {
        app.advance_active_moving_floors(DOOM_TIC_SECONDS * 1.001);
        turbo_ticks += 1;
    }
    turbo = app.active_turbo_floors[0];
    let turbo_floor_vertices = sector_flat_vertices_at_height(
        &app.draws,
        turbo.target_sector,
        doom_geometry_provider::DoomSurfacePlane::Floor,
        turbo_destination,
    );
    let turbo_observer_carried = app.spawn_observer.is_some_and(|observer| {
        observer.sector == turbo.target_sector.record_index
            && observer.floor == turbo_destination
            && (observer.position.y - (f32::from(turbo_destination) + 36.0)).abs() <= f32::EPSILON
    });
    app.active_turbo_floors.clear();

    let (platform_line_source, platform_tag) = app
        .activation_source
        .linedefs
        .iter()
        .find(|line| line.special == 88)
        .map(|line| (line.source, line.tag))
        .ok_or("E1M1 contains no code-88 down-wait-up-stay platform")?;
    let mut platform = DoomDownWaitUpStayRuntime::start_tagged(
        &app.activation_source,
        platform_tag,
        DoomDownWaitUpStayPolicy::CLASSIC,
    )
    .map_err(|error| io::Error::other(format!("platform replay start failed: {error:?}")))?
    .into_iter()
    .next()
    .ok_or("E1M1 code-88 tag selected no sector")?;
    if let Some(observer) = app.spawn_observer.as_mut() {
        observer.sector = platform.target_sector.record_index;
        observer.floor = platform.high_floor_height;
        observer.position.y = f32::from(platform.high_floor_height) + 36.0;
    }
    app.active_down_wait_up_platforms.push(platform);
    let mut platform_ticks = 0_u32;
    while app.active_down_wait_up_platforms[0].phase != DoomDownWaitUpStayPhase::Complete
        && platform_ticks < 4_096
    {
        app.advance_active_moving_floors(DOOM_TIC_SECONDS * 1.001);
        platform_ticks += 1;
    }
    platform = app.active_down_wait_up_platforms[0];
    let platform_floor_vertices = sector_flat_vertices_at_height(
        &app.draws,
        platform.target_sector,
        doom_geometry_provider::DoomSurfacePlane::Floor,
        platform.high_floor_height,
    );
    let platform_observer_carried = app.spawn_observer.is_some_and(|observer| {
        observer.sector == platform.target_sector.record_index
            && observer.floor == platform.high_floor_height
            && (observer.position.y - (f32::from(platform.high_floor_height) + 36.0)).abs()
                <= f32::EPSILON
    });
    let visual_diagnostic = app.door_visual_diagnostic.as_deref().unwrap_or("none");

    println!(
        "E1M1 Slice 8 moving-floor resource replay: turbo-line={}; turbo-sector={}; turbo={turbo_start}->{turbo_destination}; turbo-ticks={turbo_ticks}; turbo-final={:?}; turbo-floor-vertices={turbo_floor_vertices}; turbo-observer-carried={turbo_observer_carried}; platform-line={}; platform-sector={}; platform-high={}; platform-low={}; platform-ticks={platform_ticks}; platform-final={:?}; platform-floor-vertices={platform_floor_vertices}; platform-observer-carried={platform_observer_carried}; dynamic-wall-draws={}; dynamic-wall-handles={}; dirty-meshes={}; visual-diagnostic={visual_diagnostic}; source-map-mutated=false; renderer-initialized=false",
        turbo_line_source.record_index,
        turbo.target_sector.record_index,
        turbo.phase,
        platform_line_source.record_index,
        platform.target_sector.record_index,
        platform.high_floor_height,
        platform.low_floor_height,
        platform.phase,
        app.dynamic_door_draws.len(),
        app.dynamic_door_mesh_handles.len(),
        app.dirty_opaque_meshes.len(),
    );
    Ok(())
}

fn sector_flat_vertices_at_height(
    draws: &[StaticDrawPlanEntry],
    target_sector: doom_map_provider::DoomSourceRecord,
    target_plane: doom_geometry_provider::DoomSurfacePlane,
    height: i16,
) -> usize {
    let height = f32::from(height);
    draws
        .iter()
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Flat {
                    source_sector,
                    plane,
                    ..
                } if source_sector == target_sector && plane == target_plane
            )
        })
        .flat_map(|draw| &draw.mesh.positions)
        .filter(|position| (position[1] - height).abs() <= f32::EPSILON)
        .count()
}

/// Replays the dynamic-resource lifetime that exposed the E1M1 handle
/// collision without initializing a renderer.
pub(crate) fn report_door_resource_replay(app: &mut App) -> PlatformResult<()> {
    let Some(source_linedef) = app
        .activation_source
        .linedefs
        .iter()
        .find(|linedef| linedef.special == 1)
        .map(|linedef| linedef.source)
    else {
        return Err("E1M1 contains no code-1 manual door for resource replay".into());
    };
    let DoomLineActivationResolution::Accepted {
        intent: DoomLineActivationIntent::RaiseDoor { target_sector },
        ..
    } = resolve_doom_line_activation(
        &app.activation_source,
        DoomLineActivationRequest {
            source_linedef,
            activation: DoomLineActivation::Use,
        },
    )
    else {
        return Err("E1M1 code-1 manual door did not resolve for resource replay".into());
    };

    let mut door = DoomManualDoorRuntime::start(
        &app.activation_source,
        target_sector,
        DoomManualDoorPolicy::CLASSIC_NORMAL,
    )
    .map_err(|error| io::Error::other(format!("manual-door replay start failed: {error:?}")))?;
    let closed_height = door.closed_ceiling_height;
    let open_height = door.open_ceiling_height;
    app.active_manual_doors.push(door);

    app.refresh_active_manual_door_wall_meshes()?;
    let closed_initial_draws = app.dynamic_door_draws.len();
    let closed_initial_handles = app.dynamic_door_mesh_handles.clone();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = open_height;
    door.phase = DoomManualDoorPhase::Waiting { remaining_ticks: 1 };
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let opened_handles = app.dynamic_door_mesh_handles.clone();
    let opened_sources = app
        .dynamic_door_draws
        .iter()
        .map(|index| format!("{index}:{}", app.draws[*index].source_label))
        .collect::<Vec<_>>();
    let opened_enabled = app
        .dynamic_door_draws
        .iter()
        .filter(|index| app.opaque_draw_enabled[**index])
        .count();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = closed_height;
    door.phase = DoomManualDoorPhase::Closed;
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let closed_suppressed = app
        .dynamic_door_draws
        .iter()
        .filter(|index| !app.opaque_draw_enabled[**index])
        .count();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = open_height;
    door.phase = DoomManualDoorPhase::Waiting { remaining_ticks: 1 };
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let reopened_handles = app.dynamic_door_mesh_handles.clone();
    let reopened_enabled = app
        .dynamic_door_draws
        .iter()
        .filter(|index| app.opaque_draw_enabled[**index])
        .count();
    let cutout_last_handle = app
        .include_cutouts
        .then_some(app.cutout_mesh_base + app.cutout_draws.len() as u64 - 1);
    let dynamic_handles_are_after_cutouts = opened_handles
        .values()
        .all(|handle| cutout_last_handle.is_none_or(|cutout| handle.0 > cutout));

    println!(
        "E1M1 Slice 1 dynamic-resource replay: embedding={:?}; linedef={}; target-sector={}; closed-initial-draws={closed_initial_draws}; closed-initial-handles={}; opened-handles={:?}; opened-sources={}; opened-enabled={opened_enabled}; closed-suppressed={closed_suppressed}; reopened-handles={:?}; reopened-enabled={reopened_enabled}; stable-reopen={}; dynamic-after-cutouts={dynamic_handles_are_after_cutouts}; cutout-last-handle={cutout_last_handle:?}; source-map-mutated=false; renderer-initialized=false",
        app.comparative_embedding,
        source_linedef.record_index,
        target_sector.record_index,
        closed_initial_handles.len(),
        opened_handles,
        opened_sources.join(" | "),
        reopened_handles,
        opened_handles == reopened_handles,
    );
    Ok(())
}
