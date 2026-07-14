use tokimu_core::scene::{SceneDoc, SceneEntityDoc, ScenePosition};
use tokimu_rule::{ExecutionMode, RuleDefinition, RuntimeSystemPlan};

fn main() {
    let lowered_rule = RuleDefinition::new("scene-compile")
        .with_execution(ExecutionMode::Lowered)
        .with_inputs(["SceneDoc"])
        .with_outputs(["World", "SceneEntity", "SceneParent", "ScenePosition"])
        .with_signals(["scene-compiled", "scene-has-multiple-entities"]);

    let runtime_rule = RuleDefinition::new("quest-dialogue")
        .with_execution(ExecutionMode::Runtime)
        .with_inputs(["QuestState"])
        .with_outputs(["DialogueUI"])
        .with_signals(["dialogue-opened"]);

    let scene = SceneDoc {
        entities: vec![
            SceneEntityDoc {
                position: Some(ScenePosition { x: 1.0, y: 2.0 }),
                parent: None,
            },
            SceneEntityDoc {
                position: None,
                parent: Some(0),
            },
        ],
    };

    let lowered_plan = RuntimeSystemPlan::from_rule(&lowered_rule);
    let scene_plan = tokimu_rule::lower_scene_document(&scene);

    println!("lowered rule:\n{lowered_plan}");
    println!("runtime rule: {:?}", runtime_rule.lower());
    println!("scene projection:\n{scene_plan}");
    println!("scene and rule share a runtime-system plan: {}", lowered_plan == scene_plan);
}
