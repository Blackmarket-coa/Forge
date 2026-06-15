# Changelog

## Unreleased

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
