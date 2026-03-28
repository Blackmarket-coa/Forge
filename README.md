# Forge

Forge is a desktop-first visual project manager for **Tauri** applications.
It helps you discover projects, manage workspaces, run builds, inspect artifacts,
and manage release readiness from one interface.

## Current status

Forge is under active development. The repository currently includes:

- A **Tauri v2 backend** (`src-tauri/`) with IPC commands for project discovery,
  config read/write/validation, process execution, build orchestration, deploy
  readiness checks, and local state persistence.
- A **React frontend** (`web/src/`) for project browsing, workspace views,
  build orchestration, deploy dashboard, config editing, and settings.
- Commercial tier plumbing (Free/Pro/Team) with local license cache and Keygen
  validation hooks.
- A tag-driven GitHub Actions release workflow for macOS, Linux, and Windows.

## Repository layout

- `src-tauri/` — Rust backend (Tauri app, IPC handlers, state, license checks)
- `web/` — React UI and frontend API wrappers
- `.github/workflows/release.yml` — draft release pipeline on `v*` tags

## Local development

### Prerequisites

- Rust toolchain (stable)
- Node.js 20+
- Tauri dependencies for your platform

### Run frontend only

```bash
cd web
npm install
npm run dev
```

### Run Tauri app

```bash
cd web
npm install
npm run tauri dev
```

## License/tier behavior (current)

- Free tier limits project count and gates selected premium features.
- Pro/Team unlock gated feature surfaces.
- License status is persisted in `~/.forge/license.json` and app state in
  `~/.forge/forge.json`.

## Releases

Create and push a semantic tag like `v0.1.0` to trigger the cross-platform
release workflow. Releases are created as **drafts** by default.

## Notes

This README reflects the current scaffold and will evolve as Forge moves from
baseline implementation to production-ready behavior.
