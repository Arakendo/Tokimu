use ui_tools::consumer::{
    UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiLayoutFit, UiNodeId, UiNodeKind,
    UiNodeLayout, UiNodeSpec, UiRect, UiRegionKind, UiSurfaceRole, UiTextAlign, UiTextOverflow,
    UiTextRole, UiTextSpec, UiTree, UiUniformGridLayout,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeInspectorView {
    pub world_lines: Vec<String>,
    pub presentation_lines: Vec<String>,
    pub command_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeInspectorScene {
    pub tree: UiTree,
    pub viewport: UiRect,
    pub fit: UiLayoutFit,
}

pub fn build_runtime_inspector_scene(
    window_size: [f32; 2],
    view: &RuntimeInspectorView,
) -> RuntimeInspectorScene {
    let frame = UiFrameLayout::for_window(window_size, UiInsets::uniform(0.08), 0.15, 0.24, 0.035);
    let panes = UiHorizontalSplitLayout::new(frame.body, 0.5, 0.05, 0.85, 0.85);
    let footer = UiHorizontalSplitLayout::new(frame.footer.inset(0.05), 0.55, 0.05, 0.75, 0.60);
    let fit = most_severe_fit([frame.fit, panes.fit, footer.fit]);
    let layout_label = layout_status([frame.fit, panes.fit, footer.fit]);

    let root_id = UiNodeId(1);
    let header_id = UiNodeId(2);
    let body_id = UiNodeId(3);
    let footer_id = UiNodeId(4);
    let mut next_id = 10;

    let header = UiNodeSpec::new(
        header_id,
        UiNodeKind::Region(UiRegionKind::Header),
        UiSurfaceRole::Accent,
        UiNodeLayout::Explicit(frame.header),
    )
    .with_parent(root_id)
    .with_semantic_label("runtime inspector header")
    .with_child(text_node(
        header_id,
        take_id(&mut next_id),
        "RUNTIME OBSERVATION INSPECTOR",
        UiTextRole::Title,
        frame.header.inset(0.05),
    ))
    .with_child(text_node(
        header_id,
        take_id(&mut next_id),
        layout_label,
        UiTextRole::Caption,
        UiRect::new(
            [
                frame.header.center[0] + frame.header.size[0] * 0.5 - 0.38,
                frame.header.center[1],
            ],
            [0.28, frame.header.size[1] - 0.06],
        ),
    ));

    let body = if fit == UiLayoutFit::Exact {
        let leading_id = take_id(&mut next_id);
        let trailing_id = take_id(&mut next_id);
        UiNodeSpec::new(
            body_id,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Explicit(frame.body),
        )
        .with_parent(root_id)
        .with_semantic_label("runtime observation content")
        .with_child(line_panel(
            body_id,
            leading_id,
            &mut next_id,
            panes.leading,
            "world observation",
            &view.world_lines,
        ))
        .with_child(line_panel(
            body_id,
            trailing_id,
            &mut next_id,
            panes.trailing,
            "presentation and playback",
            &view.presentation_lines,
        ))
    } else {
        compact_body(body_id, root_id, &mut next_id, frame.body, view)
    };

    let footer = footer_node(footer_id, root_id, &mut next_id, frame.footer, footer, view);

    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(frame.content),
        )
        .with_semantic_label("runtime observation inspector")
        .with_child(header)
        .with_child(body)
        .with_child(footer),
    );

    RuntimeInspectorScene {
        tree,
        viewport: frame.viewport,
        fit,
    }
}

pub fn layout_status(fits: impl IntoIterator<Item = UiLayoutFit>) -> &'static str {
    match most_severe_fit(fits) {
        UiLayoutFit::Exact => "LAYOUT: FIT",
        UiLayoutFit::Adjusted => "LAYOUT: COMPACT",
        UiLayoutFit::Overflow => "LAYOUT: OVERFLOW",
        UiLayoutFit::Impossible => "LAYOUT: IMPOSSIBLE",
    }
}

fn most_severe_fit(fits: impl IntoIterator<Item = UiLayoutFit>) -> UiLayoutFit {
    fits.into_iter()
        .fold(UiLayoutFit::Exact, |current, next| match (current, next) {
            (UiLayoutFit::Impossible, _) | (_, UiLayoutFit::Impossible) => UiLayoutFit::Impossible,
            (UiLayoutFit::Overflow, _) | (_, UiLayoutFit::Overflow) => UiLayoutFit::Overflow,
            (UiLayoutFit::Adjusted, _) | (_, UiLayoutFit::Adjusted) => UiLayoutFit::Adjusted,
            _ => UiLayoutFit::Exact,
        })
}

fn compact_body(
    id: UiNodeId,
    parent: UiNodeId,
    next_id: &mut u64,
    bounds: UiRect,
    view: &RuntimeInspectorView,
) -> UiNodeSpec {
    let content = bounds.inset(0.08);
    let lines = [
        ("COMPACT OBSERVATION VIEW".to_owned(), UiTextRole::Heading),
        (
            "FULL DETAIL REQUIRES A LARGER VIEWPORT".to_owned(),
            UiTextRole::Body,
        ),
        (
            view.world_lines
                .get(1..4)
                .unwrap_or(&view.world_lines)
                .join("   "),
            UiTextRole::Caption,
        ),
    ];
    let grid = UiUniformGridLayout::new(content, lines.len(), 1, [0.0, 0.025]);
    let children = lines
        .into_iter()
        .zip(grid.cells)
        .map(|((text, role), cell)| text_node(id, take_id(next_id), &text, role, cell));

    UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label("compact runtime observation")
    .with_children(children)
}

fn line_panel(
    parent: UiNodeId,
    id: UiNodeId,
    next_id: &mut u64,
    bounds: UiRect,
    label: &str,
    lines: &[String],
) -> UiNodeSpec {
    let content = bounds.inset(0.05);
    let gap = if lines.len() > 1 { 0.025 } else { 0.0 };
    let grid = UiUniformGridLayout::new(content, lines.len().max(1), 1, [0.0, gap]);
    let children = lines
        .iter()
        .zip(grid.cells)
        .enumerate()
        .map(|(index, (line, cell))| {
            text_node(
                id,
                take_id(next_id),
                line,
                if index == 0 {
                    UiTextRole::Heading
                } else {
                    UiTextRole::Body
                },
                cell,
            )
        });

    UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Inspector),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label(label)
    .with_children(children)
}

fn footer_node(
    id: UiNodeId,
    parent: UiNodeId,
    next_id: &mut u64,
    bounds: UiRect,
    split: UiHorizontalSplitLayout,
    view: &RuntimeInspectorView,
) -> UiNodeSpec {
    let divider = UiRect::new(
        [
            bounds.center[0],
            bounds.center[1] + bounds.size[1] * 0.5 - 0.018,
        ],
        [bounds.size[0] - 0.10, 0.008],
    );
    let command_id = take_id(next_id);
    let diagnostic_id = take_id(next_id);
    let command_grid = UiUniformGridLayout::new(
        split.leading,
        view.command_lines.len().max(1),
        1,
        [0.0, 0.015],
    );
    let diagnostic_grid = UiUniformGridLayout::new(
        split.trailing,
        view.diagnostic_lines.len().max(1),
        1,
        [0.0, 0.015],
    );
    let command_children: Vec<_> = view
        .command_lines
        .iter()
        .zip(command_grid.cells)
        .enumerate()
        .map(|(index, (line, cell))| {
            text_node(
                command_id,
                take_id(next_id),
                line,
                if index == 0 {
                    UiTextRole::Heading
                } else {
                    UiTextRole::Caption
                },
                cell,
            )
        })
        .collect();
    let diagnostic_children: Vec<_> = view
        .diagnostic_lines
        .iter()
        .zip(diagnostic_grid.cells)
        .map(|(line, cell)| {
            text_node(
                diagnostic_id,
                take_id(next_id),
                line,
                UiTextRole::Caption,
                cell,
            )
        })
        .collect();

    UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::StatusBar),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label("runtime inspector commands and diagnostics")
    .with_child(
        UiNodeSpec::new(
            take_id(next_id),
            UiNodeKind::Region(UiRegionKind::StatusBar),
            UiSurfaceRole::Accent,
            UiNodeLayout::Explicit(divider),
        )
        .with_parent(id)
        .with_semantic_label("footer divider"),
    )
    .with_child(
        UiNodeSpec::new(
            command_id,
            UiNodeKind::Region(UiRegionKind::Toolbar),
            UiSurfaceRole::Background,
            UiNodeLayout::Explicit(split.leading),
        )
        .with_parent(id)
        .with_semantic_label("commands")
        .with_children(command_children),
    )
    .with_child(
        UiNodeSpec::new(
            diagnostic_id,
            UiNodeKind::Region(UiRegionKind::StatusBar),
            UiSurfaceRole::Background,
            UiNodeLayout::Explicit(split.trailing),
        )
        .with_parent(id)
        .with_semantic_label("diagnostics")
        .with_children(diagnostic_children),
    )
}

fn text_node(
    parent: UiNodeId,
    id: UiNodeId,
    text: &str,
    role: UiTextRole,
    bounds: UiRect,
) -> UiNodeSpec {
    UiNodeSpec::text(
        id,
        &UiTextSpec::new(text, bounds, role)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
            .with_overflow(UiTextOverflow::Ellipsis),
    )
    .with_parent(parent)
}

fn take_id(next_id: &mut u64) -> UiNodeId {
    let id = UiNodeId(*next_id);
    *next_id = (*next_id).saturating_add(1);
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_tools::{lower_resolved_tree_to_draw_list, UiTheme};

    fn view() -> RuntimeInspectorView {
        RuntimeInspectorView {
            world_lines: vec![
                "WORLD OBSERVATION".to_owned(),
                "SELECTED ENTITY: 1".to_owned(),
                "REVISION: 4 TICK: 8".to_owned(),
                "ENTITIES: 2".to_owned(),
                "RELATION TYPES: 1".to_owned(),
                "DETAIL: 2 COMPONENTS / 0 RELATIONS".to_owned(),
                "OUTGOING EDGES: 0".to_owned(),
                "ENABLED: TRUE".to_owned(),
                "POSITION: 2.0, 1.0, 0.5".to_owned(),
            ],
            presentation_lines: vec![
                "PRESENTATION + PLAYBACK".to_owned(),
                "PRESENTATION: SELECTED".to_owned(),
                "RESOLVED TARGETS: 1".to_owned(),
                "CLIP: STEP 1 (1/5)".to_owned(),
                "MODE: STOPPED".to_owned(),
                "LOCAL TIME: 0.00 S".to_owned(),
                "CATALOG: 5 CLIPS".to_owned(),
            ],
            command_lines: vec![
                "COMMANDS".to_owned(),
                "LEFT/RIGHT SELECT D MOVE E DISABLE X REJECT".to_owned(),
                "SPACE APPLY R TARGET A CLIP S PLAY".to_owned(),
            ],
            diagnostic_lines: vec![
                "DIAGNOSTICS: NONE".to_owned(),
                "LAST COMMAND: NONE".to_owned(),
            ],
        }
    }

    #[test]
    fn inspector_scene_resolves_without_parallel_draw_geometry() {
        let scene = build_runtime_inspector_scene([1200.0, 760.0], &view());
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let semantics = resolved.semantic_nodes(&Default::default());
        let first = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 7);
        let second = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 8);

        assert_eq!(scene.fit, UiLayoutFit::Exact);
        assert!(resolved.diagnostics.is_empty());
        assert!(!first.entries().is_empty());
        assert!(first.diagnostics.is_empty());
        assert_eq!(
            first.structural_fingerprint(),
            second.structural_fingerprint()
        );
        assert!(semantics
            .iter()
            .any(|node| node.label.as_deref() == Some("WORLD OBSERVATION")));
        assert!(semantics
            .iter()
            .any(|node| node.label.as_deref() == Some("COMMANDS")));
    }

    #[test]
    fn inspector_scene_uses_a_bounded_fallback_for_small_windows() {
        let mut observed_compact_layout = false;
        for size in [[900.0, 600.0], [640.0, 480.0], [320.0, 568.0]] {
            let scene = build_runtime_inspector_scene(size, &view());
            let resolved = scene.tree.resolve(scene.viewport).unwrap();
            assert!(resolved.diagnostics.is_empty());

            if scene.fit != UiLayoutFit::Exact {
                observed_compact_layout = true;
                assert!(resolved
                    .semantic_nodes(&Default::default())
                    .iter()
                    .any(|node| node.label.as_deref() == Some("COMPACT OBSERVATION VIEW")));
            }
        }
        assert!(observed_compact_layout);
    }

    #[test]
    fn layout_status_reports_the_most_severe_fit() {
        assert_eq!(layout_status([UiLayoutFit::Exact]), "LAYOUT: FIT");
        assert_eq!(
            layout_status([UiLayoutFit::Exact, UiLayoutFit::Adjusted]),
            "LAYOUT: COMPACT"
        );
        assert_eq!(
            layout_status([UiLayoutFit::Adjusted, UiLayoutFit::Overflow]),
            "LAYOUT: OVERFLOW"
        );
        assert_eq!(
            layout_status([UiLayoutFit::Overflow, UiLayoutFit::Impossible]),
            "LAYOUT: IMPOSSIBLE"
        );
    }
}
