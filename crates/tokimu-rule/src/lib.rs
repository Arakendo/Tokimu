use serde::{Deserialize, Serialize};
use std::fmt;

use tokimu_core::scene::SceneDoc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Auto,
    Lowered,
    Runtime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub name: String,
    pub execution: ExecutionMode,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub signals: Vec<String>,
}

impl RuleDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            execution: ExecutionMode::Auto,
            inputs: Vec::new(),
            outputs: Vec::new(),
            signals: Vec::new(),
        }
    }

    pub fn with_execution(mut self, execution: ExecutionMode) -> Self {
        self.execution = execution;
        self
    }

    pub fn with_inputs<I, S>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inputs = inputs.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_outputs<I, S>(mut self, outputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.outputs = outputs.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_signals<I, S>(mut self, signals: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.signals = signals.into_iter().map(Into::into).collect();
        self
    }

    pub fn lower(&self) -> LoweringOutcome {
        match self.execution {
            ExecutionMode::Runtime => LoweringOutcome::RuntimeOnly(self.clone()),
            ExecutionMode::Auto | ExecutionMode::Lowered => {
                LoweringOutcome::Lowered(RuntimeSystemPlan::from_rule(self))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSystemPlan {
    pub name: String,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub emits: Vec<String>,
}

impl RuntimeSystemPlan {
    pub fn from_rule(rule: &RuleDefinition) -> Self {
        Self {
            name: rule.name.clone(),
            reads: rule.inputs.clone(),
            writes: rule.outputs.clone(),
            emits: rule.signals.clone(),
        }
    }
}

impl fmt::Display for RuntimeSystemPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "rule {}", self.name)?;
        writeln!(f, "  reads: {:?}", self.reads)?;
        writeln!(f, "  writes: {:?}", self.writes)?;
        writeln!(f, "  emits: {:?}", self.emits)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweringOutcome {
    Lowered(RuntimeSystemPlan),
    RuntimeOnly(RuleDefinition),
}

pub fn lower_scene_document(scene: &SceneDoc) -> RuntimeSystemPlan {
    let mut reads = Vec::new();
    let mut writes = vec!["World".to_string()];
    let mut emits = vec!["scene-compiled".to_string()];

    reads.push("SceneDoc".to_string());
    if !scene.entities.is_empty() {
        writes.push("SceneEntity".to_string());
    }
    if scene.entities.iter().any(|entity| entity.parent.is_some()) {
        writes.push("SceneParent".to_string());
    }
    if scene
        .entities
        .iter()
        .any(|entity| entity.position.is_some())
    {
        writes.push("ScenePosition".to_string());
    }
    if scene.entities.len() > 1 {
        emits.push("scene-has-multiple-entities".to_string());
    }

    RuntimeSystemPlan {
        name: "scene-compile".to_string(),
        reads,
        writes,
        emits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokimu_core::scene::{SceneDoc, SceneEntityDoc, ScenePosition};

    #[test]
    fn lowered_rule_uses_declared_inputs_outputs_and_signals() {
        let rule = RuleDefinition::new("enemy-step")
            .with_execution(ExecutionMode::Lowered)
            .with_inputs(["Transform", "Velocity"])
            .with_outputs(["Transform"])
            .with_signals(["enemy-stepped"]);

        match rule.lower() {
            LoweringOutcome::Lowered(plan) => {
                assert_eq!(
                    plan,
                    RuntimeSystemPlan {
                        name: "enemy-step".into(),
                        reads: vec!["Transform".into(), "Velocity".into()],
                        writes: vec!["Transform".into()],
                        emits: vec!["enemy-stepped".into()],
                    }
                );
            }
            LoweringOutcome::RuntimeOnly(_) => panic!("lowered rule should lower"),
        }
    }

    #[test]
    fn runtime_rule_stays_runtime_only() {
        let rule = RuleDefinition::new("quest-dialogue")
            .with_execution(ExecutionMode::Runtime)
            .with_inputs(["QuestState"])
            .with_outputs(["DialogueUI"])
            .with_signals(["dialogue-opened"]);

        match rule.lower() {
            LoweringOutcome::RuntimeOnly(runtime_rule) => {
                assert_eq!(runtime_rule.name, "quest-dialogue");
            }
            LoweringOutcome::Lowered(_) => panic!("runtime rule should not lower"),
        }
    }

    #[test]
    fn scene_projection_and_rule_lower_to_the_same_system_plan() {
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

        let scene_plan = lower_scene_document(&scene);
        let rule_plan = RuntimeSystemPlan::from_rule(
            &RuleDefinition::new("scene-compile")
                .with_execution(ExecutionMode::Lowered)
                .with_inputs(["SceneDoc"])
                .with_outputs(["World", "SceneEntity", "SceneParent", "ScenePosition"])
                .with_signals(["scene-compiled", "scene-has-multiple-entities"]),
        );

        assert_eq!(scene_plan, rule_plan);
    }
}
