# Changelog

## Unreleased

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

### Fixed
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
