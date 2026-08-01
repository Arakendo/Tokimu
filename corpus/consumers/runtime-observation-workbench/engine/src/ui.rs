use hello_runtime_observation::{
    ObservationEnvelope, PlaybackState, PresentationObservation, RuntimeInspectionAdapter,
};
use serde::Serialize;
use ui_tools::{
    consumer::{
        UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiLayoutFit, UiNodeId, UiNodeKind,
        UiNodeLayout, UiNodeSpec, UiRect, UiRegionKind, UiResolvedNode, UiSurfaceRole, UiTextAlign,
        UiTextRole, UiTextSpec, UiTheme, UiTree, UiUniformGridLayout,
    },
    lowering::lower_resolved_tree_to_draw_list,
};

const UI_SNAPSHOT_SCHEMA: &str = "tokimu.corpus.runtime-observation-ui";
const UI_SNAPSHOT_VERSION: u16 = 1;

#[derive(Debug, Serialize)]
pub struct RuntimeUiSnapshot {
    schema: &'static str,
    version: u16,
    sequence: u64,
    selected_entity: Option<u64>,
    viewport: [u32; 2],
    layout_fit: &'static str,
    observation: ObservationEvidence,
    presentation: PresentationEvidence,
    playback: PlaybackEvidence,
    ui: UiEvidence,
}

#[derive(Debug, Serialize)]
struct ObservationEvidence {
    revision: u64,
    tick: u64,
    entity_count: usize,
    selected_resolved: bool,
    diagnostics: usize,
}

#[derive(Debug, Serialize)]
struct PresentationEvidence {
    mappings: usize,
    targets: usize,
    diagnostics: usize,
}

#[derive(Debug, Serialize)]
struct PlaybackEvidence {
    selected_clip: usize,
    mode: String,
    local_time_seconds: f32,
}

#[derive(Debug, Serialize)]
struct UiEvidence {
    nodes: Vec<UiNodeEvidence>,
    tree_diagnostics: Vec<String>,
    draw_diagnostics: Vec<String>,
    draw_entries: u32,
    surface_commands: u32,
    text_commands: u32,
    clip_pairs: u32,
    structural_fingerprint: u64,
}

#[derive(Debug, Serialize)]
struct UiNodeEvidence {
    id: u64,
    kind: String,
    label: Option<String>,
    value: Option<String>,
    center: [f32; 2],
    size: [f32; 2],
    layout_fit: String,
    visible: bool,
    enabled: bool,
}

pub fn build_runtime_ui_snapshot(
    runtime: &RuntimeInspectionAdapter,
    viewport: [u32; 2],
    sequence: u64,
    selected_entity: Option<u64>,
) -> Result<RuntimeUiSnapshot, String> {
    if viewport[0] == 0 || viewport[1] == 0 {
        return Err("runtime UI viewport dimensions must be greater than zero".to_owned());
    }

    let observation = runtime.observe_entity_id(
        sequence,
        selected_entity,
        hello_runtime_observation::ObservationLimits::default(),
    );
    let presentation = runtime.presentation();
    let playback = runtime.playback();
    let (tree, tree_viewport, layout_fit) =
        build_tree(viewport, &observation, &presentation, playback);
    let resolved = tree
        .resolve(tree_viewport)
        .map_err(|error| format!("runtime UI resolution failed: {error:?}"))?;
    let draw_list =
        lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), observation.revision);
    let statistics = draw_list.statistics();
    let mut nodes = Vec::new();
    collect_nodes(&resolved.root, &mut nodes);

    Ok(RuntimeUiSnapshot {
        schema: UI_SNAPSHOT_SCHEMA,
        version: UI_SNAPSHOT_VERSION,
        sequence,
        selected_entity,
        viewport,
        layout_fit: fit_name(layout_fit),
        observation: ObservationEvidence {
            revision: observation.revision,
            tick: observation.tick,
            entity_count: observation.payload.entity_count,
            selected_resolved: observation.payload.selected.is_some(),
            diagnostics: observation.payload.diagnostics.len(),
        },
        presentation: PresentationEvidence {
            mappings: presentation.mappings.len(),
            targets: presentation.targets.len(),
            diagnostics: presentation.diagnostics.len(),
        },
        playback: PlaybackEvidence {
            selected_clip: playback.selected_clip,
            mode: format!("{:?}", playback.mode).to_lowercase(),
            local_time_seconds: playback.local_time_seconds,
        },
        ui: UiEvidence {
            nodes,
            tree_diagnostics: resolved
                .diagnostics
                .iter()
                .map(|diagnostic| format!("node {}: {:?}", diagnostic.node.0, diagnostic.kind))
                .collect(),
            draw_diagnostics: draw_list
                .diagnostics
                .iter()
                .map(|diagnostic| format!("{:?}: {:?}", diagnostic.source, diagnostic.kind))
                .collect(),
            draw_entries: statistics.entries,
            surface_commands: statistics.surfaces,
            text_commands: statistics.text,
            clip_pairs: statistics.clip_pushes.min(statistics.clip_pops),
            structural_fingerprint: draw_list.structural_fingerprint(),
        },
    })
}

fn build_tree(
    viewport: [u32; 2],
    observation: &ObservationEnvelope,
    presentation: &PresentationObservation,
    playback: &PlaybackState,
) -> (UiTree, UiRect, UiLayoutFit) {
    let frame = UiFrameLayout::for_window(
        [viewport[0] as f32, viewport[1] as f32],
        UiInsets::uniform(0.06),
        0.16,
        0.18,
        0.035,
    );
    let panes = UiHorizontalSplitLayout::new(frame.body, 0.5, 0.04, 0.72, 0.72);
    let root_id = UiNodeId(1);
    let header_id = UiNodeId(2);
    let body_id = UiNodeId(3);
    let observation_id = UiNodeId(4);
    let presentation_id = UiNodeId(5);
    let footer_id = UiNodeId(6);
    let mut next_id = 10;

    let header = region_node(
        header_id,
        root_id,
        UiRegionKind::Header,
        UiSurfaceRole::Accent,
        frame.header,
        "runtime observation header",
    )
    .with_child(text_node(
        take_id(&mut next_id),
        header_id,
        "RUNTIME OBSERVATION",
        frame.header.inset(0.05),
        UiTextRole::Title,
    ));

    let observation_lines = [
        "WORLD OBSERVATION".to_owned(),
        format!(
            "REVISION: {}  TICK: {}",
            observation.revision, observation.tick
        ),
        format!("ENTITIES: {}", observation.payload.entity_count),
        format!(
            "SELECTED: {}",
            observation
                .payload
                .selected
                .as_ref()
                .map(|selected| selected.entity.to_string())
                .unwrap_or_else(|| "NONE".to_owned())
        ),
    ];
    let presentation_lines = [
        "PRESENTATION + PLAYBACK".to_owned(),
        format!("TARGETS: {}", presentation.targets.len()),
        format!("CLIP: {}", playback.selected_clip),
        format!("MODE: {:?}", playback.mode).to_uppercase(),
    ];

    let observation_pane = line_panel(
        observation_id,
        body_id,
        &mut next_id,
        panes.leading,
        "world observation pane",
        &observation_lines,
    );
    let presentation_pane = line_panel(
        presentation_id,
        body_id,
        &mut next_id,
        panes.trailing,
        "presentation pane",
        &presentation_lines,
    );
    let body = region_node(
        body_id,
        root_id,
        UiRegionKind::Workspace,
        UiSurfaceRole::Region,
        frame.body,
        "runtime observation body",
    )
    .with_child(observation_pane)
    .with_child(presentation_pane);

    let footer = region_node(
        footer_id,
        root_id,
        UiRegionKind::StatusBar,
        UiSurfaceRole::Panel,
        frame.footer,
        "runtime observation status",
    )
    .with_semantic_value(fit_name(most_severe_fit(frame.fit, panes.fit)))
    .with_child(text_node(
        take_id(&mut next_id),
        footer_id,
        format!(
            "{} TREE / {} OBSERVATION DIAGNOSTICS",
            fit_name(most_severe_fit(frame.fit, panes.fit)).to_uppercase(),
            observation.payload.diagnostics.len()
        ),
        frame.footer.inset(0.05),
        UiTextRole::Caption,
    ));

    let root = region_node(
        root_id,
        root_id,
        UiRegionKind::Panel,
        UiSurfaceRole::Panel,
        frame.content,
        "runtime observation workbench",
    )
    .with_child(header)
    .with_child(body)
    .with_child(footer);

    (
        UiTree::new(root),
        frame.viewport,
        most_severe_fit(frame.fit, panes.fit),
    )
}

fn line_panel(
    id: UiNodeId,
    parent: UiNodeId,
    next_id: &mut u64,
    bounds: UiRect,
    label: &str,
    lines: &[String],
) -> UiNodeSpec {
    let content = bounds.inset(0.07);
    let grid = UiUniformGridLayout::new(content, lines.len(), 1, [0.0, 0.025]);
    let mut panel = region_node(
        id,
        parent,
        UiRegionKind::Inspector,
        UiSurfaceRole::Panel,
        bounds,
        label,
    );
    for (line, cell) in lines.iter().zip(grid.cells) {
        panel = panel.with_child(text_node(
            take_id(next_id),
            id,
            line.clone(),
            cell,
            UiTextRole::Body,
        ));
    }
    panel
}

fn region_node(
    id: UiNodeId,
    parent: UiNodeId,
    kind: UiRegionKind,
    role: UiSurfaceRole,
    bounds: UiRect,
    label: &str,
) -> UiNodeSpec {
    let node = UiNodeSpec::new(
        id,
        UiNodeKind::Region(kind),
        role,
        UiNodeLayout::Explicit(bounds),
    )
    .with_semantic_label(label);
    if id == parent {
        node
    } else {
        node.with_parent(parent)
    }
}

fn text_node(
    id: UiNodeId,
    parent: UiNodeId,
    text: impl Into<String>,
    bounds: UiRect,
    role: UiTextRole,
) -> UiNodeSpec {
    let text =
        UiTextSpec::new(text, bounds, role).with_alignment(UiTextAlign::Start, UiTextAlign::Center);
    UiNodeSpec::text(id, &text)
        .with_parent(parent)
        .with_semantic_label(text.text.clone())
}

fn collect_nodes(node: &UiResolvedNode, output: &mut Vec<UiNodeEvidence>) {
    output.push(UiNodeEvidence {
        id: node.id.0,
        kind: format!("{:?}", node.kind),
        label: node.semantic_label.clone(),
        value: node.semantic_value.clone(),
        center: node.bounds.center,
        size: node.bounds.size,
        layout_fit: fit_name(node.layout_fit).to_owned(),
        visible: node.visible,
        enabled: node.enabled,
    });
    for child in &node.children {
        collect_nodes(child, output);
    }
}

fn most_severe_fit(first: UiLayoutFit, second: UiLayoutFit) -> UiLayoutFit {
    match (first, second) {
        (UiLayoutFit::Impossible, _) | (_, UiLayoutFit::Impossible) => UiLayoutFit::Impossible,
        (UiLayoutFit::Overflow, _) | (_, UiLayoutFit::Overflow) => UiLayoutFit::Overflow,
        (UiLayoutFit::Adjusted, _) | (_, UiLayoutFit::Adjusted) => UiLayoutFit::Adjusted,
        _ => UiLayoutFit::Exact,
    }
}

fn fit_name(fit: UiLayoutFit) -> &'static str {
    match fit {
        UiLayoutFit::Exact => "fit",
        UiLayoutFit::Adjusted => "compact",
        UiLayoutFit::Overflow => "overflow",
        UiLayoutFit::Impossible => "impossible",
    }
}

fn take_id(next_id: &mut u64) -> UiNodeId {
    let id = UiNodeId(*next_id);
    *next_id += 1;
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use hello_runtime_observation::verified_hole_punch_catalog_fixture;

    fn runtime() -> RuntimeInspectionAdapter {
        RuntimeInspectionAdapter::from_animation_catalog(16, verified_hole_punch_catalog_fixture())
            .expect("checked observation fixture should build")
    }

    #[test]
    fn snapshot_is_bounded_and_finite_across_browser_viewports() {
        let runtime = runtime();
        for viewport in [[1280, 720], [900, 600], [640, 480], [320, 568]] {
            let snapshot =
                build_runtime_ui_snapshot(&runtime, viewport, 7, Some(runtime.arm_id().0))
                    .expect("semantic UI should resolve");

            assert!(!snapshot.ui.nodes.is_empty());
            assert!(snapshot.ui.nodes.iter().all(|node| {
                node.center.into_iter().all(f32::is_finite)
                    && node
                        .size
                        .into_iter()
                        .all(|value| value.is_finite() && value >= 0.0)
            }));
            assert!(snapshot.ui.draw_entries > 0);
            assert!(snapshot.ui.surface_commands >= 6);
        }
    }

    #[test]
    fn equivalent_ui_snapshots_are_deterministic() {
        let runtime = runtime();
        let first = build_runtime_ui_snapshot(&runtime, [900, 600], 3, Some(runtime.arm_id().0))
            .expect("first snapshot should resolve");
        let second = build_runtime_ui_snapshot(&runtime, [900, 600], 3, Some(runtime.arm_id().0))
            .expect("second snapshot should resolve");

        assert_eq!(
            serde_json::to_string(&first).expect("snapshot should serialize"),
            serde_json::to_string(&second).expect("snapshot should serialize")
        );
    }

    #[test]
    fn snapshot_preserves_runtime_observation_identity() {
        let runtime = runtime();
        let snapshot =
            build_runtime_ui_snapshot(&runtime, [900, 600], 11, Some(runtime.arm_id().0))
                .expect("snapshot should resolve");
        let observation = runtime.observe_entity_id(
            11,
            Some(runtime.arm_id().0),
            hello_runtime_observation::ObservationLimits::default(),
        );

        assert_eq!(snapshot.sequence, observation.sequence);
        assert_eq!(snapshot.observation.revision, observation.revision);
        assert_eq!(snapshot.observation.tick, observation.tick);
        assert_eq!(
            snapshot.observation.entity_count,
            observation.payload.entity_count
        );
        assert!(snapshot.observation.selected_resolved);
    }
}
