# Changelog

## Unreleased

### Added
- Multi-framework app suite: the "Turn your website into an app" flow can now
  generate **Capacitor (iPhone & Android)**, **Electron (desktop)**,
  **PWA (install from the browser)**, and **React Native + Expo (mobile)**
  targets alongside the original Tauri desktop app. One project holds any
  combination of app types (recorded in a `forge.project.json` manifest), and
  **Add another app type** extends an existing project in place.
- A framework-adapter layer in the backend (`backend/frameworks/`) that powers
  generation, detection (including projects created outside Forge), preview and
  build command sequences, artifact discovery, per-framework config editing
  with validation, environment tool checks, and the deploy readiness matrix.
- Framework-aware UI: an app-type picker in the creation wizard, per-target
  tabs with framework-specific build outputs (and honest tooling notes) in the
  project view, framework badges on project cards and artifacts, a deploy
  matrix with one row per app type, and an environment screen that says which
  app types need each tool.
- The PWA "build" packages an uploadable web-app kit (manifest, service
  worker, icons, paste-in snippet) as a zip with no external toolchain.

### Added (W3 — extension studio MVP)
- Extensions view: scaffold a BMC extension (Featured Vendor Widget or blank
  template), validate the shared extension manifest, package with
  deterministic digests, publish through a FreeBlackMarket seller account
  (FBM signs at publish — Forge never holds keys), and browse the public
  plugin registry. Pro-gated publish (`extension_publish`) and browse
  (`plugin_browser` — the previously dangling paywall key, now real).
- Rust: `extension_manifest`/`semver` (schema mirror + SemVer §11 precedence
  sharing FBM's test vectors), `extension_scaffold`, `extension_package`,
  `fbm_client` (per-machine `~/.forge/fbm.json`, token masked), seven new IPC
  commands.
- The deploy dashboard nav is renamed "Release readiness" (real publishing
  now lives in Extensions); `web/.env` carries `REACT_APP_VERSION` and
  `scripts/check-version-sync.sh` now checks all three version sources.
- Template expansion: six more scaffold templates covering the Blackout-hosted
  artifact kinds (pinned-nav panel, theme, automation recipe, coalition kit,
  vault item, privacy tool), a backend-driven template registry
  (`get_extension_templates`), and hosted-bundle packaging — asset-carrying
  kinds produce a deterministic `dist/<name>-<version>.zip` whose SHA-256 is
  bound into FBM's signed envelope; the Extensions view collects the bundle's
  public address before publishing (manifest-kind extensions still need
  nothing hosted).

### Changed
- Persisted state schema bumped to v2: projects carry framework `targets`, and
  build history/presets record a `framework` (older entries migrate as Tauri).
- `run_dev`/`run_build` execute adapter-defined command sequences (e.g.
  `npm install` → `npx cap sync android` → Gradle) under one process id, with
  each step echoed to the activity terminal.

### Fixed
- Stopping a running preview/build actually works now: the process manager no
  longer holds the child-process lock while waiting for exit (which made
  `kill` block until the process ended on its own), and finished process ids
  can be reused, so building the same target twice no longer fails with
  "process already running".
- Docs: corrected `DEPLOYMENT.md`'s local dev instructions (JS deps live in
  `web/`, not the repo root), filled in the README's updater URL placeholder,
  and synced `package.json`/`Cargo.toml` descriptions with the product's
  current website-to-app framing.
- Marked `FORGE_IMPLEMENTATION_PLAN.md` phases 2–4 as complete to match the
  shipped feature set (config editor, build orchestrator, workspaces, deploy
  dashboard, mobile build paths, license gating, auto-updater).

### Added
- Custom design system: themeable design tokens (dark/light) and a reusable
  `ui/` primitive library (Button, Input, Select, Checkbox, Tabs, Collapsible,
  Card, Badge, Banner, EmptyState, PageHeader, Progress, Field, Spinner,
  Dialog/ConfirmDialog).
- Application shell with sidebar navigation, brand, tier badge, and theme
  toggle; global error boundary and toast notifications (notistack).
- First-run onboarding with a live environment/toolchain check.
- Config editor now previews a diff and asks for confirmation before saving.
- Atomic file writes for `forge.json`, `tauri.conf.json`, the license cache,
  and build history to prevent corruption on interrupted writes.
- Graceful (SIGTERM → SIGKILL) shutdown of spawned dev/build processes on Unix.
- Frontend and backend test suites and a CI workflow (lint, type-check,
  clippy, and tests on push/PR).

### Changed
- Re-skinned every screen (projects, project view, config editor, create
  wizard, deploy dashboard, build orchestrator, settings, license gate) with
  the design system and real loading/empty/error states.
- Build history file is bounded to the 200 most recent records.

### Fixed
- Terminal now subscribes to the process events the backend actually emits, so
  dev and build output renders correctly.

### Removed
- ~190 unused Tilt-era source files and the dependencies they pulled in
  (Material-UI, styled-components, react-router, react-table, Storybook, etc.).

## 0.1.0
- Initial fork from tilt-orchestrator (MIT)
- Rebranded as Forge
