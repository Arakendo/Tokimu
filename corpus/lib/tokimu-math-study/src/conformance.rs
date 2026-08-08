//! Shared, deterministic conformance evidence for the case study.
//!
//! These cases do **not** declare stable Tokimu math guarantees. Each one
//! names its status so a future ownership decision can either adopt it,
//! deliberately revise it, or leave it unspecified.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    /// A behavior observed from the pinned Alternative A provider.
    ObservedProviderBehavior,
    /// A behavior required by an existing caller or an explicit study case.
    RequiredConformance,
    /// A behavior intentionally not promoted to a candidate requirement.
    Unspecified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConformanceCase {
    pub id: &'static str,
    pub status: EvidenceStatus,
    pub rationale: &'static str,
}

/// The initial shared cases, scoped only to the implemented `Vec3` probe.
pub const VEC3_CASES: [ConformanceCase; 6] = [
    ConformanceCase {
        id: "vec3-construction-and-arithmetic",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Current render and corpus callers construct and combine Vec3 values.",
    },
    ConformanceCase {
        id: "vec3-dot-and-cross",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Cross is a current caller requirement; dot is a shared comparison primitive.",
    },
    ConformanceCase {
        id: "vec3-nonzero-normalization",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Current callers normalize movement, ray, and normal vectors.",
    },
    ConformanceCase {
        id: "vec3-zero-normalize-or-zero",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "The safe zero behavior is observed from the provider pending a Tokimu contract decision.",
    },
    ConformanceCase {
        id: "vec3-zero-normalize",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "The non-safe normalization edge behavior remains provider evidence, not a Tokimu promise.",
    },
    ConformanceCase {
        id: "vec3-non-finite-normalize-or-zero",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "Non-finite fallback behavior is observed before any Tokimu validation contract is chosen.",
    },
];

/// Initial transform evidence. These cases document the provider's currently
/// observed right-handed, OpenGL-depth projection conventions; adoption as a
/// Tokimu contract requires a later architectural decision.
pub const TRANSFORM_CASES: [ConformanceCase; 9] = [
    ConformanceCase {
        id: "mat4-translation-and-vector-separation",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Current render and corpus callers transform positions and direction vectors.",
    },
    ConformanceCase {
        id: "mat4-inverse-composition",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Camera and corpus paths depend on inverse transforms.",
    },
    ConformanceCase {
        id: "mat4-affine-inverse-sweep",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Camera, scene, CAD, and renderer paths require stable inversion across composed non-singular affine transforms.",
    },
    ConformanceCase {
        id: "mat4-deterministic-affine-differential-sweep",
        status: EvidenceStatus::RequiredConformance,
        rationale: "The owned inverse needs bounded differential coverage beyond hand-selected transforms before it can remain a viable experiment.",
    },
    ConformanceCase {
        id: "mat4-look-at-rh",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "The right-handed view convention is currently provider evidence.",
    },
    ConformanceCase {
        id: "mat4-perspective-rh-gl",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "The OpenGL-depth projection convention is currently provider evidence.",
    },
    ConformanceCase {
        id: "mat4-finite-camera-projection-differential-sweep",
        status: EvidenceStatus::RequiredConformance,
        rationale: "Current renderer and corpus camera paths compose finite look-at views with perspective projections.",
    },
    ConformanceCase {
        id: "mat4-degenerate-look-at",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "Degenerate view input behavior must be visible before diagnostics or recovery semantics are promised.",
    },
    ConformanceCase {
        id: "mat4-singular-inverse",
        status: EvidenceStatus::ObservedProviderBehavior,
        rationale: "Singular inversion behavior is provider evidence until Tokimu deliberately chooses a contract.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alternative_b::{Mat4 as CandidateMat4, Vec3 as CandidateVec3};
    use crate::alternative_c::{Mat4 as OwnedMat4, Vec3 as OwnedVec3};
    use crate::alternative_d::Vec3 as ForkedVec3;
    use tokimu_core::math::{Mat4 as BaselineMat4, Vec3 as BaselineVec3};

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    const EPSILON: f32 = 1.0e-6;
    const AFFINE_INVERSE_EPSILON: f32 = 3.0e-5;
    const DIFFERENTIAL_AFFINE_EPSILON: f32 = 1.0e-3;
    const CAMERA_PROJECTION_EPSILON: f32 = 1.0e-4;

    fn assert_vec3_near(actual: [f32; 3], expected: [f32; 3]) {
        assert_vec3_near_with_tolerance(actual, expected, EPSILON);
    }

    fn assert_vec3_near_with_tolerance(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= tolerance,
                "{actual} != {expected} within {tolerance}"
            );
        }
    }

    fn assert_mat4_near_with_tolerance(actual: [f32; 16], expected: [f32; 16], tolerance: f32) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= tolerance,
                "{actual} != {expected} within {tolerance}"
            );
        }
    }

    fn non_finite_mask(values: [f32; 16]) -> [bool; 16] {
        values.map(|value| !value.is_finite())
    }

    fn next_unit_interval(seed: &mut u32) -> f32 {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (*seed >> 8) as f32 / ((u32::MAX >> 8) as f32)
    }

    fn next_range(seed: &mut u32, minimum: f32, maximum: f32) -> f32 {
        minimum + (maximum - minimum) * next_unit_interval(seed)
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn baseline_a_satisfies_the_initial_vec3_cases() {
        let arithmetic = (BaselineVec3::new(1.0, 2.0, 3.0) + BaselineVec3::Y) * 2.0;
        assert_eq!(arithmetic.to_array(), [2.0, 6.0, 6.0]);

        assert_eq!(
            BaselineVec3::new(1.0, 2.0, 3.0).dot(BaselineVec3::new(4.0, 5.0, 6.0)),
            32.0
        );
        assert_eq!(BaselineVec3::X.cross(BaselineVec3::Y), BaselineVec3::Z);
        assert_vec3_near(
            BaselineVec3::new(3.0, 4.0, 0.0).normalize().to_array(),
            [0.6, 0.8, 0.0],
        );
        assert_eq!(BaselineVec3::ZERO.normalize_or_zero(), BaselineVec3::ZERO);
        assert!(BaselineVec3::ZERO.normalize().is_nan());
        assert_eq!(
            BaselineVec3::new(f32::NAN, 0.0, 0.0).normalize_or_zero(),
            BaselineVec3::ZERO
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn provider_backed_b_matches_the_initial_vec3_cases() {
        let arithmetic = (CandidateVec3::new(1.0, 2.0, 3.0) + CandidateVec3::Y) * 2.0;
        assert_eq!(arithmetic.to_array(), [2.0, 6.0, 6.0]);

        assert_eq!(
            CandidateVec3::new(1.0, 2.0, 3.0).dot(CandidateVec3::new(4.0, 5.0, 6.0)),
            32.0
        );
        assert_eq!(
            CandidateVec3::new(1.0, 0.0, 0.0)
                .cross(CandidateVec3::Y)
                .to_array(),
            [0.0, 0.0, 1.0]
        );
        assert_vec3_near(
            CandidateVec3::new(3.0, 4.0, 0.0).normalize().to_array(),
            [0.6, 0.8, 0.0],
        );
        assert_eq!(CandidateVec3::ZERO.normalize_or_zero(), CandidateVec3::ZERO);
        assert!(CandidateVec3::ZERO
            .normalize()
            .to_array()
            .into_iter()
            .all(f32::is_nan));
        assert_eq!(
            CandidateVec3::new(f32::NAN, 0.0, 0.0).normalize_or_zero(),
            CandidateVec3::ZERO
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_c_matches_the_initial_vec3_cases() {
        let arithmetic = (OwnedVec3::new(1.0, 2.0, 3.0) + OwnedVec3::Y) * 2.0;
        assert_eq!(arithmetic.to_array(), [2.0, 6.0, 6.0]);

        assert_eq!(
            OwnedVec3::new(1.0, 2.0, 3.0).dot(OwnedVec3::new(4.0, 5.0, 6.0)),
            32.0
        );
        assert_eq!(
            OwnedVec3::new(1.0, 0.0, 0.0).cross(OwnedVec3::Y).to_array(),
            [0.0, 0.0, 1.0]
        );
        assert_vec3_near(
            OwnedVec3::new(3.0, 4.0, 0.0).normalize().to_array(),
            [0.6, 0.8, 0.0],
        );
        assert_eq!(OwnedVec3::ZERO.normalize_or_zero(), OwnedVec3::ZERO);
        assert!(OwnedVec3::ZERO
            .normalize()
            .to_array()
            .into_iter()
            .all(f32::is_nan));
        assert_eq!(
            OwnedVec3::new(f32::NAN, 0.0, 0.0).normalize_or_zero(),
            OwnedVec3::ZERO
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn bounded_fork_d_matches_the_initial_vec3_cases() {
        let arithmetic = (ForkedVec3::new(1.0, 2.0, 3.0) + ForkedVec3::Y) * 2.0;
        assert_eq!(arithmetic.to_array(), [2.0, 6.0, 6.0]);
        assert_eq!(
            ForkedVec3::new(1.0, 2.0, 3.0).dot(ForkedVec3::new(4.0, 5.0, 6.0)),
            32.0
        );
        assert_eq!(
            ForkedVec3::new(1.0, 0.0, 0.0)
                .cross(ForkedVec3::Y)
                .to_array(),
            [0.0, 0.0, 1.0]
        );
        assert_vec3_near(
            ForkedVec3::new(3.0, 4.0, 0.0).normalize().to_array(),
            [0.6, 0.8, 0.0],
        );
        assert_eq!(ForkedVec3::ZERO.normalize_or_zero(), ForkedVec3::ZERO);
        assert!(ForkedVec3::ZERO
            .normalize()
            .to_array()
            .into_iter()
            .all(f32::is_nan));
        assert_eq!(
            ForkedVec3::new(f32::NAN, 0.0, 0.0).normalize_or_zero(),
            ForkedVec3::ZERO
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn initial_cases_preserve_their_evidence_classification() {
        assert_eq!(VEC3_CASES.len(), 6);
        assert!(VEC3_CASES
            .iter()
            .any(|case| case.status == EvidenceStatus::ObservedProviderBehavior));
        assert!(VEC3_CASES
            .iter()
            .any(|case| case.status == EvidenceStatus::RequiredConformance));
        assert!(TRANSFORM_CASES
            .iter()
            .any(|case| case.id == "mat4-affine-inverse-sweep"));
        assert!(TRANSFORM_CASES
            .iter()
            .any(|case| case.id == "mat4-deterministic-affine-differential-sweep"));
        assert!(TRANSFORM_CASES
            .iter()
            .any(|case| case.id == "mat4-finite-camera-projection-differential-sweep"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn baseline_a_satisfies_the_initial_transform_cases() {
        let translation = BaselineMat4::from_translation(BaselineVec3::new(4.0, 5.0, 6.0));
        assert_eq!(
            translation
                .transform_point3(BaselineVec3::new(1.0, 2.0, 3.0))
                .to_array(),
            [5.0, 7.0, 9.0]
        );
        assert_eq!(
            translation.transform_vector3(BaselineVec3::Y).to_array(),
            [0.0, 1.0, 0.0]
        );
        assert_vec3_near(
            translation
                .inverse()
                .transform_point3(BaselineVec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0],
        );
        assert_eq!(
            BaselineMat4::look_at_rh(
                BaselineVec3::new(0.0, 0.0, 5.0),
                BaselineVec3::ZERO,
                BaselineVec3::Y
            )
            .transform_point3(BaselineVec3::ZERO)
            .to_array(),
            [0.0, 0.0, -5.0]
        );
        assert!(BaselineMat4::perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0)
            .to_cols_array()
            .into_iter()
            .all(f32::is_finite));
        assert!(
            BaselineMat4::look_at_rh(BaselineVec3::ZERO, BaselineVec3::ZERO, BaselineVec3::Y)
                .to_cols_array()
                .into_iter()
                .any(f32::is_nan)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn provider_backed_b_satisfies_the_initial_transform_cases() {
        let translation = CandidateMat4::from_translation(CandidateVec3::new(4.0, 5.0, 6.0));
        assert_eq!(
            translation
                .transform_point3(CandidateVec3::new(1.0, 2.0, 3.0))
                .to_array(),
            [5.0, 7.0, 9.0]
        );
        assert_eq!(
            translation.transform_vector3(CandidateVec3::Y).to_array(),
            [0.0, 1.0, 0.0]
        );
        assert_vec3_near(
            translation
                .inverse()
                .transform_point3(CandidateVec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0],
        );
        assert_eq!(
            CandidateMat4::look_at_rh(
                CandidateVec3::new(0.0, 0.0, 5.0),
                CandidateVec3::ZERO,
                CandidateVec3::Y
            )
            .transform_point3(CandidateVec3::ZERO)
            .to_array(),
            [0.0, 0.0, -5.0]
        );
        assert!(
            CandidateMat4::perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0)
                .to_cols_array()
                .into_iter()
                .all(f32::is_finite)
        );
        assert!(CandidateMat4::look_at_rh(
            CandidateVec3::ZERO,
            CandidateVec3::ZERO,
            CandidateVec3::Y
        )
        .to_cols_array()
        .into_iter()
        .any(f32::is_nan));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn owned_c_matches_the_non_inversion_transform_cases() {
        let translation = OwnedMat4::from_translation(OwnedVec3::new(4.0, 5.0, 6.0));
        assert_eq!(
            translation
                .transform_point3(OwnedVec3::new(1.0, 2.0, 3.0))
                .to_array(),
            [5.0, 7.0, 9.0]
        );
        assert_eq!(
            translation.transform_vector3(OwnedVec3::Y).to_array(),
            [0.0, 1.0, 0.0]
        );
        assert_vec3_near(
            translation
                .inverse()
                .transform_point3(OwnedVec3::new(5.0, 7.0, 9.0))
                .to_array(),
            [1.0, 2.0, 3.0],
        );
        assert_eq!(
            OwnedMat4::look_at_rh(OwnedVec3::new(0.0, 0.0, 5.0), OwnedVec3::ZERO, OwnedVec3::Y)
                .transform_point3(OwnedVec3::ZERO)
                .to_array(),
            [0.0, 0.0, -5.0]
        );
        assert!(OwnedMat4::perspective_rh_gl(1.0, 16.0 / 9.0, 0.1, 100.0)
            .to_cols_array()
            .into_iter()
            .all(f32::is_finite));
        assert!(
            OwnedMat4::look_at_rh(OwnedVec3::ZERO, OwnedVec3::ZERO, OwnedVec3::Y)
                .to_cols_array()
                .into_iter()
                .any(f32::is_nan)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn candidates_retain_observed_degenerate_matrix_behavior_without_promoting_it() {
        let baseline_degenerate_view =
            BaselineMat4::look_at_rh(BaselineVec3::ZERO, BaselineVec3::ZERO, BaselineVec3::Y)
                .to_cols_array();
        let baseline_singular_inverse = BaselineMat4::from_scale(BaselineVec3::new(1.0, 0.0, 1.0))
            .inverse()
            .to_cols_array();

        assert_eq!(
            non_finite_mask(
                CandidateMat4::look_at_rh(
                    CandidateVec3::ZERO,
                    CandidateVec3::ZERO,
                    CandidateVec3::Y,
                )
                .to_cols_array(),
            ),
            non_finite_mask(baseline_degenerate_view),
        );
        assert_eq!(
            non_finite_mask(
                OwnedMat4::look_at_rh(OwnedVec3::ZERO, OwnedVec3::ZERO, OwnedVec3::Y)
                    .to_cols_array(),
            ),
            non_finite_mask(baseline_degenerate_view),
        );
        assert_eq!(
            non_finite_mask(
                CandidateMat4::from_scale(CandidateVec3::new(1.0, 0.0, 1.0))
                    .inverse()
                    .to_cols_array(),
            ),
            non_finite_mask(baseline_singular_inverse),
        );
        assert_eq!(
            non_finite_mask(
                OwnedMat4::from_scale(OwnedVec3::new(1.0, 0.0, 1.0))
                    .inverse()
                    .to_cols_array(),
            ),
            non_finite_mask(baseline_singular_inverse),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn candidates_round_trip_a_deterministic_non_singular_affine_sweep() {
        // These are fixed caller-shaped transforms, not a random or exhaustive
        // numerical proof. They make C's scalar pivoted inverse face combined
        // translation, rotation, non-uniform scaling, and varying point ranges.
        const CASES: [([f32; 3], [f32; 3], f32, f32, [f32; 3]); 4] = [
            (
                [2.0, -3.0, 5.0],
                [1.0, 2.0, 0.5],
                0.2,
                -0.4,
                [4.0, -2.0, 1.0],
            ),
            (
                [-10.0, 8.0, 0.25],
                [3.0, 0.25, 2.0],
                -1.1,
                0.75,
                [-3.0, 12.0, 8.0],
            ),
            (
                [0.001, -0.002, 100.0],
                [0.1, 4.0, 1.5],
                1.4,
                2.2,
                [0.5, -0.25, 9.0],
            ),
            (
                [50.0, 25.0, -75.0],
                [2.5, 1.2, 0.8],
                -2.4,
                -0.9,
                [-40.0, 20.0, 60.0],
            ),
        ];

        for (translation, scale, rotation_x, rotation_y, point) in CASES {
            let baseline_transform =
                BaselineMat4::from_translation(BaselineVec3::from_array(translation))
                    * BaselineMat4::from_rotation_y(rotation_y)
                    * BaselineMat4::from_rotation_x(rotation_x)
                    * BaselineMat4::from_scale(BaselineVec3::from_array(scale));
            let provider_backed_transform =
                CandidateMat4::from_translation(CandidateVec3::from_array(translation))
                    * CandidateMat4::from_rotation_y(rotation_y)
                    * CandidateMat4::from_rotation_x(rotation_x)
                    * CandidateMat4::from_scale(CandidateVec3::from_array(scale));
            let owned_transform = OwnedMat4::from_translation(OwnedVec3::from_array(translation))
                * OwnedMat4::from_rotation_y(rotation_y)
                * OwnedMat4::from_rotation_x(rotation_x)
                * OwnedMat4::from_scale(OwnedVec3::from_array(scale));

            let baseline_round_trip = baseline_transform
                .inverse()
                .transform_point3(
                    baseline_transform.transform_point3(BaselineVec3::from_array(point)),
                )
                .to_array();
            let provider_backed_round_trip = provider_backed_transform
                .inverse()
                .transform_point3(
                    provider_backed_transform.transform_point3(CandidateVec3::from_array(point)),
                )
                .to_array();
            let owned_round_trip = owned_transform
                .inverse()
                .transform_point3(owned_transform.transform_point3(OwnedVec3::from_array(point)))
                .to_array();

            assert_vec3_near_with_tolerance(
                provider_backed_round_trip,
                baseline_round_trip,
                AFFINE_INVERSE_EPSILON,
            );
            assert_vec3_near_with_tolerance(
                owned_round_trip,
                baseline_round_trip,
                AFFINE_INVERSE_EPSILON,
            );
            assert_vec3_near_with_tolerance(owned_round_trip, point, AFFINE_INVERSE_EPSILON);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn candidates_match_a_deterministic_affine_differential_sweep() {
        // A fixed LCG gives repeatable coverage without a random dependency.
        // The scale range intentionally excludes zero and near-zero values;
        // singular and non-finite behavior stays a separate, labelled question.
        let mut seed = 0x544F_4B49;

        for _ in 0..96 {
            let translation = [
                next_range(&mut seed, -1_000.0, 1_000.0),
                next_range(&mut seed, -1_000.0, 1_000.0),
                next_range(&mut seed, -1_000.0, 1_000.0),
            ];
            let scale = [
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
            ];
            let rotation_x = next_range(&mut seed, -core::f32::consts::PI, core::f32::consts::PI);
            let rotation_y = next_range(&mut seed, -core::f32::consts::PI, core::f32::consts::PI);
            let point = [
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
            ];

            let baseline_transform =
                BaselineMat4::from_translation(BaselineVec3::from_array(translation))
                    * BaselineMat4::from_rotation_y(rotation_y)
                    * BaselineMat4::from_rotation_x(rotation_x)
                    * BaselineMat4::from_scale(BaselineVec3::from_array(scale));
            let provider_backed_transform =
                CandidateMat4::from_translation(CandidateVec3::from_array(translation))
                    * CandidateMat4::from_rotation_y(rotation_y)
                    * CandidateMat4::from_rotation_x(rotation_x)
                    * CandidateMat4::from_scale(CandidateVec3::from_array(scale));
            let owned_transform = OwnedMat4::from_translation(OwnedVec3::from_array(translation))
                * OwnedMat4::from_rotation_y(rotation_y)
                * OwnedMat4::from_rotation_x(rotation_x)
                * OwnedMat4::from_scale(OwnedVec3::from_array(scale));

            let baseline_round_trip = baseline_transform
                .inverse()
                .transform_point3(
                    baseline_transform.transform_point3(BaselineVec3::from_array(point)),
                )
                .to_array();
            let provider_backed_round_trip = provider_backed_transform
                .inverse()
                .transform_point3(
                    provider_backed_transform.transform_point3(CandidateVec3::from_array(point)),
                )
                .to_array();
            let owned_round_trip = owned_transform
                .inverse()
                .transform_point3(owned_transform.transform_point3(OwnedVec3::from_array(point)))
                .to_array();

            assert_vec3_near_with_tolerance(
                provider_backed_round_trip,
                baseline_round_trip,
                DIFFERENTIAL_AFFINE_EPSILON,
            );
            assert_vec3_near_with_tolerance(
                owned_round_trip,
                baseline_round_trip,
                DIFFERENTIAL_AFFINE_EPSILON,
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn candidates_match_a_finite_camera_projection_differential_sweep() {
        // This is fixed-seed finite camera pressure from current 3D callers.
        // It deliberately keeps eye/target distinct, a nonparallel up vector,
        // positive aspect, and near < far. Degenerate behavior remains in the
        // separately labelled observed-provider cases.
        let mut seed = 0x4341_4D45;

        for _ in 0..128 {
            let eye = [
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
                next_range(&mut seed, -100.0, 100.0),
            ];
            let center = [
                eye[0] + next_range(&mut seed, 0.25, 20.0),
                eye[1] + next_range(&mut seed, -10.0, 10.0),
                eye[2] + next_range(&mut seed, 0.25, 20.0),
            ];
            let field_of_view = next_range(&mut seed, 0.25, 2.5);
            let aspect_ratio = next_range(&mut seed, 0.5, 2.5);
            let near = next_range(&mut seed, 0.01, 2.0);
            let far = near + next_range(&mut seed, 1.0, 500.0);

            let baseline = BaselineMat4::perspective_rh_gl(field_of_view, aspect_ratio, near, far)
                * BaselineMat4::look_at_rh(
                    BaselineVec3::from_array(eye),
                    BaselineVec3::from_array(center),
                    BaselineVec3::Y,
                );
            let provider_backed =
                CandidateMat4::perspective_rh_gl(field_of_view, aspect_ratio, near, far)
                    * CandidateMat4::look_at_rh(
                        CandidateVec3::from_array(eye),
                        CandidateVec3::from_array(center),
                        CandidateVec3::Y,
                    );
            let owned = OwnedMat4::perspective_rh_gl(field_of_view, aspect_ratio, near, far)
                * OwnedMat4::look_at_rh(
                    OwnedVec3::from_array(eye),
                    OwnedVec3::from_array(center),
                    OwnedVec3::Y,
                );

            assert_mat4_near_with_tolerance(
                provider_backed.to_cols_array(),
                baseline.to_cols_array(),
                CAMERA_PROJECTION_EPSILON,
            );
            assert_mat4_near_with_tolerance(
                owned.to_cols_array(),
                baseline.to_cols_array(),
                CAMERA_PROJECTION_EPSILON,
            );
        }
    }
}
