# Consolidation Status — Forge

Part of the 2026-08-28 seven-repo BMC consolidation review. The canonical review — audit
verdicts, decisions, and the ordered roadmap — is `docs/REPO_CONSOLIDATION_REVIEW.md` in
`Blackmarket-coa/free-black-market`.

## This repo's verdict

- **Role: the authoring tool, publishing into FBM's registry — not a registry of its own.**
  Decision D6 places the extension registry inside FBM's catalog, where `plugin-registry`,
  Ed25519 `marketplace-signing`, `digital-product` delivery, and entitlements already exist. The
  remaining platform gap (hook registry + semver handling) closes on the FBM side in workstream
  W3; Forge's side of W3 is the shared extension manifest and the build → sign → publish flow.
- What exists today is real but scoped: a Tauri v2 project manager with 30 IPC commands, the
  "website → app" scaffolding flow, and Keygen licensing (still pointed at the demo account).
  There is no extension/SDK/registry code here yet — the W3 MVP ("Featured Vendor Widget" built
  in Forge, signed, published to FBM, installed under entitlements) is where that starts.

## Queued fixes

- `web/src/lib/tier.ts` gates a `plugin_browser` feature that does not exist anywhere in the
  codebase — remove the paywall entry or implement the browser against FBM's `/store/plugins`.
- Stack modernization (React 17/CRA/TS 4.4 → React 18/Vite/TS 5), already recommended by
  `PRODUCTION_READINESS.md`, is queued behind the W3 MVP.
- Set a real `KEYGEN_ACCOUNT_ID` before any licensing-dependent release; validation always
  returns invalid on the demo account (see `src-tauri/src/backend/config.rs`).
