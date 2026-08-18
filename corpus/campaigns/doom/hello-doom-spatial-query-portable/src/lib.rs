//! Compact E1M1-derived native/WASM consumer of the shared spatial study.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use tokimu_core::math::Vec3;
    use tokimu_spatial_query_study::{Artifact, TriangleMember};
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    fn retained_e1m1_members() -> Vec<TriangleMember> {
        [
            [
                [-128.0, 104.0, -3280.0],
                [-256.0, 104.0, -3296.0],
                [-256.0, 104.0, -3264.0],
            ],
            [
                [928.0, 0.0, -3392.0],
                [1184.0, 0.0, -3552.0],
                [928.0, 0.0, -3552.0],
            ],
            [
                [-320.0, 104.0, -3168.0],
                [-320.0, 264.0, -3168.0],
                [-256.0, 264.0, -3136.0],
            ],
            [
                [-336.0, 136.0, -3168.0],
                [-336.0, 240.0, -3168.0],
                [-320.0, 240.0, -3168.0],
            ],
        ]
        .into_iter()
        .enumerate()
        .map(|(identity, vertices)| {
            TriangleMember::new(identity, format!("e1m1-retained-{identity}"), vertices)
                .expect("retained fixture coordinates are finite")
        })
        .collect()
    }

    fn assert_retained_e1m1_fixture() {
        let artifact = Artifact::build(retained_e1m1_members(), 29).expect("portable artifact");
        assert_eq!(artifact.structure_fingerprint(), 0x3189_fb35_dfba_3bdc);
        assert_eq!(artifact.audit().missing_members, 0);
        assert_eq!(artifact.audit().containment_failures, 0);

        let rays = [
            (
                Vec3::new(-80.153_22, 140.0, -3260.0718),
                Vec3::new(-0.958_382_3, -0.214_579_95, -0.188_305_15),
                0,
            ),
            (
                Vec3::new(804.535_1, 36.0, -3374.6528),
                Vec3::new(0.923_876_9, -0.188_466_03, -0.333_064_53),
                1,
            ),
            (
                Vec3::new(-97.8244, 140.0, -3256.0034),
                Vec3::new(-0.811_669_2, 0.463_742_4, 0.355_156_27),
                2,
            ),
            (
                Vec3::new(-97.8244, 140.0, -3256.0034),
                Vec3::new(-0.876_003, 0.356_287_87, 0.325_080_84),
                3,
            ),
        ];
        let mut observed = BTreeSet::new();
        for (origin, direction, expected) in rays {
            let (hit, _) = artifact
                .query_nearest_ray(artifact.revision(), origin, direction)
                .expect("matching revision");
            let identity = hit.expect("retained ray hit").identity;
            assert_eq!(identity, expected);
            observed.insert(identity);
        }
        assert_eq!(observed, BTreeSet::from([0, 1, 2, 3]));
        assert!(artifact
            .query_nearest_ray(artifact.revision().wrapping_add(1), Vec3::ZERO, Vec3::Z)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn retained_e1m1_subset_matches_on_native_and_wasm() {
        assert_retained_e1m1_fixture();
    }
}
