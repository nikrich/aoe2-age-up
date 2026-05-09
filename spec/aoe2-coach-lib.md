# AoE2 Coach — Seed BO Library (exact timings)

**Companion to** `aoe2-coach-spec.md`. This file contains the actual data for the 8 seed canonical BOs the spec calls for in §9 row 4. Each one is keyed to a published authoritative source and gives villager-by-villager assignments plus the derived benchmark snapshots at 30s intervals.

## What "exact" means here

Three categories of timing precision, decreasing in confidence:

1. **Villager-pop assignments** — what task the Nth villager does. Authoritative, taken directly from Hera's BO sheets and aoecompanion's transcriptions of his videos. These are the source of truth for `Step.at = { villagers: N }`.
2. **Action-time triggers** — "at 60% of feudal click", "around 13:00 mins", "research wheelbarrow ~16:00". These are coaching cues, not frame-perfect. Hera himself rounds to the nearest 30s in his guides.
3. **Benchmark resource snapshots** — derived, not authored. The synthesizer (spec §5.2) computes these from the assignments above plus the gather-rate table in §A. Hand-authored BOs ship with synthesizer output as a sanity check, not as ground truth.

The headline uptimes (feudal click, feudal up, castle click, castle up) are authored — these are the numbers Hera quotes — and the synthesizer is required to reproduce them within ±5s, otherwise the BO fails the conformance test in spec §10.

---

## A. Gather rate reference table

Used by the synthesizer to derive resource benchmarks from villager assignments. All rates are per-villager-second, generic civ, no upgrades unless noted.

| Source | Base rate | With first upgrade | Upgrade name | Cost |
|--------|-----------|---------------------|--------------|------|
| Sheep | 0.33 f/s | — | — | — |
| Boar (under TC) | 0.40 f/s | — | — | — |
| Berries | 0.31 f/s | 0.34 f/s | Horse Collar (also farm +75) | 75 W |
| Deer | 0.40 f/s | — | — | — |
| Farms (long-run cap) | 0.40 f/s | 0.46 f/s | Horse Collar | 75 W |
| Wood | 0.39 w/s | 0.45 w/s | Double-Bit Axe | 100 F, 50 W |
| Gold (open) | 0.38 g/s | 0.44 g/s | Gold Mining | 100 F, 75 W |
| Stone | 0.36 s/s | 0.41 s/s | Stone Mining | 100 F, 75 W |

**Construction:** first builder works at rate 1.0 (60 build-points/min). Each additional builder adds (3/(n+2)) of base — so 2 builders = 1.5x, 3 = 1.8x, 4 = 2.0x, asymptote ~3x.

**Walking time penalty:** synthesizer applies -8% to all gather rates as a "movement to drop site" approximation. This is the single biggest source of synthesizer-vs-real divergence; tune per BO if conformance test fails.

**TC villager production:** 25s/vill, costs 50F. Loom: 25s, 50F. Feudal research: 130s, 500F + 0W (Italians 425F). Castle research: 160s, 800F + 200G.

**Civ modifiers** (apply on top of the base rates above):
- Mongol hunters: +40% on sheep/boar/deer/berries (anything non-farm food)
- Franks: +25% on berries; cavalry +20% HP feudal+
- Mayans: resources last 15% longer (effectively +18% gather over a resource's lifetime for sheep/boar/berries/wood); start +1 vill, -50/-50/-50/-50 starting resources
- Chinese: start with 6 vills, -50W -200F starting; effectively must research Loom or build a house first
- Burgundians: eco upgrades available one age earlier (Wheelbarrow in feudal, Hand Cart in castle, Horse Collar in dark age... sort of — actually feudal techs in dark age, castle in feudal)
- Bengalis: +2 free vills on each age-up
- Italians: feudal age 425F, castle 700F+150G, imp 950F+250G (15% cheaper)
- Khmer: no buildings required to age up (saves ~25W and 27 build-seconds)
- Persians: TC works 5/10/15/20% faster per age (so vills produced in ~24s castle, ~22s imp)

---

## B. Seed BO #1: Hera 22-pop Scouts (Generic, Arabia)

**Source:** Hera Twitch Subscriber BO sheet, Aug 2020 (PDF page 6, "Scouts → Knights"); cross-referenced with aoecompanion Hera 20-vill scout entry.

**Identity:**
```
id: "hera-22pop-scouts-generic"
civilization: "Generic"
strategy: "ScoutRush"
map_type: "Arabia"
starting_villagers: 3
loom_timing: "Pre-feudal"
feudal_target_pop: 22  (21 vills + scout)
```

**Expected uptimes:**
```
feudal_click_seconds: 7:55  (475)
feudal_up_seconds:    10:05 (605)
castle_click_seconds: 14:30 (870)  — for the FC variant; pure scout rush doesn't FC, but the sheet continues
castle_up_seconds:    17:10 (1030)
```

**Villager-by-villager assignment table** (the canonical column from the Hera sheet):

| Vill # | Pop | Task | Cumulative running totals (F / W / G / S / Vills) |
|--------|-----|------|-----------|
| 1–3 | 4–6 | (starting) Sheep ×3 | 0 / 0 / 0 / 0 / 3 |
| 4 | 7 | Sheep (built first house en route) | 0 / 0 / 0 / 0 / 4 |
| 5 | 8 | Sheep (built first house en route, 2nd vill on it) | 0 / 0 / 0 / 0 / 5 |
| 6 | 9 | Sheep (6th shepherd) | sheep crew complete |
| 7 | 10 | Wood — build Lumber Camp #1 | LC#1 placed |
| 8 | 11 | Wood | 2 on wood |
| 9 | 12 | Wood | 3 on wood |
| 10 | 13 | Lure Boar #1 | boar 1 in progress |
| 11 | 14 | Sheep / boar consolidation | boar 1 dead → 6 on food crew |
| 12 | 15 | House + Mill on Berries | mill placed at berries |
| 13 | 16 | Berries | 1 on berries |
| 14 | 17 | Berries | 2 on berries |
| 15 | 18 | Lure Boar #2 | boar 2 in progress |
| 16 | 19 | Berries | 3 on berries |
| 17 | 20 | Boar (under TC eat) | |
| 18 | 21 | Wood (back to LC#1) | 4 on wood |
| 19 | 22 | Wood | 5 on wood |
| 20 | 23 | Wood — build Lumber Camp #2 | LC#2 placed |
| 21 | 24 | **Loom** + click Feudal | Click @ 7:55, 14W banked |
| — | — | Move 3 from sheep → LC#2 | post-click reshuffle |
| (during feudal up) | | Build Barracks at 60% feudal-up | |
| (during feudal up) | | Pre-place Stable foundation | |

**Step list (the human-readable version, drives the overlay):**

```jsonc
[
  { "action": "Send 6 villagers to sheep, build 2 houses",
    "at": { "villagers": 3, "mode": "All" },
    "villagers_assigned": { "food": 6, "wood": 0, "gold": 0, "stone": 0, "idle": 0 } },
  { "action": "7th villager: build Lumber Camp #1",
    "at": { "villagers": 7, "mode": "All" } },
  { "action": "8th–9th villagers: wood",
    "at": { "villagers": 8, "mode": "All" } },
  { "action": "10th villager: lure 1st boar",
    "at": { "villagers": 10, "mode": "All" },
    "notes": "Lure with first vill that gets to TC; replace boar bait every ~10s" },
  { "action": "11th villager: sheep (boar arriving)",
    "at": { "villagers": 11, "mode": "All" } },
  { "action": "12th villager: build house + Mill on berries",
    "at": { "villagers": 12, "mode": "All" } },
  { "action": "13th–14th villagers: berries",
    "at": { "villagers": 13, "mode": "All" } },
  { "action": "15th villager: lure 2nd boar",
    "at": { "villagers": 15, "mode": "All" },
    "notes": "Critical — late 2nd boar = late feudal" },
  { "action": "16th villager: berries (4 total)",
    "at": { "villagers": 16, "mode": "All" } },
  { "action": "17th villager: boar under TC",
    "at": { "villagers": 17, "mode": "All" } },
  { "action": "18th–19th villagers: wood (back to LC#1)",
    "at": { "villagers": 18, "mode": "All" } },
  { "action": "20th villager: build Lumber Camp #2",
    "at": { "villagers": 20, "mode": "All" } },
  { "action": "Research Loom + click Feudal Age",
    "at": { "villagers": 21, "food_min": 500, "mode": "All" },
    "notes": "Target time: 7:55. Bank 14W for the LC#2 + future barracks" },
  { "action": "Move 3 from sheep to LC#2 (10 wood total)",
    "at": { "time_seconds": 480, "mode": "Any" } },
  { "action": "At 60% feudal: build Barracks (1 vill from sheep)",
    "at": { "time_seconds": 555, "mode": "Any" },
    "notes": "60% of 130s research = 78s after click → 7:55 + 1:18 = 9:13" },
  { "action": "On feudal up: build Stable with 2 vills, research Horse Collar + Double-Bit Axe",
    "at": { "time_seconds": 605, "mode": "Any" } },
  { "action": "Make scouts continuously, seed farms as wood permits",
    "at": { "time_seconds": 605, "mode": "Any" } },
  { "action": "~13:00: send 4 vills to gold (build mining camp)",
    "at": { "time_seconds": 780, "mode": "Any" } },
  { "action": "~14:30: click Castle Age",
    "at": { "time_seconds": 870, "food_min": 800, "gold_min": 200, "mode": "All" } }
]
```

**Hand-authored benchmark snapshots** (every 60s — the synthesizer fills in the 30s gaps):

| Time | Vills | Pop | F | W | G | S | Age | Idle TC? | Primary focus |
|------|-------|-----|---|---|---|---|-----|----------|---------------|
| 1:00 | 5 | 6 | 80 | 0 | 0 | 0 | Dark | should be no | Build 1st house, 5 on sheep |
| 2:00 | 7 | 8 | 180 | 18 | 0 | 0 | Dark | no | LC#1 going up, vill 7 placing it |
| 3:00 | 9 | 10 | 230 | 70 | 0 | 0 | Dark | no | Boar #1 lure starts at vill 10 |
| 4:00 | 11 | 12 | 360 | 110 | 0 | 0 | Dark | no | Mill on berries (vill 12) |
| 5:00 | 13 | 14 | 420 | 145 | 0 | 0 | Dark | no | 2 on berries, 3 on wood |
| 6:00 | 15 | 16 | 470 | 175 | 0 | 0 | Dark | no | Boar #2 lure |
| 7:00 | 18 | 19 | 510 | 240 | 0 | 0 | Dark | no | Last vills to wood, prep LC#2 |
| 7:55 | 21 | 22 | 500 | 14 | 0 | 0 | Dark→F | clicking | Loom + Feudal click |
| 9:00 | 21 | 22 | 110 | 105 | 0 | 0 | aging | no | Barracks placed at 60% |
| 10:05 | 21 | 22 | 80 | 60 | 0 | 0 | Feudal | resume | Stable + scouts begin |
| 11:00 | 23 | 24 | 90 | 90 | 0 | 0 | Feudal | no | First scouts out, 5 farms seeded |
| 12:00 | 25 | 26 | 110 | 70 | 0 | 0 | Feudal | no | Continuous scouts + farms |
| 13:00 | 26 | 27 | 130 | 110 | 25 | 0 | Feudal | no | Mining camp going up |
| 14:00 | 28 | 29 | 350 | 130 | 110 | 0 | Feudal | no | Banking for castle click |
| 14:30 | 28 | 29 | 800 | 80 | 200 | 0 | F→C | clicking | Castle click |

The 7:55 / 14:30 / 17:10 numbers are the conformance pin-points. Synthesizer must hit them within 5s.

---

## C. Seed BO #2: 23-pop Archers (Generic, Arabia)

**Source:** aoe2.guide/archer-build-order ("created by Hera"), confirmed against ageofnotes 23-pop archers and aoecompanion archer-rush.

**Identity:**
```
id: "hera-23pop-archers-generic"
civilization: "Generic"
strategy: "ArcherRush"
map_type: "Arabia"
starting_villagers: 3
loom_timing: "Pre-feudal"
feudal_target_pop: 23  (22 vills + scout)
```

**Expected uptimes:**
```
feudal_click_seconds: 8:25  (505)   — 30s later than scouts because +1 vill of build time
feudal_up_seconds:    10:35 (635)
castle_click_seconds: 17:00 (1020)  (Hera quotes ~17min)
castle_up_seconds:    19:40 (1180)
```

**Villager assignment table:**

| Vill # | Pop | Task |
|--------|-----|------|
| 1–6 | 4–6 (starting +3) | Sheep, 2 houses built en route |
| 7–10 | 7–10 | Wood (LC#1 placed at vill 7) |
| 11 | 11 | Lure boar #1 |
| 12 | 12 | House + mill on berries |
| 13–14 | 13–14 | Berries |
| 15 | 15 | Lure boar #2 |
| 16 | 16 | Berries (4 on berries total) |
| 17–18 | 17–18 | Boar/sheep (food under TC) |
| 19 | 19 | Wood |
| 20–21 | 20–21 | Wood — LC#2 placed at vill 20 |
| 22 | 22 | **Loom** + click Feudal |
| (post-click) | | Move 3 vills from TC to LC#2 |
| (post-click) | | Move 3 vills from sheep → mining camp on gold |
| (during up) | | Build Barracks at 60% feudal |

**Key differentiator from scouts:** the 22nd vill goes wood not lumber-camp-2-builder, and you send 3 vills to gold *during the feudal click* not after up. Archers need 50W+45G/archer, so you need gold flowing the moment feudal hits.

**Castle plan:** keep producing archers + vills, research Fletching, build Blacksmith, hit ~13–14 farms, research Wheelbarrow ~16:00, +1 more on gold, click castle ~17:00.

**Step list:** identical structure to scouts up to vill 19; diverges at:
- Vill 20: wood (not LC#2 builder yet)
- Vill 21: wood (build LC#2)
- Vill 22: Loom + click feudal
- Post-click: 3 vills to mining camp on gold (instead of 3 from sheep to LC#2)
- On feudal up: build 2 archery ranges + blacksmith (not stable + barracks)

---

## D. Seed BO #3: Hera 17-pop Mongol Scouts (Mongols, Arabia)

**Source:** Hera 22-pop scouts sheet, footnote variant; and 2023 BO sheet that replaces this with the 17-pop variant for elite play. Mongol hunter bonus (+40% on hunt) makes earlier feudal viable.

**Identity:**
```
id: "hera-17pop-scouts-mongols"
civilization: "Mongols"
strategy: "ScoutRush"
map_type: "Arabia"
starting_villagers: 3
loom_timing: "Skip"     // controversial; some skip loom for 5s, some keep it
feudal_target_pop: 18   (17 vills + scout)
```

**Expected uptimes:**
```
feudal_click_seconds: 6:35  (395)   — ~80s faster than generic
feudal_up_seconds:    8:45  (525)
```

**Villager assignment:** same shape as 22-pop but truncated; the mongol sheet has 6 sheep / 3 wood / 1 boar lure / 2 boar / 2 berries / 1 boar2 / 1 wood / **click feudal at 17 vills**. The bonus is that hunters work 40% faster, so 8 vills hunting + 2 berries get the food for 17 vills + feudal in time.

**Why this matters for the coach:** the synthesizer must apply the Mongol modifier to all hunt-source rates. If a player loads this BO but accidentally selects a non-Mongol civ, the synthesizer should refuse to load it ("This BO requires Mongols; want the Generic 22-pop scouts instead?"). The validator in spec §5.2 catches this.

---

## E. Seed BO #4: 21-pop Drush → Fast Castle (Generic, Arabia)

**Source:** Hera Drush FC sheet (page 8 of the 2020 sheet); Windows Central Drush guide; Steam Community BO sheets PDF from Hibou.

**Identity:**
```
id: "drush-fc-generic"
civilization: "Generic"
strategy: "DrushFC"
map_type: "Arabia"
feudal_target_pop: 21
castle_target_pop: 30
```

**Expected uptimes:**
```
feudal_click_seconds: 8:00  (480)
feudal_up_seconds:    10:10 (610)
castle_click_seconds: 14:30 (870)
castle_up_seconds:    17:10 (1030)
```

**Distinguishing features for the coach:**
- Barracks placed at vill 18–19 in *Dark Age* (unusual — that's the "drush" part). Coach should expect a barracks building between 6:30 and 7:30.
- 3 militia produced before feudal click. Production from barracks consumes 60F per militia, so the food benchmark dips ~180F at click.
- Castle click is on 30 pop (3 extra vills made in feudal during age-up) — this is the "FC" half.

**Why this is in the seed set:** it exercises the spec's "expected military" benchmark column (most BOs have empty military through dark age; drush has 3 militia and the coach should call out "you should have 3 militia by 8:00, you have 0").

---

## F. Seed BO #5: 27-pop Fast Castle Boom (Generic, Arena)

**Source:** Hera "BEST Beginner Build Order in AoE2 (Fast Castle Boom)" YouTube guide (linked from Quora answer); ageofnotes 27+2 FC boom; chosengambit FC walkthrough.

**Identity:**
```
id: "fc-boom-27pop-generic"
civilization: "Generic"
strategy: "Boom"
map_type: "Arena"
feudal_target_pop: 27   (26 vills + scout)
castle_target_pop: 30   (29 vills + scout, made during feudal)
```

**Expected uptimes:**
```
feudal_click_seconds: 11:05 (665)
feudal_up_seconds:    13:15 (795)
castle_click_seconds: 14:45 (885)
castle_up_seconds:    17:25 (1045)
```

**Villager assignment table** (Quora's example layout matches Hera's sheet):

| Vill # | Pop | Task |
|--------|-----|------|
| 1–6 | 4–6 | Sheep (start +3) |
| 7 | 7 | House → Wood |
| 8–11 | 8–11 | Wood (LC#1 at vill 8) |
| 12 | 12 | Lure boar #1 |
| 13–14 | 13–14 | Sheep / boar |
| 15 | 15 | Mill on berries |
| 16 | 16 | Lure boar #2 |
| 17 | 17 | Boar #2 collection |
| 18 | 18 | Berries |
| 19 | 19 | Wood (LC#2 placed) |
| 20 | 20 | Wood |
| 21 | 21 | Mining camp on gold |
| 22 | 22 | Gold |
| 23 | 23 | Gold (3 on gold) |
| 24 | 24 | Wood / straggler |
| 25–26 | 25–26 | Food (push to ~12 on food) |
| 27 | 27 | Build market or blacksmith, **click Feudal** |

Post-click: more villagers to gold during feudal age-up to bank 200G for castle. Build Market + Blacksmith during feudal (the two prereq buildings for castle). Click castle as soon as 800F + 200G banked.

**Key benchmark difference:** unlike rush BOs, FC has the player banking food/gold during feudal age-up. The benchmark snapshot at 12:00 (mid-feudal-research) should expect ~600F and rising — flat resources here = late castle click.

---

## G. Seed BO #6: 18-pop Japanese Man-at-Arms (Japanese, Arabia)

**Source:** Hera 2023-06 sheet, page on "18 pop Japanese MAA"; cross-checked against the 2023-11 sheet.

**Identity:**
```
id: "hera-18pop-maa-japanese"
civilization: "Japanese"
strategy: "MAA"
map_type: "Arabia"
feudal_target_pop: 18
```

**Expected uptimes:**
```
feudal_click_seconds: 7:00  (420)  — Japanese cheap eco buildings save wood, faster lumberjack-camp build
feudal_up_seconds:    9:10  (550)
```

**Why Japanese:** Japanese have 50% cheaper Mill / Lumber Camp / Mining Camp + faster fishing. The eco-building discount is irrelevant for MAA timing per se, but the player typically lays a 2nd LC earlier, which compounds. MAA itself is a generic infantry but Japanese ones are slightly faster.

**Distinguishing benchmark column:** military — should have 3 MAA on screen by 9:30, 5 by 10:30, transitioning into archery range.

---

## H. Seed BO #7: 19-pop MAA → Archers (Generic, Arabia)

**Source:** aoecompanion's Hera-derived MAA-archers transcription; Cicero's BO reference (linked from the Steam BO sheets PDF); Hera 2023 sheet.

**Identity:**
```
id: "maa-archers-generic"
civilization: "Generic"
strategy: "MAAArchers"
feudal_target_pop: 23   (3 MAA produced means 3 vills idled equiv)
```

**Expected uptimes:**
```
feudal_click_seconds: 7:30  (450)   (the MAA delays things ~5–8s vs straight archers)
feudal_up_seconds:    9:40  (580)
castle_click_seconds: 17:30 (1050)
```

**Critical coaching cue:** at 9:00–9:30 the player should be sending the 3 MAA across the map. If the coach detects MAA still in own base at 10:00 game time → fire toast: "MAA should be moving forward".

---

## I. Seed BO #8: 19-pop Korean Skirm-Spear Defense (Koreans, Arabia)

**Source:** Hera 2023-11 sheet, page on "19 Pop Korean Spear Skirm Rush". Recommended civs from sheet: Koreans, Byzantines, Lithuanians.

**Identity:**
```
id: "hera-19pop-skirmspear-koreans"
civilization: "Koreans"
strategy: "ArcherRush"  // really "trash rush" but no separate tag
feudal_target_pop: 19
```

**Why this is in the seed set:** it's the only BO that goes archery range + barracks for *trash units* (skirms + spears) — no gold needed. The coach must distinguish "expected gold = 0" (correct) from "you forgot to send vills to gold" (wrong). The benchmark column for this BO has `expected_gold = 0` through to ~17:00 castle click, which is unusual.

---

## J. What's *not* in the seed set, and why

A v0.1 ship with 8 BOs is the right floor. Things deliberately excluded:

- **Civ-specific scout variants** (Franks, Lithuanians, Magyars). The synthesizer can produce these from the generic 22-pop scouts plus civ modifiers — separate authored BOs would be redundant. Validate with a synthesizer-vs-Hera-sheet diff.
- **Tower rush, donjon drop, dark age trush.** Niche, high-skill, lots of building micro that the OCR pipeline can't observe well. Add later.
- **Naval / hybrid BOs.** Capture pipeline doesn't currently OCR fishing-ship counts. Out of scope until OCR is extended.
- **Imperial-age push templates.** The coach is dark-age-through-castle-click. Once you're in castle age building knights, you're past the "execute the BO" phase.
- **Nomad / Empire Wars / Treaty.** Different starting conditions; different builds. Future work.

---

## K. Civ-bonus modifier table (for the synthesizer)

This is the data file the synthesizer reads to translate "generic 22-pop scouts" into "Franks 22-pop scouts" etc. Format is a list of override hooks. Showing 6 representative civs; the full table needs all 50.

| Civ | Modifier | Effect on synthesizer |
|-----|----------|------------------------|
| Mongols | `hunt_rate_mult: 1.40` | Apply to sheep / boar / deer / berries gather rates |
| Franks | `berry_rate_mult: 1.25`, `tc_garrison_heal: false` | Berries 25% faster; foragers benchmark accordingly |
| Mayans | `start_villagers: 4`, `start_food_offset: -50`, `resource_durability: 1.15` | Extra vill, less starting food, resources last longer (matters for sheep depletion timing) |
| Chinese | `start_villagers: 6`, `start_food_offset: -200`, `start_wood_offset: -50` | Forces loom-first or house-first opening |
| Burgundians | `tech_age_shift: -1` | Wheelbarrow available in feudal, Hand Cart in castle, Horse Collar / Double-Bit Axe in dark age |
| Bengalis | `feudal_up_bonus_vills: 2`, `castle_up_bonus_vills: 2`, `imperial_up_bonus_vills: 2` | Vill count jumps +2 the moment age-up completes |
| Italians | `age_up_food_mult: 0.85` | Feudal 425F, castle 680F + 170G, imp 808F+213G |
| Khmer | `age_up_skip_buildings: true` | No 2 buildings required for feudal click |
| Persians | `tc_work_rate_per_age: [1.0, 1.05, 1.10, 1.15]` | Vills produced faster each age |
| Vikings | `wheelbarrow_cost: 0`, `handcart_cost: 0` | Free eco upgrades; benchmark applies them at the moment they'd otherwise be researched |

---

## L. Concrete next deliverable

If the implementer asks "where do I start?", the answer is: write the synthesizer (`src-tauri/src/coach/synthesizer.rs`), feed it BO #1 (Hera 22-pop scouts, generic), and assert it produces the benchmark snapshots in §B's table within ±5 seconds. That's the single test that validates the whole spec.

Once that passes, the other 7 BOs are data-entry. Once those 7 pass, the civ-modifier table in §K becomes implementable as overlays on the 8 generic templates, and the BO library expands by composition rather than authoring.