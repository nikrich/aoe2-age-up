# AoE2 Build Order Coach — Live Timing Delta Spec

**Status:** v0.1 · supersedes the static "current/next step" overlay in `src/components/Overlay.tsx`
**Scope:** Age of Empires II: Definitive Edition. Single-player and multiplayer 1v1, observer-mode replays out of scope.
**Goal:** Live, in-game feedback that tells the player *exactly* how far they are from the optimal execution of the build order they loaded — not just where in the BO they are, but how late, how short on villagers, how off on resource splits.

---

## 1. The gap this fills

Existing tooling is split into two camps:

1. **Play-along trainers** (BuildOrderGuide, Rahien's PWA, in-game BO overlays): show the next step on a timeline, but have no idea what the player is actually doing. They drift the second you take damage, find an extra deer, or your scout dies.
2. **Post-game analysis** (CaptureAge, Capture Age aoe2.net stats): incredibly detailed, but only after the game is over. You learn you idled 47 vill-seconds at 6:30 *after* you've already lost the game.

What's missing is the **real-time delta**: at minute 7:30 of *this* game, with *this* civ, on *this* build order, you should have 17 villagers and 380 wood, and you actually have 15 villagers and 290 wood. Show that on the overlay. The OCR pipeline already implied by `useGameState` makes this possible — the spec below defines the comparison engine and UI that turn raw captured numbers into coaching feedback.

---

## 2. Domain model — what an "optimal" BO actually contains

The current `BuildOrder` / `Step` / `Trigger` types model a BO as a sequence of actions with trigger conditions. That's enough for "tell me what to do next" but **not** enough for delta feedback. To compute "you're 12 seconds behind", we need to know *what the world should look like at every moment of the BO*, not just at step boundaries.

The fix: every BO carries **two parallel structures**:

- `steps[]` — the human-readable action list (already exists, mostly fine)
- `benchmarks[]` — a dense timeline of expected world-state at fixed checkpoints, against which live OCR is diffed

### 2.1 Updated `BuildOrder`

```ts
interface BuildOrder {
  id: string;
  name: string;
  civilization: string;     // e.g. "Franks", "Mongols", "Generic"
  map_type: MapType;        // affects benchmarks (Arabia vs Arena timing differs ~30s)
  strategy: StrategyTag;    // "ScoutRush" | "ArcherRush" | "FastCastle" | "Drush" | "MAA" | "Boom" | "Tower" | "Custom"
  feudal_target_pop: number;       // e.g. 22 for scouts, 23 for archers, 27 for FC
  castle_target_pop?: number;      // for FC / boom orders
  author?: string;
  description?: string;
  source_url?: string;
  tags: string[];
  starting_villagers: number;      // 3 default, 4 Maya, 6 Chinese, 0 Huns w/ different setup
  loom_timing: LoomTiming;
  steps: Step[];
  benchmarks: Benchmark[];         // NEW — derived or hand-authored
  expected_uptimes: ExpectedUptimes; // NEW — feudal/castle/imperial click times
}

type MapType = "Arabia" | "Arena" | "BlackForest" | "Nomad" | "Islands" | "Hybrid" | "Generic";
type StrategyTag = "ScoutRush" | "ArcherRush" | "MAA" | "MAAArchers" | "Drush" | "DrushFC"
                 | "FastCastle" | "Boom" | "Tower" | "FastImp" | "Custom";
type LoomTiming = "Pre-feudal" | "While-clicking-up" | "Skip" | "Civ-default"; // Mayans always pre-loom
```

### 2.2 `Benchmark` — the heart of the delta engine

A benchmark is a snapshot of *expected* state at a specific game time. Benchmarks are spaced every 30 seconds of game time during Dark Age, every 60s in Feudal+, plus extra benchmarks at key moments (every age-up click, every step boundary, every TC villager production tick).

```ts
interface Benchmark {
  game_time_seconds: number;       // game-clock time, NOT wall time

  // Expected economy state
  expected_villagers: number;
  expected_villager_split: VillagerSplit; // F/W/G/S/Idle/Builder counts
  expected_resources: ResourceSnapshot;   // floor values — "you should have at least this much"
  expected_population: number;            // vills + military + scout

  // Expected progress milestones
  expected_age: Age;                      // Dark/Feudal/Castle/Imp
  expected_buildings: BuildingCount;      // mill, lc(s), barracks, range, stable, market, blacksmith, tc(s) etc.
  expected_techs: string[];               // researched techs ("Loom", "DoubleBitAxe", "HorseCollar"...)
  expected_military: MilitarySnapshot;    // unit type → count, mostly empty in dark age

  // Coaching context
  primary_focus: string;                  // "Lure 2nd boar", "Click feudal", "Walling B-line"
  tolerance_seconds: number;              // how much slip is acceptable here (lower in early game)
}

interface VillagerSplit {
  food_sheep: number;       // separated from food because sheep depletes
  food_boar: number;
  food_berry: number;
  food_farm: number;
  food_deer: number;
  wood: number;
  gold: number;
  stone: number;
  builder: number;          // actively constructing
  idle: number;             // expected to be 0 almost always
}

interface ResourceSnapshot { food: number; wood: number; gold: number; stone: number; }
interface BuildingCount { [building_id: string]: number; }
interface MilitarySnapshot { [unit_id: string]: number; }
type Age = "Dark" | "Feudal" | "Castle" | "Imperial";
```

### 2.3 `ExpectedUptimes` — the headline numbers a player actually cares about

Most coaching conversation in AoE2 happens in terms of "feudal time" and "castle time." These are the canonical metrics.

```ts
interface ExpectedUptimes {
  feudal_click_seconds: number;   // when 500F is paid
  feudal_up_seconds: number;      // feudal_click + 130s research time
  castle_click_seconds?: number;  // when 800F+200G is paid (FC/boom orders)
  castle_up_seconds?: number;     // castle_click + 160s
  imperial_click_seconds?: number;
  imperial_up_seconds?: number;   // + 190s
}
```

For reference (calibrated against the public benchmarks below):

| Strategy | Pop @ click | Feudal click | Feudal up | Castle click | Castle up |
|----------|-------------|--------------|-----------|--------------|-----------|
| Scout Rush (22 pop) | 22 | 7:55 | 10:05 | — | — |
| Archers (23 pop) | 23 | 8:25 | 10:35 | — | — |
| Drush → FC | 21 | 8:00 | 10:10 | 14:30 | 17:10 |
| Fast Castle (27 pop) | 27 | 11:05 | 13:15 | 14:45 | 17:25 |
| Boom (FC 28+) | 28 | 11:30 | 13:40 | 15:10 | 17:50 |

These are **generic-civ no-bonus** numbers, derived from: 3 starting vills, 25s/vill TC production, no idle TC time, Loom researched, +5–8s "human slop" *not* baked in (we want to coach against the perfect line and let the player see their own slop). Civ bonuses (Mayans extra vill, Chinese 6 vills no food, Bengalis +2 vills on feudal up, Khmer no buildings to age up, Italians 425F to feudal, Burgundians eco-tech-shifted-down-an-age) shift these by 10–60 seconds and live in per-civ overrides on the BO.

### 2.4 Why benchmarks need civilization awareness

The same "21 pop scouts" build means different optimal numbers per civ:

- **Mayans**: start with 4 vills → hit 21 pop ~25s faster → feudal time ~7:30
- **Chinese**: start with 6 vills but no food and -50W -200F → loom first, hit 22 pop around the same generic time despite more starting vills
- **Mongols**: hunters work 40% faster → 17-pop scouts is viable, feudal at ~6:35
- **Burgundians**: can research Wheelbarrow in feudal → benchmark vill-throughput accelerates one age earlier
- **Franks**: foragers +25% → berry vills' food contribution differs in the resource benchmark

The benchmark generator (§5) takes a generic BO + a target civ and applies civ-bonus modifiers to produce the civ-specific timeline.

---

## 3. Live state model — what we capture vs. what we infer

The existing `GameState` covers the OCR-readable fields (resources, vill count, pop, time). For coaching we need a richer picture, some of it inferred:

```ts
interface LiveState {
  // Directly OCR'd (with confidence per field)
  game_time_seconds: number;
  resources: ResourceSnapshot;
  villagers: number;            // total vill count from UI
  population: [number, number]; // current/cap
  current_age: Age;             // detected from age icon in UI
  age_up_progress?: number;     // 0..1 if currently researching feudal/castle/imp

  // Inferred from delta tracking (see §4)
  villager_split_estimate: VillagerSplit;  // we track resource trickle to back-derive vill assignments
  builder_count_estimate: number;
  idle_villager_estimate: number;
  tc_idle_seconds_total: number;           // accumulated since BO start
  tc_idle_current_seconds: number;         // current idle streak

  // Per-field OCR confidence — important for not flagging false negatives
  ocr_confidence: {
    food: number; wood: number; gold: number; stone: number;
    villagers: number; population: number; game_time: number;
  };
}
```

**TC idle detection** is critical and not trivially OCR-able. The capture pipeline detects it heuristically: if `villagers` count hasn't ticked up in >25s of game time AND we're not aged up to Feudal (where TC research blocks production), the TC was idle. This is the single most actionable feedback for sub-1500 ELO players.

**Villager split inference** (§4.3) is the harder problem. We can't see the assignment panel reliably across all UI scales, so we back-derive it from resource gain rates over rolling windows.

---

## 4. Delta engine — the comparison

Given `LiveState` at time `t` and the BO's `Benchmark` at time `t` (interpolated), produce a `Delta`:

```ts
interface Delta {
  time_seconds: number;

  // Headline scalars — what the UI surfaces front and center
  villager_delta: number;           // negative = behind. -3 means "3 vills short"
  villager_seconds_behind: number;  // -3 vills × 25s/vill = -75 vill-seconds of production lost
  resource_delta: ResourceSnapshot; // negative means short
  pop_delta: number;
  age_delta_seconds: number;        // negative = late on age-up. -42 = "feudal click is 42s late"

  // Distributional
  split_delta: VillagerSplit;       // expected − actual per resource. +2 wood means "2 more on wood than plan"
  build_delta: BuildingCount;       // missing buildings
  tech_missing: string[];

  // Quality
  tc_idle_total: number;
  tc_idle_warning: boolean;         // currently idle for >5s
  ocr_stale: boolean;               // any critical field has confidence <0.7

  // Severity classification — drives UI color/urgency
  severity: "on-track" | "minor" | "off-pace" | "broken";
  severity_reason: string;          // human-readable: "TC idle 12s — make villagers"
}
```

### 4.1 Severity rules

Severity is calculated per-field with thresholds that scale with game phase. A 2-villager deficit at 4:00 is catastrophic; at 18:00 it's noise. The thresholds:

| Phase | on-track | minor | off-pace | broken |
|-------|----------|-------|----------|--------|
| Dark Age (0–8min) | ≤−1 vill, ≤8s late | −2 vill, 9–20s late | −3 vill, 21–40s late | ≥−4 vill, >40s late |
| Feudal (8–14min) | ≤−2 vill, ≤15s late | −3 vill, 16–30s | −4 to −5 vill, 31–60s | ≥−6 vill, >60s |
| Castle+ (14min+) | ≤−3 vill, ≤20s | −5 vill, 21–45s | −7 vill, 46–90s | ≥−10 vill, >90s |

Overall severity = max severity across fields. Two off-pace fields don't escalate to broken, but one broken field does.

### 4.2 Smoothing and anti-flicker

Raw OCR jitters. Without smoothing the overlay strobes between green/yellow on every frame. Rules:

- Compute deltas at OCR cadence (default 500ms per `Settings.capture_interval_ms`)
- Severity transitions require **3 consecutive samples** at the new level to commit (1.5s minimum dwell)
- Numeric displays use a 2-second EMA so the "−3 vills" doesn't flicker to "−2.7 vills"
- TC-idle warning fires only after 6s continuous idle (one full vill cycle minus a beat)

### 4.3 Villager split inference algorithm

The splits panel in AoE2 is small, varies by resolution, and is often the worst OCR target. We don't try to read it directly. Instead:

1. Maintain a 10-second rolling window of resource snapshots
2. Compute per-resource gain rate `r_food`, `r_wood`, `r_gold`, `r_stone`
3. Use known per-vill gather rates (with applicable upgrade modifiers detected from `expected_techs` we *believe* are researched based on time + plan):
   - Sheep: 0.33 f/s · Boar 0.40 · Berry 0.30 (Forage-fed) · Farm ≈0.40 (long-term cap) · Deer 0.40
   - Wood: 0.39 w/s base, +15% with Double-Bit Axe, +20% with Bow Saw, +10% Two-Man Saw
   - Gold/Stone: 0.38/s base, +15% Gold Mining, +30% Gold Shaft, +15% Stone Mining
4. Solve `vills_on_X = r_X / per_vill_rate_X` for each resource
5. The unaccounted-for vills are builders + idle (we get builder estimate from "how many seconds since a non-house non-farm building was placed and is still constructing")

This is approximate (±1 vill at best, ±2 in messy phases like luring boar) but good enough for "you have 1 too many on wood" coaching. We expose the inferred numbers with their confidence so the UI can show e.g. "≈8 wood (low conf)" when the rate window is noisy.

### 4.4 Pacing the delta — game time vs wall time

`game_time_seconds` from OCR is the source of truth for delta calculation. Wall-clock time is irrelevant: the player may pause, the game may lag, single-player may be on slow speed. **All benchmarks are indexed by game-clock seconds.** This is already implicit in the existing `Trigger.time_seconds` field; we extend it as the universal x-axis.

---

## 5. Benchmark generation — authoring vs. computing

There are two ways a BO ends up with a `benchmarks[]` array:

### 5.1 Authored benchmarks (preferred for canonical BOs)

The 20–30 most popular BOs ship with hand-tuned benchmarks in `build_orders/canonical/`. Format is JSON5 (allows comments) — this is the "Hera scout rush, Arabia, Franks" file with timings sourced from his own VOD breakdowns. These get reviewed before each meta patch.

### 5.2 Computed benchmarks (for user-imported BOs)

When a user imports a BO that only has `steps[]`, the **benchmark synthesizer** runs. Its job: turn a list of "do X at Y trigger" into a per-30s expected state. Algorithm:

1. **Initialize** `t = 0`, vills = `starting_villagers`, resources = civ-starting (200F 200W 100G 200S generic, varies)
2. **Walk steps** forward by simulating:
   - Each villager produced costs 50F and 25s of TC time
   - Resource gather rates from §4.3
   - Building costs deducted; building build-time tracked (bare TC vill: 30/n+2 multiplier per builder count)
   - Tech research: cost deducted, time elapsed, tech added at completion
   - Age-up: 130s feudal, 160s castle, 190s imp (modified by civ — Persians −10%, Cumans 0% reduction but free TC, etc.)
3. **Snapshot** state every 30s into `Benchmark[]`
4. **Validate**: synthesizer flags impossible BOs ("step 8 requires 200 wood at 4:00 but you've assigned 2 vills to wood since 2:30, you'll have 117 wood")
5. **Annotate**: each `Step.at` trigger gets a computed `expected_time_seconds` that the runtime advances against

The synthesizer also runs on canonical BOs as a regression check — if Hera's hand-authored timings disagree with the simulator by >10s, that's either a sim bug or an undocumented civ bonus, both worth investigating.

---

## 6. UI — what the player actually sees

The current overlay shows `currentStep / totalSteps` and the next step card. Replace with a **3-zone overlay**:

```
┌───────────────────────────────────────────────┐
│ Hera 22 Pop Scouts · Franks · Arabia          │ ← header (existing-ish)
│ Step 14/31 · Feudal click in 0:42 (target)    │
├───────────────────────────────────────────────┤
│ ⏱  −18s behind feudal      🟡 OFF-PACE         │
│ 👥  16 vills (−2)          📈 18 expected     │ ← DELTA STRIP (new)
│ 🍖 320 (−60)  🪵 410 (+30) 🪙 0  🪨 0          │
├───────────────────────────────────────────────┤
│ NOW: Lure 2nd boar with TC                     │ ← current step (existing)
│      [villager assignment hint]                │
│ NEXT: Build mill on berries (at 14 vills)     │ ← next step (existing)
├───────────────────────────────────────────────┤
│ ⚠ TC idle 8s — queue villagers                │ ← coaching toasts (new)
│ ⚠ 1 extra vill on wood vs plan                │
└───────────────────────────────────────────────┘
```

### 6.1 Delta strip — design rules

- **Headline metric is `age_delta_seconds`** when player is in the same age as the benchmark expects. As soon as the player clicks feudal, it switches to "feudal up in T" countdown until they're up, then to "castle click delta" if it's an FC/boom BO.
- **Villager delta** always visible. Color: green if ≥0, yellow if −1 or −2, orange if −3 to −4, red if −5+.
- **Resource deltas** show only the resources that *should* be non-zero per benchmark. Don't shame the player for 0 gold at minute 5 if they're not supposed to have any.
- **One-glance principle:** total delta strip should be readable in <300ms. No prose, just numbers + color.

### 6.2 Coaching toasts

Bottom of the overlay reserves 2 lines for transient coaching messages. Triggered by:

- TC idle >6s (game time)
- Villager misallocation (>2 vills off plan on any single resource for >10s)
- Missed eco-tech window (Loom not researched 60s after benchmark says it should be; Wheelbarrow similarly)
- Floating resources (>200 of a resource that the BO says you should be spending — e.g. you've banked 600F at 8:00 and haven't clicked feudal)
- Population housed-out (pop = pop_cap and no house under construction)

Toasts auto-dismiss after 10s game time or when the underlying condition resolves. Severity of toasts mirrors §4.1 thresholds.

### 6.3 Post-step summary (deferred — v0.2)

When a step transitions, briefly flash the *delta-at-completion-of-that-step*: "Step 12 complete · −12s · −1 vill". Lets the player see where they lost time even after recovering. Out of scope for v0.1 because it requires reliable step-completion detection, which depends on the auto-advance feature that's currently a TODO.

### 6.4 Two new overlay modes

`Settings.overlay_mode`:

- `"coach"` — full delta strip + toasts (default)
- `"minimal"` — just the next step + age delta (current behavior, for high-ELO players who don't want the noise)
- `"silent"` — no overlay at all, just records the deltas to a log for post-game review

### 6.5 Color and accessibility

Severity colors must pass 4.5:1 contrast on the `--bg-primary` rgba(20,20,24,0.85). Current `--accent: #ffb84d` works for "minor". Add:

```css
--severity-on-track: #4ade80;   /* green */
--severity-minor:    #fbbf24;   /* amber */
--severity-off-pace: #fb923c;   /* orange */
--severity-broken:   #f87171;   /* red */
```

Color is never the sole signal — every severity also has a glyph (✓ / · / ▲ / ✗) for color-blind users.

---

## 7. Backend changes — Tauri side

### 7.1 New Rust commands

```rust
// existing
load_build_order_cmd(path: String) -> BuildOrder
list_build_orders_cmd() -> Vec<BuildOrderMeta>
advance_step() -> ()
previous_step() -> ()
reset_steps() -> ()
start_capture() / stop_capture()
get_settings() -> Settings

// new
get_benchmark_at(game_time_s: u32) -> Benchmark        // interpolated benchmark
get_current_delta() -> Option<Delta>                   // None if no BO loaded or no live state
synthesize_benchmarks(bo: BuildOrder, civ: String) -> Vec<Benchmark>  // for imported BOs
get_session_log() -> SessionLog                        // for post-game review
export_session(path: String) -> ()                     // dump deltas as CSV/JSON
```

### 7.2 New events emitted to frontend

```rust
// existing: "step-changed", "game-state"
// new:
"delta-updated"      // payload: Delta — fires on every capture tick (~2 Hz)
"coaching-toast"     // payload: CoachingToast — fires when a new condition triggers
"benchmark-passed"   // payload: { benchmark_index, delta_at_passing } — checkpoint events
"tc-idle"            // payload: { current_idle_s, total_idle_s } — fires at idle thresholds (3s, 6s, 10s, 20s)
```

### 7.3 New module structure (`src-tauri/src/`)

```
coach/
  mod.rs           // public API
  delta.rs         // delta computation, severity classification, smoothing
  benchmark.rs     // Benchmark struct, interpolation
  synthesizer.rs   // generate benchmarks from steps + civ
  inference.rs     // villager split inference from resource gain rates
  toasts.rs        // coaching toast rules
  session.rs       // session log, export

civdata/
  mod.rs
  bonuses.rs       // per-civ modifiers (vill rate, age-up cost, starting bonuses)
  rates.rs         // canonical gather/work rates table
```

The civ-data tables (`rates.rs`, `bonuses.rs`) are static — embed as `const` or `lazy_static!`. They change roughly once per balance patch (3–4 times a year), so a manual update is acceptable; no need for runtime fetch.

### 7.4 OCR confidence propagation

Existing capture loop returns `GameState` with optional fields. Extend each field to `(value, confidence: f32)` so the delta engine can decide whether to trust it. Confidence <0.5 → field treated as "unknown", delta hidden for that dimension, OCR-stale flag raised on the overlay.

---

## 8. Hotkeys

Add to `HotkeyConfig`:

```ts
toggle_coach_mode: string;        // cycle minimal/coach/silent
acknowledge_toast: string;        // dismiss current toast (some are persistent until ack)
mark_session_event: string;       // user-flagged moment for post-game review (e.g. "I got raided here")
```

---

## 9. Build sequence — what to build in what order

| # | Workstream | Output | Depends on |
|---|------------|--------|------------|
| 1 | **Domain types** | New `BuildOrder`, `Benchmark`, `Delta`, `LiveState` in both Rust and TS, in lockstep | nothing |
| 2 | **Civ data tables** | `rates.rs`, `bonuses.rs` covering all current DLC civs | nothing |
| 3 | **Benchmark synthesizer** | `synthesizer.rs` + tests against 5 hand-authored canonical BOs | 1, 2 |
| 4 | **Authored canonical BOs** | 8 BOs as seed: 22-pop scouts (3 civs), 23-pop archers (2 civs), Drush-FC, FC into knights, Boom | 1 |
| 5 | **Delta engine** | `delta.rs` + smoothing + severity, unit-tested with synthetic LiveState streams | 1, 3 |
| 6 | **Villager split inference** | `inference.rs` operating on rolling resource gain windows | 1, 2 |
| 7 | **Tauri command + event surface** | `get_current_delta`, `delta-updated`, etc. | 5, 6 |
| 8 | **Overlay UI v2** | New 3-zone overlay component + delta strip + toasts | 7 |
| 9 | **Coaching toast rules** | `toasts.rs` — TC idle, floating resources, missed techs, housed-out | 5, 7 |
| 10 | **Session log + export** | `session.rs` + CSV/JSON export from `get_session_log` | 5 |
| 11 | **Settings additions** | `overlay_mode`, new hotkeys, sensitivity sliders for severity thresholds | 8, 9 |
| 12 | **Post-step summary** (v0.2) | Flash delta on step transition; depends on reliable auto-advance | 8, separate auto-advance work |

Tackle 1–4 in parallel since they share types but no logic. 5–6 are the technically risky pieces — write them with property-based tests against simulated `LiveState` traces. 7–9 are wiring. 10–11 are polish. 12 ships in v0.2 once auto-advance is solid.

---

## 10. Test fixtures

Three categories of tests, all replayable:

1. **Synthetic LiveState streams** — generated programmatically: "perfect player", "5s late on every vill", "forgot loom", "got drushed at 8:00 lost 3 vills". Asserts delta engine produces expected severity transitions.
2. **Recorded OCR traces** — capture from real games, store as JSONL of `(timestamp, GameState)` tuples. Replay through delta engine offline. These double as regression fixtures when OCR changes.
3. **Canonical-BO conformance** — every shipped canonical BO is run through the synthesizer and the result is diffed against its hand-authored benchmarks. Tolerance of 5s on any single benchmark, 15s cumulative. Failures block release.

---

## 11. Open questions

- **Auto-advance vs manual advance.** When the live state matches the trigger of step N+1 (e.g. "at 14 villagers"), should the step auto-advance? Currently manual via hotkey. With reliable OCR, auto-advance is feasible but risks desync if OCR misreads. Recommend: dual-mode, default to "soft" auto-advance that pre-highlights the next step but doesn't commit until user acks or 10s elapse.
- **Multi-monitor / DPI.** Calibration profiles handle one resolution at a time. Player who switches between 1440p main and 1080p laptop needs profile switching. Out of scope for v0.1; prompt user to recalibrate.
- **Replay parsing.** Long term, parsing `.aoe2record` files would give frame-perfect ground truth and remove OCR entirely for replay-mode coaching. The mgz / recanalyst libraries exist in Python; a Rust port is a substantial side quest. Park this; it's an interesting v1.0 direction.
- **Cloud BO library.** Sharing user-authored BOs needs a backend. Out of scope now — file-based library in user data dir is fine.
- **Civ patch tracking.** When Microsoft patches civ bonuses (which they do regularly), `bonuses.rs` goes stale silently. Should the app fetch a manifest from a known URL on startup? Probably yes in v0.2; for v0.1 a version string in settings + a "your civ data is N patches behind" warning is enough.

---

## 12. Non-goals

- Coaching strategy choice ("you should have gone archers vs that civ") — this is a BO *execution* coach, not a meta coach.
- Opponent tracking — we never look at opponent OCR, even if it's on screen.
- Real-time reaction to enemy actions ("they made spears, switch off scouts") — too unreliable from screen scrape, too prescriptive.
- Mod support, custom scenarios, campaign missions — undefined behavior, app may show garbage deltas.