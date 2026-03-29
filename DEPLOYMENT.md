# Forge — Deployment Guide

## Prerequisites

| Tool | Minimum version | Purpose |
|------|----------------|---------|
| Rust | 1.77 (stable) | Tauri backend |
| Node.js | 20 | Frontend build |
| Yarn | 4.x (pinned via `packageManager`) | JS dependency management |
| Tauri CLI | 2.x (`cargo tauri`) | App bundling |

## Environment variables

Copy `.env.example` to `.env` and fill in the values before building a
production release.  The variables are also documented in `.env.example`.

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `KEYGEN_ACCOUNT_ID` | Production | `demo-account` | Keygen account for license validation |
| `SENTRY_DSN` | Recommended | _(none — Sentry disabled)_ | Sentry project DSN for crash reporting |
| `RUST_LOG` | Optional | `warn` | Log level filter (`forge=info` recommended) |

Forge validates these at startup and emits `warn`-level log lines for any
missing or placeholder values.

## Local development

```sh
# Install JS dependencies
yarn install

# Start the Tauri dev server (hot-reload frontend + Rust backend)
cargo tauri dev
```

## Building a release

### 1. Bump the version

Version is defined in two places — keep them in sync:

- `src-tauri/Cargo.toml` → `[package] version`
- `src-tauri/tauri.conf.json` → `"version"`

### 2. Tag and push

The GitHub Actions release workflow (`/.github/workflows/release.yml`) triggers
on `v*` tags and builds signed installers for macOS, Linux, and Windows.

```sh
git tag v0.2.0
git push origin v0.2.0
```

The workflow creates a **draft** GitHub Release.  Review the draft, edit the
release notes, then publish it.

### 3. Publish the `latest.json` update manifest

The Tauri auto-updater polls:

```
https://github.com/blackmarket-coa/forge/releases/latest/download/latest.json
```

`tauri-action` generates and uploads this file automatically when you publish
the draft release.  No manual step is needed.

## Auto-updater

`tauri-plugin-updater` is bundled into every build.  On startup it checks the
endpoint above.  Users are prompted to install the update in-app.

To test the updater locally, set a lower `version` in `tauri.conf.json` and run
`cargo tauri dev` — the updater will consider the running version outdated and
offer to upgrade.

## Rollback procedure

Because each GitHub Release ships a self-contained installer, rolling back is
straightforward:

1. Find the previous release tag on the Releases page.
2. Download the installer for the target platform.
3. Run the installer — it overwrites the current version.

To prevent users on the current version from being offered a bad release via
auto-update, unpublish or delete the bad release draft before publishing it.
If a bad release was already published:

1. Delete the release (or mark it as a pre-release to suppress the
   `latest.json` update) on GitHub.
2. Publish a patch release that increments the version so the updater points
   users to the fixed build.

## CI overview

| Pipeline | Trigger | Jobs |
|----------|---------|------|
| GitHub Actions `release.yml` | Push `v*` tag | Cross-platform installers, draft release |
| CircleCI | Push to any branch | JS tests, Go tests, integration tests, linting |

Slack notifications for `master` branch failures are configured in CircleCI.

## Secrets

| Secret | Where | Notes |
|--------|-------|-------|
| `GITHUB_TOKEN` | GitHub Actions (automatic) | Used by `tauri-action` to create releases |
| `KEYGEN_ACCOUNT_ID` | CI environment / machine `.env` | License validation account |
| `SENTRY_DSN` | CI environment / machine `.env` | Crash reporting |

Never commit `.env` files.  `.env` is listed in `.gitignore`.

## Checklist before publishing a release

- [ ] Version bumped in `Cargo.toml` and `tauri.conf.json`
- [ ] `CHANGELOG.md` updated
- [ ] All CI checks green on the release commit
- [ ] `KEYGEN_ACCOUNT_ID` set to production account in build environment
- [ ] `SENTRY_DSN` set to production project DSN in build environment
- [ ] Draft release reviewed and release notes edited
- [ ] Release published (triggers `latest.json` upload for auto-updater)
