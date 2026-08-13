use ui_tools::consumer::{
    UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiLayoutFit, UiNodeId, UiNodeKind,
    UiNodeLayout, UiNodeSpec, UiRect, UiRegionKind, UiSurfaceRole, UiTextAlign, UiTextOverflow,
    UiTextRole, UiTextSpec, UiTree,
};

pub const SOURCE_PANE_ID: UiNodeId = UiNodeId(4);
pub const VECTOR_PANE_ID: UiNodeId = UiNodeId(5);

#[derive(Clone, Debug, PartialEq)]
pub struct CgmInspectionView {
    pub metafile_name: String,
    pub source_summary: String,
    pub picture_summary: String,
    pub coordinate_summary: String,
    pub vector_summary: String,
    pub lifecycle: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CgmInspectionScene {
    pub tree: UiTree,
    pub viewport: UiRect,
    pub fit: UiLayoutFit,
}

pub fn build_cgm_inspection_scene(
    window_size: [f32; 2],
    view: &CgmInspectionView,
) -> CgmInspectionScene {
    let frame = UiFrameLayout::for_window(window_size, UiInsets::uniform(0.07), 0.17, 0.13, 0.03);
    let panes = UiHorizontalSplitLayout::new(frame.body, 0.58, 0.035, 0.95, 0.68);
    let fit = most_severe_fit([frame.fit, panes.fit]);

    let root_id = UiNodeId(1);
    let header_id = UiNodeId(2);
    let body_id = UiNodeId(3);
    let footer_id = UiNodeId(6);

    let header = UiNodeSpec::new(
        header_id,
        UiNodeKind::Region(UiRegionKind::Header),
        UiSurfaceRole::Accent,
        UiNodeLayout::Explicit(frame.header),
    )
    .with_parent(root_id)
    .with_semantic_label("CGM inspection header")
    .with_child(text_node(
        header_id,
        UiNodeId(10),
        "CGM SOURCE + VECTOR INSPECTION",
        UiTextRole::Title,
        inset_horizontal(frame.header, 0.05),
        UiTextAlign::Start,
    ))
    .with_child(text_node(
        header_id,
        UiNodeId(11),
        &view.metafile_name,
        UiTextRole::Caption,
        UiRect::new(
            [
                frame.header.center[0] + frame.header.size[0] * 0.5 - 0.34,
                frame.header.center[1],
            ],
            [0.56, frame.header.size[1] - 0.05],
        ),
        UiTextAlign::End,
    ));

    let source = pane_node(
        body_id,
        SOURCE_PANE_ID,
        panes.leading,
        "SOURCE EVIDENCE",
        [
            (&view.source_summary, UiTextRole::Caption),
            (&view.picture_summary, UiTextRole::Caption),
            (&view.coordinate_summary, UiTextRole::Caption),
        ],
        20,
    );
    let vector = pane_node(
        body_id,
        VECTOR_PANE_ID,
        panes.trailing,
        "VECTOR DIAGNOSTIC",
        [
            (&view.vector_summary, UiTextRole::Caption),
            (
                "NEUTRAL OUTLINE; CGM PAINT SEMANTICS DEFERRED",
                UiTextRole::Caption,
            ),
            ("PRESS E TO CYCLE PRESENTATION", UiTextRole::Caption),
        ],
        30,
    );
    let body = UiNodeSpec::new(
        body_id,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Region,
        UiNodeLayout::Explicit(frame.body),
    )
    .with_parent(root_id)
    .with_semantic_label("CGM inspection content")
    .with_child(source)
    .with_child(vector);

    let footer = UiNodeSpec::new(
        footer_id,
        UiNodeKind::Region(UiRegionKind::StatusBar),
        UiSurfaceRole::Panel,
        UiNodeLayout::Explicit(frame.footer),
    )
    .with_parent(root_id)
    .with_semantic_label("CGM lifecycle footer")
    .with_child(text_node(
        footer_id,
        UiNodeId(40),
        &view.lifecycle,
        UiTextRole::Caption,
        upper_half(frame.footer.inset(0.035)),
        UiTextAlign::Center,
    ))
    .with_child(text_node(
        footer_id,
        UiNodeId(41),
        layout_status(fit),
        UiTextRole::Status,
        lower_half(frame.footer.inset(0.035)),
        UiTextAlign::Center,
    ));

    let tree = UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(frame.content),
        )
        .with_semantic_label("CGM source and vector inspection")
        .with_child(header)
        .with_child(body)
        .with_child(footer),
    );

    CgmInspectionScene {
        tree,
        viewport: frame.viewport,
        fit,
    }
}

fn pane_node<const N: usize>(
    parent: UiNodeId,
    id: UiNodeId,
    bounds: UiRect,
    heading: &str,
    lines: [(&str, UiTextRole); N],
    first_text_id: u64,
) -> UiNodeSpec {
    let top = bounds.center[1] + bounds.size[1] * 0.5;
    let heading_rect = UiRect::new(
        [bounds.center[0], top - 0.07],
        [bounds.size[0] - 0.10, 0.08],
    );
    let mut node = UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Inspector),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label(heading.to_lowercase())
    .with_child(text_node(
        id,
        UiNodeId(first_text_id),
        heading,
        UiTextRole::Heading,
        heading_rect,
        UiTextAlign::Center,
    ));

    for (index, (line, role)) in lines.into_iter().enumerate() {
        let rect = UiRect::new(
            [bounds.center[0], top - 0.15 - index as f32 * 0.055],
            [bounds.size[0] - 0.10, 0.045],
        );
        node = node.with_child(text_node(
            id,
            UiNodeId(first_text_id + index as u64 + 1),
            line,
            role,
            rect,
            UiTextAlign::Center,
        ));
    }
    node
}

fn text_node(
    parent: UiNodeId,
    id: UiNodeId,
    text: &str,
    role: UiTextRole,
    bounds: UiRect,
    horizontal: UiTextAlign,
) -> UiNodeSpec {
    let spec = UiTextSpec::new(text, bounds, role)
        .with_alignment(horizontal, UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Ellipsis);
    UiNodeSpec::text(id, &spec).with_parent(parent)
}

fn inset_horizontal(rect: UiRect, amount: f32) -> UiRect {
    UiRect::new(
        rect.center,
        [(rect.size[0] - amount * 2.0).max(0.0), rect.size[1]],
    )
}

fn upper_half(rect: UiRect) -> UiRect {
    UiRect::new(
        [rect.center[0], rect.center[1] + rect.size[1] * 0.25],
        [rect.size[0], rect.size[1] * 0.5],
    )
}

fn lower_half(rect: UiRect) -> UiRect {
    UiRect::new(
        [rect.center[0], rect.center[1] - rect.size[1] * 0.25],
        [rect.size[0], rect.size[1] * 0.5],
    )
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

fn layout_status(fit: UiLayoutFit) -> &'static str {
    match fit {
        UiLayoutFit::Exact => "LAYOUT FIT | SOURCE STATE + VECTOR LOWERING COMPLETE",
        UiLayoutFit::Adjusted => "COMPACT LAYOUT | SOURCE STATE + VECTOR LOWERING COMPLETE",
        UiLayoutFit::Overflow => "LAYOUT OVERFLOW | INSPECTION REMAINS DIAGNOSTIC",
        UiLayoutFit::Impossible => "LAYOUT IMPOSSIBLE | ENLARGE THE VIEWPORT",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_tools::{lower_resolved_tree_to_draw_list, UiTheme};

    fn view() -> CgmInspectionView {
        CgmInspectionView {
            metafile_name: "POLYLN01".into(),
            source_summary: "17072 BYTES | 50 ELEMENTS | 1 PICTURE".into(),
            picture_summary: "PICTURE: PICTURE 1".into(),
            coordinate_summary: "VDC: (0, 0) -> (1000, 1000)".into(),
            vector_summary: "8 PRIMITIVES | 8 OPEN | 0 CLOSED".into(),
            lifecycle: "BEGIN MF > BEGIN PIC > BODY > END PIC > END MF".into(),
        }
    }

    #[test]
    fn scene_exposes_bounded_domain_panes_across_supported_viewports() {
        for viewport in [[1120.0, 760.0], [1024.0, 768.0], [840.0, 840.0]] {
            let scene = build_cgm_inspection_scene(viewport, &view());
            let resolved = scene.tree.resolve(scene.viewport).unwrap();
            let source = resolved.node(SOURCE_PANE_ID).unwrap();
            let vector = resolved.node(VECTOR_PANE_ID).unwrap();

            assert!(
                resolved.diagnostics.is_empty(),
                "{viewport:?}: {:#?}",
                resolved.diagnostics
            );
            assert!(contains_rect(scene.viewport, source.bounds));
            assert!(contains_rect(scene.viewport, vector.bounds));
            assert!(source.bounds.size[0] > vector.bounds.size[0]);
        }
    }

    #[test]
    fn scene_lowers_deterministically() {
        let scene = build_cgm_inspection_scene([1120.0, 760.0], &view());
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let theme = UiTheme::default();
        let first = lower_resolved_tree_to_draw_list(&resolved, &theme, 1);
        let second = lower_resolved_tree_to_draw_list(&resolved, &theme, 2);

        assert!(!first.entries().is_empty());
        assert_eq!(
            first.structural_fingerprint(),
            second.structural_fingerprint()
        );
    }

    fn contains_rect(container: UiRect, child: UiRect) -> bool {
        let container_min = [
            container.center[0] - container.size[0] * 0.5,
            container.center[1] - container.size[1] * 0.5,
        ];
        let container_max = [
            container.center[0] + container.size[0] * 0.5,
            container.center[1] + container.size[1] * 0.5,
        ];
        let child_min = [
            child.center[0] - child.size[0] * 0.5,
            child.center[1] - child.size[1] * 0.5,
        ];
        let child_max = [
            child.center[0] + child.size[0] * 0.5,
            child.center[1] + child.size[1] * 0.5,
        ];
        child_min[0] >= container_min[0]
            && child_min[1] >= container_min[1]
            && child_max[0] <= container_max[0]
            && child_max[1] <= container_max[1]
    }
}
