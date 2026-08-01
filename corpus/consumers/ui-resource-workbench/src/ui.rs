use ui_tools::consumer::{
    UiButtonId, UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiLayoutFit, UiNodeId,
    UiNodeInteraction, UiNodeKind, UiNodeLayout, UiNodeSpec, UiRect, UiRegionKind, UiSurfaceRole,
    UiTextAlign, UiTextOverflow, UiTextRole, UiTextSpec, UiTree, UiUniformGridLayout,
};

use crate::model::{ResourceDraft, ResourceWorkbenchModel};

pub const FILTER_FIELD_ID: UiNodeId = UiNodeId(20);
pub const NAME_FIELD_ID: UiNodeId = UiNodeId(21);
pub const NOTES_FIELD_ID: UiNodeId = UiNodeId(22);
pub const VISIBILITY_ID: UiNodeId = UiNodeId(23);
pub const HOTSPOT_ID: UiNodeId = UiNodeId(24);
pub const APPLY_ID: UiNodeId = UiNodeId(25);
pub const REVERT_ID: UiNodeId = UiNodeId(26);
pub const DELETE_ID: UiNodeId = UiNodeId(27);
pub const CONFIRM_DELETE_ID: UiNodeId = UiNodeId(28);
pub const CANCEL_DELETE_ID: UiNodeId = UiNodeId(29);
pub const DELETE_MODAL_ID: UiNodeId = UiNodeId(30);
pub const RESOURCE_ROW_BASE: u64 = 100;

const ROOT_ID: UiNodeId = UiNodeId(1);
const HEADER_ID: UiNodeId = UiNodeId(2);
const BODY_ID: UiNodeId = UiNodeId(3);
const FOOTER_ID: UiNodeId = UiNodeId(4);

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceScene {
    pub tree: UiTree,
    pub viewport: UiRect,
    pub fit: UiLayoutFit,
}

pub fn build_resource_scene(
    window_size: [f32; 2],
    model: &ResourceWorkbenchModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
) -> ResourceScene {
    let frame = UiFrameLayout::for_window(window_size, UiInsets::uniform(0.055), 0.12, 0.14, 0.025);
    let wide = frame.fit == UiLayoutFit::Exact && frame.body.size[0] >= 1.45;
    let fit = if wide {
        frame.fit
    } else if frame.fit == UiLayoutFit::Impossible {
        UiLayoutFit::Impossible
    } else {
        UiLayoutFit::Adjusted
    };
    let mut next_id = 1000;

    let header = UiNodeSpec::new(
        HEADER_ID,
        UiNodeKind::Region(UiRegionKind::Header),
        UiSurfaceRole::Accent,
        UiNodeLayout::Explicit(frame.header),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("resource workbench header")
    .with_child(text_node(
        HEADER_ID,
        take_id(&mut next_id),
        "RESOURCE WORKBENCH",
        UiTextRole::Title,
        frame.header.inset(0.04),
    ));

    let body = if wide {
        let split = UiHorizontalSplitLayout::new(frame.body.inset(0.025), 0.32, 0.025, 0.68, 1.05);
        workspace(
            frame.body,
            resource_list(
                BODY_ID,
                take_id(&mut next_id),
                split.leading,
                model,
                focused,
                hovered,
                1,
                &mut next_id,
            ),
            inspector(
                BODY_ID,
                take_id(&mut next_id),
                split.trailing,
                model,
                focused,
                hovered,
                &mut next_id,
            ),
        )
    } else {
        let stack = UiUniformGridLayout::new(frame.body.inset(0.025), 2, 1, [0.0, 0.025]);
        workspace(
            frame.body,
            resource_list(
                BODY_ID,
                take_id(&mut next_id),
                stack.cells[0],
                model,
                focused,
                hovered,
                2,
                &mut next_id,
            ),
            inspector(
                BODY_ID,
                take_id(&mut next_id),
                stack.cells[1],
                model,
                focused,
                hovered,
                &mut next_id,
            ),
        )
    };

    let footer_split =
        UiHorizontalSplitLayout::new(frame.footer.inset(0.035), 0.76, 0.03, 0.8, 0.45);
    let footer = UiNodeSpec::new(
        FOOTER_ID,
        UiNodeKind::Region(UiRegionKind::StatusBar),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(frame.footer),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("resource status")
    .with_child(text_node(
        FOOTER_ID,
        take_id(&mut next_id),
        &model.status,
        UiTextRole::Caption,
        footer_split.leading,
    ))
    .with_child(text_node(
        FOOTER_ID,
        take_id(&mut next_id),
        if wide {
            "LAYOUT: WIDE"
        } else {
            "LAYOUT: COMPACT"
        },
        UiTextRole::Caption,
        footer_split.trailing,
    ));

    let mut root = UiNodeSpec::new(
        ROOT_ID,
        UiNodeKind::Region(UiRegionKind::Panel),
        UiSurfaceRole::Panel,
        UiNodeLayout::Explicit(frame.content),
    )
    .with_semantic_label("resource inspection workbench")
    .with_child(header)
    .with_child(body)
    .with_child(footer);

    if model.confirm_delete {
        root = root.with_child(delete_modal(frame.content, model, &mut next_id));
    }

    ResourceScene {
        tree: UiTree::new(root),
        viewport: frame.viewport,
        fit,
    }
}

fn workspace(bounds: UiRect, list: UiNodeSpec, inspector: UiNodeSpec) -> UiNodeSpec {
    UiNodeSpec::new(
        BODY_ID,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Region,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("resource workspace")
    .with_child(list)
    .with_child(inspector)
}

#[allow(clippy::too_many_arguments)]
fn resource_list(
    parent: UiNodeId,
    id: UiNodeId,
    bounds: UiRect,
    model: &ResourceWorkbenchModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
    columns: usize,
    next_id: &mut u64,
) -> UiNodeSpec {
    let visible = model.visible_resources();
    let grid = UiUniformGridLayout::new(
        bounds.inset(0.04),
        visible.len() + 2,
        columns,
        [0.018, 0.018],
    );
    let mut panel = UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Sidebar),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label("filterable resources")
    .with_child(text_node(
        id,
        take_id(next_id),
        "RESOURCES",
        UiTextRole::Heading,
        grid.cells[0],
    ))
    .with_child(editable_field(
        id,
        FILTER_FIELD_ID,
        "FILTER",
        model.filter.value(),
        grid.cells[1],
        focused,
        hovered,
    ));

    for (index, resource) in visible.into_iter().enumerate() {
        panel = panel.with_child(resource_row(
            id,
            resource,
            grid.cells[index + 2],
            model.selected_id == resource.id,
            focused,
            hovered,
        ));
    }
    panel
}

fn resource_row(
    parent: UiNodeId,
    resource: &ResourceDraft,
    bounds: UiRect,
    selected: bool,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
) -> UiNodeSpec {
    let id = ResourceWorkbenchModel::row_id(resource.id);
    let active = selected || focused == Some(id) || hovered == Some(id);
    let label = format!("{} / {}", resource.kind.label(), resource.name.value());
    UiNodeSpec::new(
        id,
        UiNodeKind::Button(UiButtonId(resource.id as u8)),
        if active {
            UiSurfaceRole::Selected
        } else {
            UiSurfaceRole::Raised
        },
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_interaction(UiNodeInteraction::Activatable)
    .with_selected(selected)
    .with_semantic_label("resource row")
    .with_semantic_value(&label)
    .with_text(
        UiTextSpec::new(label, bounds.inset(0.045), UiTextRole::Body)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
            .with_overflow(UiTextOverflow::Ellipsis),
    )
}

fn inspector(
    parent: UiNodeId,
    id: UiNodeId,
    bounds: UiRect,
    model: &ResourceWorkbenchModel,
    focused: Option<UiNodeId>,
    hovered: Option<UiNodeId>,
    next_id: &mut u64,
) -> UiNodeSpec {
    let resource = model.selected();
    let grid = UiUniformGridLayout::new(bounds.inset(0.045), 7, 1, [0.0, 0.018]);
    let toggles = UiUniformGridLayout::new(grid.cells[4], 2, 2, [0.018, 0.0]);
    let commands = UiUniformGridLayout::new(grid.cells[5], 2, 2, [0.018, 0.0]);

    UiNodeSpec::new(
        id,
        UiNodeKind::Region(UiRegionKind::Inspector),
        UiSurfaceRole::Background,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(parent)
    .with_semantic_label("selected resource inspector")
    .with_child(text_node(
        id,
        take_id(next_id),
        &format!("{} INSPECTOR", resource.kind.label()),
        UiTextRole::Heading,
        grid.cells[0],
    ))
    .with_child(text_node(
        id,
        take_id(next_id),
        &format!(
            "RESOURCE {:02} / {}",
            resource.id,
            if resource.is_dirty() {
                "DRAFT"
            } else {
                "SAVED"
            }
        ),
        UiTextRole::Caption,
        grid.cells[1],
    ))
    .with_child(editable_field(
        id,
        NAME_FIELD_ID,
        "NAME",
        resource.name.value(),
        grid.cells[2],
        focused,
        hovered,
    ))
    .with_child(editable_field(
        id,
        NOTES_FIELD_ID,
        "NOTES",
        resource.notes.value(),
        grid.cells[3],
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        VISIBILITY_ID,
        20,
        &format!("VISIBLE: {}", on_off(resource.visible)),
        toggles.cells[0],
        true,
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        HOTSPOT_ID,
        21,
        &format!("HOTSPOT: {}", on_off(resource.hotspot)),
        toggles.cells[1],
        true,
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        APPLY_ID,
        22,
        "APPLY",
        commands.cells[0],
        resource.is_dirty(),
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        REVERT_ID,
        23,
        "REVERT",
        commands.cells[1],
        resource.is_dirty(),
        focused,
        hovered,
    ))
    .with_child(action_node(
        id,
        DELETE_ID,
        24,
        "DELETE RESOURCE",
        grid.cells[6],
        model.resources.len() > 1,
        focused,
        hovered,
    ))
}

fn delete_modal(bounds: UiRect, model: &ResourceWorkbenchModel, next_id: &mut u64) -> UiNodeSpec {
    let dialog_bounds = UiRect::new(
        bounds.center,
        [bounds.size[0] * 0.54, bounds.size[1] * 0.38],
    );
    let dialog_id = take_id(next_id);
    let grid = UiUniformGridLayout::new(dialog_bounds.inset(0.08), 4, 1, [0.0, 0.025]);
    let commands = UiUniformGridLayout::new(grid.cells[3], 2, 2, [0.025, 0.0]);
    let dialog = UiNodeSpec::new(
        dialog_id,
        UiNodeKind::Region(UiRegionKind::Panel),
        UiSurfaceRole::Panel,
        UiNodeLayout::Explicit(dialog_bounds),
    )
    .with_parent(DELETE_MODAL_ID)
    .with_semantic_label("delete confirmation dialog")
    .with_child(text_node(
        dialog_id,
        take_id(next_id),
        "DELETE RESOURCE?",
        UiTextRole::Title,
        grid.cells[0],
    ))
    .with_child(text_node(
        dialog_id,
        take_id(next_id),
        model.selected().name.value(),
        UiTextRole::Heading,
        grid.cells[1],
    ))
    .with_child(text_node(
        dialog_id,
        take_id(next_id),
        "THIS ACTION CANNOT BE UNDONE",
        UiTextRole::Caption,
        grid.cells[2],
    ))
    .with_child(action_node(
        dialog_id,
        CANCEL_DELETE_ID,
        30,
        "CANCEL",
        commands.cells[0],
        true,
        None,
        None,
    ))
    .with_child(action_node(
        dialog_id,
        CONFIRM_DELETE_ID,
        31,
        "CONFIRM DELETE",
        commands.cells[1],
        true,
        None,
        None,
    ));

    UiNodeSpec::new(
        DELETE_MODAL_ID,
        UiNodeKind::Region(UiRegionKind::Workspace),
        UiSurfaceRole::Overlay,
        UiNodeLayout::Explicit(bounds),
    )
    .with_parent(ROOT_ID)
    .with_semantic_label("delete resource modal")
    .as_modal(true)
    .with_child(dialog)
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
    .with_selected(active)
    .with_semantic_label(label)
    .with_semantic_value(value)
    .with_text(
        UiTextSpec::new(
            format!("{label}: {value}"),
            bounds.inset(0.045),
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
        UiTextSpec::new(label, bounds.inset(0.045), UiTextRole::Body)
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

const fn on_off(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
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
        lower_resolved_tree_to_draw_list, UiModalDismissReason, UiPointerEvent, UiPointerPhase,
        UiPointerRouter, UiResolvedFocus, UiTheme,
    };

    #[test]
    fn wide_and_compact_scenes_resolve_without_diagnostics() {
        let model = ResourceWorkbenchModel::default();
        for size in [[1200.0, 800.0], [560.0, 900.0]] {
            let scene = build_resource_scene(size, &model, None, None);
            let resolved = scene.tree.resolve(scene.viewport).unwrap();
            assert!(
                resolved.diagnostics.is_empty(),
                "{size:?}: {:?}",
                resolved.diagnostics
            );
        }
    }

    #[test]
    fn clean_commands_are_excluded_until_the_draft_changes() {
        let mut model = ResourceWorkbenchModel::default();
        let clean_scene = build_resource_scene([1200.0, 800.0], &model, None, None);
        let clean = clean_scene.tree.resolve(clean_scene.viewport).unwrap();
        assert!(!clean.interactive_node_ids().contains(&APPLY_ID));
        assert!(model.activate(HOTSPOT_ID));
        let dirty_scene = build_resource_scene([1200.0, 800.0], &model, None, None);
        let dirty = dirty_scene.tree.resolve(dirty_scene.viewport).unwrap();
        assert!(dirty.interactive_node_ids().contains(&APPLY_ID));
        assert!(dirty.interactive_node_ids().contains(&REVERT_ID));
    }

    #[test]
    fn row_pointer_and_focus_use_stable_resource_identity() {
        let model = ResourceWorkbenchModel::default();
        let scene = build_resource_scene([1200.0, 800.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let row = ResourceWorkbenchModel::row_id(3);
        let center = resolved.node(row).unwrap().bounds.center;
        let mut pointer = UiPointerRouter::default();
        let mut focus = UiResolvedFocus::default();
        let press = pointer.route(
            &resolved,
            UiPointerEvent::new(center, UiPointerPhase::Press),
        );
        focus.set_focus(&resolved, press.target);
        assert_eq!(press.target, Some(row));
        assert_eq!(focus.focused(), Some(row));
    }

    #[test]
    fn modal_confines_interaction_and_exposes_dismissal() {
        let mut model = ResourceWorkbenchModel::default();
        assert!(model.activate(DELETE_ID));
        let scene = build_resource_scene([1200.0, 800.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let ids = resolved.interactive_node_ids();
        assert_eq!(
            resolved.active_modal().map(|node| node.id),
            Some(DELETE_MODAL_ID)
        );
        assert!(ids.contains(&CONFIRM_DELETE_ID));
        assert!(ids.contains(&CANCEL_DELETE_ID));
        assert!(!ids.contains(&FILTER_FIELD_ID));
        assert!(resolved
            .modal_dismissal(UiModalDismissReason::Escape)
            .is_some());
    }

    #[test]
    fn lowering_is_structurally_deterministic() {
        let model = ResourceWorkbenchModel::default();
        let scene = build_resource_scene([1200.0, 800.0], &model, None, None);
        let resolved = scene.tree.resolve(scene.viewport).unwrap();
        let first = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);
        let second = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 2);
        assert_eq!(
            first.structural_fingerprint(),
            second.structural_fingerprint()
        );
    }
}
