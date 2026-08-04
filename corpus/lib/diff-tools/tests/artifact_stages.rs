use diff_tools::{
    compare_json_artifact_stages, JsonArtifactComparisonError, JsonArtifactStage,
    JsonComparisonConfig,
};

#[test]
fn identifies_the_earliest_supplied_divergent_stage() {
    let config = JsonComparisonConfig::default();
    let result = compare_json_artifact_stages([
        JsonArtifactStage {
            stage: "source".to_owned(),
            expected: serde_json::json!({ "contours": 2 }),
            actual: serde_json::json!({ "contours": 2 }),
            config: config.clone(),
        },
        JsonArtifactStage {
            stage: "vector".to_owned(),
            expected: serde_json::json!({ "points": 16 }),
            actual: serde_json::json!({ "points": 15 }),
            config: config.clone(),
        },
        JsonArtifactStage {
            stage: "mesh".to_owned(),
            expected: serde_json::json!({ "triangles": 8 }),
            actual: serde_json::json!({ "triangles": 7 }),
            config,
        },
    ])
    .expect("artifact comparison should succeed");

    assert_eq!(result.first_divergent_stage.as_deref(), Some("vector"));
    assert!(result.stages[0].comparison.equal);
    assert!(!result.stages[1].comparison.equal);

    let machine = diff_tools::json_artifact_summary(&result);
    assert_eq!(machine["first_divergent_stage"], "vector");
    assert_eq!(machine["stages"][1]["difference_count"], 1);

    let human = diff_tools::format_json_artifact_summary(&result);
    assert!(human.contains("first divergent stage: vector"));
    assert!(human.contains("source: equal"));
}

#[test]
fn rejects_duplicate_stage_names() {
    let error = compare_json_artifact_stages([
        JsonArtifactStage {
            stage: "vector".to_owned(),
            expected: serde_json::json!(1),
            actual: serde_json::json!(1),
            config: JsonComparisonConfig::default(),
        },
        JsonArtifactStage {
            stage: "vector".to_owned(),
            expected: serde_json::json!(2),
            actual: serde_json::json!(2),
            config: JsonComparisonConfig::default(),
        },
    ])
    .expect_err("duplicate stage names must be explicit");

    assert!(matches!(
        error,
        JsonArtifactComparisonError::DuplicateStageName { stage } if stage == "vector"
    ));
}
