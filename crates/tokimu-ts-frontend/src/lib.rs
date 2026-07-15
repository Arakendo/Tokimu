use serde::{Deserialize, Serialize};

use tokimu_rule::{ExecutionMode, LoweringOutcome, RuleDefinition, RuntimeSystemPlan};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TsAuthoringPackage {
    pub package_name: String,
    pub entrypoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TsLoweringDiagnostic {
    pub message: String,
    pub package_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TsRecognizedApiCall {
    Rule,
    Query,
    Signal,
    Relation,
    Command,
    DeterministicLoop,
    Arithmetic,
}

impl TsRecognizedApiCall {
    pub fn label(&self) -> &'static str {
        match self {
            TsRecognizedApiCall::Rule => "rule",
            TsRecognizedApiCall::Query => "query",
            TsRecognizedApiCall::Signal => "signal",
            TsRecognizedApiCall::Relation => "relation",
            TsRecognizedApiCall::Command => "command",
            TsRecognizedApiCall::DeterministicLoop => "deterministic-loops",
            TsRecognizedApiCall::Arithmetic => "arithmetic",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TsLoweringPlan {
    pub lowered: Vec<RuntimeSystemPlan>,
    pub runtime_only: Vec<RuleDefinition>,
    pub diagnostics: Vec<TsLoweringDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TsSourceLoweringPlan {
    pub lowered: Vec<RuntimeSystemPlan>,
    pub runtime_only: Vec<RuleDefinition>,
    pub recognized_calls: Vec<TsRecognizedApiCall>,
    pub recognized_constructs: Vec<String>,
    pub diagnostics: Vec<TsLoweringDiagnostic>,
}

#[derive(Clone, Debug, Default)]
pub struct TokimuTsFrontendHost;

impl TokimuTsFrontendHost {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_rules(
        &self,
        package: &TsAuthoringPackage,
        rules: &[RuleDefinition],
    ) -> TsLoweringPlan {
        let mut lowered = Vec::new();
        let mut runtime_only = Vec::new();
        let mut diagnostics = Vec::new();

        for rule in rules {
            match self.lower_rule(rule) {
                LoweringOutcome::Lowered(plan) => lowered.push(plan),
                LoweringOutcome::RuntimeOnly(runtime_rule) => {
                    diagnostics.push(TsLoweringDiagnostic {
                        message: format!(
                            "rule `{}` stays runtime in package `{}`",
                            runtime_rule.name, package.package_name
                        ),
                        package_name: package.package_name.clone(),
                    });
                    runtime_only.push(runtime_rule);
                }
            }
        }

        TsLoweringPlan {
            lowered,
            runtime_only,
            diagnostics,
        }
    }

    pub fn lower_rule(&self, rule: &RuleDefinition) -> LoweringOutcome {
        match rule.execution {
            ExecutionMode::Runtime => LoweringOutcome::RuntimeOnly(rule.clone()),
            ExecutionMode::Auto | ExecutionMode::Lowered => {
                LoweringOutcome::Lowered(RuntimeSystemPlan::from_rule(rule))
            }
        }
    }

    pub fn lower_authoring_source(
        &self,
        package: &TsAuthoringPackage,
        source: &str,
    ) -> TsSourceLoweringPlan {
        let mut lowered = Vec::new();
        let mut runtime_only = Vec::new();
        let mut recognized_calls = Vec::new();
        let mut recognized_constructs = Vec::new();
        let mut diagnostics = Vec::new();

        let mut cursor = 0;
        while let Some(rule_offset) = source[cursor..].find("rule(") {
            let rule_start = cursor + rule_offset;
            match parse_rule_declaration(&source[rule_start..]) {
                Ok((rule, consumed, blocked_constructs, supported_constructs)) => {
                    recognized_calls.extend(supported_constructs.iter().cloned());
                    recognized_constructs.extend(
                        supported_constructs
                            .into_iter()
                            .map(|call| call.label().to_string()),
                    );

                    if !blocked_constructs.is_empty() {
                        diagnostics.push(TsLoweringDiagnostic {
                            message: format!(
                                "rule `{}` in package `{}` cannot lower because it uses {}",
                                rule.name,
                                package.package_name,
                                blocked_constructs.join(", ")
                            ),
                            package_name: package.package_name.clone(),
                        });
                        runtime_only.push(rule);
                    } else {
                        match self.lower_rule(&rule) {
                            LoweringOutcome::Lowered(plan) => lowered.push(plan),
                            LoweringOutcome::RuntimeOnly(runtime_rule) => {
                                diagnostics.push(TsLoweringDiagnostic {
                                    message: format!(
                                        "rule `{}` stays runtime in package `{}`",
                                        runtime_rule.name, package.package_name
                                    ),
                                    package_name: package.package_name.clone(),
                                });
                                runtime_only.push(runtime_rule);
                            }
                        }
                    }

                    cursor = rule_start + consumed;
                }
                Err(message) => {
                    diagnostics.push(TsLoweringDiagnostic {
                        message: format!("{} in package `{}`", message, package.package_name),
                        package_name: package.package_name.clone(),
                    });
                    break;
                }
            }
        }

        TsSourceLoweringPlan {
            lowered,
            runtime_only,
            recognized_calls,
            recognized_constructs,
            diagnostics,
        }
    }
}

#[allow(clippy::type_complexity)]
fn parse_rule_declaration(
    source: &str,
) -> Result<
    (
        RuleDefinition,
        usize,
        Vec<&'static str>,
        Vec<TsRecognizedApiCall>,
    ),
    String,
> {
    let after_rule = source
        .strip_prefix("rule(")
        .ok_or_else(|| "expected `rule(`".to_string())?;
    let mut parser = TokenParser::new(after_rule);
    let name = parser
        .parse_string_literal()
        .map_err(|error| format!("rule name parse error: {}", error))?;
    parser.consume_whitespace();
    parser
        .consume_char(',')
        .map_err(|error| format!("rule `{}`: {}", name, error))?;
    parser.consume_whitespace();
    let object_body = parser
        .parse_braced_object()
        .map_err(|error| format!("rule `{}`: {}", name, error))?;

    let execution =
        parse_execution_mode(object_body).map_err(|error| format!("rule `{}`: {}", name, error))?;
    let inputs = parse_string_array_field(object_body, "inputs");
    let outputs = parse_string_array_field(object_body, "outputs");
    let signals = parse_string_array_field(object_body, "signals");
    let blocked_constructs = collect_blocked_constructs(object_body);
    let supported_constructs = collect_supported_constructs(object_body);

    let rule = RuleDefinition {
        name,
        execution,
        inputs,
        outputs,
        signals,
    };

    Ok((
        rule,
        parser.consumed_bytes() + "rule(".len(),
        blocked_constructs,
        supported_constructs,
    ))
}

fn parse_execution_mode(object_body: &str) -> Result<ExecutionMode, String> {
    match parse_string_property(object_body, "execution").as_deref() {
        Some("runtime") => Ok(ExecutionMode::Runtime),
        Some("lowered") => Ok(ExecutionMode::Lowered),
        Some("auto") => Ok(ExecutionMode::Auto),
        Some(other) => Err(format!("unsupported execution mode `{}`", other)),
        None => Ok(ExecutionMode::Auto),
    }
}

fn parse_string_array_field(object_body: &str, field: &str) -> Vec<String> {
    let Some(field_start) = object_body.find(field) else {
        return Vec::new();
    };

    let after_field = &object_body[field_start + field.len()..];
    let Some(open_bracket_offset) = after_field.find('[') else {
        return Vec::new();
    };

    let after_open_bracket = &after_field[open_bracket_offset + 1..];
    let Some(close_bracket_offset) = after_open_bracket.find(']') else {
        return Vec::new();
    };

    let array_source = &after_open_bracket[..close_bracket_offset];
    array_source
        .split(',')
        .filter_map(|entry| parse_quoted_string(entry.trim()))
        .collect()
}

fn parse_string_property(object_body: &str, field: &str) -> Option<String> {
    let field_start = object_body.find(field)?;
    let after_field = &object_body[field_start + field.len()..];
    let colon_offset = after_field.find(':')?;
    let after_colon = after_field[colon_offset + 1..].trim_start();
    parse_quoted_string(after_colon)
}

fn collect_supported_constructs(object_body: &str) -> Vec<TsRecognizedApiCall> {
    let mut recognized = Vec::new();

    for (needle, label) in [
        ("query(", TsRecognizedApiCall::Query),
        ("signal(", TsRecognizedApiCall::Signal),
        ("relation(", TsRecognizedApiCall::Relation),
        ("command(", TsRecognizedApiCall::Command),
    ] {
        if object_body.contains(needle) && !recognized.contains(&label) {
            recognized.push(label);
        }
    }

    if has_deterministic_loop(object_body)
        && !recognized.contains(&TsRecognizedApiCall::DeterministicLoop)
    {
        recognized.push(TsRecognizedApiCall::DeterministicLoop);
    }

    if has_arithmetic(object_body) && !recognized.contains(&TsRecognizedApiCall::Arithmetic) {
        recognized.push(TsRecognizedApiCall::Arithmetic);
    }

    recognized
}

fn has_deterministic_loop(object_body: &str) -> bool {
    ["for (", "for(", "while (", "while("]
        .iter()
        .any(|needle| object_body.contains(needle))
}

fn has_arithmetic(object_body: &str) -> bool {
    [
        " += ", " -= ", " *= ", " /= ", " + ", " - ", " * ", " / ", "%=",
    ]
    .iter()
    .any(|needle| object_body.contains(needle))
}

fn parse_quoted_string(source: &str) -> Option<String> {
    let mut characters = source.chars();
    let quote = characters.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut escaped = false;
    let mut value = String::new();
    for character in characters {
        if escaped {
            value.push(character);
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if character == quote {
            return Some(value);
        }

        value.push(character);
    }

    None
}

fn collect_blocked_constructs(object_body: &str) -> Vec<&'static str> {
    let mut blocked = Vec::new();

    for (needle, label) in [
        ("window", "window"),
        ("document", "document"),
        ("console", "console"),
        ("process", "process"),
        ("require", "require"),
        ("Deno", "Deno"),
        ("Date", "Date"),
        ("Math.random", "Math.random"),
        ("fetch", "fetch"),
        ("eval", "eval"),
        ("Promise", "Promise"),
        ("async", "async"),
        ("await", "await"),
        ("setTimeout", "setTimeout"),
        ("setInterval", "setInterval"),
        ("alert", "alert"),
        ("prompt", "prompt"),
        ("confirm", "confirm"),
        ("localStorage", "localStorage"),
        ("sessionStorage", "sessionStorage"),
    ] {
        if object_body.contains(needle) && !blocked.contains(&label) {
            blocked.push(label);
        }
    }

    blocked
}

struct TokenParser<'a> {
    source: &'a str,
    consumed: usize,
}

impl<'a> TokenParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            consumed: 0,
        }
    }

    fn consumed_bytes(&self) -> usize {
        self.consumed
    }

    fn consume_whitespace(&mut self) {
        let mut consumed = 0;
        for character in self.source[self.consumed..].chars() {
            if character.is_whitespace() {
                consumed += character.len_utf8();
            } else {
                break;
            }
        }
        self.consumed += consumed;
    }

    fn consume_char(&mut self, expected: char) -> Result<(), String> {
        self.consume_whitespace();
        let Some(character) = self.source[self.consumed..].chars().next() else {
            return Err(format!("expected `{}`", expected));
        };
        if character != expected {
            return Err(format!("expected `{}`", expected));
        }
        self.consumed += character.len_utf8();
        Ok(())
    }

    fn parse_string_literal(&mut self) -> Result<String, String> {
        self.consume_whitespace();
        let source = &self.source[self.consumed..];
        let Some(value) = parse_quoted_string(source) else {
            return Err("expected string literal".to_string());
        };

        let mut consumed = 1;
        let mut escaped = false;
        for character in source[1..].chars() {
            consumed += character.len_utf8();
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == source.chars().next().unwrap() {
                self.consumed += consumed;
                return Ok(value);
            }
        }

        Err("unterminated string literal".to_string())
    }

    fn parse_braced_object(&mut self) -> Result<&'a str, String> {
        self.consume_whitespace();
        let source = &self.source[self.consumed..];
        let Some(open_offset) = source.find('{') else {
            return Err("expected `{`".to_string());
        };
        let start = self.consumed + open_offset;
        let mut depth = 0usize;
        let mut in_string: Option<char> = None;
        let mut escaped = false;

        for (offset, character) in self.source[start..].char_indices() {
            if let Some(quote) = in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if character == '\\' {
                    escaped = true;
                    continue;
                }
                if character == quote {
                    in_string = None;
                }
                continue;
            }

            match character {
                '"' | '\'' => in_string = Some(character),
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = start + offset;
                        self.consumed = end + 1;
                        return Ok(&self.source[start + 1..end]);
                    }
                }
                _ => {}
            }
        }

        Err("unterminated object literal".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_mixed_execution_rules_explicitly() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let plan = host.plan_rules(
            &package,
            &[
                RuleDefinition::new("enemy-step")
                    .with_execution(ExecutionMode::Lowered)
                    .with_inputs(["Transform"])
                    .with_outputs(["Transform"])
                    .with_signals(["enemy-stepped"]),
                RuleDefinition::new("quest-dialogue")
                    .with_execution(ExecutionMode::Runtime)
                    .with_inputs(["QuestState"])
                    .with_outputs(["DialogueUI"])
                    .with_signals(["dialogue-opened"]),
            ],
        );

        assert_eq!(plan.lowered.len(), 1);
        assert_eq!(plan.runtime_only.len(), 1);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("quest-dialogue"));
    }

    #[test]
    fn lowers_authoring_source_and_reports_runtime_rules() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let source = r#"
                import { rule } from "tokimu";

                rule("enemy-step", {
                    execution: "lowered",
                    inputs: ["Transform", "Velocity"],
                    outputs: ["Transform"],
                    signals: ["enemy-stepped"],
                    run(ctx) {
                        ctx.emit("enemy-stepped");
                    }
                });

                rule("quest-dialogue", {
                    execution: "runtime",
                    inputs: ["QuestState"],
                    outputs: ["DialogueUI"],
                    signals: ["dialogue-opened"],
                    run(ctx) {
                        ctx.emit("dialogue-opened");
                    }
                });
        "#;

        let plan = host.lower_authoring_source(&package, source);

        assert_eq!(plan.lowered.len(), 1);
        assert_eq!(plan.runtime_only.len(), 1);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("quest-dialogue"));
    }

    #[test]
    fn rejects_unsupported_constructs_in_lowered_source() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let source = r#"
                rule("enemy-step", {
                    execution: "lowered",
                    inputs: ["Transform"],
                    outputs: ["Transform"],
                    signals: ["enemy-stepped"],
                    run(ctx) {
                        const now = Date.now();
                        ctx.emit("enemy-stepped");
                    }
                });
        "#;

        let plan = host.lower_authoring_source(&package, source);

        assert!(plan.lowered.is_empty());
        assert_eq!(plan.runtime_only.len(), 1);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("Date"));
    }

    #[test]
    fn recognizes_explicit_v0_subset_constructs() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let source = r#"
                rule("enemy-step", {
                    execution: "lowered",
                    inputs: ["Transform"],
                    outputs: ["Transform"],
                    signals: ["enemy-stepped"],
                    run(ctx) {
                        for (let step = 0; step < 3; step += 1) {
                            const target = query("enemy-targets");
                            signal("enemy-stepped");
                            relation("near", target);
                            command("move");
                            const speed = 1 + step;
                            ctx.emit("enemy-stepped");
                        }
                    }
                });
        "#;

        let plan = host.lower_authoring_source(&package, source);

        assert_eq!(plan.lowered.len(), 1);
        assert!(plan.runtime_only.is_empty());
        assert!(plan.diagnostics.is_empty());
        assert!(plan.recognized_calls.contains(&TsRecognizedApiCall::Query));
        assert!(plan.recognized_calls.contains(&TsRecognizedApiCall::Signal));
        assert!(plan
            .recognized_calls
            .contains(&TsRecognizedApiCall::Relation));
        assert!(plan
            .recognized_calls
            .contains(&TsRecognizedApiCall::Command));
        assert!(plan
            .recognized_calls
            .contains(&TsRecognizedApiCall::DeterministicLoop));
        assert!(plan
            .recognized_calls
            .contains(&TsRecognizedApiCall::Arithmetic));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "query"));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "signal"));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "relation"));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "command"));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "deterministic-loops"));
        assert!(plan
            .recognized_constructs
            .iter()
            .any(|construct| construct == "arithmetic"));
    }

    #[test]
    fn recognized_api_call_labels_match_the_lowering_boundary() {
        assert_eq!(TsRecognizedApiCall::Rule.label(), "rule");
        assert_eq!(TsRecognizedApiCall::Query.label(), "query");
        assert_eq!(TsRecognizedApiCall::Signal.label(), "signal");
        assert_eq!(TsRecognizedApiCall::Relation.label(), "relation");
        assert_eq!(TsRecognizedApiCall::Command.label(), "command");
        assert_eq!(
            TsRecognizedApiCall::DeterministicLoop.label(),
            "deterministic-loops"
        );
        assert_eq!(TsRecognizedApiCall::Arithmetic.label(), "arithmetic");
    }

    #[test]
    fn rejects_ambient_dom_and_runtime_apis_in_lowered_source() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let source = r#"
                rule("enemy-step", {
                    execution: "lowered",
                    inputs: ["Transform"],
                    outputs: ["Transform"],
                    signals: ["enemy-stepped"],
                    run(ctx) {
                        const url = window.location.href;
                        const node = document.body;
                        const sample = Math.random();
                        const response = fetch(url);
                        const timer = setTimeout(() => sample, 10);
                        return Promise.resolve([node, response, timer]);
                    }
                });
        "#;

        let plan = host.lower_authoring_source(&package, source);

        assert!(plan.lowered.is_empty());
        assert_eq!(plan.runtime_only.len(), 1);
        assert_eq!(plan.diagnostics.len(), 1);
        assert!(plan.diagnostics[0].message.contains("window"));
        assert!(plan.diagnostics[0].message.contains("document"));
        assert!(plan.diagnostics[0].message.contains("Math.random"));
        assert!(plan.diagnostics[0].message.contains("fetch"));
        assert!(plan.diagnostics[0].message.contains("Promise"));
    }

    #[test]
    fn ts_authoring_rule_lowers_to_the_same_runtime_plan_as_rust_rule() {
        let host = TokimuTsFrontendHost::new();
        let package = TsAuthoringPackage {
            package_name: "@tokimu/rules".into(),
            entrypoint: "src/index.ts".into(),
        };

        let rust_rule = RuleDefinition::new("enemy-step")
            .with_execution(ExecutionMode::Lowered)
            .with_inputs(["Transform", "Velocity"])
            .with_outputs(["Transform"])
            .with_signals(["enemy-stepped"]);

        let source = r#"
                rule("enemy-step", {
                    execution: "lowered",
                    inputs: ["Transform", "Velocity"],
                    outputs: ["Transform"],
                    signals: ["enemy-stepped"],
                    run(ctx) {
                        ctx.emit("enemy-stepped");
                    }
                });
        "#;

        let source_plan = host.lower_authoring_source(&package, source);

        assert_eq!(source_plan.lowered.len(), 1);
        assert!(source_plan.runtime_only.is_empty());
        assert!(source_plan.diagnostics.is_empty());
        assert_eq!(
            source_plan.lowered[0],
            RuntimeSystemPlan::from_rule(&rust_rule)
        );
    }
}
