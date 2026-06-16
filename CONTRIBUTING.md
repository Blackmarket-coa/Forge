# Contributing to Forge

Forge is a Tauri v2 desktop app: a Rust backend in `src-tauri/` and a React
(TypeScript) frontend in `web/`. There is no server component.

## Prerequisites

See [DEPLOYMENT.md](./DEPLOYMENT.md) for the full toolchain matrix. In short you
need Rust (stable), Node.js 20+, and the platform WebView dependencies (on Linux:
`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`).

## Local development

```bash
cd web
corepack enable
yarn install

# Frontend only (fast UI iteration, no native shell)
yarn dev

# Full desktop app (Tauri + React, rebuilds on change)
yarn tauri dev
```

## Before opening a pull request

Run the same checks CI runs (`.github/workflows/ci.yml`):

```bash
# Frontend: lint, format, type-check, tests
cd web && yarn run check && yarn run test --watchAll=false

# Backend: format, lint, tests
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Keep changes focused, write a clear commit message, and update `CHANGELOG.md`
under the `## Unreleased` heading whenever your change is user-visible.
