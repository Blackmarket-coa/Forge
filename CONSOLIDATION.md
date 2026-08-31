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
- What exists today is real but scoped: a Tauri v2 project manager (now 40 IPC commands), the
  multi-framework "website → app" scaffolding flow (Tauri, Capacitor, Electron, PWA,
  React Native/Expo), and Keygen licensing (still pointed at the demo account).
  ~~There is no extension/SDK/registry code here yet~~ **W3 landed (2026-08-29)**: the shared
  extension manifest mirror + semver (`backend/extension_manifest.rs`, `backend/semver.rs` —
  test vectors shared with FBM's `compat.unit.spec.ts`), the extension scaffolder with the
  Featured Vendor Widget template, deterministic packaging + digests, the FBM publish client
  (`backend/fbm_client.rs` — Forge holds NO signing keys; FBM signs at publish per
  `free-black-market/docs/contracts/extension-manifest.md`), and the Extensions view
  (scaffold → validate → package → publish → browse). Config lives in `~/.forge/fbm.json`
  (token masked everywhere); everything fails closed with a Settings pointer until configured.
  **Template expansion (2026-08-31)**: the scaffold registry now covers eight templates across
  the Blackout-hosted artifact kinds (pinned-nav panel, theme, automation recipe, coalition kit,
  vault item, privacy tool alongside the widget + blank), and asset-carrying kinds package a
  deterministic zip whose hash rides `code_blob_sha256` — the author hosts the zip, FBM hosts
  and signs everything else. Per D4/D6, the vault-item/privacy-tool templates are how the
  Black Mask-adjacent space is served: through the shared registry, never a blackmask-hosted
  module system.

## Queued fixes

- ~~`plugin_browser` gates a feature that does not exist~~ **resolved (W3)**: the Extensions
  view's registry browse implements it against FBM's public `/store/plugins` (sent with the
  storefront publishable key — Medusa requires one on every `/store/*` route; it is a public
  key, configured next to the seller token); publishing has its own `extension_publish` Pro key.
- Stack modernization (React 17/CRA/TS 4.4 → React 18/Vite/TS 5), already recommended by
  `PRODUCTION_READINESS.md`, is queued behind the W3 MVP.
- Set a real `KEYGEN_ACCOUNT_ID` before any licensing-dependent release; validation always
  returns invalid on the demo account (see `src-tauri/src/backend/config.rs`).
