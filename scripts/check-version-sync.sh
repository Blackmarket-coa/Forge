#!/bin/sh
# Verify that the app version is identical in src-tauri/Cargo.toml and
# src-tauri/tauri.conf.json. These two files must stay in sync — a mismatch
# ships a release whose installer/updater metadata disagrees with the binary.
#
# Exits non-zero with a clear message when the versions diverge so CI can block
# the drift before it reaches a tagged release.
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cargo_toml="$repo_root/src-tauri/Cargo.toml"
tauri_conf="$repo_root/src-tauri/tauri.conf.json"

# First `version = "x.y.z"` under [package] in Cargo.toml.
cargo_version=$(grep -m1 '^version[[:space:]]*=' "$cargo_toml" \
  | sed -E 's/.*"([^"]+)".*/\1/')

# Top-level "version": "x.y.z" in tauri.conf.json.
tauri_version=$(grep -m1 '"version"[[:space:]]*:' "$tauri_conf" \
  | sed -E 's/.*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')

if [ -z "$cargo_version" ]; then
  echo "check-version-sync: could not read version from $cargo_toml" >&2
  exit 2
fi
if [ -z "$tauri_version" ]; then
  echo "check-version-sync: could not read version from $tauri_conf" >&2
  exit 2
fi

if [ "$cargo_version" != "$tauri_version" ]; then
  echo "check-version-sync: version mismatch" >&2
  echo "  src-tauri/Cargo.toml      = $cargo_version" >&2
  echo "  src-tauri/tauri.conf.json = $tauri_version" >&2
  echo "Update both files to the same version before releasing." >&2
  exit 1
fi

echo "check-version-sync: OK ($cargo_version)"
