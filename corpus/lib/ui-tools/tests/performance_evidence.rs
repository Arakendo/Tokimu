use tokimu_core::{
    DiagnosticKind, Diagnostics, PerformanceBudget, PerformanceMonitor, PerformanceUnit,
};
use ui_tools::consumer::{
    UiNodeId, UiNodeKind, UiNodeLayout, UiNodeSpec, UiPresentationInputs,
    UiPresentationRevisionTracker, UiPresentationWorkEvidence, UiRect, UiRegionKind, UiSurfaceRole,
    UiTextRole, UiTextSpec, UiTheme,
};
use ui_tools::lowering::lower_resolved_tree_to_draw_list;

#[derive(Clone, Copy)]
struct StaticScreenBudget {
    max_entries: u32,
    max_surface_candidates: u32,
    max_text_candidates: u32,
    max_warm_rebuilds: u32,
}

const OBSERVATION_SCREEN_BUDGET: StaticScreenBudget = StaticScreenBudget {
    max_entries: 7,
    max_surface_candidates: 1,
    max_text_candidates: 2,
    max_warm_rebuilds: 0,
};

const STATUS_SCREEN_BUDGET: StaticScreenBudget = StaticScreenBudget {
    max_entries: 5,
    max_surface_candidates: 1,
    max_text_candidates: 2,
    max_warm_rebuilds: 0,
};

#[test]
fn static_consumer_screens_stay_within_declared_structural_budgets() {
    assert_static_screen_budget(
        &static_text_screen(
            &[
                "WORLD OBSERVATION",
                "ENTITY: 1",
                "REVISION: 4",
                "STATUS: READY",
            ],
            UiSurfaceRole::Panel,
        ),
        OBSERVATION_SCREEN_BUDGET,
    );
    assert_static_screen_budget(
        &static_text_screen(
            &["STATUS", "FRAME: READY", "DIAGNOSTICS: NONE"],
            UiSurfaceRole::Region,
        ),
        STATUS_SCREEN_BUDGET,
    );
}

#[test]
fn sustained_ui_stage_budget_violations_emit_one_bounded_kernel_diagnostic() {
    let evidence = UiPresentationWorkEvidence::default()
        .with_layout_time(std::time::Duration::from_micros(1_500));
    let mut diagnostics = Diagnostics::with_capacity(1);
    let mut monitor = PerformanceMonitor::new(
        PerformanceBudget::new(
            "ui.layout",
            "layout time",
            1.0,
            PerformanceUnit::Milliseconds,
        )
        .with_required_consecutive_violations(3),
    );

    let layout_millis = f64::from(evidence.layout_micros) / 1_000.0;
    monitor.observe(layout_millis, &mut diagnostics);
    monitor.observe(layout_millis, &mut diagnostics);
    assert!(diagnostics.records().is_empty());

    monitor.observe(layout_millis, &mut diagnostics);
    monitor.observe(layout_millis, &mut diagnostics);

    assert_eq!(diagnostics.records().len(), 1);
    assert_eq!(
        diagnostics.records()[0].kind,
        DiagnosticKind::PerformanceBudgetExceeded
    );
    assert_eq!(diagnostics.records()[0].source, "ui.layout");
    assert_eq!(diagnostics.dropped_records(), 0);
}

fn static_text_screen(lines: &[&str], role: UiSurfaceRole) -> ui_tools::UiResolvedTree {
    let root_id = UiNodeId(1);
    let children = lines.iter().enumerate().map(|(index, line)| {
        let id = UiNodeId(index as u64 + 2);
        UiNodeSpec::text(
            id,
            &UiTextSpec::new(
                *line,
                UiRect::new([0.0, 0.0], [0.0, 0.0]),
                if index == 0 {
                    UiTextRole::Heading
                } else {
                    UiTextRole::Body
                },
            ),
        )
        .with_parent(root_id)
        .with_layout(UiNodeLayout::Explicit(UiRect::new(
            [0.0, 0.35 - index as f32 * 0.2],
            [1.4, 0.14],
        )))
    });
    let tree = ui_tools::UiTree::new(
        UiNodeSpec::new(
            root_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            role,
            UiNodeLayout::Fill,
        )
        .with_children(children),
    );

    tree.resolve(UiRect::new([0.0, 0.0], [1.6, 1.0]))
        .expect("static corpus screen resolves")
}

fn assert_static_screen_budget(resolved: &ui_tools::UiResolvedTree, budget: StaticScreenBudget) {
    let draw_list = lower_resolved_tree_to_draw_list(resolved, &UiTheme::default(), 1);
    let statistics = draw_list.statistics();
    assert!(
        statistics.entries <= budget.max_entries,
        "draw entries exceeded budget: {statistics:?}"
    );
    assert!(
        statistics.surface_batch_candidates <= budget.max_surface_candidates,
        "surface candidates exceeded budget: {statistics:?}"
    );
    assert!(
        statistics.text_batch_candidates <= budget.max_text_candidates,
        "text candidates exceeded budget: {statistics:?}"
    );

    let mut tracker = UiPresentationRevisionTracker::default();
    let inputs = UiPresentationInputs::default();
    tracker.observe(inputs);
    let stable = tracker.observe(inputs);
    let warm_rebuilds = stable
        .semantic_rebuilds
        .saturating_add(stable.measurement_rebuilds)
        .saturating_add(stable.layout_rebuilds)
        .saturating_add(stable.geometry_rebuilds)
        .saturating_add(stable.draw_list_rebuilds);
    assert!(warm_rebuilds <= budget.max_warm_rebuilds);
}
