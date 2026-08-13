use ui_tools::consumer::{
    UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiNodeId, UiNodeKind, UiNodeLayout,
    UiNodeSpec, UiRect, UiRegionKind, UiSurfaceRole, UiTextAlign, UiTextOverflow, UiTextRole,
    UiTextSpec, UiTree, UiUniformGridLayout,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchView {
    pub catalog_lines: Vec<String>,
    pub session_lines: Vec<String>,
    pub transcript_lines: Vec<String>,
    pub watch_lines: Vec<String>,
    pub control_lines: Vec<String>,
}

pub fn build_scene(window_size: [f32; 2], view: &WorkbenchView) -> UiTree {
    let frame = UiFrameLayout::for_window(window_size, UiInsets::uniform(0.07), 0.12, 0.22, 0.035);
    let top = UiHorizontalSplitLayout::new(frame.header.inset(0.04), 0.52, 0.04, 0.72, 0.72);
    let footer = UiHorizontalSplitLayout::new(frame.footer.inset(0.04), 0.48, 0.04, 0.65, 0.65);
    let root = UiNodeId(1);
    let mut next_id = 10;
    let title_id = take_id(&mut next_id);
    let catalog_id = take_id(&mut next_id);
    let session_id = take_id(&mut next_id);
    let transcript_id = take_id(&mut next_id);
    let watches_id = take_id(&mut next_id);
    let controls_id = take_id(&mut next_id);

    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(frame.content),
        )
        .with_semantic_label("observation shell workbench")
        .with_child(
            UiNodeSpec::new(
                title_id,
                UiNodeKind::Region(UiRegionKind::Header),
                UiSurfaceRole::Accent,
                UiNodeLayout::Explicit(frame.header),
            )
            .with_parent(root)
            .with_semantic_label("observation workbench header")
            .with_child(text_node(
                title_id,
                take_id(&mut next_id),
                "OBSERVATION SHELL WORKBENCH",
                UiTextRole::Title,
                frame.header.inset(0.04),
            )),
        )
        .with_child(panel(
            root,
            catalog_id,
            &mut next_id,
            top.leading,
            "COMMAND CATALOG",
            &view.catalog_lines,
        ))
        .with_child(panel(
            root,
            session_id,
            &mut next_id,
            top.trailing,
            "SESSION CONTEXT",
            &view.session_lines,
        ))
        .with_child(panel(
            root,
            transcript_id,
            &mut next_id,
            frame.body,
            "TRANSCRIPT",
            &view.transcript_lines,
        ))
        .with_child(panel(
            root,
            watches_id,
            &mut next_id,
            footer.leading,
            "WATCHES",
            &view.watch_lines,
        ))
        .with_child(panel(
            root,
            controls_id,
            &mut next_id,
            footer.trailing,
            "CONTROLS",
            &view.control_lines,
        )),
    )
}

fn panel(
    parent: UiNodeId,
    id: UiNodeId,
    next_id: &mut u64,
    bounds: UiRect,
    title: &str,
    lines: &[String],
) -> UiNodeSpec {
    let mut content = vec![title.to_owned()];
    content.extend(lines.iter().cloned());
    let grid = UiUniformGridLayout::new(bounds.inset(0.045), content.len().max(1), 1, [0.0, 0.012]);
    let children = content
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
                    UiTextRole::Caption
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
    .with_semantic_label(title.to_ascii_lowercase())
    .with_children(children)
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
    *next_id += 1;
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbench_scene_resolves_at_normal_and_compact_sizes() {
        let view = WorkbenchView {
            catalog_lines: vec!["HELP".into()],
            session_lines: vec!["FORMAT: TEXT".into()],
            transcript_lines: vec!["> HELP".into()],
            watch_lines: vec!["NONE".into()],
            control_lines: vec!["LEFT/RIGHT SELECT".into()],
        };
        for size in [[1200.0, 760.0], [640.0, 480.0]] {
            let tree = build_scene(size, &view);
            assert!(tree
                .resolve(UiRect::new([size[0] * 0.5, size[1] * 0.5], size))
                .is_ok());
        }
    }
}
