# Forge

Forge turns websites into desktop apps — and helps you build and share them —
without writing any code.

If you have a website, Forge's **"Turn your website into an app"** flow asks for
just two things: your web address and an app name. It then generates a complete,
build-ready [Tauri](https://tauri.app) project that opens your site in its own
desktop window. No Node.js, package manager, or framework knowledge required;
the only thing you need to produce a shareable installer is the free Tauri build
toolchain, which Forge checks for and explains in plain language.

Forge is also a full visual project manager for Tauri applications: discover
projects, group them, run builds, inspect installers, and track what's left
before publishing — all from one interface.

## Turn a website into an app

1. Open Forge and click **Turn a website into an app**.
2. Enter your website address (e.g. `yoursite.com`) and a name for your app.
3. Click **Create my app**. Forge writes the project and registers it.
4. Open the app and **Build installer** to produce something you can share.

Under the hood this generates a minimal Tauri project whose main window points
directly at your URL (see `src-tauri/src/backend/web_app.rs`). Default app icons
are included so the project builds out of the box; you can replace them later.

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

The Tauri CLI must be run from the **repository root** (the folder that
contains `src-tauri/`), not from `web/`. Install the CLI once:

```bash
cargo install tauri-cli --version "^2"
```

Then, from the repository root:

```bash
cargo tauri dev
```

This launches Forge as a desktop app and rebuilds automatically on source
changes. (`cargo tauri` automatically runs the `web/` dev server via the
`beforeDevCommand` in `src-tauri/tauri.conf.json`.)

### Production build (local)

From the repository root:

```bash
cargo tauri build
```

This builds the frontend and the Rust app and produces a desktop bundle.
Artifacts are generated under `src-tauri/target/`. Use `--no-bundle` to compile
the binary without packaging installers.

### Common launch issues

- **"Couldn't recognize the current folder as a Tauri project"**: run `cargo
  tauri` from the repository root, not from `web/` — `src-tauri/` must be a
  subfolder of your working directory.
- **`tauri: command not found`**: install the CLI with
  `cargo install tauri-cli --version "^2"`.
- **Linux WebKitGTK errors**: install your distro's WebKitGTK development
  packages and related GTK build dependencies, then retry.
- **Rust target/toolchain issues**: run `rustup update` and reopen your shell.
- **Port already in use (dev server)**: stop the existing process or set a new
  port for the frontend dev server.

## Testing & quality

Forge has frontend and backend test suites, gated in CI on every push and pull
request (`.github/workflows/ci.yml`).

Frontend (`web/`):

```bash
yarn check                 # prettier + tsc + eslint
yarn test --watchAll=false # unit tests
```

Backend (`src-tauri/`):

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

> Building the desktop backend on Linux requires the Tauri system packages
> (WebKitGTK, GTK, etc.). See the release workflow for the exact package list.

## License/tier behavior (current)

- Free tier limits project count and gates selected premium features.
- Pro/Team unlock gated feature surfaces.
- License status is persisted in `~/.forge/license.json` and app state in
  `~/.forge/forge.json`.

## Releases

Create and push a semantic tag like `v0.1.0` to trigger the cross-platform
release workflow. Releases are created as **drafts** by default.

### Auto-updater

Forge ships with the Tauri updater. Built apps check
`https://github.com/<owner>/Forge/releases/latest/download/latest.json` and can
self-update from signed releases (see **Settings → Updates**).

Updater artifacts must be signed. Generate a keypair once:

```bash
cargo tauri signer generate -w ~/.forge-updater.key
```

The **public** key goes in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).
Store the **private** key and its password as repository secrets so the release
workflow can sign updates:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Keep the private key out of source control.

## Notes

The frontend ships a custom, themeable design system (`web/src/components/ui/`)
and an application shell with sidebar navigation, onboarding, and toast
feedback. Backend file writes are atomic and spawned build/dev processes are
shut down gracefully.
