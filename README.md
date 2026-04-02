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

Install these once before launching Forge:

- **Rust (stable)** via `rustup`
- **Node.js 20+**
- **Tauri system dependencies** for your OS (WebKitGTK on Linux, Xcode CLT on
  macOS, WebView2 on Windows)

Quick verification commands:

```bash
rustc --version
cargo --version
node --version
```

### First-time setup

Forge's frontend lives in `web/`, so install dependencies there first.

#### Option A (recommended): Yarn via Corepack

```bash
cd web
corepack enable
yarn install
```

#### Option B: npm

```bash
cd web
npm install
```

### Launch modes

#### 1) Frontend-only development (fast UI iteration)

```bash
cd web
npm run dev
```

This starts the React dev server without launching the desktop shell.

#### 2) Full desktop app (Tauri + frontend)

```bash
cd web
npm run tauri dev
```

This launches Forge as a desktop app and rebuilds automatically on source
changes.

### Production build (local)

From `web/`:

```bash
npm run build
npm run tauri build
```

Artifacts are generated under `src-tauri/target/`.

### Common launch issues

- **`tauri: command not found`**: install JS dependencies in `web/` and run via
  package scripts (`npm run tauri ...`) instead of a global binary.
- **Linux WebKitGTK errors**: install your distro's WebKitGTK development
  packages and related GTK build dependencies, then retry.
- **Rust target/toolchain issues**: run `rustup update` and reopen your shell.
- **Port already in use (dev server)**: stop the existing process or set a new
  port for the frontend dev server.

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
