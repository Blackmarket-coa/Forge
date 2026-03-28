export type Tier = "free" | "pro" | "team"

export const LIMITS = {
  free: { maxProjects: 2 },
  pro: { maxProjects: Infinity },
  team: { maxProjects: Infinity },
}

export function isFeatureAvailable(feature: string, tier: Tier): boolean {
  const proFeatures = [
    "workspaces",
    "build_presets",
    "build_history",
    "deploy_dashboard",
    "plugin_browser",
  ]
  if (proFeatures.includes(feature)) return tier !== "free"
  return true
}
