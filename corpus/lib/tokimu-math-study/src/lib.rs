//! Experimental evidence for AR-0019.
//!
//! This crate is deliberately corpus-local. Its baseline module observes the
//! current direct `glam` vocabulary without creating a replacement public API.
//! The provider-backed candidate currently probes only the `Vec3` slice of the
//! frozen operation inventory. It is not a proposed stable API or a completed
//! alternative-B result.

pub mod alternative_b;

pub(crate) mod alternative_b_provider {
    pub(crate) use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

    pub(crate) fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(eye, target, up)
    }

    pub(crate) fn perspective_rh_gl(
        vertical_fov_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        Mat4::perspective_rh_gl(vertical_fov_radians, aspect_ratio, near, far)
    }

    pub(crate) fn orthographic_rh_gl(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
    }
}
pub mod alternative_c;
pub mod alternative_d;
pub mod baseline_a;
pub mod bulk_reference;
pub mod chart_junction;
pub mod conformance;
pub mod hello_3d_mono_adapters;
pub mod migration_b;
pub mod migration_c;
pub mod migration_hello_3d_mono;
pub mod migration_hello_3d_stereo;
pub mod migration_hello_asteroids;
pub mod migration_hello_cad;
pub mod migration_hello_doom_observer;
pub mod migration_hello_fps;
pub mod migration_hello_glb;
pub mod migration_hello_hole_punch;
pub mod workloads;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StudyAlternative {
    DirectGlamBaseline,
    ProviderBackedVocabulary,
    OwnedSubset,
    BoundedFork,
}

impl StudyAlternative {
    pub const fn id(self) -> &'static str {
        match self {
            Self::DirectGlamBaseline => "alternative-a-direct-glam",
            Self::ProviderBackedVocabulary => "alternative-b-provider-backed",
            Self::OwnedSubset => "alternative-c-owned-subset",
            Self::BoundedFork => "alternative-d-bounded-fork",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternative_ids_are_stable_corpus_labels() {
        assert_eq!(
            StudyAlternative::DirectGlamBaseline.id(),
            "alternative-a-direct-glam"
        );
        assert_eq!(
            StudyAlternative::ProviderBackedVocabulary.id(),
            "alternative-b-provider-backed"
        );
        assert_eq!(
            StudyAlternative::OwnedSubset.id(),
            "alternative-c-owned-subset"
        );
        assert_eq!(
            StudyAlternative::BoundedFork.id(),
            "alternative-d-bounded-fork"
        );
    }
}
