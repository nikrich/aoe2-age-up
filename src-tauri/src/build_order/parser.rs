use anyhow::{bail, Context, Result};
use std::path::Path;
use tracing::warn;

use super::{BuildOrder, Trigger};

/// Load a build order from a YAML or JSON file, determined by extension.
pub fn load_build_order(path: &Path) -> Result<BuildOrder> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read build order file: {}", path.display()))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let bo: BuildOrder = match ext {
        "yaml" | "yml" => {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML from {}", path.display()))?
        }
        "json" => {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON from {}", path.display()))?
        }
        other => bail!("Unsupported file extension: '{}'. Expected yaml, yml, or json.", other),
    };

    let warnings = validate_build_order(&bo);
    for w in &warnings {
        warn!("{}", w);
    }

    Ok(bo)
}

/// Validate a build order and return a list of warning messages.
pub fn validate_build_order(bo: &BuildOrder) -> Vec<String> {
    let mut warnings = Vec::new();

    for (i, step) in bo.steps.iter().enumerate() {
        if !step.at.has_any_condition() {
            warnings.push(format!(
                "Step {} (\"{}\") has no trigger conditions — auto-advance will never fire",
                i + 1,
                step.action,
            ));
        }
    }

    // Trigger values of different kinds are incommensurable: a step fired by
    // `villagers: 14` after one fired by `time_seconds: 210` is perfectly
    // ordered, so each field is checked against its own series (in file
    // order) instead of one merged list where 14 would look "before" 210.
    let series: [(&str, fn(&Trigger) -> Option<u32>); 3] = [
        ("time_seconds", |t| t.time_seconds),
        ("villagers", |t| t.villagers),
        ("population_min", |t| t.population_min),
    ];

    for (field, get) in series {
        let values: Vec<u32> = bo.steps.iter().filter_map(|s| get(&s.at)).collect();
        for pair in values.windows(2) {
            if pair[1] < pair[0] {
                warnings.push(format!(
                    "Build order steps appear to be out of order: `{}` goes {} -> {}",
                    field, pair[0], pair[1]
                ));
                break; // one warning per field is enough
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const SAMPLE_YAML: &str = r#"
id: scouts-generic
name: "Scouts (Generic)"
civilization: Generic
author: Community
tags: [scouts, beginner-friendly]
steps:
  - action: "6 vills on sheep"
    at: { time_seconds: 0 }
  - action: "Lure boar"
    at: { villagers: 10 }
  - action: "Click up to Feudal"
    at: { villagers: 21, food_min: 500 }
"#;

    const SAMPLE_JSON: &str = r#"{
  "id": "archers-britons",
  "name": "Archers (Britons)",
  "civilization": "Britons",
  "tags": ["archers"],
  "steps": [
    { "action": "6 vills on sheep", "at": { "time_seconds": 0 } },
    { "action": "Lure boar", "at": { "villagers": 10 } }
  ]
}"#;

    fn write_temp_file(suffix: &str, content: &str) -> NamedTempFile {
        let mut f = tempfile::Builder::new()
            .suffix(suffix)
            .tempfile()
            .expect("failed to create temp file");
        f.write_all(content.as_bytes())
            .expect("failed to write temp file");
        f.flush().expect("failed to flush");
        f
    }

    #[test]
    fn test_load_yaml() {
        let f = write_temp_file(".yaml", SAMPLE_YAML);
        let bo = load_build_order(f.path()).expect("should load YAML");
        assert_eq!(bo.id, "scouts-generic");
        assert_eq!(bo.name, "Scouts (Generic)");
        assert_eq!(bo.steps.len(), 3);
    }

    #[test]
    fn test_load_json() {
        let f = write_temp_file(".json", SAMPLE_JSON);
        let bo = load_build_order(f.path()).expect("should load JSON");
        assert_eq!(bo.id, "archers-britons");
        assert_eq!(bo.steps.len(), 2);
    }

    #[test]
    fn test_load_yml_extension() {
        let f = write_temp_file(".yml", SAMPLE_YAML);
        let bo = load_build_order(f.path()).expect("should load .yml");
        assert_eq!(bo.id, "scouts-generic");
    }

    #[test]
    fn test_load_unknown_extension_fails() {
        let f = write_temp_file(".txt", SAMPLE_YAML);
        let result = load_build_order(f.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file_fails() {
        let result = load_build_order(Path::new("/tmp/nonexistent-bo-file.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_warns_on_empty_trigger() {
        let yaml = r#"
id: test-empty-trigger
name: "Test"
civilization: Generic
steps:
  - action: "Do something"
    at: {}
"#;
        let f = write_temp_file(".yaml", yaml);
        let bo = load_build_order(f.path()).expect("should load");
        let warnings = validate_build_order(&bo);
        assert!(!warnings.is_empty());
        assert!(
            warnings.iter().any(|w| w.contains("no trigger conditions")),
            "expected warning about no trigger conditions, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_validate_no_warnings_on_valid_bo() {
        let f = write_temp_file(".yaml", SAMPLE_YAML);
        let bo = load_build_order(f.path()).expect("should load");
        let warnings = validate_build_order(&bo);
        assert!(
            warnings.is_empty(),
            "expected no warnings, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_validate_allows_interleaved_time_and_villager_triggers() {
        // A time-based step between villager-based steps is normal (e.g. an
        // optional deer lure at 210s right before "6 more vills at 14").
        let yaml = r#"
id: test-interleaved
name: "Test"
civilization: Generic
steps:
  - action: "6 vills on sheep"
    at: { time_seconds: 0 }
  - action: "Optional lure at 210s"
    at: { time_seconds: 210 }
  - action: "6 more vills to hunt"
    at: { villagers: 14 }
  - action: "2 more vills to wood"
    at: { villagers: 17 }
  - action: "More time-based steps"
    at: { time_seconds: 570 }
"#;
        let f = write_temp_file(".yaml", yaml);
        let bo = load_build_order(f.path()).expect("should load");
        let warnings = validate_build_order(&bo);
        assert!(
            warnings.is_empty(),
            "interleaved triggers should not warn, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_validate_warns_when_time_series_goes_backwards() {
        let yaml = r#"
id: test-time-backwards
name: "Test"
civilization: Generic
steps:
  - action: "Later step"
    at: { time_seconds: 300 }
  - action: "Earlier step"
    at: { time_seconds: 210 }
"#;
        let f = write_temp_file(".yaml", yaml);
        let bo = load_build_order(f.path()).expect("should load");
        let warnings = validate_build_order(&bo);
        assert!(
            warnings.iter().any(|w| w.contains("time_seconds") && w.contains("300 -> 210")),
            "expected a backwards time_seconds warning, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_validate_warns_when_villager_series_goes_backwards() {
        let yaml = r#"
id: test-vills-backwards
name: "Test"
civilization: Generic
steps:
  - action: "Later step"
    at: { villagers: 21 }
  - action: "Earlier step"
    at: { villagers: 14 }
"#;
        let f = write_temp_file(".yaml", yaml);
        let bo = load_build_order(f.path()).expect("should load");
        let warnings = validate_build_order(&bo);
        assert!(
            warnings.iter().any(|w| w.contains("villagers") && w.contains("21 -> 14")),
            "expected a backwards villagers warning, got: {:?}",
            warnings
        );
    }
}
