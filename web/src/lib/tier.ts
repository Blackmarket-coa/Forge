export type Tier = "free" | "pro" | "team"

export const LIMITS = {
  free: { maxProjects: 2 },
  pro: { maxProjects: Infinity },
  team: { maxProjects: Infinity },
}

// Keep this list in sync with src-tauri/src/backend/tier.rs, which enforces
// the same gates in the IPC commands (publish_extension, browse_plugins).
export function isFeatureAvailable(feature: string, tier: Tier): boolean {
  const proFeatures = [
    "workspaces",
    "build_presets",
    "build_history",
    "deploy_dashboard",
    "plugin_browser",
    "extension_publish",
  ]
  if (proFeatures.includes(feature)) return tier !== "free"
  return true
}
