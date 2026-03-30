# Changelog

All notable changes to Forge are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Forge uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-03-30

### Added
- Initial release of Forge — visual project manager for Tauri apps
- Project registration, workspace management, and build history
- `run_dev` / `run_build` IPC commands with live log streaming via Tauri events
- Build presets with sequential and parallel step execution
- Deploy status dashboard with per-platform build matrix
- License tier gating (free / pro / team) via Keygen integration with offline grace period
- Tauri auto-updater (`tauri-plugin-updater`) pointed at GitHub Releases
- Structured logging via `env_logger` (controlled by `RUST_LOG`)
- Sentry crash reporting (opt-in via `SENTRY_DSN` environment variable)
- State persistence in `~/.forge/forge.json` with schema versioning and migration framework
- Config validation on startup with actionable warnings for missing env vars
- Cross-platform release CI (macOS, Linux, Windows) via GitHub Actions + `tauri-action`
- `cargo test` and `cargo clippy` gate in CI before release builds

### Changed
- Forked and rebranded from tilt-orchestrator (MIT licence retained)
