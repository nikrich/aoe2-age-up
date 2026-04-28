# Phase 1 Design — Tauri Skeleton with Manual-Advance Overlay

**Date:** 2026-04-28
**Scope:** Phase 1 of spec.md (Section 14, steps 1-5)
**Milestone:** Usable as a static cheat sheet with manual step advancement

---

## Decisions

- **Platform:** Windows-first development (Windows machine available, AoE2 is Windows-only)
- **Scaffold:** Vite React+TS template, then `cargo tauri init` on top (Approach A)
- **Styling:** Plain CSS, no framework
- **Stubs:** capture/ and ocr/ modules stubbed with trait definitions, no implementation
- **Deferred:** Screen capture (Phase 2), OCR (Phase 3), auto-advance (Phase 4), editor/settings/installer (Phase 5)

---

## 1. Project Scaffolding & Window Config

Scaffold via `npm create vite` (React+TS) then `cargo tauri init` to add `src-tauri/`.

### Tauri window config (spec S8.1)

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

On Windows, apply `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW` via the `windows` crate in Tauri's `setup` hook so the overlay never steals focus or appears in Alt+Tab.

### Cargo.toml (Phase 1 deps)

Match spec S17 but skip `windows` crate capture features, `xcap`, `image`, `imageproc` (Phase 2-3). Phase 1 dependencies:

- `tauri` 2.0 with `tray-icon` feature
- `tauri-plugin-shell` 2.0
- `serde` + `serde_json` + `serde_yaml`
- `tokio` (full features)
- `anyhow` + `thiserror`
- `global-hotkey` 0.6
- `tracing` + `tracing-subscriber`
- `uuid` (v4, serde)

---

## 2. Rust Backend

### state.rs

Types from spec S3:

- `AppState` — holds current build order, step index, game state, calibration, settings
- `GameState` — all `Option<T>` fields (food, wood, gold, stone, villagers, population, game_time_seconds, last_updated)
- `Calibration` — profile name, resolution, UI scale, regions hashmap
- `Region`, `RegionKind` — screen rectangle definitions
- `Settings` — capture interval, auto_advance, click_through, overlay_opacity, hotkeys, OCR backend
- `HotkeyConfig` — configurable key bindings

All types derive `Serialize`, `Deserialize`, `Clone`, `Debug`. `AppState` wrapped in `Arc<Mutex<>>` and managed by Tauri.

### build_order/mod.rs

Types from spec S3.2:

- `BuildOrder` — id, name, civilization, author, description, source_url, tags, steps
- `Step` — action, trigger, notes, villager assignment
- `Trigger` — optional thresholds for time/villagers/population/resources + TriggerMode (All/Any)
- `VillagerAssignment` — food/wood/gold/stone/idle counts
- `TriggerMode` enum — All (AND) | Any (OR)

- `BuildOrderMeta` — lightweight summary (id, name, civilization, tags) returned by `list_build_orders` to avoid sending full step data to the frontend for the library view

### build_order/parser.rs

- Load from YAML (`.yaml`/`.yml`) or JSON (`.json`) based on file extension
- Validation on load:
  - Every step's trigger must specify at least one condition
  - Warn (don't fail) if villager assignment totals exceed step villager count
  - Warn if steps aren't sorted by trigger time/villager count
- Returns `Result<BuildOrder, anyhow::Error>` with descriptive messages

### build_order/engine.rs

Trigger evaluation from spec S6.1:

- `evaluate(trigger: &Trigger, state: &GameState) -> bool`
- Collects active conditions (where trigger field is `Some`), checks against game state
- `TriggerMode::All` — all must be met; `TriggerMode::Any` — at least one
- Empty active set returns `false`
- Phase 1: exposed as a standalone function, not wired to capture loop

### Stubs (trait definitions only)

- `capture/mod.rs` — `CaptureBackend` trait: `fn next_frame(&mut self) -> Result<CaptureFrame>`
- `capture/windows.rs` — empty, `#[cfg(target_os = "windows")]`
- `capture/fallback.rs` — empty
- `ocr/mod.rs` — `OcrPipeline` trait: `fn read_number(&self, image: &[u8], kind: RegionKind) -> Result<Option<u32>>`
- `ocr/preprocess.rs`, `ocr/segment.rs`, `ocr/template.rs`, `ocr/tesseract.rs` — empty files with module declarations

### ipc.rs

Tauri commands (frontend -> backend):

| Command | Args | Returns |
|---|---|---|
| `load_build_order` | `path: String` | `BuildOrder` |
| `list_build_orders` | -- | `Vec<BuildOrderMeta>` |
| `advance_step` | -- | `usize` |
| `previous_step` | -- | `usize` |
| `reset_steps` | -- | `()` |
| `get_settings` | -- | `Settings` |
| `get_calibration` | -- | `Calibration` |

Events (backend -> frontend):

| Event | Payload |
|---|---|
| `step-changed` | `{ index: usize, step: Step }` |

### hotkeys.rs

Register global hotkeys on startup via `global-hotkey` crate. Default bindings from spec S10:

| Action | Default |
|---|---|
| Advance step | Ctrl+Alt+Right |
| Previous step | Ctrl+Alt+Left |
| Reset | Ctrl+Alt+R |
| Pause capture | Ctrl+Alt+P |
| Toggle visibility | Ctrl+Alt+H |
| Toggle click-through | Ctrl+Alt+C |

Hotkey presses invoke the same underlying logic as the IPC commands.

### storage.rs

File-based persistence using Tauri's `app_data_dir()`:

```
{app_data}/
  settings.json
  calibration/
  build-orders/
    (bundled samples copied here on first run)
    user/
```

On first run, copy bundled sample build orders into `build-orders/` if not already present.

---

## 3. React Frontend

### Components

**App.tsx** — Simple router between Overlay (default), Library, and Settings views.

**Overlay.tsx** — Compact view (~320x200px):
- Current step large with AoE-amber accent
- Next step smaller below
- Step counter (e.g. "3 / 12")
- Listens to `step-changed` events

**StepCard.tsx** — Renders one step:
- Action text (primary)
- Optional notes (secondary, muted)
- Optional villager assignment breakdown
- Highlighted variant for current step

**BuildOrderLibrary.tsx** — Lists available build orders from `list_build_orders`. Shows name, civilization, tags, description. Click to load.

**Settings.tsx** — Phase 1 stub. Read-only display of current hotkey bindings.

### Hooks

- `useGameState.ts` — Subscribes to `game-state` events (empty in Phase 1, ready for Phase 2+)
- `useBuildOrder.ts` — Subscribes to `step-changed` events, tracks current step index and loaded build order
- `useTauriCommand.ts` — Thin wrapper around Tauri `invoke()`

### Styles (overlay.css)

- Background: `rgba(20, 20, 24, 0.85)`
- Primary text: white; secondary: muted gray
- Current step accent: `#ffb84d` (AoE-amber)
- No heavy shadows or glows
- All transitions 150ms max
- Compact mode target: ~320x200px

### Sample build orders

Three YAML files in `build-orders/`:
- `scouts-generic.yaml`
- `archers-britons.yaml`
- `fast-castle-cavalier.yaml`

---

## 4. Testing

### Rust unit tests

- **parser** — Valid YAML/JSON loading, rejection of invalid input (missing triggers, malformed), warnings on unsorted steps
- **engine** — Trigger evaluation: All mode (AND), Any mode (OR), partial OCR data (None fields), empty triggers return false, single condition, all conditions met/unmet
- **state** — Serialization round-trips for GameState, Calibration, Settings

### Integration test

Load a sample build order -> advance through all steps via IPC commands -> verify step indices and events.

### Manual verification

- `cargo tauri dev` launches the overlay window
- Overlay displays a loaded build order with current step highlighted
- Global hotkeys advance/go back steps
- Build order library lists and loads sample YAMLs
