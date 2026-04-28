use anyhow::{bail, Context, Result};
use std::path::Path;
use tracing::warn;

use super::BuildOrder;

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

    // Check if steps are in a reasonable order based on trigger values.
    let trigger_values: Vec<Option<u32>> = bo
        .steps
        .iter()
        .map(|s| {
            s.at.time_seconds
                .or(s.at.villagers)
                .or(s.at.population_min)
        })
        .collect();

    let mut warned_order = false;
    for pair in trigger_values.windows(2) {
        if let (Some(a), Some(b)) = (pair[0], pair[1]) {
            if b < a && !warned_order {
                warnings.push(
                    "Build order steps appear to be out of order based on trigger values".to_string(),
                );
                warned_order = true;
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
}
