# Forge

Forge turns websites into desktop, mobile, and web apps — and helps you build
and share them — without writing any code.

If you have a website, Forge's **"Turn your website into an app"** flow asks
for your web address, an app name, and where the app should run. It then
generates a complete, build-ready project for every kind of app you picked —
all wrapping the same website, so updating your site updates every app. No
Node.js, package manager, or framework knowledge is required at generation
time; each app type's free build tools are only needed to produce installers,
and Forge checks for them and explains anything missing in plain language.

### Supported app types

| App type | Framework | Runs on | Build tools needed |
|---|---|---|---|
| Desktop app (small & fast) | [Tauri](https://tauri.app) | Windows, macOS, Linux | Rust + Tauri CLI |
| Desktop app (classic web stack) | [Electron](https://www.electronjs.org) | Windows, macOS, Linux | Node.js |
| iPhone & Android app | [Capacitor](https://capacitorjs.com) | iOS, Android | Node.js + Android SDK / Xcode |
| Mobile app (Expo preview) | [React Native](https://reactnative.dev) + [Expo](https://expo.dev) | iOS, Android | Node.js (+ Android SDK / Xcode / EAS for store builds) |
| Install from the browser | PWA | Any modern browser | none — upload a kit to your site |

One project can hold several app types (a *multi-target* project): start with a
desktop app and add a mobile one later with **Add another app type**. Forge is
also a full visual project manager for these projects: discover them on disk,
group them, run previews and builds with live logs, inspect installers, edit
each app's settings safely, and track what's left before publishing — all from
one interface.

## Turn a website into an app

1. Open Forge and click **Turn a website into an app**.
2. Enter your website address (e.g. `yoursite.com`) and a name for your app.
3. Pick where your app should run — desktop, mobile, and/or the browser.
4. Click **Create my app**. Forge writes the project and registers it.
5. Open the app and **Build installer** to produce something you can share
   (each app type lists its own installer formats and required tools).

Under the hood this generates one project folder with a target per framework
(`src-tauri/`, `capacitor/`, `electron/`, `pwa/`, `react-native/`), described
by a `forge.project.json` manifest. Each target wraps your URL directly — see
`src-tauri/src/backend/frameworks/` for the generators. Default app icons are
included so projects build out of the box; you can replace them later.

## Extensions for the BMC ecosystem

Forge is also the **authoring tool for BMC ecosystem extensions** — the
modules users create and sell on the FreeBlackMarket marketplace and install
into the Blackout host. The pipeline (the shared contract is
`docs/contracts/extension-manifest.md` in `Blackmarket-coa/free-black-market`):

1. **Create** in Forge's Extensions view from a template: Featured Vendor
   Widget or a pinned-nav panel (pure manifest — nothing to host), or an
   asset-carrying kind — theme, automation recipe, coalition kit, vault item,
   privacy tool.
2. **Validate & package**: Forge mirrors the shared manifest schema in Rust,
   checks it locally, and computes deterministic digests. Asset-carrying kinds
   get a reproducible `dist/<name>-<version>.zip`.
3. **Publish** through your FreeBlackMarket seller account
   (`~/.forge/fbm.json`): FBM validates, **signs with the platform Ed25519
   key** (Forge never holds signing keys), and lists the extension in its
   catalog with immutable version history. Manifest-kind extensions are hosted
   entirely by the marketplace; for asset kinds you upload the packaged zip
   anywhere public and paste its address — the signed envelope binds the
   zip's hash, so the bytes are tamper-evident wherever they live.
4. **Install**: buyers install under FBM entitlements; the Blackout client
   verifies signatures against FBM's published keys and renders the declared
   surfaces (home cards, pinned nav, panels).

Per the ecosystem's consolidation decisions, there is one registry (FBM's
catalog) and one host (Blackout); Black Mask stays a persona/credential
manager — the vault-item and privacy-tool templates are how that space is
served, through the shared registry.

## Current status

Forge is under active development. The repository currently includes:

- A **Tauri v2 backend** (`src-tauri/`) with a framework-adapter layer
  (`backend/frameworks/`) behind IPC commands for project discovery, config
  read/write/validation, process execution, build orchestration, deploy
  readiness checks, and local state persistence.
- A **BMC extension studio** (W3): scaffold an extension from eight templates
  (widget, pinned-nav panel, theme, automation recipe, coalition kit, vault
  item, privacy tool, blank), validate the shared extension manifest, package
  with deterministic digests (asset kinds get a reproducible bundle zip), and
  publish into the FreeBlackMarket plugin registry through your seller
  account — FBM signs at publish, Forge never holds signing keys. Connection
  config lives in `~/.forge/fbm.json`; the contract is
  `docs/contracts/extension-manifest.md` in `Blackmarket-coa/free-black-market`.
- A **React frontend** (`web/src/`) for project browsing, workspace views,
  build orchestration, deploy dashboard, per-framework config editing, and
  settings.
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
`https://github.com/blackmarket-coa/Forge/releases/latest/download/latest.json`
and can self-update from signed releases (see **Settings → Updates**).

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
