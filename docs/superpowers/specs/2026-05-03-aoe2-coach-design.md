# AoE2 Build Order Coach — Design Spec

**Date:** 2026-05-03
**Status:** Approved
**Scope:** Live timing delta engine, coaching overlay, tiered free/coach modes
**Supersedes:** Static step overlay in `src/components/Overlay.tsx`

---

## 1. Overview

Add a real-time coaching engine that compares the player's live game state (via OCR) against a benchmark timeline derived from the loaded build order. The overlay shows how far ahead or behind the player is on villagers, resources, and age-up timing, plus actionable coaching toasts (TC idle, floating resources, housed out).

The feature ships behind a tier gate: free tier retains the current step-by-step guide overlay; coach tier unlocks the full delta engine and coaching UI.

---

## 2. Tier System

- `Settings.tier: "free" | "coach"` (default `"free"`)
- Configured in the Settings page via a "Mode" dropdown
- **Free tier**: current step-by-step overlay (guide mode), nav buttons visible, no delta computation
- **Coach tier**: 3-zone overlay with delta strip + coaching toasts, no nav buttons, full delta engine active
- Gate checked at two points:
  1. Frontend renders `GuideOverlay` or `CoachOverlay` based on tier
  2. Backend skips delta pipeline in capture loop when tier is `"free"`

---

## 3. Domain Types

### 3.1 Extended BuildOrder

Existing `BuildOrder` struct gains two new optional fields (both `#[serde(default)]` for backward compat):

```rust
benchmarks: Vec<Benchmark>,            // empty until synthesizer runs or hand-authored
expected_uptimes: Option<ExpectedUptimes>,
```

Existing `Step`, `Trigger`, `VillagerAssignment` types are unchanged.

### 3.2 Benchmark

Snapshot of expected state at a specific game time:

```rust
struct Benchmark {
    game_time_seconds: u32,
    expected_villagers: u32,
    expected_villager_split: VillagerSplit,
    expected_resources: ResourceSnapshot,
    expected_population: u32,
    expected_age: Age,
    expected_buildings: HashMap<String, u32>,
    expected_techs: Vec<String>,
    expected_military: HashMap<String, u32>,
    primary_focus: String,
    tolerance_seconds: u32,
}
```

Spaced every 30s during Dark Age, every 60s in Feudal+, plus at step boundaries and age-up clicks.

### 3.3 VillagerSplit

```rust
struct VillagerSplit {
    food_sheep: u32,
    food_boar: u32,
    food_berry: u32,
    food_farm: u32,
    food_deer: u32,
    wood: u32,
    gold: u32,
    stone: u32,
    builder: u32,
    idle: u32,
}
```

### 3.4 ResourceSnapshot

```rust
struct ResourceSnapshot {
    food: i32,
    wood: i32,
    gold: i32,
    stone: i32,
}
```

Signed integers — deltas can be negative.

### 3.5 ExpectedUptimes

```rust
struct ExpectedUptimes {
    feudal_click_seconds: u32,
    feudal_up_seconds: u32,
    castle_click_seconds: Option<u32>,
    castle_up_seconds: Option<u32>,
    imperial_click_seconds: Option<u32>,
    imperial_up_seconds: Option<u32>,
}
```

### 3.6 OcrField

Replaces `Option<u32>` on GameState fields:

```rust
struct OcrField {
    value: u32,
    confidence: f32, // 0.0-1.0
}
```

Confidence < 0.5 means field treated as unknown; delta hidden for that dimension.

### 3.7 LiveState

Extended game state with inferred fields:

```rust
struct LiveState {
    game_time_seconds: u32,
    resources: ResourceSnapshot,
    villagers: u32,
    population: (u32, u32),
    current_age: Age,
    age_up_progress: Option<f32>,
    villager_split_estimate: VillagerSplit,
    builder_count_estimate: u32,
    idle_villager_estimate: u32,
    tc_idle_seconds_total: f32,
    tc_idle_current_seconds: f32,
    ocr_confidence: OcrConfidence,
}

struct OcrConfidence {
    food: f32,
    wood: f32,
    gold: f32,
    stone: f32,
    villagers: f32,
    population: f32,
    game_time: f32,
}
```

### 3.8 Delta

```rust
struct Delta {
    time_seconds: u32,
    villager_delta: i32,
    villager_seconds_behind: i32,
    resource_delta: ResourceSnapshot,
    pop_delta: i32,
    age_delta_seconds: i32,
    split_delta: VillagerSplit, // as signed differences
    build_delta: HashMap<String, i32>,
    tech_missing: Vec<String>,
    tc_idle_total: f32,
    tc_idle_warning: bool,
    ocr_stale: bool,
    severity: Severity,
    severity_reason: String,
}

enum Severity { OnTrack, Minor, OffPace, Broken }
```

### 3.9 CoachingToast

```rust
struct CoachingToast {
    id: String,
    message: String,
    severity: Severity,
    triggered_at: u32,
    auto_dismiss_at: u32,
    dismissed: bool,
}
```

### 3.10 Age

```rust
enum Age { Dark, Feudal, Castle, Imperial }
```

---

## 4. Civ Data

### Location: `src-tauri/src/coach/civdata/`

### 4.1 Base Rates (`rates.rs`)

Static constants for all gather/production/research rates:

| Resource | Source | Rate (per vill/s) |
|----------|--------|--------------------|
| Food | Sheep | 0.33 |
| Food | Boar | 0.40 |
| Food | Berry | 0.30 |
| Food | Farm | 0.40 |
| Food | Deer | 0.40 |
| Wood | Base | 0.39 |
| Gold | Base | 0.38 |
| Stone | Base | 0.38 |

Tech modifiers: Double-Bit Axe +15%, Bow Saw +20%, Two-Man Saw +10%, Horse Collar/Heavy Plow/Crop Rotation for farms, Gold Mining +15%, Gold Shaft Mining +30%, Stone Mining +15%, Stone Shaft Mining +15%.

TC production: 25s per villager. Age-up: 130s feudal, 160s castle, 190s imperial. Costs: 500F feudal, 800F+200G castle, 1000F+800G imperial.

### 4.2 Civ Bonuses (`bonuses.rs`)

All 45+ civs. Each as a `CivBonus` struct:

```rust
struct CivBonus {
    id: &'static str,
    name: &'static str,
    starting_food_bonus: i32,
    starting_wood_bonus: i32,
    starting_villagers: u8,       // 3 default, 4 Mayans, 6 Chinese
    starting_food_override: Option<i32>,
    forager_bonus: f32,           // 1.0 = no bonus
    shepherd_bonus: f32,
    hunter_bonus: f32,
    lumberjack_bonus: f32,
    gold_miner_bonus: f32,
    stone_miner_bonus: f32,
    farmer_bonus: f32,
    age_up_cost_mod: Option<AgeCostMod>,
    age_up_time_mod: f32,         // 1.0 = standard
    tc_work_rate_bonus: f32,
    free_techs: Vec<&'static str>,
}
```

Stored as a `const` array or `lazy_static!` lookup by civ name. A `CIV_DATA_VERSION` constant tracks the balance patch.

---

## 5. Benchmark Synthesizer

### Location: `src-tauri/src/coach/synthesizer.rs`

### Algorithm

1. **Initialize** at `t=0`: villagers = `civ.starting_villagers`, resources = civ starting amounts + bonuses, age = Dark
2. **Assign villagers** to gather tasks based on current step's `villagers_assigned`
3. **Tick forward** in 1-second increments:
   - Each villager generates resources at civ-modified gather rate
   - TC produces a villager every 25s (50F deducted). If food < 50, TC idles
   - Track queued researches (cost at queue, effect at completion)
   - Track building construction (cost at placement, complete after build time)
4. **Snapshot** every 30s in Dark Age, 60s in Feudal+, plus at step boundaries and age-up clicks
5. **Fill benchmark fields**: vills, split, resources, pop, age, buildings, techs, military, primary_focus from step action, tolerance_seconds

### Step-to-time Mapping

Steps with triggers (e.g. `at: { villagers: 14 }`) get resolved to concrete `expected_time_seconds` by the simulator hitting that condition.

### Validation

- Flag impossible timings (step requires resources sim says aren't available)
- Flag gaps (>60s between step triggers with no assignment change)
- Regression check: synthesized vs hand-authored benchmarks must agree within 10s per checkpoint, 15s cumulative

### Interpolation

`interpolate_benchmark(benchmarks: &[Benchmark], game_time_s: u32) -> Benchmark` — linear interpolation between two nearest snapshots for all numeric fields.

---

## 6. Delta Engine

### Location: `src-tauri/src/coach/delta.rs`

### Core: `compute_delta(live: &LiveState, benchmark: &Benchmark) -> Delta`

### Computations

- **Villager delta**: `live.villagers - benchmark.expected_villagers`
- **Villager-seconds behind**: `villager_delta * 25`
- **Resource delta**: per-resource `live - expected`
- **Age delta**: seconds since benchmark said age-up should have happened (if behind), or time difference on age-up click
- **Pop delta**: `live.population.0 - benchmark.expected_population`
- **Split delta**: `expected - actual` per resource
- **TC idle**: accumulated and current streak from LiveState

### Severity Thresholds (scaled by game phase)

| Phase | on-track | minor | off-pace | broken |
|-------|----------|-------|----------|--------|
| Dark (0-8min) | <=1 vill, <=8s late | 2 vill, 9-20s | 3 vill, 21-40s | >=4 vill, >40s |
| Feudal (8-14min) | <=2 vill, <=15s | 3 vill, 16-30s | 4-5 vill, 31-60s | >=6 vill, >60s |
| Castle+ (14min+) | <=3 vill, <=20s | 5 vill, 21-45s | 7 vill, 46-90s | >=10 vill, >90s |

Overall severity = max across all fields.

### Smoothing (`DeltaSmoother`)

- Ring buffer of last 6 severity samples (3s at 2Hz)
- Severity transitions require 3 consecutive samples at new level
- Numeric values use 2-second EMA
- TC idle warning fires after 6s continuous idle

---

## 7. Villager Split Inference

### Location: `src-tauri/src/coach/inference.rs`

### Algorithm

1. Rolling 10-second window of ResourceSnapshot samples
2. Linear regression for per-resource gain rate
3. Apply civ-modified gather rates (using benchmark's expected techs as prior)
4. Solve `vills_on_X = rate_X / per_vill_rate_X` per resource
5. Food decomposed using benchmark's expected split as prior for sheep/boar/berry/farm rates
6. Unaccounted vills = builders + idle
7. Confidence per resource: high >0.8 (stable), medium 0.5-0.8 (noisy), low <0.5 (unreliable)

### Edge Cases

- Loom/age-up researching: don't flag TC idle
- Boar lure: 3s settling delay before updating food split
- Market transactions: outlier detection (>3 sigma), excluded from rate computation

### State

```rust
struct SplitInferencer {
    resource_history: VecDeque<(u32, ResourceSnapshot)>,
    max_window_seconds: u32, // 10
    last_estimate: VillagerSplit,
    last_confidence: SplitConfidence,
}
```

---

## 8. Coaching Toasts

### Location: `src-tauri/src/coach/toasts.rs`

### Rules

| Rule | Condition | Severity |
|------|-----------|----------|
| TC idle | `tc_idle_current > 6s` | minor@6s, off-pace@12s, broken@20s |
| Vill misallocation | Single resource off by >2 vills for >10s | minor |
| Missed eco tech | Expected tech not researched 60s past benchmark | off-pace |
| Floating resources | >200 of a resource BO says to spend | off-pace |
| Housed out | `pop == pop_cap` for >3s | broken |

### Lifecycle

- Max 2 visible toasts (highest severity wins)
- Auto-dismiss after 10s game time or when condition resolves
- Dismissable via `acknowledge_toast` hotkey
- Same rule can't re-fire within 15s cooldown

---

## 9. Overlay UI

### Free (Guide) Mode — `GuideOverlay.tsx`

Current overlay extracted as-is: header, step cards, nav buttons, capture toggle, game state bar.

### Coach Mode — `CoachOverlay.tsx`

3-zone layout:

```
+---------------------------------------+
| Header: BO name / Civ / Step X/Y     |
| Feudal click in 0:42 (target)         |
+---------------------------------------+
| Delta Strip:                          |
|  -18s behind feudal           [glyph] |
|  16 vills (-2)     18 expected        |
|  F 320 (-60)  W 410 (+30)            |
+---------------------------------------+
| NOW: Lure 2nd boar with TC           |
| NEXT: Build mill on berries           |
+---------------------------------------+
| ! TC idle 8s -- queue villagers       |
| ! 1 extra vill on wood vs plan       |
+---------------------------------------+
```

No nav buttons. Step advancement via delta engine / auto-advance.

### Delta Strip Rules

- Headline metric: `age_delta_seconds` when in expected age; countdown during age-up research
- Villager delta always visible, colored by threshold
- Resource deltas only for resources expected to be non-zero
- One-glance readable in <300ms

### Severity Colors & Glyphs

```css
--severity-on-track: #4ade80;  /* checkmark */
--severity-minor:    #fbbf24;  /* dot */
--severity-off-pace: #fb923c;  /* triangle */
--severity-broken:   #f87171;  /* cross */
```

All pass 4.5:1 contrast on `rgba(20,20,24,0.85)`. Color never sole signal.

### Component Structure

- `Overlay.tsx` — router, renders GuideOverlay or CoachOverlay
- `GuideOverlay.tsx` — extracted current overlay
- `CoachOverlay.tsx` — composed of DeltaStrip, StepDisplay, CoachingToasts
- `DeltaStrip.tsx` — headline delta metrics
- `CoachingToasts.tsx` — bottom toast area
- `OverlayHeader.tsx` — shared header
- `StepDisplay.tsx` — current + next step (reuses StepCard)

### New Hooks

- `useDelta.ts` — listens to `"delta-updated"`, returns current Delta
- `useCoachingToasts.ts` — listens to `"coaching-toast"`, manages queue

---

## 10. Backend Wiring

### New Tauri Commands

```rust
get_benchmark_at(game_time_s: u32) -> Option<Benchmark>
get_current_delta() -> Option<Delta>
synthesize_benchmarks(bo_id: String, civ: String) -> Vec<Benchmark>
get_session_log() -> SessionLog
export_session(path: String) -> ()
```

### New Events

| Event | Payload | Frequency |
|-------|---------|-----------|
| `"delta-updated"` | Delta | Every capture tick (~2Hz) when coach tier |
| `"coaching-toast"` | CoachingToast | On condition trigger |
| `"tc-idle"` | `{ current_idle_s, total_idle_s }` | At 3s, 6s, 10s, 20s thresholds |

Existing `"step-changed"` and `"game-state"` events unchanged.

### Capture Loop Changes

When coach tier active, after OCR:
1. GameState (with OcrField confidence) -> LiveState
2. Feed through SplitInferencer, update TC idle tracker
3. Interpolate benchmark for current game time
4. `compute_delta()` -> DeltaSmoother -> emit `"delta-updated"`
5. Evaluate toast rules -> emit `"coaching-toast"` if triggered

All coach state lives in `CoachState` struct inside AppState.

### New Hotkeys

- `toggle_coach_mode` — cycles overlay mode
- `acknowledge_toast` — dismiss current toast
- `mark_session_event` — flag moment for post-game review

### Session Log (`coach/session.rs`)

- `Vec<(u32, Delta)>` — timestamped deltas
- `Vec<CoachingToast>` — all fired toasts
- `Vec<u32>` — user-marked events
- Export as JSON or CSV

---

## 11. File Layout

### New Rust files

```
src-tauri/src/coach/
  mod.rs
  delta.rs
  benchmark.rs
  synthesizer.rs
  inference.rs
  toasts.rs
  session.rs
  civdata/
    mod.rs
    rates.rs
    bonuses.rs
```

### Modified Rust files

| File | Change |
|------|--------|
| `state.rs` | Add tier to Settings, CoachState to AppState, OcrField, LiveState |
| `build_order/mod.rs` | Add benchmarks + expected_uptimes to BuildOrder |
| `capture/loop_task.rs` | Add coach pipeline stage |
| `ipc.rs` | 5 new commands, 3 new events |
| `hotkeys.rs` | 3 new hotkey bindings |
| `lib.rs` | Register commands, declare coach module |

### New frontend files

```
src/components/CoachOverlay.tsx
src/components/GuideOverlay.tsx
src/components/DeltaStrip.tsx
src/components/CoachingToasts.tsx
src/components/OverlayHeader.tsx
src/components/StepDisplay.tsx
src/hooks/useDelta.ts
src/hooks/useCoachingToasts.ts
```

### Modified frontend files

| File | Change |
|------|--------|
| `Overlay.tsx` | Router: GuideOverlay or CoachOverlay based on tier |
| `lib/types.ts` | All new types |
| `components/Settings.tsx` | Tier dropdown |
| `App.tsx` | Pass tier to Overlay |

### New canonical build orders

```
build-orders/canonical/
  scouts-22pop-franks.yaml
  scouts-22pop-generic.yaml
  scouts-22pop-mongols.yaml
  archers-23pop-britons.yaml
  archers-23pop-generic.yaml
  drush-fc-generic.yaml
  fc-knights-generic.yaml
  boom-generic.yaml
```

---

## 12. Build Sequence

| # | Workstream | Depends on |
|---|------------|------------|
| 1 | Domain types (Rust + TS) | nothing |
| 2 | Civ data tables (rates.rs, bonuses.rs) | nothing |
| 3 | Benchmark synthesizer + tests | 1, 2 |
| 4 | Canonical BOs with hand-authored benchmarks | 1 |
| 5 | Delta engine + smoothing + severity | 1, 3 |
| 6 | Villager split inference | 1, 2 |
| 7 | Tauri commands + events | 5, 6 |
| 8 | Overlay UI (GuideOverlay + CoachOverlay) | 7 |
| 9 | Coaching toast rules | 5, 7 |
| 10 | Session log + export | 5 |
| 11 | Settings additions (tier, hotkeys) | 8, 9 |

Workstreams 1-4 are parallelizable. 5-6 are the technically risky pieces. 7-9 are wiring. 10-11 are polish.

---

## 13. Testing Strategy

1. **Synthetic LiveState streams** — programmatic: "perfect player", "5s late per vill", "forgot loom", "drushed at 8:00". Assert severity transitions.
2. **Canonical BO conformance** — every shipped BO run through synthesizer, diffed against hand-authored benchmarks. Tolerance 5s per checkpoint, 15s cumulative.
3. **Inference accuracy** — synthetic resource rate streams, verify split estimation within +/-1 vill.
4. **Toast rules** — unit tests per rule with synthetic delta streams.
5. **Smoothing** — verify severity dwell time (3 samples) and EMA convergence.
