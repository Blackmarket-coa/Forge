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
Forge does not load `.env` itself — export the values into the environment of
the build (or of the running app, where noted).

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `KEYGEN_ACCOUNT_ID` | Production (**build time**) | _(none — licensing not configured)_ | Keygen account for license validation; baked into the binary at compile time |
| `SENTRY_DSN` | Recommended | _(none — Sentry disabled)_ | Sentry project DSN for crash reporting |
| `RUST_LOG` | Optional | `warn` | Log level filter (`forge=info` recommended) |

`SENTRY_DSN` and `RUST_LOG` are read when the app starts.
`KEYGEN_ACCOUNT_ID` is read when the Rust backend is **compiled**
(`option_env!` in `src-tauri/src/backend/license.rs`): a binary built without
it cannot validate license keys and reports "Licensing is not configured in
this build" instead of treating keys as invalid. Debug builds (`cargo tauri
dev`) also honor a runtime `KEYGEN_ACCOUNT_ID` override for development;
release builds ignore it.

Forge checks these at startup and emits `warn`-level log lines for any
missing values (including a build without licensing configured).

## Local development

```sh
# Install JS dependencies (frontend lives in web/)
cd web && yarn install

# Start the Tauri dev server (hot-reload frontend + Rust backend), from the repo root
cargo tauri dev
```

## Building a release

> **Status:** no release has been published yet — there are no `v*` tags or
> GitHub Releases, and `release.yml` has never run. The steps below describe
> the intended process; the first tagged release will be its first real test.

### 1. Bump the version

Version is defined in two places — keep them in sync:

- `src-tauri/Cargo.toml` → `[package] version`
- `src-tauri/tauri.conf.json` → `"version"`

### 2. Tag and push

The GitHub Actions release workflow (`/.github/workflows/release.yml`) triggers
on `v*` tags and builds installers for macOS, Linux, and Windows. The updater
artifacts and `latest.json` are signed with `TAURI_SIGNING_PRIVATE_KEY`; the
installers themselves are **not** code-signed or notarized (no Apple/Windows
signing identity is configured), so macOS Gatekeeper and Windows SmartScreen
will warn on first launch.

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

`tauri-action` generates and uploads this file to the draft release. The
`latest/download` URL only resolves once a non-draft release is published —
until then (i.e. today) the updater finds nothing.

## Auto-updater

`tauri-plugin-updater` is bundled into every build.  It does not check on
startup: users run **Settings → Updates → Check for updates**, which queries the
endpoint above and offers to download and install a newer signed build.

Because no release has been published, the endpoint currently returns nothing
and the check fails. Once a release exists, you can test the flow by running a
build with a lower `version` in `tauri.conf.json` and checking for updates.

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
| GitHub Actions `ci.yml` | Push to `master` / any PR | Version-sync check; frontend lint, types, tests (+coverage artifact); backend `cargo fmt`/`clippy`/`test` |
| GitHub Actions `release.yml` | Push `v*` tag | Cross-platform installers + signed updater artifacts / `latest.json`, draft release (never run yet) |

The `version-sync` job runs `scripts/check-version-sync.sh`, which fails the
build if `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` disagree on the
version — so the manual two-file bump in step 1 above is enforced by CI.

## Secrets

| Secret | Where | Notes |
|--------|-------|-------|
| `GITHUB_TOKEN` | GitHub Actions (automatic) | Used by `tauri-action` to create releases |
| `KEYGEN_ACCOUNT_ID` | Repository secret, passed to the build by `release.yml` | License validation account (compiled into the binary) |
| `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Repository secrets | Sign updater artifacts and `latest.json` |
| `SENTRY_DSN` | CI environment / machine `.env` | Crash reporting |

Never commit `.env` files.  `.env` is listed in `.gitignore`.

## Checklist before publishing a release

- [ ] Version bumped in `Cargo.toml` and `tauri.conf.json` (`sh scripts/check-version-sync.sh` passes)
- [ ] `CHANGELOG.md` updated
- [ ] All CI checks green on the release commit
- [ ] `KEYGEN_ACCOUNT_ID` repository secret set to the production account (it is compiled in — a release built without it cannot unlock Pro/Team)
- [ ] `SENTRY_DSN` set to production project DSN in build environment
- [ ] Draft release reviewed and release notes edited
- [ ] Release published (triggers `latest.json` upload for auto-updater)
