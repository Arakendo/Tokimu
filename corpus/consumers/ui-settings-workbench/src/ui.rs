use ui_tools::consumer::{
    UiButtonId, UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiLayoutFit, UiNodeId,
    UiNodeInteraction, UiNodeKind, UiNodeLayout, UiNodeSpec, UiRect, UiRegionKind, UiSurfaceRole,
    UiTextAlign, UiTextOverflow, UiTextRole, UiTextSpec, UiTree, UiUniformGridLayout,
};

use crate::model::SettingsModel;

pub const PROJECT_FIELD_ID: UiNodeId = UiNodeId(20);
pub const AUTHOR_FIELD_ID: UiNodeId = UiNodeId(21);
pub const DIAGNOSTICS_ID: UiNodeId = UiNodeId(22);
pub const QUALITY_ID: UiNodeId = UiNodeId(23);
pub const APPLY_ID: UiNodeId = UiNodeId(24);
pub const RESET_ID: UiNodeId = UiNodeId(25);

const ROOT_ID: UiNodeId = UiNodeId(1);
const HEADER_ID: UiNodeId = UiNodeId(2);
const BODY_ID: UiNodeId = UiNodeId(3);
const FOOTER_ID: UiNodeId = UiNodeId(4);

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsScene {
    pub tree: UiTree,
    pub viewport: UiRect,
    pub fit: UiLayoutFit,
}

pub fn build_settings_scene(
    window_size: [f32; 2],
    model: &SettingsModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
) -> SettingsScene {
    let frame = UiFrameLayout::for_window(window_size, UiInsets::uniform(0.075), 0.14, 0.20, 0.03);
    let wide = frame.fit == UiLayoutFit::Exact && frame.body.size[0] >= 1.45;
    let fit = if wide {
        frame.fit
    } else if frame.fit == UiLayoutFit::Impossible {
        UiLayoutFit::Impossible
    } else {
        UiLayoutFit::Adjusted
    };
    let mut next_id = 100;

    let header = UiNodeSpec::new(
        HEADER_ID,
        UiNodeKind::Region(UiRegionKind::Header),
        UiSurfaceRole::Accent,
        UiNodeLayout::Explicit(frame.header),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("settings workbench header")
    .with_child(text_node(
        HEADER_ID,
        take_id(&mut next_id),
        "PROJECT SETTINGS",
        UiTextRole::Title,
        frame.header.inset(0.05),
    ));

    let body = if wide {
        wide_body(frame.body, model, focused, hovered, &mut next_id)
    } else {
        compact_body(frame.body, model, focused, hovered, &mut next_id)
    };

    let footer_grid = UiHorizontalSplitLayout::new(frame.footer.inset(0.05), 0.72, 0.04, 0.8, 0.8);
    let footer = UiNodeSpec::new(
        FOOTER_ID,
        UiNodeKind::Region(UiRegionKind::StatusBar),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(frame.footer),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("settings status")
    .with_child(text_node(
        FOOTER_ID,
        take_id(&mut next_id),
        &model.status,
        UiTextRole::Caption,
        footer_grid.leading,
    ))
    .with_child(text_node(
        FOOTER_ID,
        take_id(&mut next_id),
        if fit == UiLayoutFit::Exact {
            "LAYOUT: WIDE"
        } else {
            "LAYOUT: COMPACT"
        },
        UiTextRole::Caption,
        footer_grid.trailing,
    ));

    let tree = UiTree::new(
        UiNodeSpec::new(
            ROOT_ID,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(frame.content),
        )
        .with_semantic_label("project settings workbench")
        .with_child(header)
        .with_child(body)
        .with_child(footer),
    );

    SettingsScene {
        tree,
        viewport: frame.viewport,
        fit,
    }
}

fn wide_body(
    bounds: UiRect,
    model: &SettingsModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
    next_id: &mut u64,
) -> UiNodeSpec {
    let split = UiHorizontalSplitLayout::new(bounds.inset(0.04), 0.28, 0.04, 0.75, 0.85);
    let navigation_id = take_id(next_id);
    let form_id = take_id(next_id);
    UiNodeSpec::new(
        BODY_ID,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Region,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("settings workspace")
    .with_child(
        UiNodeSpec::new(
            navigation_id,
            UiNodeKind::Region(UiRegionKind::Sidebar),
            UiSurfaceRole::Background,
            UiNodeLayout::Explicit(split.leading),
        )
        .with_parent(BODY_ID)
        .with_semantic_label("settings categories")
        .with_child(text_node(
            navigation_id,
            take_id(next_id),
            "GENERAL",
            UiTextRole::Heading,
            split.leading.inset(0.12),
        )),
    )
    .with_child(form_panel(
        BODY_ID,
        form_id,
        split.trailing,
        model,
        focused,
        hovered,
        next_id,
    ))
}

fn compact_body(
    bounds: UiRect,
    model: &SettingsModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
    next_id: &mut u64,
) -> UiNodeSpec {
    let form_id = take_id(next_id);
    UiNodeSpec::new(
        BODY_ID,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Region,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("compact settings workspace")
    .with_child(form_panel(
        BODY_ID,
        form_id,
        bounds.inset(0.06),
        model,
        focused,
        hovered,
        next_id,
    ))
}

fn form_panel(
    parent: UiNodeId,
    id: UiNodeId,
    bounds: UiRect,
    model: &SettingsModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
    next_id: &mut u64,
) -> UiNodeSpec {
    let rows = UiUniformGridLayout::new(bounds.inset(0.06), 7, 1, [0.0, 0.025]);
    let cells = rows.cells;
    UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Inspector),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label("general settings form")
    .with_child(text_node(
        id,
        take_id(next_id),
        "GENERAL SETTINGS",
        UiTextRole::Heading,
        cells[0],
    ))
    .with_child(editable_field(
        id,
        PROJECT_FIELD_ID,
        "PROJECT NAME",
        model.project_name.value(),
        cells[1],
        focused,
        hovered,
    ))
    .with_child(editable_field(
        id,
        AUTHOR_FIELD_ID,
        "AUTHOR",
        model.author.value(),
        cells[2],
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        DIAGNOSTICS_ID,
        2,
        &format!(
            "DIAGNOSTICS: {}",
            if model.diagnostics { "ON" } else { "OFF" }
        ),
        cells[3],
        true,
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        QUALITY_ID,
        3,
        &format!("QUALITY: {}", model.quality.label()),
        cells[4],
        true,
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        APPLY_ID,
        4,
        "APPLY CHANGES",
        cells[5],
        model.is_dirty(),
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        RESET_ID,
        5,
        "RESET DRAFT",
        cells[6],
        model.is_dirty(),
        focused,
        hovered,
    ))
}

fn editable_field(
    parent: UiNodeId,
    id: UiNodeId,
    label: &str,
    value: &str,
    bounds: UiRect,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
) -> UiNodeSpec {
    let active = focused == Some(id) || hovered == Some(id);
    UiNodeSpec::new(
        id,
        UiNodeKind::TextInput,
        if active {
            UiSurfaceRole::Selected
        } else {
            UiSurfaceRole::Raised
        },
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_interaction(UiNodeInteraction::Editable)
    .with_semantic_label(label)
    .with_semantic_value(value)
    .with_selected(active)
    .with_text(
        UiTextSpec::new(
            format!("{label}: {value}"),
            bounds.inset(0.06),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Ellipsis),
    )
}

#[allow(clippy::too_many_arguments)]
fn action_node(
    parent: UiNodeId,
    id: UiNodeId,
    button_id: u8,
    label: &str,
    bounds: UiRect,
    enabled: bool,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
) -> UiNodeSpec {
    let active = focused == Some(id) || hovered == Some(id);
    UiNodeSpec::new(
        id,
        UiNodeKind::Button(UiButtonId(button_id)),
        if active {
            UiSurfaceRole::Selected
        } else {
            UiSurfaceRole::Raised
        },
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_interaction(UiNodeInteraction::Activatable)
    .with_enabled(enabled)
    .with_selected(active)
    .with_semantic_label(label)
    .with_text(
        UiTextSpec::new(label, bounds.inset(0.06), UiTextRole::Body)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
            .with_overflow(UiTextOverflow::Ellipsis),
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
    *next_id = next_id.saturating_add(1);
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_tools::{
        lower_resolved_tree_to_draw_list, UiPointerEvent, UiPointerPhase, UiPointerRouter,
        UiResolvedFocus, UiTextInputEvent, UiTextInputOperation, UiTextInputRouter, UiTheme,
    };

    #[test]
    fn wide_and_compact_consumers_resolve_inside_their_viewports() {
        let model = SettingsModel::default();
        let mut saw_compact = false;
        for size in [[1100.0, 760.0], [720.0, 560.0], [420.0, 720.0]] {
            let scene = build_settings_scene(size, &model, None, None);
            let resolved = scene.tree.resolve(scene.viewport).unwrap();
            assert!(
                resolved.diagnostics.is_empty(),
                "{size:?}: {:?}",
                resolved.diagnostics
            );
            if scene.fit != UiLayoutFit::Exact {
                saw_compact = true;
            }
        }
        assert!(saw_compact);
    }

    #[test]
    fn clean_actions_are_not_pointer_or_focus_targets() {
        let model = SettingsModel::default();
        let scene = build_settings_scene([1100.0, 760.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let ids = resolved.interactive_node_ids();

        assert!(!ids.contains(&APPLY_ID));
        assert!(!ids.contains(&RESET_ID));
        assert!(ids.contains(&PROJECT_FIELD_ID));
        assert!(ids.contains(&DIAGNOSTICS_ID));
    }

    #[test]
    fn pointer_and_focus_share_the_same_resolved_field_identity() {
        let model = SettingsModel::default();
        let scene = build_settings_scene([1100.0, 760.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let center = resolved.node(PROJECT_FIELD_ID).unwrap().bounds.center;
        let mut pointer = UiPointerRouter::default();
        let mut focus = UiResolvedFocus::default();

        let press = pointer.route(
            &resolved,
            UiPointerEvent::new(center, UiPointerPhase::Press),
        );
        focus.set_focus(&resolved, press.target);

        assert_eq!(press.target, Some(PROJECT_FIELD_ID));
        assert_eq!(focus.focused(), Some(PROJECT_FIELD_ID));
    }

    #[test]
    fn focused_text_routing_mutates_only_the_targeted_application_field() {
        let mut model = SettingsModel::default();
        let author_before = model.author.value().to_owned();
        let scene = build_settings_scene([1100.0, 760.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let mut focus = UiResolvedFocus::default();
        focus.set_focus(&resolved, Some(PROJECT_FIELD_ID));

        let routed = UiTextInputRouter.route(
            &resolved,
            &mut focus,
            UiTextInputEvent::new(UiTextInputOperation::Insert('!')),
        );
        assert_eq!(routed.target, Some(PROJECT_FIELD_ID));
        assert!(model.apply_edit(routed.target.unwrap(), routed.operation));
        assert!(model.project_name.value().ends_with('!'));
        assert_eq!(model.author.value(), author_before);
    }

    #[test]
    fn dirty_actions_join_the_shared_interaction_order() {
        let mut model = SettingsModel::default();
        assert!(model.activate(DIAGNOSTICS_ID));
        let scene = build_settings_scene([1100.0, 760.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let ids = resolved.interactive_node_ids();

        assert!(ids.contains(&APPLY_ID));
        assert!(ids.contains(&RESET_ID));
    }

    #[test]
    fn lowering_is_structurally_deterministic() {
        let model = SettingsModel::default();
        let scene = build_settings_scene([1100.0, 760.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let first = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);
        let second = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 2);

        assert!(!first.entries().is_empty());
        assert_eq!(
            first.structural_fingerprint(),
            second.structural_fingerprint()
        );
    }
}
