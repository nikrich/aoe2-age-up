# Recommended Claude Code Skills for Age Up

A curated list of skills to install before handing off the spec, ordered by impact for this specific project.

---

## Tier 1 — Install these first

These give you the biggest leverage for the AoE overlay specifically.

### 1. obra/superpowers
**Repo:** https://github.com/obra/superpowers
**Install:**
```bash
/plugin marketplace add obra/superpowers-marketplace
/plugin install superpowers@superpowers-marketplace
```

A 20+ skill framework that enforces planning, TDD (RED-GREEN-REFACTOR), and systematic debugging. The killer features for your project:

- **`/superpowers:brainstorm`** — refines requirements before any code. Useful at the start of each phase from the spec to pressure-test assumptions (e.g., "should we really start with template matching or Tesseract?").
- **`/superpowers:write-plan`** — turns the spec phases into detailed, file-level implementation plans with verification commands. This is the natural complement to your spec — feed it Phase 1, get back a 200+ line concrete plan.
- **`/superpowers:execute-plan`** — runs plans in batches with code review checkpoints between tasks. Lets Claude Code work autonomously for hours without drifting from the plan.
- **systematic-debugging** — 4-phase root cause process. Critical for OCR debugging when reads are flaky and you'd otherwise just tweak the threshold randomly.
- **verification-before-completion** — forces actual verification that things work, not just "compiles cleanly."

This is a force multiplier across the whole project. Install first.

### 2. dchuk/claude-code-tauri-skills
**Repo:** https://github.com/dchuk/claude-code-tauri-skills
**Install (manual):**
```bash
git clone https://github.com/dchuk/claude-code-tauri-skills.git ~/.claude/skills/tauri
```

**39 skills covering Tauri v2 specifically** — setup, security, IPC, distribution, code signing, mobile. Directly maps to almost every section of your spec:

- Tauri command/event patterns (your §8.4 IPC table)
- Window configuration including transparent/always-on-top (your §8.1)
- Calling Rust from the frontend
- MSI/NSIS bundling (your §12)
- Code signing setup (your §12.3 risk)

This is the single highest-signal skill for the project. The official Anthropic Tauri skill exists but is less comprehensive.

### 3. leonardomso/rust-skills
**Repo:** https://github.com/leonardomso/rust-skills
**Install:**
```bash
git clone https://github.com/leonardomso/rust-skills.git ~/.claude/skills/rust-skills
```

179 Rust rules across type safety, testing (proptest, mockall, criterion), performance, project structure, and anti-patterns. Each rule has bad/good code examples plus links to official docs.

Especially relevant for your project:
- **Performance rules** — iterators, entry API, collect patterns (matters for the capture loop hot path).
- **Newtypes / parse don't validate** — fits the `GameState`/`Region`/`Trigger` type design in §3.
- **Anti-patterns** — catches `Arc<Mutex>` overuse and other common Rust traps you'll hit in the capture/state code.

---

## Tier 2 — Install for specific phases

### 4. Rust Testing & rstest Manager
**Source:** https://mcpmarket.com/tools/skills/rust-testing-rstest-manager

Enforces `rstest` patterns (parametrized tests + fixtures) and uses `cargo-llvm-cov` for coverage. Lines up directly with your §13.1 test plan — trigger evaluation across all condition combinations is a perfect fit for parametrized tests.

### 5. Anthropic webapp-testing (Playwright)
**Source:** https://github.com/anthropics/skills

Only useful once you have the calibration UI and build order editor in Phase 2/5. Tests Tauri webview UIs in the same way it tests web apps — Playwright works against the Tauri webview.

### 6. Anthropic frontend-design
**Source:** https://github.com/anthropics/skills

For the calibration wizard, build order editor, and overlay styling (§8.3 visual design). Avoids generic AI-looking UIs, which is exactly the risk for an overlay where polish matters because users see it constantly.

---

## Tier 3 — Skip these despite the obvious match

### OCR skills (Code From Image, LlamaFarm OCR)
The OCR skills available are oriented toward general document/code OCR — Tesseract/PaddleOCR/Surya pipelines on photos and PDFs. **Your project needs specialized OCR for a known fixed font in known regions** — those general skills will pull you toward heavyweight backends and away from the template-matching approach your spec correctly chose. Skip them.

### Tauri App Consultant / Tauri Setup
Both overlap heavily with `dchuk/claude-code-tauri-skills` and offer less depth. The dchuk repo's 39 skills supersede these.

---

## Recommended setup sequence

```bash
# 1. Superpowers (the core workflow framework)
/plugin marketplace add obra/superpowers-marketplace
/plugin install superpowers@superpowers-marketplace

# 2. Tauri-specific knowledge
git clone https://github.com/dchuk/claude-code-tauri-skills.git \
  ~/.claude/skills/tauri

# 3. Rust idioms and patterns
git clone https://github.com/leonardomso/rust-skills.git \
  ~/.claude/skills/rust-skills

# 4. Testing patterns (install after Phase 1 scaffolding)
# See https://mcpmarket.com/tools/skills/rust-testing-rstest-manager
```

---

## Recommended workflow with the spec

1. Drop the spec into the project root as `SPEC.md`.
2. Start a Claude Code session in the project directory.
3. Run `/superpowers:brainstorm "Phase 1 of SPEC.md — Tauri skeleton with manual-advance overlay"` to pressure-test the approach.
4. Run `/superpowers:write-plan "Implement Phase 1 of SPEC.md"` to get a file-level plan.
5. Run `/superpowers:execute-plan` and let it work through Phase 1 with TDD enforced.
6. At Phase 1 milestone, repeat from step 3 for Phase 2.

The Tauri and Rust skills will auto-activate based on file context — no need to invoke them explicitly. Superpowers' systematic-debugging skill will kick in when something breaks, which is especially valuable during the OCR phase where the failure modes are subtle (off-by-one region offsets, threshold values, character segmentation edge cases).

---

## A note on skill verification

Skills can execute arbitrary code. Before installing community skills (anything in Tier 1 or Tier 3), spot-check the `SKILL.md` files and any bundled scripts. The repos linked above are all reputable but do verify before adding to `~/.claude/skills/`.