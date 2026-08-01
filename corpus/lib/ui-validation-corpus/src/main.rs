use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};

use ui_tools::consumer::{
    UiActivationKey, UiButton, UiButtonId, UiCrossAxisAlignment, UiFocusDirection, UiFrameLayout,
    UiHorizontalSplitLayout, UiInsets, UiMainAxisAllocation, UiMeasureContext, UiNodeId,
    UiNodeInteraction, UiNodeKind, UiNodeLayout, UiNodeSpec, UiPointerEvent, UiPointerPhase,
    UiPointerRouter, UiRect, UiRegionKind, UiResolvedFocus, UiResolvedNode, UiResolvedTree,
    UiSurfaceRole, UiTextInputEvent, UiTextInputOperation, UiTextInputRouter, UiTextInputState,
    UiTextRole, UiTextSpec, UiTheme, UiTree, UiUniformGridLayout, UiVerticalScroll,
    UiVerticalStack,
};
use ui_tools::lowering::{lower_resolved_tree_to_draw_list, UiDrawList};

mod cpu_image;

const ARTIFACT_SCHEMA: &str = "tokimu-ui-structural-v1";
const GENERATOR: &str = concat!("ui-validation-corpus/", env!("CARGO_PKG_VERSION"));
const SELECTION: &str = include_str!("../selection-v1.toml");
const RESOLVER_ALGORITHM: &str = "ui-tree-resolve-v1";
const LOWERING_ALGORITHM: &str = "ui-draw-list-lowering-v1";
const TEXT_PROVIDER: &str = "none-provider-neutral";
const CASES: &[&str] = &[
    "runtime-observation",
    "command-toolbar",
    "text-entry",
    "scroll-modal",
    "content-stress",
    "composition-layout",
];
const VIEWPORTS: &[(&str, u32, u32)] = &[
    ("desktop-large", 1920, 1080),
    ("desktop", 1280, 720),
    ("compact", 900, 600),
    ("small", 640, 480),
    ("mobile", 320, 568),
];
const SCALES: &[(&str, f32)] = &[("1.0", 1.0), ("1.5", 1.5), ("2.0", 2.0)];

fn main() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command] if command == "list" => {
            for case in CASES {
                println!("{case}");
            }
            Ok(())
        }
        [] => run_selection(None),
        [command] if command == "run" => run_selection(None),
        [command, case] if command == "run" => run_selection(Some(case)),
        _ => Err("usage: ui-validation-corpus [list|run [case-id]]".to_owned()),
    }
}

fn run_selection(selected: Option<&str>) -> Result<(), String> {
    if let Some(case) = selected {
        if !CASES.contains(&case) {
            return Err(format!("unknown UI corpus case `{case}`"));
        }
    }

    let executed_cases = CASES
        .iter()
        .copied()
        .filter(|case| selected.is_none_or(|selected| selected == *case))
        .collect::<Vec<_>>();

    for case in &executed_cases {
        for &(viewport_id, width, height) in VIEWPORTS {
            for &(scale_id, scale) in SCALES {
                run_case(case, viewport_id, width, height, scale_id, scale)?;
            }
        }
    }
    let output_root = PathBuf::from("target").join("ui-validation-corpus");
    fs::create_dir_all(&output_root).map_err(|error| error.to_string())?;
    let coverage_name = selected
        .map(|case| format!("coverage-{case}.json"))
        .unwrap_or_else(|| "coverage.json".to_owned());
    write(
        &output_root.join(coverage_name),
        &coverage_json(selected, &executed_cases),
    )?;
    Ok(())
}

fn run_case(
    case: &str,
    viewport_id: &str,
    width: u32,
    height: u32,
    scale_id: &str,
    scale: f32,
) -> Result<(), String> {
    let logical_width = width as f32 / (100.0 * scale);
    let logical_height = height as f32 / (100.0 * scale);
    let viewport = UiRect::new([0.0, 0.0], [logical_width, logical_height]);
    let tree = case_tree(case, viewport)?;
    let resolve_started = Instant::now();
    let resolved = tree
        .resolve(viewport)
        .map_err(|error| format!("{error:?}"))?;
    let resolve_micros = resolve_started.elapsed().as_micros();
    let lowering_started = Instant::now();
    let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);
    let lowering_micros = lowering_started.elapsed().as_micros();
    let semantics = semantics_json(&resolved);
    let content = content_json(&resolved);
    let layout = layout_json(&resolved);
    let interaction = interaction_json(&resolved);
    let input_sequence = input_sequence_json(case, &resolved, viewport);
    let draw_list_artifact = draw_list_json(&draw_list);
    let diagnostics = diagnostics_json(&resolved, &draw_list);
    let input_hash = stable_hash(&semantics);
    let selection_hash = stable_hash(SELECTION);

    let root = PathBuf::from("target")
        .join("ui-validation-corpus")
        .join(case)
        .join(viewport_id)
        .join(format!("scale-{scale_id}"));
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write(
        &root.join("manifest.txt"),
        &format!(
            "schema={ARTIFACT_SCHEMA}\ngenerator={GENERATOR}\nselection=selection-v1.toml\nselection_hash={selection_hash:016x}\ncase={case}\ninput_hash={input_hash:016x}\nviewport={viewport_id}\nphysical_width={width}\nphysical_height={height}\nlogical_width={logical_width:.4}\nlogical_height={logical_height:.4}\nscale={scale_id}\ntext_provider={TEXT_PROVIDER}\nresolver={RESOLVER_ALGORITHM}\nlowering={LOWERING_ALGORITHM}\nheadless=true\ngpu_capture=false\n"
        ),
    )?;
    write(&root.join("semantics.json"), &semantics)?;
    write(&root.join("content.json"), &content)?;
    write(&root.join("layout.json"), &layout)?;
    write(&root.join("interaction.json"), &interaction)?;
    write(&root.join("input-sequence.json"), &input_sequence)?;
    write(&root.join("draw-list.json"), &draw_list_artifact)?;
    write(&root.join("diagnostics.json"), &diagnostics)?;
    write(
        &root.join("timing.json"),
        &format!(
            "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"deterministic\": false,\n  \"resolve_micros\": {resolve_micros},\n  \"lowering_micros\": {lowering_micros}\n}}\n"
        ),
    )?;
    if viewport_id == "desktop" && scale_id == "1.0" {
        cpu_image::rasterize(&draw_list, viewport, width, height)
            .write_artifacts(&root, &draw_list)?;
    }

    println!(
        "{case}/{viewport_id}/scale-{scale_id}: nodes={}, entries={}, diagnostics={}",
        count_nodes(&resolved.root),
        draw_list.statistics().entries,
        resolved.diagnostics.len() + draw_list.diagnostics.len()
    );
    Ok(())
}

fn case_tree(case: &str, viewport: UiRect) -> Result<UiTree, String> {
    match case {
        "runtime-observation" => Ok(runtime_observation_tree(viewport)),
        "command-toolbar" => Ok(command_toolbar_tree(viewport)),
        "text-entry" => Ok(text_entry_tree(viewport)),
        "scroll-modal" => Ok(scroll_modal_tree(viewport)),
        "content-stress" => Ok(content_stress_tree(viewport)),
        "composition-layout" => Ok(composition_layout_tree(viewport)),
        _ => Err(format!("unknown UI corpus case `{case}`")),
    }
}

fn runtime_observation_tree(viewport: UiRect) -> UiTree {
    let root = UiNodeId(1);
    let content = viewport.inset(0.3);
    let line_height = (content.size[1] / 8.0).clamp(0.32, 0.72);
    let top = content.center[1] + content.size[1] * 0.5 - line_height * 0.5;
    let lines = [
        ("RUNTIME OBSERVATION", UiTextRole::Heading),
        ("SELECTED ENTITY: 1", UiTextRole::Body),
        ("REVISION: 4", UiTextRole::Body),
        ("RELATIONS: 2", UiTextRole::Body),
        ("PRESENTATION: SELECTED", UiTextRole::Body),
        ("DIAGNOSTICS: NONE", UiTextRole::Status),
    ];
    let children = lines.into_iter().enumerate().map(|(index, (text, role))| {
        let id = UiNodeId(index as u64 + 2);
        UiNodeSpec::text(id, &UiTextSpec::new(text, viewport, role))
            .with_parent(root)
            .with_layout(UiNodeLayout::Explicit(UiRect::new(
                [content.center[0], top - index as f32 * line_height],
                [content.size[0], line_height * 0.8],
            )))
    });
    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Inspector),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("runtime observation inspector")
        .with_children(children),
    )
}

fn content_stress_tree(viewport: UiRect) -> UiTree {
    let root = UiNodeId(1);
    let inset = (viewport.size[0].min(viewport.size[1]) * 0.08).clamp(0.08, 0.3);
    let content = viewport.inset(inset);
    let line_height = (content.size[1] / 4.5).clamp(0.28, 0.72);
    let top = content.center[1] + content.size[1] * 0.5 - line_height * 0.5;
    let samples = [
        ("", "empty text"),
        ("Ordinary text", "ordinary text"),
        (
            "A deliberately long provider-neutral text sample that remains intact when its visual bounds are constrained.",
            "long text",
        ),
        ("First line\nSecond line\nThird line", "multiline text"),
    ];
    let children = samples
        .into_iter()
        .enumerate()
        .map(|(index, (text, label))| {
            let id = UiNodeId(index as u64 + 2);
            UiNodeSpec::text(id, &UiTextSpec::new(text, viewport, UiTextRole::Body))
                .with_parent(root)
                .with_semantic_label(label)
                .with_layout(UiNodeLayout::Explicit(UiRect::new(
                    [content.center[0], top - index as f32 * line_height],
                    [content.size[0], line_height * 0.82],
                )))
        });
    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Inspector),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("content stress")
        .with_children(children),
    )
}

fn composition_layout_tree(viewport: UiRect) -> UiTree {
    let theme = UiTheme::default();
    let unit = viewport.size[0].min(viewport.size[1]);
    let inset = (unit * 0.04).clamp(0.04, 0.18);
    let gap = (unit * 0.025).clamp(0.025, 0.10);
    let frame = UiFrameLayout::new(
        viewport,
        UiInsets::uniform(inset),
        (unit * 0.12).clamp(0.22, 0.56),
        (unit * 0.10).clamp(0.18, 0.46),
        gap,
    );
    let split = UiHorizontalSplitLayout::new(frame.body, 0.66, gap, 0.20, 0.20);
    let grid = UiUniformGridLayout::new(split.leading, 4, 2, [gap, gap]);
    let stack_buttons = ["ACTION 1", "ACTION 2", "ACTION 3"]
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            UiButton::from_intrinsic(UiButtonId(index as u8 + 1), label, [0.0, 0.0], &theme)
        })
        .collect::<Vec<_>>();
    let stack_context = UiMeasureContext::new(&theme, split.trailing.size);
    let stack = UiVerticalStack::new(stack_buttons, gap)
        .with_cross_axis_alignment(UiCrossAxisAlignment::Fill)
        .with_main_axis_allocation(UiMainAxisAllocation::Fill)
        .layout(split.trailing, &stack_context);

    let root = UiNodeId(1);
    let grid_id = UiNodeId(3);
    let actions_id = UiNodeId(8);
    let grid_children = grid.cells.into_iter().enumerate().map(|(index, rect)| {
        let id = UiNodeId(index as u64 + 4);
        UiNodeSpec::new(
            id,
            UiNodeKind::Region(UiRegionKind::Card),
            UiSurfaceRole::Card,
            UiNodeLayout::Explicit(rect),
        )
        .with_parent(grid_id)
        .with_semantic_label(format!("grid card {}", index + 1))
        .with_text(UiTextSpec::new(
            format!("CARD {}", index + 1),
            viewport,
            UiTextRole::Caption,
        ))
    });
    let action_children = stack
        .children
        .into_iter()
        .enumerate()
        .map(|(index, layout)| {
            let id = UiNodeId(index as u64 + 9);
            UiNodeSpec::new(
                id,
                UiNodeKind::Button(UiButtonId(index as u8 + 1)),
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(layout.rect),
            )
            .with_parent(actions_id)
            .with_interaction(UiNodeInteraction::Activatable)
            .with_semantic_label(format!("action {}", index + 1))
            .with_text(UiTextSpec::new(
                format!("ACTION {}", index + 1),
                viewport,
                UiTextRole::Button,
            ))
        });

    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Background,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("composition layout")
        .with_child(
            UiNodeSpec::new(
                UiNodeId(2),
                UiNodeKind::Region(UiRegionKind::Header),
                UiSurfaceRole::Toolbar,
                UiNodeLayout::Explicit(frame.header),
            )
            .with_parent(root)
            .with_semantic_label("composition header")
            .with_text(UiTextSpec::new(
                "COMPOSITION",
                viewport,
                UiTextRole::Heading,
            )),
        )
        .with_child(
            UiNodeSpec::new(
                grid_id,
                UiNodeKind::Region(UiRegionKind::CardGrid),
                UiSurfaceRole::Region,
                UiNodeLayout::Explicit(split.leading),
            )
            .with_parent(root)
            .with_semantic_label("uniform card grid")
            .with_children(grid_children),
        )
        .with_child(
            UiNodeSpec::new(
                actions_id,
                UiNodeKind::Region(UiRegionKind::Inspector),
                UiSurfaceRole::Panel,
                UiNodeLayout::Explicit(split.trailing),
            )
            .with_parent(root)
            .with_semantic_label("stacked actions")
            .with_children(action_children),
        )
        .with_child(
            UiNodeSpec::new(
                UiNodeId(12),
                UiNodeKind::Region(UiRegionKind::StatusBar),
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(frame.footer),
            )
            .with_parent(root)
            .with_semantic_label("composition status")
            .with_text(UiTextSpec::new(
                "FRAME / SPLIT / STACK / GRID",
                viewport,
                UiTextRole::Status,
            )),
        ),
    )
}

fn command_toolbar_tree(viewport: UiRect) -> UiTree {
    let root = UiNodeId(1);
    let content = viewport.inset(0.3);
    let button_width = (content.size[0] / 3.5).min(2.4);
    let gap = (button_width * 0.15).max(0.1);
    let total = button_width * 3.0 + gap * 2.0;
    let left = content.center[0] - total * 0.5 + button_width * 0.5;
    let labels = ["OPEN", "SAVE", "RESET"];
    let children = labels.into_iter().enumerate().map(|(index, label)| {
        let id = UiNodeId(index as u64 + 2);
        UiNodeSpec::new(
            id,
            UiNodeKind::Button(UiButtonId(index as u8 + 1)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new(
                [
                    left + index as f32 * (button_width + gap),
                    content.center[1],
                ],
                [button_width, 0.8],
            )),
        )
        .with_parent(root)
        .with_interaction(UiNodeInteraction::Activatable)
        .with_enabled(label != "RESET")
        .with_semantic_label(label)
        .with_text(UiTextSpec::new(label, viewport, UiTextRole::Button))
    });
    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Toolbar),
            UiSurfaceRole::Toolbar,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("file commands")
        .with_children(children),
    )
}

fn text_entry_tree(viewport: UiRect) -> UiTree {
    let root = UiNodeId(1);
    let field = UiNodeId(2);
    let submit = UiNodeId(3);
    let content = viewport.inset((viewport.size[0].min(viewport.size[1]) * 0.1).max(0.12));
    let narrow = content.size[0] < 4.0;
    let field_bounds = if narrow {
        UiRect::new(
            [
                content.center[0],
                content.center[1] + content.size[1] * 0.16,
            ],
            [
                content.size[0] * 0.86,
                (content.size[1] * 0.22).clamp(0.48, 0.9),
            ],
        )
    } else {
        UiRect::new(
            [
                content.center[0] - content.size[0] * 0.12,
                content.center[1],
            ],
            [
                content.size[0] * 0.58,
                (content.size[1] * 0.24).clamp(0.48, 0.9),
            ],
        )
    };
    let submit_bounds = if narrow {
        UiRect::new(
            [
                content.center[0],
                content.center[1] - content.size[1] * 0.16,
            ],
            [content.size[0] * 0.52, field_bounds.size[1]],
        )
    } else {
        UiRect::new(
            [
                content.center[0] + content.size[0] * 0.32,
                content.center[1],
            ],
            [content.size[0] * 0.22, field_bounds.size[1]],
        )
    };

    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("text entry form")
        .with_child(
            UiNodeSpec::new(
                field,
                UiNodeKind::TextInput,
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(field_bounds),
            )
            .with_parent(root)
            .with_interaction(UiNodeInteraction::Editable)
            .with_semantic_label("command name")
            .with_semantic_value("")
            .with_text(UiTextSpec::new("", viewport, UiTextRole::Body)),
        )
        .with_child(
            UiNodeSpec::new(
                submit,
                UiNodeKind::Button(UiButtonId(1)),
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(submit_bounds),
            )
            .with_parent(root)
            .with_semantic_label("submit command")
            .with_text(UiTextSpec::new("SUBMIT", viewport, UiTextRole::Button)),
        ),
    )
}

fn scroll_modal_tree(viewport: UiRect) -> UiTree {
    let root = UiNodeId(1);
    let scroll_id = UiNodeId(2);
    let background_button_id = UiNodeId(3);
    let modal_id = UiNodeId(4);
    let modal_button_id = UiNodeId(5);
    let inset = (viewport.size[0].min(viewport.size[1]) * 0.08).max(0.12);
    let content = viewport.inset(inset);
    let scroll_bounds = UiRect::new(
        [
            content.center[0] - content.size[0] * 0.24,
            content.center[1],
        ],
        [content.size[0] * 0.42, content.size[1] * 0.8],
    );
    let mut scroll = UiVerticalScroll::new(scroll_bounds, scroll_bounds.size[1] * 2.0);
    scroll.set_offset(scroll.max_offset() * 0.35);
    let background_bounds = UiRect::new(
        [
            scroll_bounds.center[0],
            scroll_bounds.center[1] - scroll_bounds.size[1] * 0.32,
        ],
        [
            scroll_bounds.size[0] * 0.72,
            (scroll_bounds.size[1] * 0.32).max(0.3),
        ],
    );
    let modal_bounds = UiRect::new(
        [content.center[0] + content.size[0] * 0.2, content.center[1]],
        [content.size[0] * 0.48, content.size[1] * 0.56],
    );

    UiTree::new(
        UiNodeSpec::new(
            root,
            UiNodeKind::Region(UiRegionKind::Workspace),
            UiSurfaceRole::Region,
            UiNodeLayout::Fill,
        )
        .with_semantic_label("scroll and modal composition")
        .with_child(
            UiNodeSpec::new(
                scroll_id,
                UiNodeKind::Region(UiRegionKind::Panel),
                UiSurfaceRole::Panel,
                UiNodeLayout::Explicit(scroll_bounds),
            )
            .with_parent(root)
            .with_semantic_label("scroll viewport")
            .clips_children()
            .with_child_translation(scroll.content_translation())
            .with_child(
                UiNodeSpec::new(
                    background_button_id,
                    UiNodeKind::Button(UiButtonId(1)),
                    UiSurfaceRole::Raised,
                    UiNodeLayout::Explicit(background_bounds),
                )
                .with_parent(scroll_id)
                .with_semantic_label("background action")
                .with_text(UiTextSpec::new(
                    "BACKGROUND",
                    viewport,
                    UiTextRole::Button,
                )),
            ),
        )
        .with_child(
            UiNodeSpec::new(
                modal_id,
                UiNodeKind::Region(UiRegionKind::Panel),
                UiSurfaceRole::Overlay,
                UiNodeLayout::Explicit(modal_bounds),
            )
            .with_parent(root)
            .with_semantic_label("confirmation dialog")
            .as_modal(true)
            .with_child(
                UiNodeSpec::new(
                    modal_button_id,
                    UiNodeKind::Button(UiButtonId(2)),
                    UiSurfaceRole::Raised,
                    UiNodeLayout::Inset(ui_tools::consumer::UiInsets::uniform(
                        (modal_bounds.size[0].min(modal_bounds.size[1]) * 0.18).max(0.08),
                    )),
                )
                .with_parent(modal_id)
                .with_semantic_label("confirm")
                .with_text(UiTextSpec::new(
                    "CONFIRM",
                    viewport,
                    UiTextRole::Button,
                )),
            ),
        ),
    )
}

fn semantics_json(tree: &UiResolvedTree) -> String {
    let nodes = tree.semantic_nodes(&UiResolvedFocus::default());
    let records = nodes
        .iter()
        .map(|node| {
            format!(
                "    {{\"id\": {}, \"role\": \"{:?}\", \"label\": {}, \"enabled\": {}, \"focusable\": {}}}",
                node.id.0,
                node.role,
                json_optional(node.label.as_deref()),
                node.enabled,
                node.focusable
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"nodes\": [\n{records}\n  ]\n}}\n")
}

fn content_json(tree: &UiResolvedTree) -> String {
    let mut records = Vec::new();
    collect_content(&tree.root, &mut records);
    format!(
        "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"text_nodes\": [\n{}\n  ]\n}}\n",
        records.join(",\n")
    )
}

fn collect_content(node: &UiResolvedNode, records: &mut Vec<String>) {
    if let Some(text) = &node.text {
        let classification = if text.text.is_empty() {
            "empty"
        } else if text.text.contains('\n') {
            "multiline"
        } else if text.text.chars().count() > 48 {
            "long"
        } else {
            "ordinary"
        };
        records.push(format!(
            "    {{\"id\": {}, \"classification\": \"{classification}\", \"characters\": {}, \"lines\": {}, \"text\": \"{}\"}}",
            node.id.0,
            text.text.chars().count(),
            text.text.lines().count().max(1),
            json_escape(&text.text)
        ));
    }
    for child in &node.children {
        collect_content(child, records);
    }
}

fn layout_json(tree: &UiResolvedTree) -> String {
    let mut records = Vec::new();
    collect_layout(&tree.root, &mut records);
    format!(
        "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"nodes\": [\n{}\n  ]\n}}\n",
        records.join(",\n")
    )
}

fn collect_layout(node: &UiResolvedNode, records: &mut Vec<String>) {
    records.push(format!(
        "    {{\"id\": {}, \"center\": [{:.4}, {:.4}], \"size\": [{:.4}, {:.4}], \"fit\": \"{:?}\", \"visible\": {}}}",
        node.id.0,
        node.bounds.center[0],
        node.bounds.center[1],
        node.bounds.size[0],
        node.bounds.size[1],
        node.layout_fit,
        node.visible
    ));
    for child in &node.children {
        collect_layout(child, records);
    }
}

fn interaction_json(tree: &UiResolvedTree) -> String {
    let records = tree
        .interactive_node_ids()
        .into_iter()
        .filter_map(|id| tree.interactive_node(id))
        .map(|node| {
            let mut router = UiPointerRouter::default();
            let pressed = router.route(
                tree,
                UiPointerEvent::new(node.bounds.center, UiPointerPhase::Press),
            );
            let released = router.route(
                tree,
                UiPointerEvent::new(node.bounds.center, UiPointerPhase::Release),
            );
            format!(
                "    {{\"id\": {}, \"center\": [{:.4}, {:.4}], \"size\": [{:.4}, {:.4}], \"press_target\": {}, \"release_target\": {}, \"activated\": {}}}",
                node.id.0,
                node.bounds.center[0],
                node.bounds.center[1],
                node.bounds.size[0],
                node.bounds.size[1],
                json_node_id(pressed.target),
                json_node_id(released.target),
                json_node_id(released.activated)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"targets\": [\n{records}\n  ]\n}}\n")
}

fn input_sequence_json(case: &str, tree: &UiResolvedTree, viewport: UiRect) -> String {
    match case {
        "command-toolbar" => {
            let mut focus = UiResolvedFocus::default();
            let first = focus.move_focus(tree, UiFocusDirection::Forward);
            let enter = focus.activate(tree, UiActivationKey::Enter);
            let second = focus.move_focus(tree, UiFocusDirection::Forward);
            let space = focus.activate(tree, UiActivationKey::Space);
            let mut pointer = UiPointerRouter::default();
            let press = tree.interactive_node(UiNodeId(2)).map(|node| {
                pointer.route(
                    tree,
                    UiPointerEvent::new(node.bounds.center, UiPointerPhase::Press),
                )
            });
            let outside = [
                viewport.center[0] + viewport.size[0],
                viewport.center[1] + viewport.size[1],
            ];
            let moved = pointer.route(tree, UiPointerEvent::new(outside, UiPointerPhase::Move));
            let released =
                pointer.route(tree, UiPointerEvent::new(outside, UiPointerPhase::Release));
            format!(
                "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"sequence\": \"toolbar-keyboard-and-capture\",\n  \"focus_first\": {},\n  \"enter_activated\": {},\n  \"focus_second\": {},\n  \"space_activated\": {},\n  \"press_captured\": {},\n  \"move_target\": {},\n  \"release_activated\": {}\n}}\n",
                json_node_id(first),
                json_node_id(enter),
                json_node_id(second),
                json_node_id(space),
                json_node_id(press.and_then(|resolution| resolution.captured)),
                json_node_id(moved.target),
                json_node_id(released.activated)
            )
        }
        "text-entry" => {
            let mut focus = UiResolvedFocus::default();
            let focused = focus.move_focus(tree, UiFocusDirection::Forward);
            let router = UiTextInputRouter;
            let mut state = UiTextInputState::default();
            let operations = [
                UiTextInputOperation::Insert('A'),
                UiTextInputOperation::Insert(' '),
                UiTextInputOperation::Insert('7'),
                UiTextInputOperation::DeleteBackward,
                UiTextInputOperation::Insert('9'),
            ];
            let mut targets = Vec::new();
            for operation in operations {
                let resolution = router.route(tree, &mut focus, UiTextInputEvent::new(operation));
                targets.push(json_node_id(resolution.target));
                if resolution.target.is_some() {
                    state.apply(resolution.operation);
                }
            }
            let submit_focus = focus.move_focus(tree, UiFocusDirection::Forward);
            let submit_activation = focus.activate(tree, UiActivationKey::Enter);
            format!(
                "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"sequence\": \"text-edit-and-submit\",\n  \"field_focus\": {},\n  \"edit_targets\": [{}],\n  \"value\": \"{}\",\n  \"caret\": {},\n  \"submit_focus\": {},\n  \"submit_activated\": {}\n}}\n",
                json_node_id(focused),
                targets.join(", "),
                json_escape(state.value()),
                state.caret(),
                json_node_id(submit_focus),
                json_node_id(submit_activation)
            )
        }
        "scroll-modal" => {
            let mut focus = UiResolvedFocus::default();
            let focused = focus.move_focus(tree, UiFocusDirection::Forward);
            let activated = focus.activate(tree, UiActivationKey::Enter);
            format!(
                "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"sequence\": \"modal-focus-confinement\",\n  \"focus\": {},\n  \"activated\": {}\n}}\n",
                json_node_id(focused),
                json_node_id(activated)
            )
        }
        _ => format!("{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"sequence\": \"none\"\n}}\n"),
    }
}

fn draw_list_json(draw_list: &UiDrawList) -> String {
    let statistics = draw_list.statistics();
    format!(
        "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"revision\": {},\n  \"cache_key\": {},\n  \"entries\": {},\n  \"surfaces\": {},\n  \"text\": {},\n  \"clip_pushes\": {},\n  \"clip_pops\": {},\n  \"surface_batch_candidates\": {},\n  \"text_batch_candidates\": {}\n}}\n",
        draw_list.revision,
        draw_list.cache_key().value(),
        statistics.entries,
        statistics.surfaces,
        statistics.text,
        statistics.clip_pushes,
        statistics.clip_pops,
        statistics.surface_batch_candidates,
        statistics.text_batch_candidates
    )
}

fn diagnostics_json(tree: &UiResolvedTree, draw_list: &UiDrawList) -> String {
    format!(
        "{{\n  \"schema\": \"{ARTIFACT_SCHEMA}\",\n  \"tree_count\": {},\n  \"draw_list_count\": {},\n  \"tree\": \"{}\",\n  \"draw_list\": \"{}\"\n}}\n",
        tree.diagnostics.len(),
        draw_list.diagnostics.len(),
        json_escape(&format!("{:?}", tree.diagnostics)),
        json_escape(&format!("{:?}", draw_list.diagnostics))
    )
}

fn coverage_json(selected: Option<&str>, executed_cases: &[&str]) -> String {
    let viewport_count = format!("\"executed_viewports_per_case\": {},", VIEWPORTS.len());
    let viewport_and_scale_count = format!(
        "{viewport_count}\n  \"executed_scales_per_viewport\": {},",
        SCALES.len()
    );
    coverage_json_base(selected, executed_cases)
        .replace(
            "\"id\": \"deterministic-cpu-image\", \"status\": \"open\"",
            "\"id\": \"deterministic-cpu-image\", \"status\": \"covered\"",
        )
        .replace(
            &viewport_count,
            &viewport_and_scale_count,
        )
        .replace(
            "\"dimension\": \"scale\", \"status\": \"partial\", \"required\": [\"1.0\", \"1.5\", \"2.0\"], \"observed\": [\"1.0\"]",
            "\"dimension\": \"scale\", \"status\": \"covered\", \"required\": [\"1.0\", \"1.5\", \"2.0\"], \"observed\": [\"1.0\", \"1.5\", \"2.0\"]",
        )
        .replace(
            "\"dimension\": \"content\", \"status\": \"partial\", \"required\": [\"empty\", \"ordinary\", \"long\", \"multiline\", \"missing-glyph\"], \"observed\": [\"ordinary\"]",
            "\"dimension\": \"content\", \"status\": \"partial\", \"required\": [\"empty\", \"ordinary\", \"long\", \"multiline\", \"missing-glyph\"], \"observed\": [\"empty\", \"ordinary\", \"long\", \"multiline\"]",
        )
        .replace(
            "\"dimension\": \"composition\", \"status\": \"partial\", \"required\": [\"frame\", \"split\", \"stack\", \"grid\", \"scroll\", \"overlay\", \"modal\"], \"observed\": [\"scroll\", \"overlay\", \"modal\"]",
            "\"dimension\": \"composition\", \"status\": \"covered\", \"required\": [\"frame\", \"split\", \"stack\", \"grid\", \"scroll\", \"overlay\", \"modal\"], \"observed\": [\"frame\", \"split\", \"stack\", \"grid\", \"scroll\", \"overlay\", \"modal\"]",
        )
}

fn coverage_json_base(selected: Option<&str>, executed_cases: &[&str]) -> String {
    let executed = executed_cases
        .iter()
        .map(|case| format!("\"{}\"", json_escape(case)))
        .collect::<Vec<_>>()
        .join(", ");
    let scope = selected.unwrap_or("full-selection");
    format!(
        "{{\n  \"schema\": \"tokimu-ui-coverage-v1\",\n  \"generator\": \"{GENERATOR}\",\n  \"selection\": \"selection-v1.toml\",\n  \"run_scope\": \"{scope}\",\n  \"executed_cases\": [{executed}],\n  \"executed_viewports_per_case\": {},\n  \"behaviors\": [\n    {{\"id\": \"semantic-resolution\", \"status\": \"covered\"}},\n    {{\"id\": \"constraint-safe-layout\", \"status\": \"covered\"}},\n    {{\"id\": \"interaction-routing\", \"status\": \"covered\"}},\n    {{\"id\": \"normalized-input-sequences\", \"status\": \"covered\"}},\n    {{\"id\": \"ordered-draw-lowering\", \"status\": \"covered\"}},\n    {{\"id\": \"bounded-diagnostics\", \"status\": \"covered\"}},\n    {{\"id\": \"deterministic-cpu-image\", \"status\": \"open\"}},\n    {{\"id\": \"native-backend-image\", \"status\": \"manual\"}}\n  ],\n  \"matrix\": [\n    {{\"dimension\": \"viewport\", \"status\": \"covered\", \"required\": [\"1920x1080\", \"1280x720\", \"900x600\", \"640x480\", \"320x568\"], \"observed\": [\"1920x1080\", \"1280x720\", \"900x600\", \"640x480\", \"320x568\"]}},\n    {{\"dimension\": \"scale\", \"status\": \"partial\", \"required\": [\"1.0\", \"1.5\", \"2.0\"], \"observed\": [\"1.0\"]}},\n    {{\"dimension\": \"text-provider\", \"status\": \"open\", \"required\": [\"built-in\", \"TTF\", \"OTF\", \"missing-provider\"], \"observed\": [\"none-provider-neutral\"]}},\n    {{\"dimension\": \"content\", \"status\": \"partial\", \"required\": [\"empty\", \"ordinary\", \"long\", \"multiline\", \"missing-glyph\"], \"observed\": [\"ordinary\"]}},\n    {{\"dimension\": \"input\", \"status\": \"covered\", \"required\": [\"pointer\", \"keyboard\", \"text-input\", \"capture\", \"disabled-control\"], \"observed\": [\"pointer\", \"keyboard\", \"text-input\", \"capture\", \"disabled-control\"]}},\n    {{\"dimension\": \"composition\", \"status\": \"partial\", \"required\": [\"frame\", \"split\", \"stack\", \"grid\", \"scroll\", \"overlay\", \"modal\"], \"observed\": [\"scroll\", \"overlay\", \"modal\"]}},\n    {{\"dimension\": \"mutation\", \"status\": \"partial\", \"required\": [\"static\", \"interaction\", \"text\", \"theme\", \"resize\", \"content-replacement\"], \"observed\": [\"static\", \"interaction\", \"text\"]}},\n    {{\"dimension\": \"target\", \"status\": \"partial\", \"required\": [\"headless\", \"native\", \"WASM\"], \"observed\": [\"headless\"]}}\n  ]\n}}\n",
        VIEWPORTS.len()
    )
}

fn count_nodes(node: &UiResolvedNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn json_optional(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_node_id(value: Option<UiNodeId>) -> String {
    value
        .map(|id| id.0.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn stable_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_selected_case_resolves_at_every_required_viewport() {
        for case in CASES {
            for &(_, width, height) in VIEWPORTS {
                for &(_, scale) in SCALES {
                    let viewport = UiRect::new(
                        [0.0, 0.0],
                        [
                            width as f32 / (100.0 * scale),
                            height as f32 / (100.0 * scale),
                        ],
                    );
                    let tree = case_tree(case, viewport).unwrap();
                    let resolved = tree.resolve(viewport).unwrap();
                    let draw_list =
                        lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);
                    assert!(!draw_list.entries().is_empty());
                    assert!(resolved.diagnostics.is_empty());
                    assert!(draw_list.diagnostics.is_empty());
                }
            }
        }
    }

    #[test]
    fn structural_artifacts_are_deterministic() {
        for case in CASES {
            let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
            let first = case_tree(case, viewport)
                .unwrap()
                .resolve(viewport)
                .unwrap();
            let second = case_tree(case, viewport)
                .unwrap()
                .resolve(viewport)
                .unwrap();
            let first_draw = lower_resolved_tree_to_draw_list(&first, &UiTheme::default(), 1);
            let second_draw = lower_resolved_tree_to_draw_list(&second, &UiTheme::default(), 1);

            assert_eq!(semantics_json(&first), semantics_json(&second));
            assert_eq!(content_json(&first), content_json(&second));
            assert_eq!(layout_json(&first), layout_json(&second));
            assert_eq!(interaction_json(&first), interaction_json(&second));
            assert_eq!(
                input_sequence_json(case, &first, viewport),
                input_sequence_json(case, &second, viewport)
            );
            assert_eq!(draw_list_json(&first_draw), draw_list_json(&second_draw));
            assert_eq!(
                diagnostics_json(&first, &first_draw),
                diagnostics_json(&second, &second_draw)
            );
        }
    }

    #[test]
    fn versioned_selection_declares_the_compiled_matrix() {
        assert!(SELECTION.contains("schema = \"tokimu-ui-selection-v1\""));
        for case in CASES {
            assert!(SELECTION.contains(&format!("id = \"{case}\"")));
        }
        for &(viewport, width, height) in VIEWPORTS {
            assert!(SELECTION.contains(&format!("id = \"{viewport}\"")));
            assert!(SELECTION.contains(&format!("width = {width}")));
            assert!(SELECTION.contains(&format!("height = {height}")));
        }
        for &(scale_id, scale) in SCALES {
            assert!(SELECTION.contains(&format!("id = \"{scale_id}\"")));
            assert!(SELECTION.contains(&format!("value = {scale:.1}")));
        }
    }

    #[test]
    fn coverage_report_names_covered_partial_and_open_dimensions() {
        let report = coverage_json(None, CASES);

        assert!(report.contains("\"run_scope\": \"full-selection\""));
        assert!(report.contains("\"dimension\": \"viewport\", \"status\": \"covered\""));
        assert!(report.contains("\"dimension\": \"input\", \"status\": \"covered\""));
        assert!(report.contains("\"dimension\": \"scale\", \"status\": \"covered\""));
        assert!(report.contains("\"dimension\": \"composition\", \"status\": \"covered\""));
        assert!(report.contains("\"dimension\": \"text-provider\", \"status\": \"open\""));
        assert!(report.contains("\"observed\": [\"empty\", \"ordinary\", \"long\", \"multiline\"]"));
        assert!(report.contains("\"id\": \"deterministic-cpu-image\", \"status\": \"covered\""));
    }

    #[test]
    fn content_stress_preserves_provider_neutral_text() {
        let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
        let resolved = content_stress_tree(viewport).resolve(viewport).unwrap();
        let artifact = content_json(&resolved);

        assert!(artifact.contains("\"classification\": \"empty\""));
        assert!(artifact.contains("\"classification\": \"ordinary\""));
        assert!(artifact.contains("\"classification\": \"long\""));
        assert!(artifact.contains("\"classification\": \"multiline\""));
        assert!(artifact.contains("First line\\nSecond line\\nThird line"));
    }

    #[test]
    fn composition_case_uses_shared_layout_primitives_without_overlap() {
        let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
        let resolved = composition_layout_tree(viewport).resolve(viewport).unwrap();
        let grid = resolved
            .root
            .children
            .iter()
            .find(|node| node.id == UiNodeId(3))
            .unwrap();
        let actions = resolved
            .root
            .children
            .iter()
            .find(|node| node.id == UiNodeId(8))
            .unwrap();

        assert_eq!(grid.children.len(), 4);
        assert_eq!(actions.children.len(), 3);
        assert!(grid.bounds.intersection(actions.bounds).is_none());
        for (index, child) in grid.children.iter().enumerate() {
            for other in grid.children.iter().skip(index + 1) {
                assert!(child.bounds.intersection(other.bounds).is_none());
            }
        }
    }

    #[test]
    fn selected_coverage_report_does_not_claim_a_full_run() {
        let report = coverage_json(Some("text-entry"), &["text-entry"]);

        assert!(report.contains("\"run_scope\": \"text-entry\""));
        assert!(report.contains("\"executed_cases\": [\"text-entry\"]"));
        assert!(!report.contains("\"executed_cases\": [\"runtime-observation\""));
    }

    #[test]
    fn disabled_toolbar_control_is_present_but_not_routable() {
        let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
        let resolved = command_toolbar_tree(viewport).resolve(viewport).unwrap();
        let semantics = semantics_json(&resolved);
        let interaction = interaction_json(&resolved);

        assert!(semantics.contains("\"id\": 4, \"role\": \"Button\""));
        assert!(semantics.contains("\"label\": \"RESET\", \"enabled\": false"));
        assert!(!interaction.contains("\"id\": 4"));
        assert!(interaction.contains("\"id\": 2"));
        assert!(interaction.contains("\"id\": 3"));
    }

    #[test]
    fn modal_case_preserves_background_draw_evidence_but_confines_interaction() {
        let viewport = UiRect::new([0.0, 0.0], [3.2, 5.68]);
        let resolved = scroll_modal_tree(viewport).resolve(viewport).unwrap();
        let draw_list = lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), 1);

        assert_eq!(
            resolved.active_modal().map(|node| node.id),
            Some(UiNodeId(4))
        );
        assert_eq!(resolved.interactive_node_ids(), vec![UiNodeId(5)]);
        assert!(layout_json(&resolved).contains("\"id\": 3"));
        assert!(draw_list
            .entries()
            .iter()
            .any(|entry| entry.source == Some(UiNodeId(3))));
        assert!(!semantics_json(&resolved).contains("background action"));
        assert!(interaction_json(&resolved).contains("\"activated\": 5"));
    }

    #[test]
    fn recorded_input_sequences_cover_keyboard_text_and_pointer_capture() {
        let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
        let toolbar = command_toolbar_tree(viewport).resolve(viewport).unwrap();
        let toolbar_sequence = input_sequence_json("command-toolbar", &toolbar, viewport);
        assert!(toolbar_sequence.contains("\"enter_activated\": 2"));
        assert!(toolbar_sequence.contains("\"space_activated\": 3"));
        assert!(toolbar_sequence.contains("\"press_captured\": 2"));
        assert!(toolbar_sequence.contains("\"move_target\": 2"));
        assert!(toolbar_sequence.contains("\"release_activated\": null"));

        let text_entry = text_entry_tree(viewport).resolve(viewport).unwrap();
        let text_sequence = input_sequence_json("text-entry", &text_entry, viewport);
        assert!(text_sequence.contains("\"field_focus\": 2"));
        assert!(text_sequence.contains("\"edit_targets\": [2, 2, 2, 2, 2]"));
        assert!(text_sequence.contains("\"value\": \"A 9\""));
        assert!(text_sequence.contains("\"caret\": 3"));
        assert!(text_sequence.contains("\"submit_activated\": 3"));
    }
}
