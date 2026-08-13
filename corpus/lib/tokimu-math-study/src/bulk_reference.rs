//! CPU-only semantic references for the two Slice 7 bulk candidates.
//!
//! These types are corpus scaffolding, not a renderer/culling/compute API. The
//! caller owns all identities, coordinate frames, and decisions made from the
//! ordered classifications.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rejection {
    Invalid,
    Plane(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Classification {
    Candidate,
    Rejected(Rejection),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: [f32; 3],
    pub offset: f32,
}

impl Plane {
    #[must_use]
    pub fn signed_distance(self, point: [f32; 3]) -> f32 {
        self.normal[0] * point[0]
            + self.normal[1] * point[1]
            + self.normal[2] * point[2]
            + self.offset
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub minimum: [f32; 3],
    pub maximum: [f32; 3],
}

impl Aabb {
    fn is_finite_and_ordered(self) -> bool {
        self.minimum
            .into_iter()
            .chain(self.maximum)
            .all(f32::is_finite)
            && (0..3).all(|axis| self.minimum[axis] <= self.maximum[axis])
    }

    fn positive_vertex(self, normal: [f32; 3]) -> [f32; 3] {
        core::array::from_fn(|axis| {
            if normal[axis] >= 0.0 {
                self.maximum[axis]
            } else {
                self.minimum[axis]
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IdentifiedAabb {
    pub id: u32,
    pub bounds: Aabb,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IdentifiedPoint {
    pub id: u32,
    pub position: [f32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClassificationRecord {
    pub id: u32,
    pub result: Classification,
}

/// Classifies ordered world AABBs conservatively against caller-supplied planes.
/// A box touching a plane remains a candidate; rejecting a potentially visible
/// box would violate this reference's one-sided correctness rule.
#[must_use]
pub fn classify_aabbs(planes: &[Plane], input: &[IdentifiedAabb]) -> Vec<ClassificationRecord> {
    let mut output = Vec::with_capacity(input.len());
    classify_aabbs_into(planes, input, &mut output);
    output
}

/// Reuses caller-provided result storage while preserving input order and IDs.
/// This makes residency/output-allocation controls explicit in the corpus
/// measurement without pretending the caller has accepted a public API.
pub fn classify_aabbs_into(
    planes: &[Plane],
    input: &[IdentifiedAabb],
    output: &mut Vec<ClassificationRecord>,
) {
    output.clear();
    output.reserve(input.len().saturating_sub(output.capacity()));
    output.extend(input.iter().map(|item| ClassificationRecord {
        id: item.id,
        result: classify_aabb(planes, item.bounds),
    }));
}

#[must_use]
pub fn classify_points(planes: &[Plane], input: &[IdentifiedPoint]) -> Vec<ClassificationRecord> {
    let mut output = Vec::with_capacity(input.len());
    classify_points_into(planes, input, &mut output);
    output
}

/// Reuses caller-provided result storage while preserving input order and IDs.
pub fn classify_points_into(
    planes: &[Plane],
    input: &[IdentifiedPoint],
    output: &mut Vec<ClassificationRecord>,
) {
    output.clear();
    output.reserve(input.len().saturating_sub(output.capacity()));
    output.extend(input.iter().map(|item| ClassificationRecord {
        id: item.id,
        result: classify_point(planes, item.position),
    }));
}

/// Returns a caller-owned compacted candidate list in its original input order.
#[must_use]
pub fn candidate_ids(records: &[ClassificationRecord]) -> Vec<u32> {
    records
        .iter()
        .filter_map(|record| (record.result == Classification::Candidate).then_some(record.id))
        .collect()
}

/// Returns the count-only observation without changing or reordering records.
#[must_use]
pub fn candidate_count(records: &[ClassificationRecord]) -> usize {
    records
        .iter()
        .filter(|record| record.result == Classification::Candidate)
        .count()
}

/// Produces a deterministic, order-sensitive digest for corpus replay controls.
#[must_use]
pub fn classification_checksum(records: &[ClassificationRecord]) -> u64 {
    records
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |checksum, record| {
            let status = match record.result {
                Classification::Candidate => 0_u64,
                Classification::Rejected(Rejection::Invalid) => 1,
                Classification::Rejected(Rejection::Plane(index)) => {
                    2_u64 + u64::try_from(index).expect("plane index fits u64")
                }
            };
            checksum
                .wrapping_mul(0x0000_0100_0000_01b3)
                .wrapping_add(u64::from(record.id).rotate_left(17) ^ status)
        })
}

#[must_use]
pub fn unit_cube_planes() -> [Plane; 6] {
    [
        Plane {
            normal: [1.0, 0.0, 0.0],
            offset: 1.0,
        },
        Plane {
            normal: [-1.0, 0.0, 0.0],
            offset: 1.0,
        },
        Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 1.0,
        },
        Plane {
            normal: [0.0, -1.0, 0.0],
            offset: 1.0,
        },
        Plane {
            normal: [0.0, 0.0, 1.0],
            offset: 1.0,
        },
        Plane {
            normal: [0.0, 0.0, -1.0],
            offset: 1.0,
        },
    ]
}

#[must_use]
pub fn generated_aabbs(count: usize) -> Vec<IdentifiedAabb> {
    let mut seed = 0xAABB_C0DE_u32;
    (0..count)
        .map(|id| {
            let center = core::array::from_fn(|_| next_signed(&mut seed));
            let half_extent = 0.01 + next_unit(&mut seed) * 0.15;
            IdentifiedAabb {
                id: u32::try_from(id).expect("bounded corpus count fits u32"),
                bounds: Aabb {
                    minimum: center.map(|value| value - half_extent),
                    maximum: center.map(|value| value + half_extent),
                },
            }
        })
        .collect()
}

#[must_use]
pub fn generated_points(count: usize) -> Vec<IdentifiedPoint> {
    let mut seed = 0xB01D_7C0D_u32;
    (0..count)
        .map(|id| IdentifiedPoint {
            id: u32::try_from(id).expect("bounded corpus count fits u32"),
            position: core::array::from_fn(|_| next_signed(&mut seed)),
        })
        .collect()
}

fn classify_aabb(planes: &[Plane], bounds: Aabb) -> Classification {
    if !bounds.is_finite_and_ordered() {
        return Classification::Rejected(Rejection::Invalid);
    }
    for (index, plane) in planes.iter().enumerate() {
        if !plane.normal.into_iter().all(f32::is_finite) || !plane.offset.is_finite() {
            return Classification::Rejected(Rejection::Invalid);
        }
        if plane.signed_distance(bounds.positive_vertex(plane.normal)) < 0.0 {
            return Classification::Rejected(Rejection::Plane(index));
        }
    }
    Classification::Candidate
}

fn classify_point(planes: &[Plane], point: [f32; 3]) -> Classification {
    if !point.into_iter().all(f32::is_finite) {
        return Classification::Rejected(Rejection::Invalid);
    }
    for (index, plane) in planes.iter().enumerate() {
        if !plane.normal.into_iter().all(f32::is_finite) || !plane.offset.is_finite() {
            return Classification::Rejected(Rejection::Invalid);
        }
        if plane.signed_distance(point) < 0.0 {
            return Classification::Rejected(Rejection::Plane(index));
        }
    }
    Classification::Candidate
}

fn next_unit(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 8) as f32 / ((u32::MAX >> 8) as f32)
}

fn next_signed(seed: &mut u32) -> f32 {
    next_unit(seed) * 4.0 - 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_bounds_classification_retains_identity_and_conservative_plane_touching() {
        let planes = unit_cube_planes();
        let input = [
            IdentifiedAabb {
                id: 7,
                bounds: Aabb {
                    minimum: [-0.5, -0.5, -0.5],
                    maximum: [0.5, 0.5, 0.5],
                },
            },
            IdentifiedAabb {
                id: 9,
                bounds: Aabb {
                    minimum: [1.0, 0.0, 0.0],
                    maximum: [1.2, 0.2, 0.2],
                },
            },
            IdentifiedAabb {
                id: 11,
                bounds: Aabb {
                    minimum: [1.1, 0.0, 0.0],
                    maximum: [1.2, 0.2, 0.2],
                },
            },
        ];
        assert_eq!(
            classify_aabbs(&planes, &input),
            vec![
                ClassificationRecord {
                    id: 7,
                    result: Classification::Candidate
                },
                ClassificationRecord {
                    id: 9,
                    result: Classification::Candidate
                },
                ClassificationRecord {
                    id: 11,
                    result: Classification::Rejected(Rejection::Plane(1))
                },
            ]
        );
    }

    #[test]
    fn point_reference_rejects_invalid_values_without_reordering() {
        let input = [
            IdentifiedPoint {
                id: 3,
                position: [0.0, 0.0, 0.0],
            },
            IdentifiedPoint {
                id: 4,
                position: [f32::NAN, 0.0, 0.0],
            },
            IdentifiedPoint {
                id: 5,
                position: [-2.0, 0.0, 0.0],
            },
        ];
        assert_eq!(
            classify_points(&unit_cube_planes(), &input),
            vec![
                ClassificationRecord {
                    id: 3,
                    result: Classification::Candidate
                },
                ClassificationRecord {
                    id: 4,
                    result: Classification::Rejected(Rejection::Invalid)
                },
                ClassificationRecord {
                    id: 5,
                    result: Classification::Rejected(Rejection::Plane(0))
                },
            ]
        );
    }

    #[test]
    fn generators_are_deterministic_and_memory_bounded_by_requested_count() {
        assert_eq!(generated_aabbs(32), generated_aabbs(32));
        assert_eq!(generated_points(32), generated_points(32));
        assert_eq!(generated_aabbs(32).len(), 32);
        assert_eq!(generated_points(32).len(), 32);
    }

    #[test]
    fn reused_outputs_and_result_views_preserve_order_and_identity() {
        let input = [
            IdentifiedPoint {
                id: 12,
                position: [0.0, 0.0, 0.0],
            },
            IdentifiedPoint {
                id: 3,
                position: [2.0, 0.0, 0.0],
            },
            IdentifiedPoint {
                id: 9,
                position: [0.5, 0.0, 0.0],
            },
        ];
        let planes = unit_cube_planes();
        let mut reused = Vec::with_capacity(input.len());
        classify_points_into(&planes, &input, &mut reused);
        assert_eq!(
            reused.iter().map(|record| record.id).collect::<Vec<_>>(),
            vec![12, 3, 9]
        );
        assert_eq!(candidate_ids(&reused), vec![12, 9]);
        assert_eq!(candidate_count(&reused), 2);
        assert_eq!(
            classification_checksum(&reused),
            classification_checksum(&reused)
        );
    }
}
