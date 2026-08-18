//! Bounded audit of the representation actually covered by classic BSP boxes.
//!
//! Raw NODES fields, decoded node fields, descendant SEG endpoints and inferred
//! plane-support regions remain distinct evidence. None of these observations
//! grants renderer or presentation-removal authority.

use std::{collections::BTreeSet, io};

use doom_map_provider::{DoomBspChild, DoomMapCore};
use doom_wad_package::{DoomMapSelection, RequiredDoomMapLump};
use tokimu::PlatformResult;

#[derive(Clone, Debug, Default)]
pub(crate) struct DoomBspBoundsAudit {
    pub(crate) nodes: usize,
    pub(crate) child_boxes: usize,
    pub(crate) raw_decode_matches: usize,
    pub(crate) raw_decode_mismatches: usize,
    pub(crate) seg_endpoint_envelopes: usize,
    pub(crate) seg_endpoint_contained: usize,
    pub(crate) seg_endpoint_underbounded: usize,
    pub(crate) inferred_region_envelopes: usize,
    pub(crate) inferred_region_contained: usize,
    pub(crate) inferred_region_underbounded: usize,
    pub(crate) samples: Vec<String>,
}

impl DoomBspBoundsAudit {
    pub(crate) fn report(&self) -> String {
        format!(
            "nodes={}; child-boxes={}; raw-decoded=[match:{},mismatch:{}]; descendant-seg-envelope=[available:{},contained:{},underbounded:{}]; inferred-plane-region-envelope=[available:{},contained:{},underbounded:{}]; samples=[{}]; meaning=source-bsp-box-representation-audit-not-visibility-authority",
            self.nodes,
            self.child_boxes,
            self.raw_decode_matches,
            self.raw_decode_mismatches,
            self.seg_endpoint_envelopes,
            self.seg_endpoint_contained,
            self.seg_endpoint_underbounded,
            self.inferred_region_envelopes,
            self.inferred_region_contained,
            self.inferred_region_underbounded,
            self.samples.join(" | "),
        )
    }
}

pub(crate) fn audit_doom_bsp_bounds(
    wad_bytes: &[u8],
    selection: &DoomMapSelection,
    map: &DoomMapCore,
    inferred_region_bounds: &[Option<[f64; 4]>],
) -> PlatformResult<DoomBspBoundsAudit> {
    let lump = selection
        .required_lumps
        .iter()
        .find(|lump| lump.kind == RequiredDoomMapLump::Nodes)
        .ok_or_else(|| io::Error::other("selected map has no NODES audit source"))?;
    let start = usize::try_from(lump.source.offset)?;
    let size = usize::try_from(lump.source.size)?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| io::Error::other("NODES audit range overflow"))?;
    let raw = wad_bytes
        .get(start..end)
        .ok_or_else(|| io::Error::other("NODES audit range is outside the WAD"))?;
    if raw.len() != map.nodes.len() * 28 {
        return Err(io::Error::other(format!(
            "NODES audit length {} does not match {} decoded records",
            raw.len(),
            map.nodes.len()
        ))
        .into());
    }

    let mut audit = DoomBspBoundsAudit {
        nodes: map.nodes.len(),
        child_boxes: map.nodes.len() * 2,
        ..DoomBspBoundsAudit::default()
    };

    for (index, node) in map.nodes.iter().enumerate() {
        let record = &raw[index * 28..][..28];
        if raw_node_matches_decoded(record, node) {
            audit.raw_decode_matches += 1;
        } else {
            audit.raw_decode_mismatches += 1;
            push_sample(
                &mut audit.samples,
                format!("node={index}:raw-decoded-mismatch"),
            );
        }

        for (side, child, bbox) in [
            ("right", node.right_child, node.right_bbox),
            ("left", node.left_child, node.left_bbox),
        ] {
            let mut subsectors = BTreeSet::new();
            let mut visited_nodes = BTreeSet::new();
            collect_child_subsectors(map, child, &mut visited_nodes, &mut subsectors)?;

            if let Some(envelope) = descendant_seg_endpoint_envelope(map, &subsectors)? {
                audit.seg_endpoint_envelopes += 1;
                if bbox_contains_i16_envelope(bbox, envelope) {
                    audit.seg_endpoint_contained += 1;
                } else {
                    audit.seg_endpoint_underbounded += 1;
                    push_sample(
                        &mut audit.samples,
                        format!(
                            "node={index}:{side}:seg-underbound:bbox={bbox:?}:envelope={envelope:?}"
                        ),
                    );
                }
            }

            if let Some(envelope) =
                descendant_inferred_region_envelope(&subsectors, inferred_region_bounds)
            {
                audit.inferred_region_envelopes += 1;
                if bbox_contains_f64_envelope(bbox, envelope) {
                    audit.inferred_region_contained += 1;
                } else {
                    audit.inferred_region_underbounded += 1;
                    push_sample(
                        &mut audit.samples,
                        format!(
                            "node={index}:{side}:plane-region-overrun:bbox={bbox:?}:envelope=[{:.3},{:.3},{:.3},{:.3}]:subsectors={subsectors:?}",
                            envelope[0], envelope[1], envelope[2], envelope[3]
                        ),
                    );
                }
            }
        }
    }

    Ok(audit)
}

fn raw_node_matches_decoded(raw: &[u8], node: &doom_map_provider::DoomNode) -> bool {
    let i16_at = |offset| i16::from_le_bytes([raw[offset], raw[offset + 1]]);
    let u16_at = |offset| u16::from_le_bytes([raw[offset], raw[offset + 1]]);
    node.x == i16_at(0)
        && node.y == i16_at(2)
        && node.delta_x == i16_at(4)
        && node.delta_y == i16_at(6)
        && node.right_bbox == [i16_at(8), i16_at(10), i16_at(12), i16_at(14)]
        && node.left_bbox == [i16_at(16), i16_at(18), i16_at(20), i16_at(22)]
        && encode_child(node.right_child) == u16_at(24)
        && encode_child(node.left_child) == u16_at(26)
}

fn encode_child(child: DoomBspChild) -> u16 {
    match child {
        DoomBspChild::Node(index) => index,
        DoomBspChild::Subsector(index) => index | 0x8000,
    }
}

fn collect_child_subsectors(
    map: &DoomMapCore,
    child: DoomBspChild,
    visited_nodes: &mut BTreeSet<u16>,
    subsectors: &mut BTreeSet<u16>,
) -> PlatformResult<()> {
    match child {
        DoomBspChild::Subsector(index) => {
            if usize::from(index) >= map.subsectors.len() {
                return Err(io::Error::other(format!(
                    "BSP audit subsector {index} is out of range"
                ))
                .into());
            }
            subsectors.insert(index);
        }
        DoomBspChild::Node(index) => {
            if !visited_nodes.insert(index) {
                return Err(io::Error::other(format!(
                    "BSP audit encountered repeated/cyclic node {index}"
                ))
                .into());
            }
            let node = map.nodes.get(usize::from(index)).ok_or_else(|| {
                io::Error::other(format!("BSP audit node {index} is out of range"))
            })?;
            collect_child_subsectors(map, node.right_child, visited_nodes, subsectors)?;
            collect_child_subsectors(map, node.left_child, visited_nodes, subsectors)?;
        }
    }
    Ok(())
}

fn descendant_seg_endpoint_envelope(
    map: &DoomMapCore,
    subsectors: &BTreeSet<u16>,
) -> PlatformResult<Option<[i16; 4]>> {
    let mut envelope: Option<[i16; 4]> = None;
    for &subsector_index in subsectors {
        let subsector = &map.subsectors[usize::from(subsector_index)];
        let first = usize::from(subsector.first_seg);
        let end = first
            .checked_add(usize::from(subsector.seg_count))
            .ok_or_else(|| io::Error::other("BSP audit SEG range overflow"))?;
        let segs = map
            .segs
            .get(first..end)
            .ok_or_else(|| io::Error::other("BSP audit SEG range is out of bounds"))?;
        for seg in segs {
            for vertex_index in [seg.start_vertex, seg.end_vertex] {
                let vertex = map.vertices.get(usize::from(vertex_index)).ok_or_else(|| {
                    io::Error::other("BSP audit vertex reference is out of bounds")
                })?;
                envelope = Some(expand_i16_envelope(envelope, vertex.x, vertex.y));
            }
        }
    }
    Ok(envelope)
}

fn descendant_inferred_region_envelope(
    subsectors: &BTreeSet<u16>,
    bounds: &[Option<[f64; 4]>],
) -> Option<[f64; 4]> {
    subsectors.iter().fold(None, |envelope, &index| {
        let Some(bounds) = bounds.get(usize::from(index)).copied().flatten() else {
            return envelope;
        };
        Some(match envelope {
            None => bounds,
            Some(current) => [
                current[0].min(bounds[0]),
                current[1].min(bounds[1]),
                current[2].max(bounds[2]),
                current[3].max(bounds[3]),
            ],
        })
    })
}

fn expand_i16_envelope(current: Option<[i16; 4]>, x: i16, y: i16) -> [i16; 4] {
    current.map_or([x, y, x, y], |bounds| {
        [
            bounds[0].min(x),
            bounds[1].min(y),
            bounds[2].max(x),
            bounds[3].max(y),
        ]
    })
}

fn bbox_contains_i16_envelope(bbox: [i16; 4], envelope: [i16; 4]) -> bool {
    bbox[2] <= envelope[0]
        && bbox[1] <= envelope[1]
        && bbox[3] >= envelope[2]
        && bbox[0] >= envelope[3]
}

fn bbox_contains_f64_envelope(bbox: [i16; 4], envelope: [f64; 4]) -> bool {
    f64::from(bbox[2]) <= envelope[0]
        && f64::from(bbox[1]) <= envelope[1]
        && f64::from(bbox[3]) >= envelope[2]
        && f64::from(bbox[0]) >= envelope[3]
}

fn push_sample(samples: &mut Vec<String>, sample: String) {
    const MAX_SAMPLES: usize = 16;
    let priority = sample.starts_with("node=95:") || sample.starts_with("node=96:");
    if priority {
        samples.insert(0, sample);
        samples.truncate(MAX_SAMPLES);
        return;
    }
    if samples.len() < MAX_SAMPLES {
        samples.push(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::{bbox_contains_f64_envelope, bbox_contains_i16_envelope};

    #[test]
    fn classic_bbox_order_contains_only_its_declared_representation() {
        let bbox = [-3392, -3552, 928, 1184];
        assert!(bbox_contains_i16_envelope(bbox, [928, -3552, 1184, -3392]));
        assert!(!bbox_contains_f64_envelope(
            bbox,
            [928.0, -3560.0, 1184.0, -3392.0]
        ));
    }
}
