# Forge Post-Fork Implementation Plan

This repository is the Forge desktop app: a Tauri v2 (Rust) backend in `src-tauri/`
and a React + TypeScript frontend in `web/`. The upstream Tilt Go codebase has been
removed; this document is retained as the **execution order + roadmap** for the
remaining phases.

## Repository Readiness Gate (before Phase 0) — satisfied

- [x] Repo root contains `src-tauri/`, `web/`, and the app configuration.
- [x] `src-tauri/tauri.conf.json` exists and is configured for Forge.
- [x] Fork origin confirmed; MIT attribution preserved in `LICENSE`.
- [x] Branch naming convention established for rollout.

## Phase 0 — Rebrand (Day 1)

Goal: rename Tilt identity to Forge without behavior changes.

### Core file updates
- `package.json`: name = `forge`, update description.
- `src-tauri/tauri.conf.json`: `productName`, bundle identifier, and window title = Forge.
- `src-tauri/Cargo.toml`: package metadata rename.
- Rewrite `README.md` and reset `CHANGELOG.md` with fork provenance.
- Replace logos/icons in `public/` and app icon assets.
- Remove stale screenshots.

### String replacement sweep
- `forge` → `forge`
- `Forge` → `Forge`
- `forge` → `forge`

### Exit criteria
- [ ] `npm install` passes.
- [ ] `npm run tauri dev` launches with Forge name.

## Phase 1 — Remove Tilt-Domain Logic (Days 1–2)

Goal: keep reusable app shell patterns, remove Tilt/Kubernetes service concepts.

### Backend
- Delete Tilt-only modules:
  - `backend/generator.rs`
  - `backend/dependency_graph.rs`
  - `backend/ports.rs`
  - `backend/tilt_manager.rs` (replaced)
- Add/replace with Forge core:
  - `backend/process_manager.rs`
  - `backend/config_manager.rs`
  - `backend/build_manager.rs`
  - `backend/environment_checker.rs`
- Refactor `backend/ipc.rs` command surface to Forge commands.
- Update `backend/project_manager.rs` for Tauri detection/scanning/workspaces.
- Update state and persistence (`app_state/model.rs`, `app_state/store.rs`) to `.forge` schema.

### Frontend
- Remove service-centric components (ServiceCard/AddServiceDialog/TiltControls).
- Rewrite landing, create/open flows, project dashboard, and settings around Tauri builds.
- Add config/build/terminal/environment/workspace/deploy components.
- Replace IPC client APIs and TypeScript models with Forge equivalents.

### Exit criteria
- [ ] App compiles with no Tilt command handlers/components referenced.
- [ ] App opens and basic navigation works with new state model.

## Phase 2 — Free Tier Features (Days 3–7)

Primary deliverables:
- project registration + Tauri detection
- environment checker UI
- config read/write/validate/editor
- process execution for dev/build
- terminal output streaming
- artifact collection/listing
- create project/init existing project workflows

### Exit criteria
- [ ] User can register/open project.
- [ ] User can edit `tauri.conf.json` safely.
- [ ] User can run dev/build and view logs/artifacts.

## Phase 3 — Pro Features (Days 8–12)

Primary deliverables:
- workspaces
- build presets and orchestrated workspace builds
- persisted build history/artifact browser
- deployment readiness dashboard
- iOS/Android build support

### Exit criteria
- [ ] Workspace orchestration works end-to-end.
- [ ] Build history + re-run works.
- [ ] Mobile build paths wired and preflight checks implemented.

## Phase 4 — Launch (Days 13–16)

Primary deliverables:
- license gating
- updater pipeline + release automation
- landing page/docs/demo assets
- final visual polish and public launch checklist

### Exit criteria
- [ ] Free tier binaries released.
- [ ] Pro gating validated with online/offline behavior.
- [ ] Auto-update flow tested with version bump.

## Risk Register (recommended)

1. **Wrong baseline repo risk** (highest): verify Tauri app fork first.
2. **Scope risk**: enforce phase gates and avoid premature Pro work.
3. **Cross-platform build risk**: continuously validate Linux/macOS/Windows behavior.
4. **Command-safety risk**: sandbox and validate all shell-command wrappers.
5. **Config integrity risk**: backup + diff preview + validation on every config write.

## Suggested branch/release cadence

- `forge/phase-0-rebrand`
- `forge/phase-1-shell`
- `forge/phase-2-free-core`
- `forge/phase-3-pro`
- `forge/phase-4-launch`

Tag after each phase gate (e.g. `forge-v0.1.0-phase0`).
