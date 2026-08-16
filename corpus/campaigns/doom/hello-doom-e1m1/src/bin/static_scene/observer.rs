//! Source-spawn observer state for the native E1M1 evidence application.
//!
//! This module owns only the corpus camera's retained source identity and its
//! presentation-only look deltas. It deliberately does not own input policy,
//! player simulation, collision, or Doom runtime state.

use hello_doom_e1m1::{observer_direction, DoomComparativeEmbedding};
use tokimu::Camera;
use tokimu_core::math::Vec3;

/// Corpus-local source-spawn observer. This is a fixed visual evidence camera,
/// not runtime player state, movement, collision, or an original-Doom claim.
#[derive(Clone, Copy, Debug)]
pub(super) struct SpawnObserver {
    pub(super) position: Vec3,
    pub(super) forward: Vec3,
    pub(super) source_record: u32,
    pub(super) source_position: [i16; 2],
    pub(super) source_angle: u16,
    pub(super) sector: u32,
    pub(super) floor: i16,
    pub(super) ceiling: i16,
}

/// Presentation-only look state for the opt-in source-spawn observer. It is
/// deliberately not imported player orientation, runtime state, or input
/// policy beyond this native evidence application.
#[derive(Clone, Copy, Debug)]
pub(super) struct ObserverLook {
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) last_cursor: Option<[f32; 2]>,
}

pub(super) fn apply_look_delta(look: &mut ObserverLook, delta_x: f32, delta_y: f32) {
    // `look_at_rh` receives the source-world forward vector, whose horizontal
    // view sign is opposite the screen-space cursor delta on the native path:
    // moving right therefore subtracts yaw to turn the displayed view right.
    // Moving down looks down. This is a first-person observer convention, not
    // the AR-0021 model-orbit convention.
    look.yaw -= delta_x * 0.0032;
    look.pitch = (look.pitch - delta_y * 0.0024).clamp(-0.7, 0.7);
}

/// Lowers a corpus observer pose through the explicit AR-0028 comparison
/// embedding. This is diagnostic/source-adapter machinery, not camera API.
pub(super) fn doom_source_pose(
    observer: SpawnObserver,
    look: ObserverLook,
    embedding: DoomComparativeEmbedding,
) -> ([i16; 2], f64) {
    let (source_xy, _) = embedding.lower_direction(observer.position);
    let source_position = [source_xy[0].round() as i16, source_xy[1].round() as i16];
    let direction = observer_direction(look.yaw, look.pitch);
    let (source_forward, _) = embedding.lower_direction(direction);
    let source_angle = f64::from(source_forward[1].atan2(source_forward[0]));
    (source_position, source_angle)
}

/// Builds the corpus camera from either the retained source-spawn observer or
/// the explicit overview control. Projection range is application evidence,
/// not a renderer-wide Doom policy.
pub(super) fn scene_camera(
    size: [f32; 2],
    center: Vec3,
    radius: f32,
    spawn_observer: Option<SpawnObserver>,
    observer_look: Option<ObserverLook>,
) -> Camera {
    let mut camera = Camera::perspective_3d(size[0], size[1]);
    // `Camera::perspective_3d` deliberately serves small corpus fixtures
    // with a 100-unit far plane. E1M1's ordinary source coordinates span
    // thousands of units, so this consumer owns an explicit overview
    // projection rather than treating that convenience default as a
    // renderer-wide Doom policy.
    let aspect = size[0] / size[1].max(1.0);
    camera.projection = tokimu_core::math::try_projection_perspective_rh_gl(
        60.0_f32.to_radians(),
        aspect,
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    )
    .expect("perspective parameters must be finite and ordered");
    camera.view = if let (Some(observer), Some(look)) = (spawn_observer, observer_look) {
        tokimu_core::math::try_view_look_at_rh(
            observer.position,
            observer.position + observer_direction(look.yaw, look.pitch) * 128.0,
            Vec3::Y,
        )
        .expect("camera basis must be finite and non-degenerate")
    } else {
        tokimu_core::math::try_view_look_at_rh(
            center + Vec3::new(radius, radius * 0.72, radius),
            center,
            Vec3::Y,
        )
        .expect("camera basis must be finite and non-degenerate")
    };
    camera
}

#[cfg(test)]
mod tests {
    use super::{apply_look_delta, ObserverLook};
    use hello_doom_e1m1::{observer_direction, observer_right};
    use tokimu_core::math::Vec3;

    #[test]
    fn observer_look_uses_first_person_pointer_signs_and_bounded_pitch() {
        let mut look = ObserverLook {
            yaw: 0.0,
            pitch: 0.0,
            last_cursor: None,
        };

        apply_look_delta(&mut look, 100.0, -100.0);
        assert!(look.yaw < 0.0);
        assert!(look.pitch > 0.0);
        assert!(observer_direction(look.yaw, 0.0).dot(observer_right(Vec3::Z)) > 0.0);

        apply_look_delta(&mut look, 0.0, -10_000.0);
        assert_eq!(look.pitch, 0.7);
    }
}
