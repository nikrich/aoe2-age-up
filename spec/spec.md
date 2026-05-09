# AoE Overlay — Technical Specification

**Project codename:** `aoe-overlay`
**Target platform:** Windows 10/11 (primary), with macOS/Linux as nice-to-have
**Stack:** Rust + Tauri 2.0 + TypeScript/React frontend
**Author handoff:** This spec is intended for direct ingestion by Claude Code as the initial scaffolding brief.

---

## 1. Product overview

A lightweight, transparent, always-on-top desktop overlay that helps players learn and execute build orders in real-time strategy games — primarily Age of Empires II: Definitive Edition, with an architecture that allows extension to AoE4 and other RTS titles.

The overlay reads game state from the screen via OCR (no memory injection, no anti-cheat risk) and either auto-advances build order steps based on resource/villager/time triggers, or allows manual advancement via global hotkey.

### 1.1 Goals

- **Memory footprint < 100MB resident** under normal operation.
- **CPU usage < 3%** on a mid-range modern CPU during active capture.
- **Zero interference** with the running game (no input injection, no memory reads, no DLL injection).
- **Sub-second responsiveness** for step transitions when triggers are met.
- **Build order portability** — easy import/export, sharable as plain text files.

### 1.2 Non-goals (v1)

- Multi-game support beyond AoE2:DE (architecture should allow it; implementation should not).
- Replay analysis (post-game `.aoe2record` parsing).
- Cloud sync of build orders.
- Voice cues or TTS.
- AI-generated build orders.

---

## 2. Architecture

### 2.1 High-level component diagram

```
┌──────────────────────────────────────────────────────────────┐
│ Tauri Webview (frontend)                                     │
│ - React + TypeScript + Vite                                  │
│ - Renders overlay UI (transparent, click-through optional)   │
│ - Receives game state events via Tauri IPC                   │
│ - Calibration UI, build order editor, settings               │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri IPC (commands + events)
┌──────────────────────────▼───────────────────────────────────┐
│ Rust core (backend)                                          │
│                                                              │
│  ┌────────────────────┐    ┌────────────────────────────┐    │
│  │ Capture loop       │───▶│ OCR pipeline               │    │
│  │ (Windows.Graphics. │    │ - grayscale + threshold    │    │
│  │  Capture, 1Hz)     │    │ - template matching        │    │
│  └─────────┬──────────┘    │   (default, lightweight)   │    │
│            │               │ - Tesseract (optional)     │    │
│            │               └─────────────┬──────────────┘    │
│            │                             │                   │
│            │                             ▼                   │
│            │               ┌────────────────────────────┐    │
│            └──────────────▶│ Game state aggregator      │    │
│                            │ - resource values, time,   │    │
│                            │   villagers, population    │    │
│                            └─────────────┬──────────────┘    │
│                                          │                   │
│                                          ▼                   │
│                            ┌────────────────────────────┐    │
│                            │ Build order engine         │    │
│                            │ - trigger evaluation       │    │
│                            │ - step advancement         │    │
│                            └─────────────┬──────────────┘    │
│                                          │                   │
│                                          ▼                   │
│                            ┌────────────────────────────┐    │
│                            │ Event emitter              │    │
│                            │ - emits to frontend        │    │
│                            └────────────────────────────┘    │
│                                                              │
│  ┌────────────────────┐    ┌────────────────────────────┐    │
│  │ Global hotkeys     │    │ Persistent storage         │    │
│  │ - manual advance   │    │ - calibration profiles     │    │
│  │ - reset / pause    │    │ - build order library      │    │
│  │ - toggle overlay   │    │ - user settings            │    │
│  └────────────────────┘    └────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Process model

- **Single Tauri app process** owns the webview and the Rust core.
- **One dedicated Tokio task** for the capture loop (runs at 1Hz, configurable).
- **One Tokio task** for global hotkey listener.
- All shared state behind `Arc<Mutex<AppState>>` (or `RwLock` where reads dominate).

### 2.3 Crate layout

```
aoe-overlay/
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs               # Entry point, Tauri setup, command handlers
│       ├── state.rs              # AppState, GameState, Calibration types
│       ├── capture/
│       │   ├── mod.rs            # Capture loop driver
│       │   ├── windows.rs        # Windows.Graphics.Capture impl (primary)
│       │   └── fallback.rs       # xcap-based fallback for cross-platform
│       ├── ocr/
│       │   ├── mod.rs            # OCR pipeline orchestration
│       │   ├── preprocess.rs     # grayscale, threshold, denoise
│       │   ├── segment.rs        # connected-components character segmentation
│       │   ├── template.rs       # template matching (default backend)
│       │   ├── templates/        # embedded digit/symbol PNGs
│       │   └── tesseract.rs      # optional Tesseract backend (feature-gated)
│       ├── build_order/
│       │   ├── mod.rs            # BuildOrder, Step, Trigger types
│       │   ├── parser.rs         # YAML/JSON loading + validation
│       │   └── engine.rs         # Trigger evaluation + step advancement
│       ├── hotkeys.rs            # Global hotkey registration
│       ├── storage.rs            # File-based persistence (calibration, BOs, settings)
│       └── ipc.rs                # Tauri command/event definitions
├── src/                          # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── Overlay.tsx           # Main overlay display
│   │   ├── StepCard.tsx          # Individual step rendering
│   │   ├── ResourceBar.tsx       # Live resource readout (debug/QA)
│   │   ├── CalibrationWizard.tsx # Region-selection UI
│   │   ├── BuildOrderEditor.tsx  # In-app BO authoring
│   │   ├── BuildOrderLibrary.tsx # Browse/load BOs
│   │   └── Settings.tsx          # Hotkeys, capture rate, etc.
│   ├── hooks/
│   │   ├── useGameState.ts       # Subscribes to game-state events
│   │   ├── useBuildOrder.ts      # Subscribes to build-order events
│   │   └── useTauriCommand.ts    # Wrapper for invoke()
│   ├── lib/
│   │   ├── tauri-bindings.ts     # Generated TypeScript types from Rust
│   │   └── format.ts             # Display formatting utilities
│   └── styles/
│       └── overlay.css           # Transparent-friendly styles
├── build-orders/                 # Sample build orders shipped with app
│   ├── scouts-generic.yaml
│   ├── archers-britons.yaml
│   └── fast-castle-cavalier.yaml
├── package.json
├── vite.config.ts
└── README.md
```

---

## 3. Data model

### 3.1 GameState

Represents the latest known game state. All fields are `Option<T>` because OCR may fail or the user may not have calibrated all regions.

```rust
pub struct GameState {
    pub food: Option<u32>,
    pub wood: Option<u32>,
    pub gold: Option<u32>,
    pub stone: Option<u32>,
    pub villagers: Option<u32>,
    pub population: Option<(u32, u32)>,   // (current, max)
    pub game_time_seconds: Option<u32>,
    pub last_updated: Instant,             // for staleness detection
}
```

### 3.2 BuildOrder

```rust
pub struct BuildOrder {
    pub id: String,                        // uuid or slug
    pub name: String,
    pub civilization: String,              // "Generic", "Britons", etc.
    pub author: Option<String>,
    pub description: Option<String>,
    pub source_url: Option<String>,        // attribution
    pub tags: Vec<String>,                 // e.g. ["scouts", "feudal-aggression"]
    pub steps: Vec<Step>,
}

pub struct Step {
    pub action: String,                    // primary instruction shown to user
    pub at: Trigger,                       // when this step becomes active
    pub notes: Option<String>,             // expanded guidance
    pub villagers_assigned: Option<VillagerAssignment>, // optional resource breakdown
}

pub struct Trigger {
    pub time_seconds: Option<u32>,
    pub villagers: Option<u32>,
    pub population_min: Option<u32>,
    pub food_min: Option<u32>,
    pub wood_min: Option<u32>,
    pub gold_min: Option<u32>,
    pub stone_min: Option<u32>,
    pub mode: TriggerMode,                 // All (AND) or Any (OR), default All
}

pub enum TriggerMode { All, Any }

pub struct VillagerAssignment {
    pub food: u32,
    pub wood: u32,
    pub gold: u32,
    pub stone: u32,
    pub idle: u32,
}
```

### 3.3 Calibration

```rust
pub struct Calibration {
    pub profile_name: String,
    pub resolution: (u32, u32),
    pub ui_scale: f32,
    pub regions: HashMap<RegionKind, Region>,
}

pub enum RegionKind {
    Food, Wood, Gold, Stone,
    Villagers, Population, GameTime,
}

pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

### 3.4 AppState

```rust
pub struct AppState {
    pub current_build_order: Option<BuildOrder>,
    pub current_step_index: usize,
    pub last_game_state: GameState,
    pub capture_running: bool,
    pub calibration: Calibration,
    pub settings: Settings,
}

pub struct Settings {
    pub capture_interval_ms: u64,          // default 1000
    pub auto_advance: bool,                // default true
    pub click_through: bool,                // default false
    pub overlay_opacity: f32,              // 0.0–1.0, default 0.9
    pub hotkeys: HotkeyConfig,
    pub ocr_backend: OcrBackend,           // Template (default) | Tesseract
}
```

---

## 4. Capture subsystem

### 4.1 Primary backend (Windows)

Use **Windows.Graphics.Capture** via the `windows` crate. This is the modern, low-overhead capture API introduced in Windows 10 1903. Benefits:

- DXGI-based, no GDI overhead.
- No flashing yellow border in 1903+ (suppressed via `IsBorderRequired = false` on Windows 11).
- Captures DirectX-rendered content reliably (which AoE2:DE uses).
- Works with hardware-accelerated games without performance hit.

Implementation outline:

1. Get the primary monitor's `GraphicsCaptureItem` via `GraphicsCaptureItem::CreateFromMonitor`.
2. Create a `Direct3D11CaptureFramePool` with `BGRA8` format and a small frame buffer count (2).
3. Create a `GraphicsCaptureSession`, set `IsBorderRequired = false` and `IsCursorCaptureEnabled = false`.
4. On each frame arrival event, lock the texture, copy only the calibrated regions to CPU memory, release the frame.
5. Throttle frame processing to 1Hz — drop frames in between.

Critically: **do not copy the full screen to CPU memory each frame**. Use `ID3D11DeviceContext::CopySubresourceRegion` to copy only the rectangles you care about (resource bar, time, villager counter). This is the single biggest perf win.

### 4.2 Fallback backend

Use the `xcap` crate for non-Windows or for environments where the modern API is unavailable. It captures the full screen as an `RgbaImage` which is then cropped in CPU memory. Slower but portable.

Selection at runtime:

```rust
#[cfg(target_os = "windows")]
fn create_capture() -> Box<dyn CaptureBackend> {
    Box::new(WindowsGraphicsCapture::new()?)
}
#[cfg(not(target_os = "windows"))]
fn create_capture() -> Box<dyn CaptureBackend> {
    Box::new(XcapFallback::new()?)
}
```

### 4.3 Capture loop pseudocode

```
loop:
    if !state.capture_running: break
    frame = capture_backend.next_frame()
    for region in calibration.regions:
        cropped = frame.crop(region)
        hash = quick_hash(cropped.pixels)
        if hash == last_hash[region]: continue  # skip unchanged regions
        last_hash[region] = hash
        result = ocr.read_number(cropped, region.kind)
        update_game_state(region.kind, result)
    if game_state.changed:
        emit_event("game-state", game_state)
        engine.evaluate_triggers(game_state)
    sleep(capture_interval_ms)
```

### 4.4 Optimisations

| Tactic | Estimated saving |
|---|---|
| Region-only D3D copy | ~95% reduction in CPU↔GPU transfer |
| 1Hz sample rate | ~98% reduction vs 60Hz |
| Pixel-hash skip when unchanged | Skips OCR on most frames; resources only change every few seconds |
| Reused capture/preprocess buffers | Eliminates per-frame allocation |
| Threshold to binary before OCR | 4× faster OCR, ~10% better accuracy |

---

## 5. OCR subsystem

### 5.1 Pipeline

For each captured region:

1. **Crop** to region bounds (already done in capture).
2. **Grayscale conversion** using luminance formula `0.299R + 0.587G + 0.114B`.
3. **Threshold** to binary image. Default threshold `180`; expose as setting per region kind.
4. **(Optional) Morphological cleanup** — single-pass dilate then erode to close gaps in characters. Use `imageproc::morphology`.
5. **Character segmentation** — find connected components, return bounding boxes left-to-right.
6. **Per-character recognition** — run primary backend.
7. **Post-processing** — strip whitespace, validate against expected format (digits-only for resources, `M:SS` for time, `N/M` for population).

### 5.2 Primary backend: template matching

**Why not Tesseract by default:** Tesseract pulls in ~40MB of trained data, has a startup cost, and is overkill for a known fixed font with only ~12 possible characters (`0-9`, `/`, `:`).

**Approach:**

1. Generate templates once from screenshots of the AoE2 resource bar at the user's resolution. Templates are 12×18 (configurable) grayscale binary images of each character, stored as PNGs in `src-tauri/src/ocr/templates/{digit_0..9, slash, colon}.png` and embedded via `include_bytes!`.
2. For each segmented character, resize to template dimensions and compute similarity against each template using **normalized cross-correlation** or **sum of absolute differences**.
3. Pick the best match if its score exceeds a confidence threshold (e.g., NCC > 0.8). Otherwise return `None` for that character (treated as OCR failure for that region this tick).

**Performance target:** < 5ms per region on modern hardware.

**Memory:** ~3KB for all templates combined.

### 5.3 Fallback backend: Tesseract

Feature-gated behind `cargo feature = "tesseract"`. Use `rusty-tesseract` crate. On Windows the Tesseract binary and `tessdata/eng.traineddata` are bundled in `resources/` and `TESSDATA_PREFIX` is set at startup.

Configure with:

- `tessedit_char_whitelist = "0123456789/:"` to massively improve accuracy and speed.
- `psm = 7` (single text line).
- `oem = 3` (default LSTM engine).

### 5.4 Calibration-driven template generation

Provide a CLI/dev command `aoe-overlay generate-templates --screenshot path/to/full.png --calibration path/to/cal.json` that:

1. Loads the user's screenshot and calibration.
2. Crops each region.
3. Walks the user through labelling each character (terminal prompt: "Character 1 of region 'food':").
4. Saves templates into the templates directory, scoped by resolution.

This is dev/power-user territory; v1 ships templates for 1080p and 1440p baked in.

---

## 6. Build order engine

### 6.1 Trigger evaluation

On every game-state update, evaluate the **current step's trigger** against the latest `GameState`. If the trigger is satisfied AND `auto_advance` is enabled, advance to the next step.

```rust
fn evaluate(trigger: &Trigger, state: &GameState) -> bool {
    let checks = [
        (trigger.time_seconds, state.game_time_seconds),
        (trigger.villagers, state.villagers),
        (trigger.population_min, state.population.map(|p| p.0)),
        (trigger.food_min, state.food),
        (trigger.wood_min, state.wood),
        (trigger.gold_min, state.gold),
        (trigger.stone_min, state.stone),
    ];
    let active: Vec<bool> = checks.iter()
        .filter_map(|(target, actual)| match (target, actual) {
            (Some(t), Some(a)) => Some(*a >= *t),
            (Some(_), None) => Some(false),  // condition specified but unreadable
            _ => None,
        })
        .collect();

    if active.is_empty() { return false; }
    match trigger.mode {
        TriggerMode::All => active.iter().all(|&b| b),
        TriggerMode::Any => active.iter().any(|&b| b),
    }
}
```

### 6.2 Step advancement rules

- Never skip backwards automatically. Backwards is manual only.
- Never skip forwards more than one step per evaluation tick. Prevents runaway advancement on noisy OCR.
- Emit `step-changed` event with the new index and the new step payload.
- Optionally play a soft audio cue (configurable, default off).

### 6.3 Manual override

Hotkey-bound commands:

- `Ctrl+Alt+→` — advance one step
- `Ctrl+Alt+←` — go back one step
- `Ctrl+Alt+R` — reset to step 0
- `Ctrl+Alt+P` — pause/resume capture
- `Ctrl+Alt+H` — show/hide overlay

All hotkeys configurable via Settings.

---

## 7. Build order file format

### 7.1 YAML schema (preferred)

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

  - action: "Lure boar with TC, 4 more vills to wood"
    at: { villagers: 10 }
    notes: "Don't let boar reach lure spot before vills are eating it."

  - action: "Build mill on berries, 4 vills to berries"
    at: { villagers: 14 }

  - action: "Click up to Feudal"
    at: { villagers: 21, food_min: 500 }
    notes: "Send 2 vills to wood while clicking up."

  - action: "Build barracks, then stable. Switch wood vills to scout production."
    at: { time_seconds: 720 }  # roughly 12:00 game time
```

### 7.2 JSON schema

Equivalent JSON is supported for tooling that prefers it. Detected by file extension.

### 7.3 Validation

On load:

- All step `at` triggers must specify at least one condition (otherwise auto-advance is impossible).
- `villagers_assigned` totals (if present) must not exceed the step's villager count by more than a small margin (warn, don't fail).
- Steps should be sorted by trigger time/villager count; warn if not.

---

## 8. Frontend (Tauri webview)

### 8.1 Overlay window properties

```json
{
  "transparent": true,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "shadow": false,
  "focus": false,
  "width": 320,
  "height": 480
}
```

On Windows, additionally apply `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW` styles via the `windows` crate so the overlay never steals focus from the game and doesn't appear in Alt+Tab.

For fullscreen-exclusive games: the overlay only appears reliably over **borderless fullscreen**. Document this clearly in setup. AoE2:DE defaults to borderless, so this is fine.

### 8.2 UI states

1. **Compact (default while playing):** current step large, next step small below, optional resource readout for confidence. ~320×200px.
2. **Expanded:** full step list with current highlighted, scrollable.
3. **Calibration:** full-screen overlay with draggable rectangles for each region.
4. **Library:** browse/select build orders.
5. **Editor:** create/edit build orders (form-based, with a preview).
6. **Settings:** hotkeys, capture rate, opacity, OCR backend, etc.

### 8.3 Visual design

- Dark, semi-transparent background (`rgba(20, 20, 24, 0.85)`).
- High-contrast text. White primary, muted gray for secondary.
- Accent color: AoE-amber (`#ffb84d`) for current step.
- Avoid heavy shadows or glows that look cheap on transparent windows.
- All animations 150ms or less; no flashy transitions.

### 8.4 Tauri IPC

**Commands (frontend → backend):**

| Command | Args | Returns |
|---|---|---|
| `load_build_order` | `path: string` | `BuildOrder` |
| `start_capture` | — | `()` |
| `stop_capture` | — | `()` |
| `advance_step` | — | `usize` |
| `previous_step` | — | `usize` |
| `reset_steps` | — | `()` |
| `save_calibration` | `Calibration` | `()` |
| `get_calibration` | — | `Calibration` |
| `list_build_orders` | — | `Vec<BuildOrderMeta>` |
| `save_build_order` | `BuildOrder` | `()` |
| `get_settings` | — | `Settings` |
| `update_settings` | `Settings` | `()` |

**Events (backend → frontend):**

| Event | Payload |
|---|---|
| `game-state` | `GameState` |
| `step-changed` | `{ index: usize, step: Step }` |
| `capture-error` | `{ message: string }` |
| `ocr-debug` | `{ region: RegionKind, raw: string, parsed: Option<u32> }` (only when debug mode on) |

---

## 9. Persistence

All persisted data lives under `%APPDATA%\aoe-overlay\` on Windows (or platform equivalent via Tauri's `path::app_data_dir()`).

```
%APPDATA%\aoe-overlay\
├── settings.json
├── calibration\
│   ├── 1920x1080.json
│   └── 2560x1440.json
└── build-orders\
    ├── scouts-generic.yaml
    ├── archers-britons.yaml
    └── user\
        └── my-custom-bo.yaml
```

Bundled sample build orders are copied into `build-orders/` on first run if not present.

---

## 10. Hotkeys

Use the `global-hotkey` crate. Register on startup, deregister on shutdown.

Default bindings (all configurable):

| Action | Hotkey |
|---|---|
| Advance step | Ctrl+Alt+Right |
| Previous step | Ctrl+Alt+Left |
| Reset | Ctrl+Alt+R |
| Pause capture | Ctrl+Alt+P |
| Toggle overlay visibility | Ctrl+Alt+H |
| Toggle click-through | Ctrl+Alt+C |

---

## 11. Performance budget

| Resource | Budget |
|---|---|
| Resident memory | < 100 MB |
| CPU (capture+OCR active) | < 3% on a 4-core 3GHz CPU |
| GPU usage | Negligible (D3D copy only, no shaders) |
| Disk I/O | < 1 MB/s during normal operation |
| Startup time | < 1.5s cold, < 500ms warm |
| Frame-to-event latency | < 200ms from pixel change to UI update |

Add a simple profiler view in dev builds (`--features dev`) showing per-frame timings.

---

## 12. Build & distribution

### 12.1 Development

```bash
# Prerequisites: Rust 1.75+, Node 20+, Tauri CLI
cargo install tauri-cli --version "^2.0"
npm install
cargo tauri dev
```

### 12.2 Release build

```bash
cargo tauri build
```

Outputs:

- `src-tauri/target/release/bundle/msi/AoE Overlay_0.1.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/AoE Overlay_0.1.0_x64-setup.exe`

### 12.3 Code signing (post-MVP)

Windows SmartScreen will warn on unsigned binaries. Plan to obtain an EV code-signing certificate before public distribution. For early distribution, document the SmartScreen workaround.

---

## 13. Testing strategy

### 13.1 Unit tests

- `build_order::engine` — trigger evaluation across all condition combinations.
- `ocr::preprocess` — golden-image tests for grayscale + threshold output.
- `ocr::segment` — known-input character segmentation.
- `ocr::template` — accuracy on a fixture set of resource bar crops (target: 99%+ on clean inputs).

### 13.2 Integration tests

- End-to-end: feed a sequence of fixture screenshots, verify the engine advances correctly.
- Latency: assert event delivery < 200ms after pixel change.

### 13.3 Manual QA checklist

- [ ] Overlay appears over AoE2:DE in borderless fullscreen at 1080p, 1440p, 4K.
- [ ] Calibration wizard produces accurate regions on each resolution.
- [ ] OCR reads resources correctly across a 30-minute game.
- [ ] Build order auto-advances through all steps in `scouts-generic.yaml`.
- [ ] Hotkeys work while game has focus.
- [ ] Click-through toggle works (mouse passes through to game).
- [ ] CPU and memory stay within budget across a full match.
- [ ] No crash on monitor resolution change mid-session.

---

## 14. Build sequence (recommended order for Claude Code)

This is the order I'd build features in. Each step should be a green-tested, working state before moving on.

### Phase 1 — Skeleton (Week 1)
1. Initialize Tauri project with the config in §8.1.
2. Implement `state.rs`, `build_order/mod.rs`, `build_order/parser.rs` with full test coverage.
3. Build minimal React UI showing a static `BuildOrder` with current step highlighted.
4. Wire up `advance_step` / `previous_step` commands and global hotkeys.
5. Ship a demoable manual-advance overlay. **Milestone: usable as a static cheat sheet.**

### Phase 2 — Capture + display only (Week 2)
6. Implement Windows.Graphics.Capture backend in `capture/windows.rs`.
7. Build calibration UI (CalibrationWizard.tsx) — draggable rectangles over a screenshot.
8. Pipe raw cropped region images to the frontend for visual inspection (no OCR yet).
9. Verify capture works during an actual AoE2:DE game.

### Phase 3 — OCR (Week 3)
10. Implement preprocessing pipeline (`ocr/preprocess.rs`).
11. Generate digit templates from a 1080p AoE2 screenshot.
12. Implement `ocr/segment.rs` and `ocr/template.rs`.
13. Hook OCR into the capture loop, emit `game-state` events.
14. Validate accuracy against a fixture set.

### Phase 4 — Auto-advance (Week 4)
15. Implement `build_order/engine.rs` trigger evaluation.
16. Wire engine into capture loop.
17. Add `ocr-debug` event and debug panel for QA.
18. Test full BO execution in a real game.

### Phase 5 — Polish (Week 5+)
19. Build order editor (BuildOrderEditor.tsx).
20. Settings UI.
21. Bundle additional sample build orders.
22. MSI/NSIS installer config.
23. README, documentation, screenshots.

---

## 15. Open questions / decisions deferred

- **Multi-monitor support**: v1 captures primary only. Multi-monitor selection deferred to v1.1.
- **Click-through implementation**: Win32 `SetWindowLong` with `WS_EX_TRANSPARENT`. Toggle behaviour TBD — modal? Hotkey? Both.
- **Fullscreen-exclusive capture**: needs investigation; may require driver-level capture which is out of scope.
- **AoE4 support**: similar architecture, different calibration & OCR templates. Plan for v1.2.
- **Build order sharing**: simple file export now; web-based registry later.

---

## 16. Risks and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| AoE2:DE patch changes UI layout | Medium | Calibration is user-driven; templates regenerated from screenshots. Document re-calibration. |
| OCR accuracy below 99% causes spurious step advances | Medium | Hash-based change detection + require condition met for 2 consecutive ticks before advancing. |
| Anti-cheat false positive | Very low | Read-only screen capture; no input or memory access. Should be indistinguishable from OBS. |
| Windows SmartScreen blocks installer | High initially | Document workaround; budget for code signing cert. |
| Performance regression on low-end hardware | Low | Configurable capture interval; ship with 2Hz default for older CPUs. |

---

## 17. Appendix: Cargo.toml

```toml
[package]
name = "aoe-overlay"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["tray-icon"] }
tauri-plugin-shell = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
xcap = "0.0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
imageproc = "0.25"
global-hotkey = "0.6"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4", "serde"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Graphics_Capture",
  "Graphics_DirectX",
  "Graphics_DirectX_Direct3D11",
  "Win32_Graphics_Direct3D11",
  "Win32_Graphics_Dxgi",
  "Win32_System_WinRT_Graphics_Capture",
  "Win32_UI_WindowsAndMessaging",
] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
tesseract = ["dep:rusty-tesseract"]

[dependencies.rusty-tesseract]
version = "1.1"
optional = true

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

---

**End of spec.** This document should be sufficient for Claude Code to scaffold the project, implement Phase 1 in a single session, and incrementally build out the remaining phases. Hand it off as the initial context, then issue specific phase-by-phase build commands.