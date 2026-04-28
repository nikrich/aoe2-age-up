# Phase 1: Manual-Advance Overlay — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working Tauri overlay that displays AoE2 build orders and lets users advance steps via global hotkeys.

**Architecture:** Tauri 2.0 app with a Rust backend managing build order state and a React/TypeScript frontend rendering the overlay. Backend exposes IPC commands for step navigation and emits events on state change. Global hotkeys provide control while the game has focus.

**Tech Stack:** Rust 1.95, Tauri 2.0, React 18, TypeScript, Vite, serde_yaml, global-hotkey crate, plain CSS.

---

## File Map

### Rust backend (`src-tauri/src/`)

| File | Responsibility |
|---|---|
| `main.rs` | Tauri setup, command registration, state init |
| `state.rs` | `AppState`, `GameState`, `Calibration`, `Settings` types |
| `build_order/mod.rs` | `BuildOrder`, `Step`, `Trigger`, `BuildOrderMeta` types |
| `build_order/parser.rs` | YAML/JSON loading + validation |
| `build_order/engine.rs` | Trigger evaluation logic |
| `capture/mod.rs` | `CaptureBackend` trait stub |
| `capture/windows.rs` | Empty Windows capture stub |
| `capture/fallback.rs` | Empty fallback stub |
| `ocr/mod.rs` | `OcrPipeline` trait stub |
| `ocr/preprocess.rs` | Empty stub |
| `ocr/segment.rs` | Empty stub |
| `ocr/template.rs` | Empty stub |
| `ocr/tesseract.rs` | Empty stub |
| `hotkeys.rs` | Global hotkey registration |
| `storage.rs` | File-based persistence |
| `ipc.rs` | Tauri command + event definitions |
| `error.rs` | App-level error types |

### React frontend (`src/`)

| File | Responsibility |
|---|---|
| `main.tsx` | React entry point |
| `App.tsx` | View router (Overlay, Library, Settings) |
| `components/Overlay.tsx` | Main overlay compact view |
| `components/StepCard.tsx` | Single step rendering |
| `components/BuildOrderLibrary.tsx` | Browse/load build orders |
| `components/Settings.tsx` | Read-only hotkey display (Phase 1 stub) |
| `hooks/useGameState.ts` | Subscribe to game-state events |
| `hooks/useBuildOrder.ts` | Subscribe to step-changed events |
| `hooks/useTauriCommand.ts` | Wrapper for invoke() |
| `lib/types.ts` | TypeScript types matching Rust structs |
| `styles/overlay.css` | Transparent overlay styles |

### Other

| File | Responsibility |
|---|---|
| `build-orders/scouts-generic.yaml` | Sample build order |
| `build-orders/archers-britons.yaml` | Sample build order |
| `build-orders/fast-castle-cavalier.yaml` | Sample build order |

---

### Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`

- [ ] **Step 1: Initialize Vite React+TS project**

Run from the project root:

```bash
npm create vite@latest . -- --template react-ts
```

If it prompts about existing files, allow overwrite — the only files that matter are `spec.md`, `skills.md`, `README.md`, and the `docs/` directory.

- [ ] **Step 2: Install frontend dependencies**

```bash
npm install
```

- [ ] **Step 3: Initialize Tauri**

```bash
cargo tauri init
```

When prompted:
- App name: `aoe-overlay`
- Window title: `AoE Overlay`
- Frontend dev URL: `http://localhost:5173`
- Frontend dev command: `npm run dev`
- Frontend build command: `npm run build`

- [ ] **Step 4: Configure Tauri window**

Replace the window config in `src-tauri/tauri.conf.json`. The `windows` array should contain:

```json
{
  "label": "main",
  "title": "AoE Overlay",
  "transparent": true,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "shadow": false,
  "focus": false,
  "width": 320,
  "height": 480,
  "x": null,
  "y": null
}
```

Also set the `identifier` to `com.aoe-overlay.app` and `productName` to `AoE Overlay`.

- [ ] **Step 5: Update Cargo.toml with Phase 1 dependencies**

Replace `src-tauri/Cargo.toml` with:

```toml
[package]
name = "aoe-overlay"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "2.0"
global-hotkey = "0.6"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

- [ ] **Step 6: Write minimal main.rs**

Replace `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Verify scaffold builds and launches**

```bash
cargo tauri dev
```

Expected: A transparent, borderless window appears showing the Vite React starter page. Close the window to shut down.

- [ ] **Step 8: Commit scaffold**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2.0 + Vite React+TS project"
```

---

### Task 2: Rust Data Model — Error and State Types

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write error.rs**

Create `src-tauri/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Build order not found: {0}")]
    BuildOrderNotFound(String),

    #[error("Invalid build order: {0}")]
    InvalidBuildOrder(String),

    #[error("No build order loaded")]
    NoBuildOrderLoaded,

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
```

- [ ] **Step 2: Write state.rs**

Create `src-tauri/src/state.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use crate::build_order::BuildOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub food: Option<u32>,
    pub wood: Option<u32>,
    pub gold: Option<u32>,
    pub stone: Option<u32>,
    pub villagers: Option<u32>,
    pub population: Option<(u32, u32)>,
    pub game_time_seconds: Option<u32>,
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            food: None,
            wood: None,
            gold: None,
            stone: None,
            villagers: None,
            population: None,
            game_time_seconds: None,
            last_updated: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calibration {
    pub profile_name: String,
    pub resolution: (u32, u32),
    pub ui_scale: f32,
    pub regions: HashMap<RegionKind, Region>,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            profile_name: "default".to_string(),
            resolution: (1920, 1080),
            ui_scale: 1.0,
            regions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionKind {
    Food,
    Wood,
    Gold,
    Stone,
    Villagers,
    Population,
    GameTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub capture_interval_ms: u64,
    pub auto_advance: bool,
    pub click_through: bool,
    pub overlay_opacity: f32,
    pub hotkeys: HotkeyConfig,
    pub ocr_backend: OcrBackend,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            capture_interval_ms: 1000,
            auto_advance: true,
            click_through: false,
            overlay_opacity: 0.9,
            hotkeys: HotkeyConfig::default(),
            ocr_backend: OcrBackend::Template,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub advance_step: String,
    pub previous_step: String,
    pub reset: String,
    pub pause_capture: String,
    pub toggle_visibility: String,
    pub toggle_click_through: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            advance_step: "Ctrl+Alt+Right".to_string(),
            previous_step: "Ctrl+Alt+Left".to_string(),
            reset: "Ctrl+Alt+R".to_string(),
            pause_capture: "Ctrl+Alt+P".to_string(),
            toggle_visibility: "Ctrl+Alt+H".to_string(),
            toggle_click_through: "Ctrl+Alt+C".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OcrBackend {
    #[default]
    Template,
    Tesseract,
}

pub struct AppState {
    pub current_build_order: Option<BuildOrder>,
    pub current_step_index: usize,
    pub last_game_state: GameState,
    pub capture_running: bool,
    pub calibration: Calibration,
    pub settings: Settings,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_build_order: None,
            current_step_index: 0,
            last_game_state: GameState::default(),
            capture_running: false,
            calibration: Calibration::default(),
            settings: Settings::default(),
        }
    }
}
```

- [ ] **Step 3: Register modules in main.rs**

Replace `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_order;
mod error;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Note: This won't compile yet — `build_order` module doesn't exist. That's the next task.

- [ ] **Step 4: Verify compiles after Task 3 completes**

This step is deferred until Task 3 creates the build_order module.

---

### Task 3: Build Order Types

**Files:**
- Create: `src-tauri/src/build_order/mod.rs`

- [ ] **Step 1: Create build_order/mod.rs with all types**

Create `src-tauri/src/build_order/mod.rs`:

```rust
pub mod engine;
pub mod parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrder {
    pub id: String,
    pub name: String,
    pub civilization: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub action: String,
    pub at: Trigger,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub villagers_assigned: Option<VillagerAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(default)]
    pub time_seconds: Option<u32>,
    #[serde(default)]
    pub villagers: Option<u32>,
    #[serde(default)]
    pub population_min: Option<u32>,
    #[serde(default)]
    pub food_min: Option<u32>,
    #[serde(default)]
    pub wood_min: Option<u32>,
    #[serde(default)]
    pub gold_min: Option<u32>,
    #[serde(default)]
    pub stone_min: Option<u32>,
    #[serde(default)]
    pub mode: TriggerMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TriggerMode {
    #[default]
    All,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VillagerAssignment {
    pub food: u32,
    pub wood: u32,
    pub gold: u32,
    pub stone: u32,
    #[serde(default)]
    pub idle: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrderMeta {
    pub id: String,
    pub name: String,
    pub civilization: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub path: String,
}

impl BuildOrder {
    pub fn to_meta(&self, path: &str) -> BuildOrderMeta {
        BuildOrderMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            civilization: self.civilization.clone(),
            tags: self.tags.clone(),
            description: self.description.clone(),
            path: path.to_string(),
        }
    }
}

impl Trigger {
    pub fn has_any_condition(&self) -> bool {
        self.time_seconds.is_some()
            || self.villagers.is_some()
            || self.population_min.is_some()
            || self.food_min.is_some()
            || self.wood_min.is_some()
            || self.gold_min.is_some()
            || self.stone_min.is_some()
    }
}
```

- [ ] **Step 2: Create empty parser.rs and engine.rs stubs so the module compiles**

Create `src-tauri/src/build_order/parser.rs`:

```rust
// Build order YAML/JSON parsing — implemented in Task 4
```

Create `src-tauri/src/build_order/engine.rs`:

```rust
// Trigger evaluation engine — implemented in Task 5
```

- [ ] **Step 3: Verify the project compiles**

```bash
cd src-tauri && cargo check
```

Expected: Compiles with no errors. There may be dead code warnings — that's fine.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/state.rs src-tauri/src/build_order/ src-tauri/src/main.rs
git commit -m "feat: add data model types (state, build order, calibration, settings)"
```

---

### Task 4: Build Order Parser (TDD)

**Files:**
- Modify: `src-tauri/src/build_order/parser.rs`
- Test: inline `#[cfg(test)]` module

- [ ] **Step 1: Write failing tests for parser**

Replace `src-tauri/src/build_order/parser.rs` with:

```rust
use anyhow::{Context, Result};
use std::path::Path;
use tracing::warn;

use super::BuildOrder;

pub fn load_build_order(path: &Path) -> Result<BuildOrder> {
    todo!()
}

pub fn validate_build_order(bo: &BuildOrder) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_yaml() -> &'static str {
        r#"
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
"#
    }

    fn sample_json() -> &'static str {
        r#"{
  "id": "archers-britons",
  "name": "Archers (Britons)",
  "civilization": "Britons",
  "tags": ["archers"],
  "steps": [
    { "action": "6 vills on sheep", "at": { "time_seconds": 0 } },
    { "action": "Lure boar", "at": { "villagers": 10 } }
  ]
}"#
    }

    fn write_temp_file(content: &str, ext: &str) -> NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(ext)
            .tempfile()
            .unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_load_yaml() {
        let file = write_temp_file(sample_yaml(), ".yaml");
        let bo = load_build_order(file.path()).unwrap();
        assert_eq!(bo.id, "scouts-generic");
        assert_eq!(bo.name, "Scouts (Generic)");
        assert_eq!(bo.steps.len(), 3);
        assert_eq!(bo.steps[0].at.time_seconds, Some(0));
        assert_eq!(bo.steps[1].at.villagers, Some(10));
    }

    #[test]
    fn test_load_json() {
        let file = write_temp_file(sample_json(), ".json");
        let bo = load_build_order(file.path()).unwrap();
        assert_eq!(bo.id, "archers-britons");
        assert_eq!(bo.steps.len(), 2);
    }

    #[test]
    fn test_load_yml_extension() {
        let file = write_temp_file(sample_yaml(), ".yml");
        let bo = load_build_order(file.path()).unwrap();
        assert_eq!(bo.id, "scouts-generic");
    }

    #[test]
    fn test_load_unknown_extension_fails() {
        let file = write_temp_file("whatever", ".txt");
        let result = load_build_order(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file_fails() {
        let result = load_build_order(Path::new("/nonexistent/file.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_warns_on_empty_trigger() {
        let yaml = r#"
id: bad
name: "Bad BO"
civilization: Generic
steps:
  - action: "Do something"
    at: {}
"#;
        let file = write_temp_file(yaml, ".yaml");
        let bo = load_build_order(file.path()).unwrap();
        let warnings = validate_build_order(&bo);
        assert!(warnings.iter().any(|w| w.contains("no trigger conditions")));
    }

    #[test]
    fn test_validate_no_warnings_on_valid_bo() {
        let file = write_temp_file(sample_yaml(), ".yaml");
        let bo = load_build_order(file.path()).unwrap();
        let warnings = validate_build_order(&bo);
        assert!(warnings.is_empty());
    }
}
```

- [ ] **Step 2: Add tempfile dev-dependency to Cargo.toml**

Add to `src-tauri/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --lib build_order::parser::tests
```

Expected: All tests FAIL with `not yet implemented`.

- [ ] **Step 4: Implement the parser**

Replace the `todo!()` bodies in `src-tauri/src/build_order/parser.rs`:

```rust
pub fn load_build_order(path: &Path) -> Result<BuildOrder> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read build order file: {}", path.display()))?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let bo: BuildOrder = match ext {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML: {}", path.display()))?,
        "json" => serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON: {}", path.display()))?,
        _ => anyhow::bail!("Unsupported file extension: '{}'. Use .yaml, .yml, or .json", ext),
    };

    let warnings = validate_build_order(&bo);
    for w in &warnings {
        warn!("{}", w);
    }

    Ok(bo)
}

pub fn validate_build_order(bo: &BuildOrder) -> Vec<String> {
    let mut warnings = Vec::new();

    for (i, step) in bo.steps.iter().enumerate() {
        if !step.at.has_any_condition() {
            warnings.push(format!(
                "Step {} (\"{}\") has no trigger conditions — auto-advance will never fire",
                i + 1,
                step.action
            ));
        }
    }

    // Check if steps are roughly sorted by time/villager count
    let trigger_values: Vec<Option<u32>> = bo.steps.iter().map(|s| {
        s.at.time_seconds.or(s.at.villagers).or(s.at.population_min)
    }).collect();

    for window in trigger_values.windows(2) {
        if let (Some(a), Some(b)) = (window[0], window[1]) {
            if b < a {
                warnings.push(format!(
                    "Steps may be out of order: trigger value {} followed by {}",
                    a, b
                ));
                break;
            }
        }
    }

    warnings
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --lib build_order::parser::tests
```

Expected: All 7 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/build_order/parser.rs src-tauri/Cargo.toml
git commit -m "feat: build order YAML/JSON parser with validation"
```

---

### Task 5: Build Order Engine (TDD)

**Files:**
- Modify: `src-tauri/src/build_order/engine.rs`
- Test: inline `#[cfg(test)]` module

- [ ] **Step 1: Write failing tests for trigger evaluation**

Replace `src-tauri/src/build_order/engine.rs` with:

```rust
use super::{Trigger, TriggerMode};
use crate::state::GameState;

pub fn evaluate(trigger: &Trigger, state: &GameState) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(food: u32, wood: u32, gold: u32, stone: u32, vils: u32, time: u32) -> GameState {
        GameState {
            food: Some(food),
            wood: Some(wood),
            gold: Some(gold),
            stone: Some(stone),
            villagers: Some(vils),
            population: Some((vils, 200)),
            game_time_seconds: Some(time),
            last_updated: None,
        }
    }

    fn trigger(mode: TriggerMode) -> Trigger {
        Trigger {
            time_seconds: None,
            villagers: None,
            population_min: None,
            food_min: None,
            wood_min: None,
            gold_min: None,
            stone_min: None,
            mode,
        }
    }

    #[test]
    fn test_empty_trigger_returns_false() {
        let t = trigger(TriggerMode::All);
        let s = state_with(100, 100, 0, 0, 5, 60);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_single_villager_condition_met() {
        let t = Trigger { villagers: Some(10), ..trigger(TriggerMode::All) };
        let s = state_with(0, 0, 0, 0, 10, 0);
        assert!(evaluate(&t, &s));
    }

    #[test]
    fn test_single_villager_condition_not_met() {
        let t = Trigger { villagers: Some(10), ..trigger(TriggerMode::All) };
        let s = state_with(0, 0, 0, 0, 9, 0);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_all_mode_requires_all_conditions() {
        let t = Trigger {
            villagers: Some(21),
            food_min: Some(500),
            ..trigger(TriggerMode::All)
        };
        // Villagers met but food not met
        let s = state_with(499, 0, 0, 0, 21, 0);
        assert!(!evaluate(&t, &s));

        // Both met
        let s = state_with(500, 0, 0, 0, 21, 0);
        assert!(evaluate(&t, &s));
    }

    #[test]
    fn test_any_mode_requires_one_condition() {
        let t = Trigger {
            villagers: Some(21),
            food_min: Some(500),
            ..trigger(TriggerMode::Any)
        };
        // Only villagers met
        let s = state_with(0, 0, 0, 0, 21, 0);
        assert!(evaluate(&t, &s));

        // Only food met
        let s = state_with(500, 0, 0, 0, 5, 0);
        assert!(evaluate(&t, &s));

        // Neither met
        let s = state_with(100, 0, 0, 0, 5, 0);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_time_condition() {
        let t = Trigger { time_seconds: Some(720), ..trigger(TriggerMode::All) };
        let s = state_with(0, 0, 0, 0, 0, 720);
        assert!(evaluate(&t, &s));

        let s = state_with(0, 0, 0, 0, 0, 719);
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_none_game_state_field_fails_condition() {
        let t = Trigger { villagers: Some(10), ..trigger(TriggerMode::All) };
        let s = GameState::default(); // all None
        assert!(!evaluate(&t, &s));
    }

    #[test]
    fn test_population_min_condition() {
        let t = Trigger { population_min: Some(50), ..trigger(TriggerMode::All) };
        let s = GameState {
            population: Some((50, 200)),
            ..GameState::default()
        };
        assert!(evaluate(&t, &s));
    }

    #[test]
    fn test_all_resource_conditions() {
        let t = Trigger {
            food_min: Some(100),
            wood_min: Some(200),
            gold_min: Some(50),
            stone_min: Some(25),
            ..trigger(TriggerMode::All)
        };
        let s = state_with(100, 200, 50, 25, 0, 0);
        assert!(evaluate(&t, &s));

        let s = state_with(100, 199, 50, 25, 0, 0);
        assert!(!evaluate(&t, &s));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --lib build_order::engine::tests
```

Expected: All tests FAIL with `not yet implemented`.

- [ ] **Step 3: Implement trigger evaluation**

Replace the `todo!()` body in the `evaluate` function:

```rust
pub fn evaluate(trigger: &Trigger, state: &GameState) -> bool {
    let checks = [
        (trigger.time_seconds, state.game_time_seconds),
        (trigger.villagers, state.villagers),
        (trigger.population_min, state.population.map(|p| p.0)),
        (trigger.food_min, state.food),
        (trigger.wood_min, state.wood),
        (trigger.gold_min, state.gold),
        (trigger.stone_min, state.stone),
    ];

    let active: Vec<bool> = checks
        .iter()
        .filter_map(|(target, actual)| match (target, actual) {
            (Some(t), Some(a)) => Some(*a >= *t),
            (Some(_), None) => Some(false),
            _ => None,
        })
        .collect();

    if active.is_empty() {
        return false;
    }

    match trigger.mode {
        TriggerMode::All => active.iter().all(|&b| b),
        TriggerMode::Any => active.iter().any(|&b| b),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --lib build_order::engine::tests
```

Expected: All 9 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/build_order/engine.rs
git commit -m "feat: trigger evaluation engine with AND/OR modes"
```

---

### Task 6: Capture and OCR Stubs

**Files:**
- Create: `src-tauri/src/capture/mod.rs`
- Create: `src-tauri/src/capture/windows.rs`
- Create: `src-tauri/src/capture/fallback.rs`
- Create: `src-tauri/src/ocr/mod.rs`
- Create: `src-tauri/src/ocr/preprocess.rs`
- Create: `src-tauri/src/ocr/segment.rs`
- Create: `src-tauri/src/ocr/template.rs`
- Create: `src-tauri/src/ocr/tesseract.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create capture/mod.rs with trait**

```rust
pub mod fallback;
#[cfg(target_os = "windows")]
pub mod windows;

use anyhow::Result;

pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub trait CaptureBackend: Send {
    fn next_frame(&mut self) -> Result<CaptureFrame>;
}
```

- [ ] **Step 2: Create capture/windows.rs and capture/fallback.rs stubs**

`src-tauri/src/capture/windows.rs`:

```rust
// Windows.Graphics.Capture implementation — Phase 2
```

`src-tauri/src/capture/fallback.rs`:

```rust
// xcap-based fallback capture — Phase 2
```

- [ ] **Step 3: Create ocr/mod.rs with trait**

```rust
pub mod preprocess;
pub mod segment;
pub mod template;
pub mod tesseract;

use anyhow::Result;
use crate::state::RegionKind;

pub trait OcrPipeline: Send {
    fn read_number(&self, image: &[u8], width: u32, height: u32, kind: RegionKind) -> Result<Option<u32>>;
}
```

- [ ] **Step 4: Create ocr sub-module stubs**

`src-tauri/src/ocr/preprocess.rs`:

```rust
// Image preprocessing (grayscale, threshold, denoise) — Phase 3
```

`src-tauri/src/ocr/segment.rs`:

```rust
// Character segmentation via connected components — Phase 3
```

`src-tauri/src/ocr/template.rs`:

```rust
// Template matching OCR backend — Phase 3
```

`src-tauri/src/ocr/tesseract.rs`:

```rust
// Tesseract OCR backend (feature-gated) — Phase 3
```

- [ ] **Step 5: Register modules in main.rs**

Update `src-tauri/src/main.rs` to include:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_order;
mod capture;
mod error;
mod ocr;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify compiles**

```bash
cd src-tauri && cargo check
```

Expected: Compiles with warnings about unused code (expected for stubs).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/capture/ src-tauri/src/ocr/ src-tauri/src/main.rs
git commit -m "feat: stub capture and OCR modules with trait definitions"
```

---

### Task 7: Storage Layer

**Files:**
- Create: `src-tauri/src/storage.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write storage.rs**

Create `src-tauri/src/storage.rs`:

```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;

use crate::build_order::{BuildOrder, BuildOrderMeta};
use crate::build_order::parser::load_build_order;
use crate::state::{Calibration, Settings};

pub struct Storage {
    app_data_dir: PathBuf,
}

impl Storage {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        let storage = Self { app_data_dir };
        storage.ensure_directories()?;
        Ok(storage)
    }

    fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            self.build_orders_dir(),
            self.build_orders_dir().join("user"),
            self.calibration_dir(),
        ];
        for dir in &dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            }
        }
        Ok(())
    }

    pub fn build_orders_dir(&self) -> PathBuf {
        self.app_data_dir.join("build-orders")
    }

    fn calibration_dir(&self) -> PathBuf {
        self.app_data_dir.join("calibration")
    }

    fn settings_path(&self) -> PathBuf {
        self.app_data_dir.join("settings.json")
    }

    pub fn copy_bundled_build_orders(&self, bundled_dir: &Path) -> Result<()> {
        if !bundled_dir.exists() {
            return Ok(());
        }
        let target_dir = self.build_orders_dir();
        for entry in std::fs::read_dir(bundled_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml" || ext == "json") {
                let target = target_dir.join(entry.file_name());
                if !target.exists() {
                    std::fs::copy(&path, &target)?;
                    info!("Copied bundled build order: {}", entry.file_name().to_string_lossy());
                }
            }
        }
        Ok(())
    }

    pub fn list_build_orders(&self) -> Result<Vec<BuildOrderMeta>> {
        let mut metas = Vec::new();
        self.scan_build_orders_in(&self.build_orders_dir(), &mut metas)?;
        let user_dir = self.build_orders_dir().join("user");
        if user_dir.exists() {
            self.scan_build_orders_in(&user_dir, &mut metas)?;
        }
        Ok(metas)
    }

    fn scan_build_orders_in(&self, dir: &Path, metas: &mut Vec<BuildOrderMeta>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "yaml" | "yml" | "json") {
                    match load_build_order(&path) {
                        Ok(bo) => metas.push(bo.to_meta(&path.to_string_lossy())),
                        Err(e) => tracing::warn!("Skipping {}: {}", path.display(), e),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_settings(&self) -> Result<Settings> {
        let path = self.settings_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Settings::default())
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(self.settings_path(), content)?;
        Ok(())
    }

    pub fn load_calibration(&self, resolution: (u32, u32)) -> Result<Calibration> {
        let path = self.calibration_dir().join(format!("{}x{}.json", resolution.0, resolution.1));
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Calibration::default())
        }
    }

    pub fn save_calibration(&self, calibration: &Calibration) -> Result<()> {
        let path = self.calibration_dir().join(format!(
            "{}x{}.json",
            calibration.resolution.0, calibration.resolution.1
        ));
        let content = serde_json::to_string_pretty(calibration)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

- [ ] **Step 2: Add `mod storage;` to main.rs**

Add `mod storage;` to the module declarations in `src-tauri/src/main.rs`.

- [ ] **Step 3: Verify compiles**

```bash
cd src-tauri && cargo check
```

Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/storage.rs src-tauri/src/main.rs
git commit -m "feat: file-based storage for settings, calibration, and build orders"
```

---

### Task 8: Sample Build Orders

**Files:**
- Create: `build-orders/scouts-generic.yaml`
- Create: `build-orders/archers-britons.yaml`
- Create: `build-orders/fast-castle-cavalier.yaml`

- [ ] **Step 1: Create scouts-generic.yaml**

Create `build-orders/scouts-generic.yaml`:

```yaml
id: scouts-generic
name: "Scouts (Generic)"
civilization: Generic
author: Community
description: "Standard 21+2 pop scout rush. Works for most cavalry civs."
tags: [scouts, feudal-aggression, beginner-friendly]

steps:
  - action: "6 vills on sheep, 1 vill builds house then to sheep"
    at: { time_seconds: 0 }
    villagers_assigned: { food: 7, wood: 0, gold: 0, stone: 0, idle: 0 }

  - action: "Next 3 vills to sheep under TC"
    at: { villagers: 10 }
    villagers_assigned: { food: 10, wood: 0, gold: 0, stone: 0, idle: 0 }

  - action: "4 vills to wood (build lumber camp)"
    at: { villagers: 11 }
    notes: "Send to the closest woodline. Build a lumber camp."
    villagers_assigned: { food: 10, wood: 4, gold: 0, stone: 0, idle: 0 }

  - action: "Lure first boar"
    at: { villagers: 14 }
    notes: "Shoot boar once with a villager, then garrison in TC. Don't let boar rot."

  - action: "Build mill on berries, 3 vills to berries"
    at: { villagers: 15 }
    villagers_assigned: { food: 11, wood: 4, gold: 0, stone: 0, idle: 0 }

  - action: "Lure second boar"
    at: { villagers: 17 }

  - action: "2 more vills to wood"
    at: { villagers: 19 }
    villagers_assigned: { food: 11, wood: 6, gold: 0, stone: 0, idle: 0 }

  - action: "Build second house, 2 vills to farms"
    at: { villagers: 20 }

  - action: "Click up to Feudal Age"
    at: { villagers: 21, food_min: 500 }
    notes: "While advancing: send 2 sheep vills to wood."

  - action: "Build barracks while advancing"
    at: { time_seconds: 600 }
    notes: "Use a villager that finished a farm."

  - action: "Build stable immediately in Feudal"
    at: { time_seconds: 720 }
    notes: "Start producing scouts. Keep making farms."
```

- [ ] **Step 2: Create archers-britons.yaml**

Create `build-orders/archers-britons.yaml`:

```yaml
id: archers-britons
name: "Archers (Britons)"
civilization: Britons
author: Community
description: "22 pop flush into archers. Britons get +1 range in Castle Age."
tags: [archers, feudal-aggression, britons]

steps:
  - action: "6 vills on sheep"
    at: { time_seconds: 0 }
    villagers_assigned: { food: 6, wood: 0, gold: 0, stone: 0, idle: 0 }

  - action: "4 vills to wood (lumber camp)"
    at: { villagers: 10 }
    villagers_assigned: { food: 6, wood: 4, gold: 0, stone: 0, idle: 0 }

  - action: "Lure boar, next vill to berries (build mill)"
    at: { villagers: 11 }

  - action: "3 more vills to berries"
    at: { villagers: 14 }
    villagers_assigned: { food: 10, wood: 4, gold: 0, stone: 0, idle: 0 }

  - action: "Lure second boar"
    at: { villagers: 15 }

  - action: "3 vills to gold (build mining camp)"
    at: { villagers: 18 }
    villagers_assigned: { food: 10, wood: 4, gold: 3, stone: 0, idle: 0 }

  - action: "1 more to wood, build house"
    at: { villagers: 20 }

  - action: "Click up to Feudal Age"
    at: { villagers: 22, food_min: 500 }
    notes: "Build barracks with one of the new wood villagers while advancing."

  - action: "Build 2 archery ranges in Feudal"
    at: { time_seconds: 660 }
    notes: "Non-stop archer production. Research fletching."

  - action: "Send archers across the map"
    at: { time_seconds: 780 }
    notes: "Target enemy woodlines and gold miners."
```

- [ ] **Step 3: Create fast-castle-cavalier.yaml**

Create `build-orders/fast-castle-cavalier.yaml`:

```yaml
id: fast-castle-cavalier
name: "Fast Castle into Knights"
civilization: Generic
author: Community
description: "27+2 pop fast castle into knight production. Works for Franks, Berbers, Huns, etc."
tags: [fast-castle, knights, castle-age, intermediate]

steps:
  - action: "6 vills on sheep"
    at: { time_seconds: 0 }
    villagers_assigned: { food: 6, wood: 0, gold: 0, stone: 0, idle: 0 }

  - action: "4 vills to wood (lumber camp)"
    at: { villagers: 10 }

  - action: "Lure first boar"
    at: { villagers: 11 }

  - action: "Build mill on berries, 2 to berries"
    at: { villagers: 13 }

  - action: "Lure second boar, 2 more to berries"
    at: { villagers: 15 }
    villagers_assigned: { food: 11, wood: 4, gold: 0, stone: 0, idle: 0 }

  - action: "3 more to wood (second lumber camp if needed)"
    at: { villagers: 18 }

  - action: "5 to farms"
    at: { villagers: 21 }
    notes: "Reseed farms as sheep and boar run out."

  - action: "3 to gold (mining camp)"
    at: { villagers: 24 }

  - action: "Click up to Feudal Age"
    at: { villagers: 27, food_min: 500 }
    notes: "Build barracks while advancing."

  - action: "Click up to Castle Age immediately"
    at: { food_min: 800, gold_min: 200 }
    notes: "Build blacksmith + stable while advancing to Feudal. Click Castle as soon as Feudal hits."

  - action: "Build 2 stables, start knight production"
    at: { time_seconds: 1020 }
    notes: "Research bloodlines and husbandry when affordable."

  - action: "Boom behind knights — add TCs"
    at: { time_seconds: 1200 }
    notes: "Build 2 additional TCs. Keep producing knights and villagers."
```

- [ ] **Step 4: Commit**

```bash
git add build-orders/
git commit -m "feat: add sample build order YAML files"
```

---

### Task 9: IPC Commands

**Files:**
- Create: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write ipc.rs**

Create `src-tauri/src/ipc.rs`:

```rust
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, State};

use crate::build_order::parser::load_build_order;
use crate::build_order::{BuildOrder, BuildOrderMeta, Step};
use crate::error::AppError;
use crate::state::{AppState, Calibration, Settings};
use crate::storage::Storage;

#[derive(Clone, serde::Serialize)]
pub struct StepChangedPayload {
    pub index: usize,
    pub step: Step,
    pub total: usize,
}

fn emit_step_changed(app: &AppHandle, state: &AppState) {
    if let Some(ref bo) = state.current_build_order {
        if let Some(step) = bo.steps.get(state.current_step_index) {
            let _ = app.emit(
                "step-changed",
                StepChangedPayload {
                    index: state.current_step_index,
                    step: step.clone(),
                    total: bo.steps.len(),
                },
            );
        }
    }
}

#[tauri::command]
pub fn load_build_order_cmd(
    path: String,
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<BuildOrder, AppError> {
    let bo = load_build_order(std::path::Path::new(&path))
        .map_err(|e| AppError::InvalidBuildOrder(e.to_string()))?;
    let mut app_state = state.lock().unwrap();
    app_state.current_build_order = Some(bo.clone());
    app_state.current_step_index = 0;
    emit_step_changed(&app, &app_state);
    Ok(bo)
}

#[tauri::command]
pub fn list_build_orders_cmd(
    storage: State<'_, Storage>,
) -> Result<Vec<BuildOrderMeta>, AppError> {
    storage
        .list_build_orders()
        .map_err(|e| AppError::Storage(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
}

#[tauri::command]
pub fn advance_step(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<usize, AppError> {
    let mut app_state = state.lock().unwrap();
    let max = app_state
        .current_build_order
        .as_ref()
        .map(|bo| bo.steps.len())
        .ok_or(AppError::NoBuildOrderLoaded)?;

    if app_state.current_step_index < max - 1 {
        app_state.current_step_index += 1;
    }
    emit_step_changed(&app, &app_state);
    Ok(app_state.current_step_index)
}

#[tauri::command]
pub fn previous_step(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<usize, AppError> {
    let mut app_state = state.lock().unwrap();
    if app_state.current_build_order.is_none() {
        return Err(AppError::NoBuildOrderLoaded);
    }
    if app_state.current_step_index > 0 {
        app_state.current_step_index -= 1;
    }
    emit_step_changed(&app, &app_state);
    Ok(app_state.current_step_index)
}

#[tauri::command]
pub fn reset_steps(
    app: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    let mut app_state = state.lock().unwrap();
    if app_state.current_build_order.is_none() {
        return Err(AppError::NoBuildOrderLoaded);
    }
    app_state.current_step_index = 0;
    emit_step_changed(&app, &app_state);
    Ok(())
}

#[tauri::command]
pub fn get_settings(
    state: State<'_, Mutex<AppState>>,
) -> Result<Settings, AppError> {
    let app_state = state.lock().unwrap();
    Ok(app_state.settings.clone())
}

#[tauri::command]
pub fn get_calibration(
    state: State<'_, Mutex<AppState>>,
) -> Result<Calibration, AppError> {
    let app_state = state.lock().unwrap();
    Ok(app_state.calibration.clone())
}
```

- [ ] **Step 2: Wire commands and state into main.rs**

Replace `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_order;
mod capture;
mod error;
mod hotkeys;
mod ipc;
mod ocr;
mod state;
mod storage;

use std::sync::Mutex;

use state::AppState;
use storage::Storage;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            let storage = Storage::new(app_data_dir)?;

            // Copy bundled build orders on first run
            let bundled_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir")
                .join("build-orders");
            let _ = storage.copy_bundled_build_orders(&bundled_dir);

            let settings = storage.load_settings().unwrap_or_default();
            let app_state = AppState {
                settings,
                ..AppState::default()
            };

            app.manage(Mutex::new(app_state));
            app.manage(storage);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::load_build_order_cmd,
            ipc::list_build_orders_cmd,
            ipc::advance_step,
            ipc::previous_step,
            ipc::reset_steps,
            ipc::get_settings,
            ipc::get_calibration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Configure Tauri to bundle build-orders as resources**

In `src-tauri/tauri.conf.json`, add to the `bundle` section:

```json
"resources": [
  "../build-orders/*"
]
```

- [ ] **Step 4: Add `use tauri::Manager;` if needed and verify compiles**

```bash
cd src-tauri && cargo check
```

Fix any import issues. The `app.path()` method requires the `Manager` trait from Tauri.

Add `use tauri::Manager;` in `main.rs` inside the `setup` closure if the compiler requests it.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/main.rs src-tauri/tauri.conf.json
git commit -m "feat: IPC commands for build order loading and step navigation"
```

---

### Task 10: Global Hotkeys

**Files:**
- Create: `src-tauri/src/hotkeys.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write hotkeys.rs**

Create `src-tauri/src/hotkeys.rs`:

```rust
use std::sync::Mutex;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};

use crate::state::AppState;
use crate::ipc::{StepChangedPayload, emit_step_changed};

struct HotkeyIds {
    advance: u32,
    previous: u32,
    reset: u32,
    toggle_visibility: u32,
}

pub fn setup_hotkeys(app: &AppHandle) {
    let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

    let advance = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowRight);
    let previous = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft);
    let reset = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyR);
    let toggle_visibility = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyH);

    let ids = HotkeyIds {
        advance: advance.id(),
        previous: previous.id(),
        reset: reset.id(),
        toggle_visibility: toggle_visibility.id(),
    };

    manager.register(advance).expect("Failed to register advance hotkey");
    manager.register(previous).expect("Failed to register previous hotkey");
    manager.register(reset).expect("Failed to register reset hotkey");
    manager.register(toggle_visibility).expect("Failed to register toggle hotkey");

    info!("Global hotkeys registered: Ctrl+Alt+Arrow (advance/previous), Ctrl+Alt+R (reset), Ctrl+Alt+H (toggle)");

    let app_handle = app.clone();

    // Leak the manager so it lives for the lifetime of the app
    // (it must not be dropped or hotkeys stop working)
    std::mem::forget(manager);

    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                handle_hotkey_event(&app_handle, &ids, event.id());
            }
        }
    });
}

fn handle_hotkey_event(app: &AppHandle, ids: &HotkeyIds, id: u32) {
    let state = app.state::<Mutex<AppState>>();

    if id == ids.advance {
        let mut app_state = state.lock().unwrap();
        if let Some(ref bo) = app_state.current_build_order {
            let max = bo.steps.len();
            if app_state.current_step_index < max - 1 {
                app_state.current_step_index += 1;
                emit_step_changed(app, &app_state);
            }
        }
    } else if id == ids.previous {
        let mut app_state = state.lock().unwrap();
        if app_state.current_build_order.is_some() && app_state.current_step_index > 0 {
            app_state.current_step_index -= 1;
            emit_step_changed(app, &app_state);
        }
    } else if id == ids.reset {
        let mut app_state = state.lock().unwrap();
        if app_state.current_build_order.is_some() {
            app_state.current_step_index = 0;
            emit_step_changed(app, &app_state);
        }
    } else if id == ids.toggle_visibility {
        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
            }
        }
    }
}
```

- [ ] **Step 2: Make `emit_step_changed` public in ipc.rs**

In `src-tauri/src/ipc.rs`, change:

```rust
fn emit_step_changed(app: &AppHandle, state: &AppState) {
```

to:

```rust
pub fn emit_step_changed(app: &AppHandle, state: &AppState) {
```

- [ ] **Step 3: Call setup_hotkeys in main.rs setup**

In `src-tauri/src/main.rs`, inside the `.setup(|app| { ... })` closure, after managing state, add:

```rust
hotkeys::setup_hotkeys(&app.handle().clone());
```

- [ ] **Step 4: Verify compiles**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/hotkeys.rs src-tauri/src/ipc.rs src-tauri/src/main.rs
git commit -m "feat: global hotkeys for step navigation and overlay toggle"
```

---

### Task 11: TypeScript Types and Hooks

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/hooks/useTauriCommand.ts`
- Create: `src/hooks/useBuildOrder.ts`
- Create: `src/hooks/useGameState.ts`

- [ ] **Step 1: Create TypeScript types**

Create `src/lib/types.ts`:

```typescript
export interface BuildOrder {
  id: string;
  name: string;
  civilization: string;
  author?: string;
  description?: string;
  source_url?: string;
  tags: string[];
  steps: Step[];
}

export interface Step {
  action: string;
  at: Trigger;
  notes?: string;
  villagers_assigned?: VillagerAssignment;
}

export interface Trigger {
  time_seconds?: number;
  villagers?: number;
  population_min?: number;
  food_min?: number;
  wood_min?: number;
  gold_min?: number;
  stone_min?: number;
  mode: "All" | "Any";
}

export interface VillagerAssignment {
  food: number;
  wood: number;
  gold: number;
  stone: number;
  idle: number;
}

export interface BuildOrderMeta {
  id: string;
  name: string;
  civilization: string;
  tags: string[];
  description?: string;
  path: string;
}

export interface StepChangedPayload {
  index: number;
  step: Step;
  total: number;
}

export interface GameState {
  food?: number;
  wood?: number;
  gold?: number;
  stone?: number;
  villagers?: number;
  population?: [number, number];
  game_time_seconds?: number;
}

export interface Settings {
  capture_interval_ms: number;
  auto_advance: boolean;
  click_through: boolean;
  overlay_opacity: number;
  hotkeys: HotkeyConfig;
  ocr_backend: "Template" | "Tesseract";
}

export interface HotkeyConfig {
  advance_step: string;
  previous_step: string;
  reset: string;
  pause_capture: string;
  toggle_visibility: string;
  toggle_click_through: string;
}
```

- [ ] **Step 2: Install Tauri API package**

```bash
npm install @tauri-apps/api
```

- [ ] **Step 3: Create useTauriCommand hook**

Create `src/hooks/useTauriCommand.ts`:

```typescript
import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useTauriCommand<T>(command: string) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (args?: Record<string, unknown>): Promise<T | null> => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<T>(command, args);
        return result;
      } catch (e) {
        const message = typeof e === "string" ? e : String(e);
        setError(message);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [command]
  );

  return { execute, loading, error };
}
```

- [ ] **Step 4: Create useBuildOrder hook**

Create `src/hooks/useBuildOrder.ts`:

```typescript
import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { BuildOrder, StepChangedPayload } from "../lib/types";

export function useBuildOrder() {
  const [buildOrder, setBuildOrder] = useState<BuildOrder | null>(null);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    const unlisten = listen<StepChangedPayload>("step-changed", (event) => {
      setCurrentStep(event.payload.index);
      setTotalSteps(event.payload.total);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadBuildOrder = useCallback(async (path: string) => {
    try {
      const bo = await invoke<BuildOrder>("load_build_order_cmd", { path });
      setBuildOrder(bo);
      setCurrentStep(0);
      setTotalSteps(bo.steps.length);
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  }, []);

  const advance = useCallback(async () => {
    try {
      await invoke("advance_step");
    } catch (e) {
      console.error("Failed to advance step:", e);
    }
  }, []);

  const previous = useCallback(async () => {
    try {
      await invoke("previous_step");
    } catch (e) {
      console.error("Failed to go to previous step:", e);
    }
  }, []);

  const reset = useCallback(async () => {
    try {
      await invoke("reset_steps");
    } catch (e) {
      console.error("Failed to reset steps:", e);
    }
  }, []);

  return {
    buildOrder,
    currentStep,
    totalSteps,
    loadBuildOrder,
    advance,
    previous,
    reset,
  };
}
```

- [ ] **Step 5: Create useGameState hook**

Create `src/hooks/useGameState.ts`:

```typescript
import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { GameState } from "../lib/types";

export function useGameState() {
  const [gameState, setGameState] = useState<GameState | null>(null);

  useEffect(() => {
    const unlisten = listen<GameState>("game-state", (event) => {
      setGameState(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return gameState;
}
```

- [ ] **Step 6: Commit**

```bash
git add src/lib/ src/hooks/
git commit -m "feat: TypeScript types and Tauri hooks for build order and game state"
```

---

### Task 12: Overlay Styles and StepCard Component

**Files:**
- Create: `src/styles/overlay.css`
- Create: `src/components/StepCard.tsx`

- [ ] **Step 1: Create overlay.css**

Create `src/styles/overlay.css`:

```css
:root {
  --bg-primary: rgba(20, 20, 24, 0.85);
  --bg-secondary: rgba(30, 30, 36, 0.9);
  --text-primary: #ffffff;
  --text-secondary: #9ca3af;
  --accent: #ffb84d;
  --accent-dim: rgba(255, 184, 77, 0.15);
  --border: rgba(255, 255, 255, 0.08);
  --transition: 150ms ease;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #root {
  height: 100%;
  overflow: hidden;
  background: transparent;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  color: var(--text-primary);
  font-size: 14px;
  -webkit-font-smoothing: antialiased;
}

.app {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Overlay compact view */
.overlay {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border);
}

.overlay-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}

.overlay-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.step-counter {
  font-size: 12px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

/* Step card */
.step-card {
  padding: 10px 12px;
  border-radius: 6px;
  transition: background var(--transition);
}

.step-card--current {
  background: var(--accent-dim);
  border-left: 3px solid var(--accent);
  padding-left: 9px;
}

.step-card--next {
  opacity: 0.6;
}

.step-action {
  font-size: 14px;
  font-weight: 500;
  line-height: 1.4;
}

.step-card--current .step-action {
  color: var(--accent);
}

.step-notes {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.3;
}

.step-villagers {
  display: flex;
  gap: 8px;
  margin-top: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}

.step-villager-item {
  display: flex;
  align-items: center;
  gap: 2px;
}

/* Navigation buttons */
.overlay-nav {
  display: flex;
  gap: 6px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
}

.nav-btn {
  flex: 1;
  padding: 6px 8px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition);
}

.nav-btn:hover {
  color: var(--text-primary);
  border-color: rgba(255, 255, 255, 0.2);
}

/* No build order loaded */
.no-build-order {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px 20px;
  text-align: center;
  color: var(--text-secondary);
}

/* Library view */
.library {
  padding: 12px;
  background: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border);
  overflow-y: auto;
  max-height: 100%;
}

.library-header {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.library-item {
  padding: 10px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  margin-bottom: 8px;
  cursor: pointer;
  transition: all var(--transition);
}

.library-item:hover {
  border-color: var(--accent);
  background: var(--accent-dim);
}

.library-item-name {
  font-size: 14px;
  font-weight: 500;
}

.library-item-civ {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.library-item-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.library-item-tags {
  display: flex;
  gap: 4px;
  margin-top: 6px;
  flex-wrap: wrap;
}

.tag {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

/* View switcher */
.view-switcher {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border);
}

.view-btn {
  padding: 4px 10px;
  background: none;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition);
}

.view-btn:hover {
  color: var(--text-primary);
}

.view-btn--active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-dim);
}

/* Settings stub */
.settings {
  padding: 12px;
  background: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border);
}

.settings-header {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.settings-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 12px;
  border-bottom: 1px solid var(--border);
}

.settings-label {
  color: var(--text-secondary);
}

.settings-value {
  color: var(--text-primary);
  font-family: monospace;
}
```

- [ ] **Step 2: Create StepCard component**

Create `src/components/StepCard.tsx`:

```tsx
import type { Step } from "../lib/types";

interface StepCardProps {
  step: Step;
  variant: "current" | "next" | "default";
}

export function StepCard({ step, variant }: StepCardProps) {
  const className = `step-card step-card--${variant}`;

  return (
    <div className={className}>
      <div className="step-action">{step.action}</div>
      {step.notes && <div className="step-notes">{step.notes}</div>}
      {step.villagers_assigned && (
        <div className="step-villagers">
          {step.villagers_assigned.food > 0 && (
            <span className="step-villager-item">F:{step.villagers_assigned.food}</span>
          )}
          {step.villagers_assigned.wood > 0 && (
            <span className="step-villager-item">W:{step.villagers_assigned.wood}</span>
          )}
          {step.villagers_assigned.gold > 0 && (
            <span className="step-villager-item">G:{step.villagers_assigned.gold}</span>
          )}
          {step.villagers_assigned.stone > 0 && (
            <span className="step-villager-item">S:{step.villagers_assigned.stone}</span>
          )}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Commit**

```bash
git add src/styles/ src/components/StepCard.tsx
git commit -m "feat: overlay styles and StepCard component"
```

---

### Task 13: Overlay, Library, Settings, and App Components

**Files:**
- Create: `src/components/Overlay.tsx`
- Create: `src/components/BuildOrderLibrary.tsx`
- Create: `src/components/Settings.tsx`
- Replace: `src/App.tsx`
- Replace: `src/main.tsx`

- [ ] **Step 1: Create Overlay component**

Create `src/components/Overlay.tsx`:

```tsx
import { useBuildOrder } from "../hooks/useBuildOrder";
import { StepCard } from "./StepCard";

interface OverlayProps {
  onOpenLibrary: () => void;
}

export function Overlay({ onOpenLibrary }: OverlayProps) {
  const { buildOrder, currentStep, totalSteps, advance, previous, reset } =
    useBuildOrder();

  if (!buildOrder) {
    return (
      <div className="overlay">
        <div className="no-build-order">
          <div>No build order loaded</div>
          <button className="nav-btn" onClick={onOpenLibrary}>
            Open Library
          </button>
        </div>
      </div>
    );
  }

  const currentStepData = buildOrder.steps[currentStep];
  const nextStepData =
    currentStep < buildOrder.steps.length - 1
      ? buildOrder.steps[currentStep + 1]
      : null;

  return (
    <div className="overlay">
      <div className="overlay-header">
        <span className="overlay-title">{buildOrder.name}</span>
        <span className="step-counter">
          {currentStep + 1} / {totalSteps}
        </span>
      </div>

      {currentStepData && (
        <StepCard step={currentStepData} variant="current" />
      )}

      {nextStepData && <StepCard step={nextStepData} variant="next" />}

      <div className="overlay-nav">
        <button className="nav-btn" onClick={previous}>
          Prev
        </button>
        <button className="nav-btn" onClick={reset}>
          Reset
        </button>
        <button className="nav-btn" onClick={advance}>
          Next
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create BuildOrderLibrary component**

Create `src/components/BuildOrderLibrary.tsx`:

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { BuildOrderMeta } from "../lib/types";

interface BuildOrderLibraryProps {
  onSelect: (path: string) => void;
}

export function BuildOrderLibrary({ onSelect }: BuildOrderLibraryProps) {
  const [buildOrders, setBuildOrders] = useState<BuildOrderMeta[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<BuildOrderMeta[]>("list_build_orders_cmd")
      .then(setBuildOrders)
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return (
      <div className="library">
        <div className="library-header">Build Orders</div>
        <div style={{ color: "var(--text-secondary)" }}>Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="library">
      <div className="library-header">Build Orders</div>
      {buildOrders.length === 0 && (
        <div style={{ color: "var(--text-secondary)" }}>
          No build orders found.
        </div>
      )}
      {buildOrders.map((bo) => (
        <div
          key={bo.id}
          className="library-item"
          onClick={() => onSelect(bo.path)}
        >
          <div className="library-item-name">{bo.name}</div>
          <div className="library-item-civ">{bo.civilization}</div>
          {bo.description && (
            <div className="library-item-desc">{bo.description}</div>
          )}
          {bo.tags.length > 0 && (
            <div className="library-item-tags">
              {bo.tags.map((tag) => (
                <span key={tag} className="tag">
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: Create Settings stub component**

Create `src/components/Settings.tsx`:

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings as SettingsType } from "../lib/types";

export function Settings() {
  const [settings, setSettings] = useState<SettingsType | null>(null);

  useEffect(() => {
    invoke<SettingsType>("get_settings")
      .then(setSettings)
      .catch(console.error);
  }, []);

  if (!settings) {
    return (
      <div className="settings">
        <div className="settings-header">Settings</div>
        <div style={{ color: "var(--text-secondary)" }}>Loading...</div>
      </div>
    );
  }

  const hotkeys = settings.hotkeys;

  return (
    <div className="settings">
      <div className="settings-header">Hotkeys</div>
      <div className="settings-row">
        <span className="settings-label">Advance step</span>
        <span className="settings-value">{hotkeys.advance_step}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Previous step</span>
        <span className="settings-value">{hotkeys.previous_step}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Reset</span>
        <span className="settings-value">{hotkeys.reset}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Pause capture</span>
        <span className="settings-value">{hotkeys.pause_capture}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Toggle overlay</span>
        <span className="settings-value">{hotkeys.toggle_visibility}</span>
      </div>
      <div className="settings-row">
        <span className="settings-label">Toggle click-through</span>
        <span className="settings-value">{hotkeys.toggle_click_through}</span>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Replace App.tsx**

Replace `src/App.tsx` with:

```tsx
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Overlay } from "./components/Overlay";
import { BuildOrderLibrary } from "./components/BuildOrderLibrary";
import { Settings } from "./components/Settings";
import "./styles/overlay.css";

type View = "overlay" | "library" | "settings";

function App() {
  const [view, setView] = useState<View>("overlay");

  const handleSelectBuildOrder = async (path: string) => {
    try {
      await invoke("load_build_order_cmd", { path });
      setView("overlay");
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  };

  return (
    <div className="app">
      <div className="view-switcher">
        <button
          className={`view-btn ${view === "overlay" ? "view-btn--active" : ""}`}
          onClick={() => setView("overlay")}
        >
          Overlay
        </button>
        <button
          className={`view-btn ${view === "library" ? "view-btn--active" : ""}`}
          onClick={() => setView("library")}
        >
          Library
        </button>
        <button
          className={`view-btn ${view === "settings" ? "view-btn--active" : ""}`}
          onClick={() => setView("settings")}
        >
          Settings
        </button>
      </div>

      {view === "overlay" && (
        <Overlay onOpenLibrary={() => setView("library")} />
      )}
      {view === "library" && (
        <BuildOrderLibrary onSelect={handleSelectBuildOrder} />
      )}
      {view === "settings" && <Settings />}
    </div>
  );
}

export default App;
```

- [ ] **Step 5: Replace main.tsx**

Replace `src/main.tsx` with:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

- [ ] **Step 6: Clean up Vite starter files**

Remove Vite's default starter files that we don't need:

```bash
rm -f src/App.css src/index.css src/assets/react.svg public/vite.svg
```

Remove any imports of `App.css` or `index.css` from remaining files if present.

- [ ] **Step 7: Verify it compiles and launches**

```bash
cargo tauri dev
```

Expected: The overlay window appears with the view switcher (Overlay, Library, Settings tabs). The Overlay tab shows "No build order loaded" with an "Open Library" button. The Library tab lists available build orders from the app data directory.

- [ ] **Step 8: Commit**

```bash
git add src/ 
git commit -m "feat: React overlay UI with library, settings, and step navigation"
```

---

### Task 14: Integration Test

**Files:**
- Create: `src-tauri/tests/build_order_integration.rs`

- [ ] **Step 1: Write integration test**

Create `src-tauri/tests/build_order_integration.rs`:

```rust
use std::io::Write;

use tempfile::NamedTempFile;

// Integration test: load a build order, advance through all steps, verify state
#[test]
fn test_full_step_advancement_through_build_order() {
    // We test the parser + engine together without Tauri runtime
    let yaml = r#"
id: test-bo
name: "Test Build Order"
civilization: Generic
steps:
  - action: "Step 1"
    at: { time_seconds: 0 }
  - action: "Step 2"
    at: { villagers: 10 }
  - action: "Step 3"
    at: { villagers: 15, food_min: 200 }
  - action: "Step 4"
    at: { time_seconds: 600 }
"#;

    let mut file = tempfile::Builder::new()
        .suffix(".yaml")
        .tempfile()
        .unwrap();
    file.write_all(yaml.as_bytes()).unwrap();

    // Load
    let bo = aoe_overlay::build_order::parser::load_build_order(file.path()).unwrap();
    assert_eq!(bo.steps.len(), 4);

    // Simulate step advancement via trigger evaluation
    use aoe_overlay::build_order::engine::evaluate;
    use aoe_overlay::state::GameState;

    // Step 0: time_seconds >= 0
    let state = GameState {
        game_time_seconds: Some(0),
        ..GameState::default()
    };
    assert!(evaluate(&bo.steps[0].at, &state));

    // Step 1: villagers >= 10
    let state = GameState {
        villagers: Some(10),
        ..GameState::default()
    };
    assert!(evaluate(&bo.steps[1].at, &state));

    // Step 1: villagers = 9 should NOT trigger
    let state = GameState {
        villagers: Some(9),
        ..GameState::default()
    };
    assert!(!evaluate(&bo.steps[1].at, &state));

    // Step 2: villagers >= 15 AND food >= 200 (All mode)
    let state = GameState {
        villagers: Some(15),
        food: Some(200),
        ..GameState::default()
    };
    assert!(evaluate(&bo.steps[2].at, &state));

    // Step 2: villagers met but food not met
    let state = GameState {
        villagers: Some(15),
        food: Some(199),
        ..GameState::default()
    };
    assert!(!evaluate(&bo.steps[2].at, &state));

    // Step 3: time >= 600
    let state = GameState {
        game_time_seconds: Some(600),
        ..GameState::default()
    };
    assert!(evaluate(&bo.steps[3].at, &state));
}

#[test]
fn test_load_all_sample_build_orders() {
    let sample_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("build-orders");

    if !sample_dir.exists() {
        panic!("build-orders directory not found at {}", sample_dir.display());
    }

    let mut count = 0;
    for entry in std::fs::read_dir(&sample_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |e| e == "yaml") {
            let bo = aoe_overlay::build_order::parser::load_build_order(&path)
                .unwrap_or_else(|e| panic!("Failed to load {}: {}", path.display(), e));
            assert!(!bo.steps.is_empty(), "Build order {} has no steps", bo.name);

            // Validate all triggers have conditions
            let warnings = aoe_overlay::build_order::parser::validate_build_order(&bo);
            assert!(
                warnings.is_empty(),
                "Build order {} has validation warnings: {:?}",
                bo.name,
                warnings
            );

            count += 1;
        }
    }
    assert_eq!(count, 3, "Expected 3 sample build orders");
}
```

- [ ] **Step 2: Make crate modules public for integration tests**

In `src-tauri/src/main.rs`, the modules need to be accessible from integration tests. Create a `src-tauri/src/lib.rs` that re-exports the modules:

```rust
pub mod build_order;
pub mod capture;
pub mod error;
pub mod ocr;
pub mod state;
pub mod storage;
```

Keep `main.rs` as-is — it still uses `mod` declarations for the binary target. Update `main.rs` module declarations to import from the lib:

Actually, the cleaner approach: move module declarations to `lib.rs` and have `main.rs` use the library crate. Replace `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use aoe_overlay::state::AppState;
use aoe_overlay::storage::Storage;
use aoe_overlay::ipc;
use aoe_overlay::hotkeys;

use tauri::Manager;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            let storage = Storage::new(app_data_dir)?;

            let bundled_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir")
                .join("build-orders");
            let _ = storage.copy_bundled_build_orders(&bundled_dir);

            let settings = storage.load_settings().unwrap_or_default();
            let app_state = AppState {
                settings,
                ..AppState::default()
            };

            app.manage(Mutex::new(app_state));
            app.manage(storage);

            hotkeys::setup_hotkeys(&app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::load_build_order_cmd,
            ipc::list_build_orders_cmd,
            ipc::advance_step,
            ipc::previous_step,
            ipc::reset_steps,
            ipc::get_settings,
            ipc::get_calibration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

And create `src-tauri/src/lib.rs`:

```rust
pub mod build_order;
pub mod capture;
pub mod error;
pub mod hotkeys;
pub mod ipc;
pub mod ocr;
pub mod state;
pub mod storage;
```

- [ ] **Step 3: Run integration tests**

```bash
cd src-tauri && cargo test --test build_order_integration
```

Expected: Both tests PASS.

- [ ] **Step 4: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All unit tests (parser: 7, engine: 9) and integration tests (2) pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs src-tauri/tests/
git commit -m "feat: integration tests for build order loading and step advancement"
```

---

### Task 15: Final Verification

- [ ] **Step 1: Run full test suite**

```bash
cd src-tauri && cargo test
```

Expected: All tests pass.

- [ ] **Step 2: Run cargo clippy**

```bash
cd src-tauri && cargo clippy -- -D warnings 2>&1 || true
```

Fix any clippy warnings that appear. Common ones: unused imports, unnecessary clones.

- [ ] **Step 3: Launch the app**

```bash
cargo tauri dev
```

Manual checks:
1. Overlay window appears (transparent, no decorations, always on top)
2. View switcher shows Overlay / Library / Settings tabs
3. Overlay shows "No build order loaded"
4. Library tab lists the 3 sample build orders
5. Clicking a build order loads it and switches to overlay view
6. Current step shows with amber accent, next step dimmed below
7. Step counter shows "1 / N"
8. Nav buttons (Prev/Reset/Next) work
9. Global hotkeys (Ctrl+Alt+Arrow, Ctrl+Alt+R, Ctrl+Alt+H) work
10. Ctrl+Alt+H toggles overlay visibility

- [ ] **Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: address clippy warnings and final polish"
```
