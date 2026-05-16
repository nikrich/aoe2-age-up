# Age Up — Claude Code Project Guide

## What is this?

AoE2 build order overlay built with Tauri 2.0 (Rust backend) + React/TypeScript frontend.

## Commands

- `cargo tauri dev` — Run in development mode
- `cd src-tauri && cargo test` — Run all Rust tests (16 unit + 2 integration)
- `npm run build` — Build frontend only
- `cargo tauri build` — Build release installer

## Architecture

- **Rust backend** (`src-tauri/src/`): state management, build order parsing/engine, IPC commands, global hotkeys, file storage
- **React frontend** (`src/`): overlay UI, build order library, settings display
- **Build orders** (`build-orders/`): YAML files defining step-by-step strategies

## Key Patterns

- All shared state behind `Arc<Mutex<AppState>>` managed by Tauri
- IPC: frontend calls Tauri commands, backend emits events (`step-changed`, `game-state`)
- Build order engine uses trigger evaluation with AND/OR modes
- Global hotkeys via `global-hotkey` crate, registered on startup

## Development Phases

- Phase 1 (complete): Manual-advance overlay with build order library
- Phase 2 (next): Windows.Graphics.Capture screen capture + calibration UI
- Phase 3: OCR pipeline (template matching for known AoE2 font)
- Phase 4: Auto-advance wiring (trigger evaluation against live game state)
- Phase 5: Build order editor, full settings UI, installer polish

## Workflow

- All feature work happens on feature branches (e.g., `feat/phase-2-capture`)
- Submit PRs to `main` when work is complete
- Never push directly to `main`

## Conventions

- Commits use conventional commits: `feat:`, `fix:`, `docs:`, `ci:`
- Rust tests use inline `#[cfg(test)]` modules
- Integration tests in `src-tauri/tests/`
- Frontend types mirror Rust structs exactly (see `src/lib/types.ts`)
