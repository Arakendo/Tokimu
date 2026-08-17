//! Inventory and conservation evidence for source-topology admission.
//!
//! Records in this module refer to original E1M1 draw indices and mesh handles.
//! They never own reconstructed geometry and never cross into `tokimu-render`.

use std::collections::{BTreeMap, BTreeSet};

use doom_geometry_provider::{DoomSurfacePlane, DoomWallTextureRole};
use hello_doom_e1m1::{StaticDrawPlanEntry, StaticDrawSource};
use tokimu::{Mesh, MeshHandle};

use crate::{DoomLineActivationSource, DIAGNOSTIC_SKY_MESH_BASE};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum TopologyContributionFamily {
    Floor,
    Ceiling,
    WallUpper,
    WallLower,
    WallMiddle,
    CutoutMiddle,
    SkyPlane,
}

impl TopologyContributionFamily {
    const fn label(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Ceiling => "ceiling",
            Self::WallUpper => "wall-upper",
            Self::WallLower => "wall-lower",
            Self::WallMiddle => "wall-middle",
            Self::CutoutMiddle => "cutout-middle",
            Self::SkyPlane => "sky-plane",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TopologyContributionDomain {
    Opaque,
    Cutout,
    DiagnosticSky,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopologyContributionRecord {
    pub(crate) domain: TopologyContributionDomain,
    pub(crate) draw_index: usize,
    pub(crate) mesh_handle: MeshHandle,
    pub(crate) family: TopologyContributionFamily,
    pub(crate) source: StaticDrawSource,
    pub(crate) runtime_related: bool,
    pub(crate) mesh_hash: u64,
    pub(crate) contribution_hash: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TopologyContributionInventory {
    pub(crate) records: Vec<TopologyContributionRecord>,
    pub(crate) presentation_global: usize,
    pub(crate) family_counts: BTreeMap<TopologyContributionFamily, usize>,
    pub(crate) runtime_related: usize,
    pub(crate) duplicate_samples: Vec<String>,
    pub(crate) aggregate_hash: u64,
}

impl TopologyContributionInventory {
    pub(crate) fn report(&self) -> String {
        let families = self
            .family_counts
            .iter()
            .map(|(family, count)| format!("{}={count}", family.label()))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "records={}; presentation_global={}; runtime_related={}; families=[{}]; aggregate_hash={:016x}; duplicate_samples=[{}]",
            self.records.len(),
            self.presentation_global,
            self.runtime_related,
            families,
            self.aggregate_hash,
            self.duplicate_samples.join(" | "),
        )
    }

    pub(crate) fn verify_unchanged(
        &self,
        opaque: &[StaticDrawPlanEntry],
        cutouts: &[StaticDrawPlanEntry],
        diagnostic_sky: &[StaticDrawPlanEntry],
    ) -> bool {
        self.records.iter().all(|record| {
            let draw = match record.domain {
                TopologyContributionDomain::Opaque => opaque.get(record.draw_index),
                TopologyContributionDomain::Cutout => cutouts.get(record.draw_index),
                TopologyContributionDomain::DiagnosticSky => diagnostic_sky.get(record.draw_index),
            };
            draw.is_some_and(|draw| {
                mesh_structural_hash(&draw.mesh) == record.mesh_hash
                    && contribution_structural_hash(draw) == record.contribution_hash
            })
        })
    }
}

pub(crate) fn build_original_contribution_inventory(
    opaque: &[StaticDrawPlanEntry],
    cutouts: &[StaticDrawPlanEntry],
    diagnostic_sky: &[StaticDrawPlanEntry],
    cutout_mesh_base: u64,
    activation: &DoomLineActivationSource,
) -> TopologyContributionInventory {
    let runtime_sectors = runtime_related_sectors(activation);
    let runtime_linedefs = activation
        .linedefs
        .iter()
        .filter(|line| line.special != 0)
        .map(|line| line.source.record_index)
        .collect::<BTreeSet<_>>();
    let mut inventory = TopologyContributionInventory {
        // The panorama/background is presentation-global rather than a
        // topology-admitted map contribution.
        presentation_global: 1,
        ..TopologyContributionInventory::default()
    };
    let mut duplicate_keys = BTreeMap::<String, usize>::new();

    for (domain, draws, mesh_base) in [
        (TopologyContributionDomain::Opaque, opaque, 1_u64),
        (
            TopologyContributionDomain::Cutout,
            cutouts,
            cutout_mesh_base,
        ),
        (
            TopologyContributionDomain::DiagnosticSky,
            diagnostic_sky,
            DIAGNOSTIC_SKY_MESH_BASE,
        ),
    ] {
        for (draw_index, draw) in draws.iter().enumerate() {
            let family = contribution_family(domain, draw.source);
            let runtime_related =
                source_is_runtime_related(draw.source, &runtime_linedefs, &runtime_sectors);
            let mesh_hash = mesh_structural_hash(&draw.mesh);
            let contribution_hash = contribution_structural_hash(draw);
            let record = TopologyContributionRecord {
                domain,
                draw_index,
                mesh_handle: MeshHandle(mesh_base + draw_index as u64),
                family,
                source: draw.source,
                runtime_related,
                mesh_hash,
                contribution_hash,
            };
            *inventory.family_counts.entry(family).or_default() += 1;
            inventory.runtime_related += usize::from(runtime_related);
            inventory.aggregate_hash = hash_u64(inventory.aggregate_hash, contribution_hash);

            let key = source_key(family, draw.source);
            let occurrences = duplicate_keys.entry(key.clone()).or_default();
            *occurrences += 1;
            if *occurrences == 2 && inventory.duplicate_samples.len() < 12 {
                inventory.duplicate_samples.push(key);
            }
            inventory.records.push(record);
        }
    }
    inventory
}

fn contribution_family(
    domain: TopologyContributionDomain,
    source: StaticDrawSource,
) -> TopologyContributionFamily {
    if domain == TopologyContributionDomain::DiagnosticSky {
        return TopologyContributionFamily::SkyPlane;
    }
    match source {
        StaticDrawSource::Flat {
            plane: DoomSurfacePlane::Floor,
            ..
        } => TopologyContributionFamily::Floor,
        StaticDrawSource::Flat {
            plane: DoomSurfacePlane::Ceiling,
            ..
        } => TopologyContributionFamily::Ceiling,
        StaticDrawSource::Wall {
            role: DoomWallTextureRole::Upper,
            ..
        } => TopologyContributionFamily::WallUpper,
        StaticDrawSource::Wall {
            role: DoomWallTextureRole::Lower,
            ..
        } => TopologyContributionFamily::WallLower,
        StaticDrawSource::Wall {
            role: DoomWallTextureRole::Middle,
            ..
        } if domain == TopologyContributionDomain::Cutout => {
            TopologyContributionFamily::CutoutMiddle
        }
        StaticDrawSource::Wall {
            role: DoomWallTextureRole::Middle,
            ..
        } => TopologyContributionFamily::WallMiddle,
    }
}

fn source_is_runtime_related(
    source: StaticDrawSource,
    linedefs: &BTreeSet<u32>,
    sectors: &BTreeSet<u32>,
) -> bool {
    match source {
        StaticDrawSource::Flat { source_sector, .. } => {
            sectors.contains(&source_sector.record_index)
        }
        StaticDrawSource::Wall {
            source_linedef,
            source_sector,
            ..
        } => {
            linedefs.contains(&source_linedef.record_index)
                || sectors.contains(&source_sector.record_index)
        }
    }
}

fn runtime_related_sectors(activation: &DoomLineActivationSource) -> BTreeSet<u32> {
    let mut sectors = BTreeSet::new();
    for line in activation.linedefs.iter().filter(|line| line.special != 0) {
        if line.tag != 0 {
            sectors.extend(
                activation
                    .sectors
                    .iter()
                    .filter(|sector| sector.tag == line.tag)
                    .map(|sector| sector.source.record_index),
            );
        }
        if line.special == 1 {
            for sidedef_index in [line.right_sidedef, line.left_sidedef]
                .into_iter()
                .flatten()
            {
                if let Some(sidedef) = activation.sidedefs.get(usize::from(sidedef_index)) {
                    if let Some(sector) = activation.sectors.get(usize::from(sidedef.sector)) {
                        sectors.insert(sector.source.record_index);
                    }
                }
            }
        }
    }
    sectors
}

fn source_key(family: TopologyContributionFamily, source: StaticDrawSource) -> String {
    match source {
        StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            ..
        } => format!(
            "{}:subsector={}:sector={}",
            family.label(),
            source_subsector.record_index,
            source_sector.record_index
        ),
        StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            ..
        } => format!(
            "{}:linedef={}:sidedef={}:sector={}",
            family.label(),
            source_linedef.record_index,
            source_sidedef.record_index,
            source_sector.record_index
        ),
    }
}

fn mesh_structural_hash(mesh: &Mesh) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    hash = hash_u64(hash, mesh.positions.len() as u64);
    for position in &mesh.positions {
        for component in position {
            hash = hash_bytes(hash, &component.to_bits().to_le_bytes());
        }
    }
    hash = hash_u64(hash, mesh.normals.len() as u64);
    for normal in &mesh.normals {
        for component in normal {
            hash = hash_bytes(hash, &component.to_bits().to_le_bytes());
        }
    }
    hash = hash_u64(hash, mesh.texture_coordinates.len() as u64);
    for uv in &mesh.texture_coordinates {
        for component in uv {
            hash = hash_bytes(hash, &component.to_bits().to_le_bytes());
        }
    }
    hash
}

fn contribution_structural_hash(draw: &StaticDrawPlanEntry) -> u64 {
    let mut hash = mesh_structural_hash(&draw.mesh);
    hash = hash_u64(hash, draw.material.0);
    hash_bytes(hash, draw.source_label.as_bytes())
}

fn hash_u64(hash: u64, value: u64) -> u64 {
    hash_bytes(hash, &value.to_le_bytes())
}

fn hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use doom_geometry_provider::DoomSurfacePlane;
    use doom_map_provider::DoomSourceRecord;
    use hello_doom_e1m1::{StaticDrawPlanEntry, StaticDrawSource};
    use tokimu::{MaterialHandle, Mesh};

    use super::{build_original_contribution_inventory, DoomLineActivationSource};

    fn source(index: u32) -> DoomSourceRecord {
        DoomSourceRecord {
            lump_index: 1,
            record_index: index,
        }
    }

    #[test]
    fn all_fail_open_inventory_references_and_preserves_original_meshes() {
        let opaque = vec![StaticDrawPlanEntry {
            mesh: Mesh::triangle(),
            material: MaterialHandle(7),
            source_label: "flat:3:FLOOR".to_owned(),
            source: StaticDrawSource::Flat {
                source_subsector: source(2),
                source_sector: source(3),
                plane: DoomSurfacePlane::Floor,
            },
        }];
        let activation = DoomLineActivationSource {
            linedefs: Vec::new(),
            sidedefs: Vec::new(),
            sectors: Vec::new(),
        };
        let inventory = build_original_contribution_inventory(&opaque, &[], &[], 2, &activation);

        assert_eq!(inventory.records.len(), 1);
        assert_eq!(inventory.records[0].draw_index, 0);
        assert!(inventory.verify_unchanged(&opaque, &[], &[]));

        let mut changed = opaque.clone();
        changed[0].mesh.positions[0][0] += 1.0;
        assert!(!inventory.verify_unchanged(&changed, &[], &[]));
    }
}
