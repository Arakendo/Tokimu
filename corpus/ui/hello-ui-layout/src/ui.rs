use ui_tools::consumer::{
    UiNodeId, UiNodeKind, UiNodeLayout, UiNodeSpec, UiTextAlign, UiTextOverflow, UiTextRole,
    UiTextSpec, UiTree,
};
use ui_tools::{UiRect, UiRegion, UiSurfaceRole, UiTheme, UiWorkspaceLayout};

pub struct UiLayoutScene {
    pub tree: UiTree,
    pub viewport: ui_tools::UiRect,
}

pub fn build_layout_scene(window_size: [f32; 2], theme: &UiTheme) -> UiLayoutScene {
    let width = window_size[0].max(1.0);
    let height = window_size[1].max(1.0);
    let viewport = UiRect::new([0.0, 0.0], [2.0 * width / height, 2.0]);
    let layout = UiWorkspaceLayout::new_with_theme(
        window_size,
        [
            ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(0), "HEADER"),
            ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(1), "WORKSPACE"),
            ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(2), "STATUS"),
        ],
        [
            ui_tools::UiCardSpec::new(
                ui_tools::UiCardRole::Browser,
                "Sidebar",
                "FILTERS + NAVIGATION",
            ),
            ui_tools::UiCardSpec::new(ui_tools::UiCardRole::Editor, "Canvas", "MAIN CONTENT AREA"),
            ui_tools::UiCardSpec::new(
                ui_tools::UiCardRole::Inspector,
                "Inspector",
                "PROPERTIES + STATE",
            ),
        ],
        theme,
    );

    let root_id = UiNodeId(1);
    let regions = [
        (&layout.header, "HEADER", UiTextRole::Heading),
        (&layout.toolbar, "TOOLBAR", UiTextRole::Caption),
        (&layout.sidebar, "SIDEBAR", UiTextRole::Caption),
        (&layout.canvas, "WORKSPACE", UiTextRole::Heading),
        (&layout.inspector, "INSPECTOR", UiTextRole::Caption),
        (&layout.card_grid, "CARD GRID", UiTextRole::Caption),
        (&layout.status_bar, "STATUS", UiTextRole::Caption),
    ];
    let children = regions
        .into_iter()
        .enumerate()
        .map(|(index, (region, label, role))| {
            region_node(root_id, UiNodeId(index as u64 + 2), region, label, role)
        });

    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(layout.workspace.kind),
            UiSurfaceRole::Background,
            UiNodeLayout::Explicit(viewport),
        )
        .with_semantic_label("layout corpus workspace")
        .with_children(children),
    );

    UiLayoutScene { tree, viewport }
}

fn region_node(
    parent: UiNodeId,
    id: UiNodeId,
    region: &UiRegion,
    label: &str,
    role: UiTextRole,
) -> UiNodeSpec {
    let text_id = UiNodeId(id.0 + 100);
    let text = UiTextSpec::new(label, region.rect, role)
        .with_alignment(UiTextAlign::Center, UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Ellipsis);

    UiNodeSpec::new(
        id,
        UiNodeKind::Region(region.kind),
        region.role,
        UiNodeLayout::Explicit(region.rect),
    )
    .with_parent(parent)
    .with_semantic_label(label.to_lowercase())
    .with_child(UiNodeSpec::text(text_id, &text).with_parent(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_tools::lower_resolved_tree_to_draw_list;

    #[test]
    fn layout_scene_has_one_deterministic_semantic_lowering_path() {
        let theme = UiTheme::default();
        let scene = build_layout_scene([1360.0, 840.0], &theme);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let first = lower_resolved_tree_to_draw_list(&resolved, &theme, 1);
        let second = lower_resolved_tree_to_draw_list(&resolved, &theme, 2);

        assert!(
            resolved.diagnostics.is_empty(),
            "unexpected layout diagnostics: {:#?}",
            resolved.diagnostics
        );
        assert_eq!(first.statistics().surfaces, 8);
        assert_eq!(first.statistics().text, 7);
        assert_eq!(
            first.structural_fingerprint(),
            second.structural_fingerprint()
        );
    }

    #[test]
    fn layout_scene_stays_bounded_across_supported_viewports() {
        let theme = UiTheme::default();

        for viewport in [[1360.0, 840.0], [1024.0, 768.0], [840.0, 840.0]] {
            let scene = build_layout_scene(viewport, &theme);
            let resolved = scene.tree.resolve(scene.viewport).unwrap();

            assert!(
                resolved.diagnostics.is_empty(),
                "unexpected diagnostics at {viewport:?}: {:#?}",
                resolved.diagnostics
            );
            assert!(resolved.root.bounds.size[0].is_finite());
            assert!(resolved.root.bounds.size[1].is_finite());
            assert!(resolved.root.bounds.size[0] > 0.0);
            assert!(resolved.root.bounds.size[1] > 0.0);
        }
    }
}
