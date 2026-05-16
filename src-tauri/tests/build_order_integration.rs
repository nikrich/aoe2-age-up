use std::io::Write;
use tempfile::NamedTempFile;

use aoe_overlay::build_order::engine::evaluate;
use aoe_overlay::build_order::parser::{load_build_order, validate_build_order};
use aoe_overlay::state::GameState;

fn write_temp_yaml(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(".yaml")
        .tempfile()
        .expect("failed to create temp file");
    f.write_all(content.as_bytes())
        .expect("failed to write temp file");
    f.flush().expect("failed to flush");
    f
}

#[test]
fn test_full_step_advancement_through_build_order() {
    let yaml = r#"
id: test-advancement
name: "Test Step Advancement"
civilization: Generic
steps:
  - action: "Step 1 — start"
    at: { time_seconds: 0 }
  - action: "Step 2 — 10 vills"
    at: { villagers: 10 }
  - action: "Step 3 — 15 vills AND 200 food"
    at: { villagers: 15, food_min: 200 }
  - action: "Step 4 — 10 min mark"
    at: { time_seconds: 600 }
"#;

    let f = write_temp_yaml(yaml);
    let bo = load_build_order(f.path()).expect("should load test build order");
    assert_eq!(bo.steps.len(), 4);

    // Step 0 trigger: time_seconds >= 0 → true
    let state = GameState {
        game_time_seconds: Some(0),
        ..Default::default()
    };
    assert!(evaluate(&bo.steps[0].at, &state), "Step 0 should fire at time=0");

    // Step 1 trigger: villagers >= 10 → true
    let state = GameState {
        villagers: Some(10),
        ..Default::default()
    };
    assert!(evaluate(&bo.steps[1].at, &state), "Step 1 should fire at villagers=10");

    // Step 1 trigger: villagers=9 → false
    let state = GameState {
        villagers: Some(9),
        ..Default::default()
    };
    assert!(!evaluate(&bo.steps[1].at, &state), "Step 1 should NOT fire at villagers=9");

    // Step 2 trigger (AND mode): villagers=15, food=200 → true
    let state = GameState {
        villagers: Some(15),
        food: Some(200),
        ..Default::default()
    };
    assert!(evaluate(&bo.steps[2].at, &state), "Step 2 should fire at vills=15, food=200");

    // Step 2 trigger (AND mode): villagers=15, food=199 → false
    let state = GameState {
        villagers: Some(15),
        food: Some(199),
        ..Default::default()
    };
    assert!(!evaluate(&bo.steps[2].at, &state), "Step 2 should NOT fire at vills=15, food=199");

    // Step 3 trigger: time_seconds >= 600 → true
    let state = GameState {
        game_time_seconds: Some(600),
        ..Default::default()
    };
    assert!(evaluate(&bo.steps[3].at, &state), "Step 3 should fire at time=600");
}

#[test]
fn test_load_all_sample_build_orders() {
    let sample_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("build-orders");

    assert!(sample_dir.exists(), "build-orders directory should exist at {:?}", sample_dir);

    let mut count = 0;
    for entry in std::fs::read_dir(&sample_dir).expect("should read build-orders dir") {
        let entry = entry.expect("should read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let bo = load_build_order(&path)
                .unwrap_or_else(|e| panic!("Failed to load {:?}: {}", path, e));
            assert!(
                !bo.steps.is_empty(),
                "Build order {:?} should have non-empty steps",
                path
            );

            let warnings = validate_build_order(&bo);
            assert!(
                warnings.is_empty(),
                "Build order {:?} has warnings: {:?}",
                path,
                warnings
            );

            count += 1;
        }
    }

    assert_eq!(count, 5, "Expected exactly 5 sample build orders, found {}", count);
}
