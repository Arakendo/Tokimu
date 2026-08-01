use super::super::*;

#[test]
fn frame_layout_uses_the_available_window_instead_of_a_fixed_card() {
    let frame =
        UiFrameLayout::for_window([1200.0, 760.0], UiInsets::uniform(0.08), 0.14, 0.20, 0.03);

    assert!(frame.content.size[0] > 2.8);
    assert!(frame.header.center[1] > frame.body.center[1]);
    assert!(frame.body.center[1] > frame.footer.center[1]);
    assert!(frame.header.intersection(frame.body).is_none());
    assert!(frame.body.intersection(frame.footer).is_none());
    assert_eq!(frame.fit, UiLayoutFit::Exact);
}

#[test]
fn frame_layout_clamps_gaps_before_regions_can_overlap() {
    let frame = UiFrameLayout::new(
        UiRect::new([0.0, 0.0], [1.0, 1.0]),
        UiInsets::uniform(0.1),
        0.4,
        0.4,
        0.5,
    );

    assert!(frame.header.intersection(frame.body).is_none());
    assert!(frame.body.intersection(frame.footer).is_none());
    assert_eq!(frame.fit, UiLayoutFit::Impossible);
}

#[test]
fn frame_layout_reports_viewport_capacity_without_overlapping_regions() {
    let cases = [
        (UiRect::new([0.0, 0.0], [3.2, 2.0]), UiLayoutFit::Exact),
        (UiRect::new([0.0, 0.0], [1.2, 0.5]), UiLayoutFit::Impossible),
        (UiRect::new([0.0, 0.0], [0.0, 0.5]), UiLayoutFit::Impossible),
    ];

    for (viewport, expected_fit) in cases {
        let frame = UiFrameLayout::new(viewport, UiInsets::uniform(0.08), 0.14, 0.20, 0.03);

        assert_eq!(frame.fit, expected_fit, "viewport: {viewport:?}");
        if frame.fit != UiLayoutFit::Impossible {
            assert!(frame.header.intersection(frame.body).is_none());
            assert!(frame.body.intersection(frame.footer).is_none());
        }
    }
}

#[test]
fn horizontal_split_preserves_declared_minimums_when_the_viewport_allows_them() {
    let split =
        UiHorizontalSplitLayout::new(UiRect::new([0.0, 0.0], [2.0, 0.8]), 0.5, 0.04, 0.70, 0.70);

    assert!(split.fits_minimums);
    assert_eq!(split.fit, UiLayoutFit::Exact);
    assert!(split.leading.size[0] >= 0.70);
    assert!(split.trailing.size[0] >= 0.70);
    assert!(split.leading.intersection(split.trailing).is_none());
}

#[test]
fn horizontal_split_reports_unfit_minimums_without_overlapping_panes() {
    let split =
        UiHorizontalSplitLayout::new(UiRect::new([0.0, 0.0], [1.0, 0.8]), 0.5, 0.04, 0.70, 0.70);

    assert!(!split.fits_minimums);
    assert_eq!(split.fit, UiLayoutFit::Adjusted);
    assert!(split.leading.intersection(split.trailing).is_none());
}

#[test]
fn horizontal_split_with_no_usable_pane_is_impossible_not_compact() {
    let split =
        UiHorizontalSplitLayout::new(UiRect::new([0.0, 0.0], [0.0, 0.8]), 0.5, 0.04, 0.10, 0.10);

    assert_eq!(split.fit, UiLayoutFit::Impossible);
}

#[test]
fn uniform_grid_resolves_row_major_cells_without_overlap() {
    let grid = UiUniformGridLayout::new(UiRect::new([0.0, 0.0], [1.2, 0.8]), 5, 3, [0.06, 0.08]);

    assert_eq!(grid.fit, UiLayoutFit::Exact);
    assert_eq!(grid.rows, 2);
    assert_eq!(grid.cells.len(), 5);
    assert!(grid.cells[0].center[0] < grid.cells[1].center[0]);
    assert!(grid.cells[0].center[1] > grid.cells[3].center[1]);
    for (index, cell) in grid.cells.iter().enumerate() {
        for other in grid.cells.iter().skip(index + 1) {
            assert!(cell.intersection(*other).is_none());
        }
    }
}

#[test]
fn uniform_grid_adjusts_excessive_gaps_and_preserves_usable_cells() {
    let grid = UiUniformGridLayout::new(UiRect::new([0.0, 0.0], [1.0, 1.0]), 4, 2, [10.0, 10.0]);

    assert_eq!(grid.fit, UiLayoutFit::Adjusted);
    assert_eq!(grid.cells.len(), 4);
    assert!(grid
        .cells
        .iter()
        .all(|cell| cell.size[0] > 0.0 && cell.size[1] > 0.0));
    assert!(grid.cells[0].intersection(grid.cells[1]).is_none());
    assert!(grid.cells[0].intersection(grid.cells[2]).is_none());
}

#[test]
fn uniform_grid_handles_empty_and_impossible_requests_explicitly() {
    let container = UiRect::new([0.0, 0.0], [1.0, 1.0]);
    let empty = UiUniformGridLayout::new(container, 0, 3, [0.1, 0.1]);
    let no_columns = UiUniformGridLayout::new(container, 3, 0, [0.1, 0.1]);
    let no_container =
        UiUniformGridLayout::new(UiRect::new([0.0, 0.0], [0.0, 1.0]), 3, 2, [0.1, 0.1]);
    let invalid_gap = UiUniformGridLayout::new(container, 1, 2, [f32::NAN, -1.0]);
    let extreme_columns = UiUniformGridLayout::new(container, 1, usize::MAX, [0.0, 0.0]);

    assert_eq!(empty.fit, UiLayoutFit::Exact);
    assert!(empty.cells.is_empty());
    assert_eq!(no_columns.fit, UiLayoutFit::Impossible);
    assert_eq!(no_container.fit, UiLayoutFit::Impossible);
    assert_eq!(invalid_gap.fit, UiLayoutFit::Adjusted);
    assert_eq!(invalid_gap.gap, [0.0, 0.0]);
    assert!(extreme_columns.cells[0]
        .size
        .iter()
        .all(|value| value.is_finite() && *value > 0.0));
}

#[test]
fn toolbar_layout_can_hit_test_buttons() {
    let layout = UiToolbarLayout::new(
        [1280.0, 720.0],
        [
            UiButtonSpec::new(UiButtonId(0), "browse"),
            UiButtonSpec::new(UiButtonId(1), "edit"),
            UiButtonSpec::new(UiButtonId(2), "preview"),
        ],
        [
            UiCardSpec::new(UiCardRole::Browser, "browse", "shell"),
            UiCardSpec::new(UiCardRole::Editor, "edit", "select"),
            UiCardSpec::new(UiCardRole::Preview, "preview", "hover"),
        ],
    );
    let point = layout.buttons[1].rect.center;

    assert_eq!(layout.button_at(point), Some(UiButtonId(1)));
    assert_eq!(layout.title_chip.label, "WORKSPACE");
    assert_eq!(layout.workspace.kind, UiRegionKind::Workspace);
    assert_eq!(layout.sidebar.kind, UiRegionKind::Sidebar);
    assert!(layout
        .footer_chip
        .contains(layout.footer_chip.region().rect.center));
    assert_eq!(layout.cards[0].title, "browse");
}

#[test]
fn label_and_card_metadata_are_usable() {
    let label = UiLabelSpec::new("hello", UiLabelAnchor::Start);
    let card = UiCardSpec::new(UiCardRole::Editor, "title", "body");
    let region = UiRegion::new(
        UiRegionKind::Card,
        UiSurfaceRole::Panel,
        UiRect::new([0.0, 0.0], [1.0, 1.0]),
    );
    let structured_card = UiCard::new(UiCardRole::Editor, "title", "body", region);

    assert_eq!(label.text, "hello");
    assert_eq!(card.body, "body");
    assert_eq!(structured_card.region.kind, UiRegionKind::Card);
    assert_eq!(structured_card.header.role, UiSurfaceRole::Raised);
}

#[test]
fn intrinsic_card_width_follows_content_measurement() {
    let theme = UiTheme::default();
    let short = UiCard::from_intrinsic(UiCardRole::Editor, "Title", "Body", [0.0, 0.0], &theme);
    let long = UiCard::from_intrinsic(
        UiCardRole::Editor,
        "Title",
        "A much longer body",
        [0.0, 0.0],
        &theme,
    );

    assert!(long.region.rect.size[0] > short.region.rect.size[0]);
    assert_eq!(long.region.rect.size[1], short.region.rect.size[1]);
}

#[test]
fn measurement_applies_parent_constraints() {
    let theme = UiTheme::default();
    let button = UiButton::from_intrinsic(UiButtonId(0), "A LONG LABEL", [0.0, 0.0], &theme);
    let context = UiMeasureContext::new(&theme, [0.12, 0.08])
        .with_constraints(UiConstraints::new([0.06, 0.04], [0.12, 0.08]));

    assert_eq!(button.measure(&context), [0.12, 0.08]);
}

#[test]
fn malformed_constraints_are_normalized() {
    let constraints = UiConstraints::new([0.8, 0.6], [0.2, 0.1]);

    assert_eq!(constraints.min, [0.2, 0.1]);
    assert_eq!(constraints.max, [0.8, 0.6]);
    assert_eq!(constraints.constrain([0.0, 1.0]), [0.2, 0.6]);
}

#[test]
fn size_policy_resolves_intrinsic_fill_fixed_min_and_max() {
    let constraints = UiConstraints::new([0.2, 0.1], [0.8, 0.6]);
    let intrinsic = [0.4, 0.3];

    assert_eq!(
        UiSizePolicy::Intrinsic.resolve(intrinsic, constraints),
        intrinsic
    );
    assert_eq!(
        UiSizePolicy::Fill.resolve(intrinsic, constraints),
        [0.8, 0.6]
    );
    assert_eq!(
        UiSizePolicy::Fixed([0.5, 0.2]).resolve(intrinsic, constraints),
        [0.5, 0.2]
    );
    assert_eq!(
        UiSizePolicy::Min([0.6, 0.5]).resolve(intrinsic, constraints),
        [0.6, 0.5]
    );
    assert_eq!(
        UiSizePolicy::Max([0.3, 0.2]).resolve(intrinsic, constraints),
        [0.3, 0.2]
    );
}

#[test]
fn horizontal_stack_produces_ordered_nested_layout() {
    let theme = UiTheme::default();
    let first = UiButton::from_intrinsic(UiButtonId(0), "browse", [0.0, 0.0], &theme);
    let second = UiButton::from_intrinsic(UiButtonId(1), "edit", [0.0, 0.0], &theme);
    let stack = UiHorizontalStack::new(vec![first, second], 0.02);
    let context = UiMeasureContext::new(&theme, [0.8, 0.3]);
    let parent = UiRect::new([0.0, 0.0], [0.8, 0.3]);

    let measured = stack.measure(&context);
    let result = stack.layout(parent, &context);

    assert_eq!(result.rect, parent);
    assert_eq!(result.children.len(), 2);
    assert!(measured[0] > first.intrinsic_size(&theme)[0]);
    assert!(result.children[0].rect.center[0] < result.children[1].rect.center[0]);
    assert!(result
        .children
        .iter()
        .all(|child| child.rect.size[0] <= parent.size[0]));
}

#[test]
fn horizontal_stack_clamps_oversized_gaps_to_parent_width() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
    ];
    let stack = UiHorizontalStack::new(children, 1.0);
    let context = UiMeasureContext::new(&theme, [0.1, 0.2]);
    let parent = UiRect::new([0.0, 0.0], [0.1, 0.2]);

    let result = stack.layout(parent, &context);
    let left = result.children[0].rect.center[0] - result.children[0].rect.size[0] * 0.5;
    let right = result.children[1].rect.center[0] + result.children[1].rect.size[0] * 0.5;

    assert!(left >= parent.center[0] - parent.size[0] * 0.5);
    assert!(right <= parent.center[0] + parent.size[0] * 0.5);
    assert_eq!(result.fit, UiLayoutFit::Adjusted);
}

#[test]
fn horizontal_stack_can_preserve_overflow_for_a_caller_selected_fallback() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "LONG LABEL", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "LONG LABEL", [0.0, 0.0], &theme),
    ];
    let parent = UiRect::new([0.0, 0.0], [0.1, 0.2]);
    let context = UiMeasureContext::new(&theme, [0.1, 0.2]);

    let result = UiHorizontalStack::new(children, 0.02)
        .with_overflow_policy(UiOverflowPolicy::Preserve)
        .layout(parent, &context);

    assert_eq!(result.fit, UiLayoutFit::Overflow);
    assert!(result.overflow[0] > 0.0);
    let right = result.children[1].rect.center[0] + result.children[1].rect.size[0] * 0.5;
    assert!(right > parent.center[0] + parent.size[0] * 0.5);
}

#[test]
fn horizontal_stack_space_between_preserves_children_and_uses_available_width() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(2), "C", [0.0, 0.0], &theme),
    ];
    let context = UiMeasureContext::new(&theme, [1.2, 0.2]);
    let parent = UiRect::new([0.0, 0.0], [1.2, 0.2]);
    let intrinsic_widths: Vec<f32> = children
        .iter()
        .map(|child| child.measure(&context)[0])
        .collect();

    let result = UiHorizontalStack::new(children, 0.02)
        .with_main_axis_allocation(UiMainAxisAllocation::SpaceBetween)
        .layout(parent, &context);

    assert_eq!(result.fit, UiLayoutFit::Adjusted);
    assert_eq!(
        result
            .children
            .iter()
            .map(|child| child.rect.size[0])
            .collect::<Vec<_>>(),
        intrinsic_widths
    );
    let left = result.children[0].rect.center[0] - result.children[0].rect.size[0] * 0.5;
    let right = result.children[2].rect.center[0] + result.children[2].rect.size[0] * 0.5;
    assert!((left - (parent.center[0] - parent.size[0] * 0.5)).abs() < 0.00001);
    assert!((right - (parent.center[0] + parent.size[0] * 0.5)).abs() < 0.00001);
}

#[test]
fn vertical_stack_space_between_preserves_children_and_uses_available_height() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
    ];
    let context = UiMeasureContext::new(&theme, [0.4, 1.0]);
    let parent = UiRect::new([0.0, 0.0], [0.4, 1.0]);
    let intrinsic_heights: Vec<f32> = children
        .iter()
        .map(|child| child.measure(&context)[1])
        .collect();

    let result = UiVerticalStack::new(children, 0.02)
        .with_main_axis_allocation(UiMainAxisAllocation::SpaceBetween)
        .layout(parent, &context);

    assert_eq!(result.fit, UiLayoutFit::Adjusted);
    assert_eq!(
        result
            .children
            .iter()
            .map(|child| child.rect.size[1])
            .collect::<Vec<_>>(),
        intrinsic_heights
    );
    let top = result.children[0].rect.center[1] + result.children[0].rect.size[1] * 0.5;
    let bottom = result.children[1].rect.center[1] - result.children[1].rect.size[1] * 0.5;
    assert!((top - (parent.center[1] + parent.size[1] * 0.5)).abs() < 0.00001);
    assert!((bottom - (parent.center[1] - parent.size[1] * 0.5)).abs() < 0.00001);
}

#[test]
fn specialized_layouts_share_one_resolved_result_contract() {
    use crate::UiResolvedLayout;

    let container = UiRect::new([0.0, 0.0], [12.0, 8.0]);
    let frame = UiFrameLayout::new(container, UiInsets::uniform(0.5), 1.0, 1.0, 0.25);
    let split = UiHorizontalSplitLayout::new(container, 0.4, 0.5, 2.0, 2.0);
    let grid = UiUniformGridLayout::new(container, 5, 3, [0.25, 0.25]);

    let frame_result = frame.layout_result();
    let split_result = split.layout_result();
    let grid_result = grid.layout_result();

    assert_eq!(frame_result.rect, container);
    assert_eq!(frame_result.fit, frame.fit);
    assert_eq!(frame_result.children.len(), 3);
    assert_eq!(frame_result.children[0].rect, frame.header);
    assert_eq!(frame_result.children[1].rect, frame.body);
    assert_eq!(frame_result.children[2].rect, frame.footer);

    assert_eq!(split_result.rect, container);
    assert_eq!(split_result.fit, split.fit);
    assert_eq!(split_result.children.len(), 2);
    assert_eq!(split_result.children[0].rect, split.leading);
    assert_eq!(split_result.children[1].rect, split.trailing);

    assert_eq!(grid_result.rect, container);
    assert_eq!(grid_result.fit, grid.fit);
    assert_eq!(grid_result.children.len(), 5);
    assert_eq!(grid_result.children[4].rect, grid.cells[4]);
}

#[test]
fn stacks_report_impossible_for_zero_sized_containers() {
    let theme = UiTheme::default();
    let stack = UiVerticalStack::new(
        vec![UiButton::from_intrinsic(
            UiButtonId(0),
            "A",
            [0.0, 0.0],
            &theme,
        )],
        0.0,
    );
    let parent = UiRect::new([0.0, 0.0], [0.0, 0.2]);
    let context = UiMeasureContext::new(&theme, [0.0, 0.2]);

    let result = stack.layout(parent, &context);

    assert_eq!(result.fit, UiLayoutFit::Impossible);
    assert!(result.children.is_empty());
}

#[test]
fn horizontal_stack_accepts_cards_as_measurable_children() {
    let theme = UiTheme::default();
    let cards = vec![
        UiCard::from_intrinsic(UiCardRole::Browser, "Files", "One", [0.0, 0.0], &theme),
        UiCard::from_intrinsic(UiCardRole::Preview, "Preview", "Two", [0.0, 0.0], &theme),
    ];
    let stack = UiHorizontalStack::new(cards, 0.02);
    let context = UiMeasureContext::new(&theme, [1.0, 0.5]);

    let result = stack.layout(UiRect::new([0.0, 0.0], [1.0, 0.5]), &context);

    assert_eq!(result.children.len(), 2);
    assert!(result.children[0].rect.size[0] > 0.0);
    assert!(result.children[1].rect.size[0] > 0.0);
}

#[test]
fn vertical_stack_produces_top_to_bottom_nested_layout() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "TOP", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "BOTTOM", [0.0, 0.0], &theme),
    ];
    let stack = UiVerticalStack::new(children, 0.02);
    let context = UiMeasureContext::new(&theme, [0.5, 0.6]);
    let parent = UiRect::new([0.0, 0.0], [0.5, 0.6]);

    let result = stack.layout(parent, &context);

    assert_eq!(result.rect, parent);
    assert_eq!(result.children.len(), 2);
    assert!(result.children[0].rect.center[1] > result.children[1].rect.center[1]);
    assert!(result
        .children
        .iter()
        .all(|child| child.rect.center[0] == parent.center[0]));
}

#[test]
fn vertical_stack_clamps_oversized_gaps_to_parent_height() {
    let theme = UiTheme::default();
    let children = vec![
        UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
    ];
    let stack = UiVerticalStack::new(children, 1.0);
    let context = UiMeasureContext::new(&theme, [0.2, 0.1]);
    let parent = UiRect::new([0.0, 0.0], [0.2, 0.1]);

    let result = stack.layout(parent, &context);
    let top = result.children[0].rect.center[1] + result.children[0].rect.size[1] * 0.5;
    let bottom = result.children[1].rect.center[1] - result.children[1].rect.size[1] * 0.5;

    assert!(top <= parent.center[1] + parent.size[1] * 0.5);
    assert!(bottom >= parent.center[1] - parent.size[1] * 0.5);
}

#[test]
fn stacks_apply_cross_axis_start_end_and_fill_alignment() {
    let theme = UiTheme::default();
    let button = UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme);
    let context = UiMeasureContext::new(&theme, [0.6, 0.6]);
    let parent = UiRect::new([0.0, 0.0], [0.6, 0.6]);

    let horizontal_start = UiHorizontalStack::new(vec![button], 0.0)
        .with_cross_axis_alignment(UiCrossAxisAlignment::Start)
        .layout(parent, &context)
        .children[0]
        .rect;
    let horizontal_fill = UiHorizontalStack::new(vec![button], 0.0)
        .with_cross_axis_alignment(UiCrossAxisAlignment::Fill)
        .layout(parent, &context)
        .children[0]
        .rect;
    let vertical_end = UiVerticalStack::new(vec![button], 0.0)
        .with_cross_axis_alignment(UiCrossAxisAlignment::End)
        .layout(parent, &context)
        .children[0]
        .rect;

    assert!(horizontal_start.center[1] > 0.0);
    assert_eq!(horizontal_fill.size[1], parent.size[1]);
    assert!(vertical_end.center[0] > 0.0);
}

#[test]
fn stacks_allocate_remaining_main_axis_space_in_fill_mode() {
    let theme = UiTheme::default();
    let buttons = vec![
        UiButton::from_intrinsic(UiButtonId(0), "A", [0.0, 0.0], &theme),
        UiButton::from_intrinsic(UiButtonId(1), "B", [0.0, 0.0], &theme),
    ];
    let context = UiMeasureContext::new(&theme, [0.8, 0.8]);
    let horizontal_parent = UiRect::new([0.0, 0.0], [0.8, 0.3]);
    let vertical_parent = UiRect::new([0.0, 0.0], [0.3, 0.8]);
    let gap = 0.02;

    let horizontal = UiHorizontalStack::new(buttons.clone(), gap)
        .with_main_axis_allocation(UiMainAxisAllocation::Fill)
        .layout(horizontal_parent, &context);
    let vertical = UiVerticalStack::new(buttons, gap)
        .with_main_axis_allocation(UiMainAxisAllocation::Fill)
        .layout(vertical_parent, &context);
    let horizontal_width = horizontal
        .children
        .iter()
        .map(|child| child.rect.size[0])
        .sum::<f32>()
        + gap;
    let vertical_height = vertical
        .children
        .iter()
        .map(|child| child.rect.size[1])
        .sum::<f32>()
        + gap;

    assert!((horizontal_width - horizontal_parent.size[0]).abs() < 0.00001);
    assert!((vertical_height - vertical_parent.size[1]).abs() < 0.00001);
}

#[test]
fn contained_stack_viewports_keep_children_finite_ordered_and_disjoint() {
    let theme = UiTheme::default();
    let context = UiMeasureContext::new(&theme, [4.0, 4.0]);
    let viewports = [[2.0, 0.6], [0.8, 0.4], [0.2, 0.2]];

    for size in viewports {
        let parent = UiRect::new([0.0, 0.0], size);
        let horizontal = UiHorizontalStack::new(
            vec![
                UiButton::from_intrinsic(UiButtonId(0), "LONG LABEL", [0.0, 0.0], &theme),
                UiButton::from_intrinsic(UiButtonId(1), "SECOND", [0.0, 0.0], &theme),
                UiButton::from_intrinsic(UiButtonId(2), "THIRD", [0.0, 0.0], &theme),
            ],
            0.08,
        )
        .layout(parent, &context);
        let vertical = UiVerticalStack::new(
            vec![
                UiButton::from_intrinsic(UiButtonId(3), "LONG LABEL", [0.0, 0.0], &theme),
                UiButton::from_intrinsic(UiButtonId(4), "SECOND", [0.0, 0.0], &theme),
                UiButton::from_intrinsic(UiButtonId(5), "THIRD", [0.0, 0.0], &theme),
            ],
            0.08,
        )
        .layout(parent, &context);

        assert_contained_stack(&horizontal, parent, true);
        assert_contained_stack(&vertical, parent, false);
        assert_ne!(horizontal.fit, UiLayoutFit::Overflow);
        assert_ne!(vertical.fit, UiLayoutFit::Overflow);
        assert_ne!(horizontal.fit, UiLayoutFit::Impossible);
        assert_ne!(vertical.fit, UiLayoutFit::Impossible);
    }
}

#[test]
fn empty_stacks_resolve_to_an_exact_empty_layout() {
    let theme = UiTheme::default();
    let context = UiMeasureContext::new(&theme, [0.4, 0.3]);
    let parent = UiRect::new([0.0, 0.0], [0.4, 0.3]);

    let horizontal = UiHorizontalStack::<UiButton>::new(Vec::new(), 0.2).layout(parent, &context);
    let vertical = UiVerticalStack::<UiButton>::new(Vec::new(), 0.2).layout(parent, &context);

    assert_eq!(horizontal.fit, UiLayoutFit::Exact);
    assert_eq!(vertical.fit, UiLayoutFit::Exact);
    assert!(horizontal.children.is_empty());
    assert!(vertical.children.is_empty());
}

fn assert_contained_stack(result: &UiLayoutResult, parent: UiRect, horizontal: bool) {
    let parent_left = parent.center[0] - parent.size[0] * 0.5;
    let parent_right = parent.center[0] + parent.size[0] * 0.5;
    let parent_bottom = parent.center[1] - parent.size[1] * 0.5;
    let parent_top = parent.center[1] + parent.size[1] * 0.5;

    for child in &result.children {
        assert!(child.rect.center.iter().all(|value| value.is_finite()));
        assert!(child
            .rect
            .size
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0));
        assert!(child.rect.center[0] - child.rect.size[0] * 0.5 >= parent_left);
        assert!(child.rect.center[0] + child.rect.size[0] * 0.5 <= parent_right);
        assert!(child.rect.center[1] - child.rect.size[1] * 0.5 >= parent_bottom);
        assert!(child.rect.center[1] + child.rect.size[1] * 0.5 <= parent_top);
    }

    for pair in result.children.windows(2) {
        assert!(pair[0].rect.intersection(pair[1].rect).is_none());
        if horizontal {
            assert!(pair[0].rect.center[0] <= pair[1].rect.center[0]);
        } else {
            assert!(pair[0].rect.center[1] >= pair[1].rect.center[1]);
        }
    }
}
