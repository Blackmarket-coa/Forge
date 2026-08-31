import { Framework, FrameworkInfo } from "../api/api"

/** Short display metadata for framework badges and pickers. */
export interface FrameworkBadge {
  /** Compact label for badges/chips (e.g. "Tauri"). */
  short: string
  emoji: string
}

export const FRAMEWORK_BADGES: Record<Framework, FrameworkBadge> = {
  tauri: { short: "Tauri", emoji: "🖥️" },
  capacitor: { short: "Capacitor", emoji: "📱" },
  electron: { short: "Electron", emoji: "🖥️" },
  pwa: { short: "PWA", emoji: "🌐" },
  "react-native": { short: "React Native", emoji: "📱" },
}

export function frameworkBadge(framework: string): FrameworkBadge {
  return (
    FRAMEWORK_BADGES[framework as Framework] || {
      short: framework,
      emoji: "📦",
    }
  )
}

/** The editable config file per framework, for UI copy. */
export const CONFIG_FILE_NAMES: Record<Framework, string> = {
  tauri: "tauri.conf.json",
  capacitor: "capacitor.config.json",
  electron: "package.json",
  pwa: "manifest.webmanifest",
  "react-native": "app.json",
}

/**
 * Fallback framework descriptions when `get_frameworks` isn't reachable
 * (e.g. plain-browser dev). Mirrors the backend's adapter metadata.
 */
export const FALLBACK_FRAMEWORKS: FrameworkInfo[] = [
  {
    id: "tauri",
    label: "Desktop app (Tauri)",
    tagline: "Small, fast desktop app for Windows, Mac, and Linux.",
    platforms: ["macOS", "Linux", "Windows"],
    bundle_kinds: [],
    tools: [],
    config_file: "tauri.conf.json",
    dev_label: "Preview app",
    dev_available: true,
  },
  {
    id: "capacitor",
    label: "iPhone & Android app (Capacitor)",
    tagline:
      "Your website as a real mobile app for Google Play and the App Store.",
    platforms: ["Android", "iOS"],
    bundle_kinds: [],
    tools: [],
    config_file: "capacitor.config.json",
    dev_label: "Preview on Android device",
    dev_available: true,
  },
  {
    id: "electron",
    label: "Desktop app (Electron)",
    tagline:
      "The classic web-tech desktop app — bigger downloads, huge ecosystem.",
    platforms: ["macOS", "Linux", "Windows"],
    bundle_kinds: [],
    tools: [],
    config_file: "package.json",
    dev_label: "Preview app",
    dev_available: true,
  },
  {
    id: "pwa",
    label: "Install from the browser (PWA)",
    tagline:
      "Visitors install your site straight from the browser — no app store, no tools needed.",
    platforms: ["Web"],
    bundle_kinds: [],
    tools: [],
    config_file: "manifest.webmanifest",
    dev_label: "Open site in browser",
    dev_available: false,
  },
  {
    id: "react-native",
    label: "Mobile app (React Native + Expo)",
    tagline:
      "A React Native shell around your site — preview instantly with the Expo Go app.",
    platforms: ["Android", "iOS"],
    bundle_kinds: [],
    tools: [],
    config_file: "app.json",
    dev_label: "Preview with Expo Go",
    dev_available: true,
  },
]

/** One editable field in the generic (non-Tauri) config form. */
export interface ConfigField {
  /** JSON path into the config document. */
  path: string[]
  label: string
  help?: string
  kind: "text" | "number"
  placeholder?: string
}

export interface ConfigSection {
  title: string
  hint?: string
  fields: ConfigField[]
}

/**
 * Form-mode field mapping for each framework's config file. Tauri is not
 * listed here — it keeps its original richer form in ConfigEditor.
 */
export const CONFIG_FORMS: Partial<Record<Framework, ConfigSection[]>> = {
  capacitor: [
    {
      title: "About your app",
      hint: "Shown on the phone and in the app stores.",
      fields: [
        {
          path: ["appName"],
          label: "App name",
          kind: "text",
          placeholder: "My App",
        },
        {
          path: ["appId"],
          label: "Bundle ID",
          help: "Reverse-domain format, e.g. com.example.myapp",
          kind: "text",
          placeholder: "com.example.myapp",
        },
      ],
    },
    {
      title: "Website",
      fields: [
        {
          path: ["server", "url"],
          label: "Website address",
          help: "The page your app opens.",
          kind: "text",
          placeholder: "https://yoursite.com",
        },
      ],
    },
  ],
  electron: [
    {
      title: "About your app",
      hint: "Shown in the installer and the app's details.",
      fields: [
        {
          path: ["build", "productName"],
          label: "App name",
          kind: "text",
          placeholder: "My App",
        },
        {
          path: ["build", "appId"],
          label: "Bundle ID",
          help: "Reverse-domain format, e.g. com.example.myapp",
          kind: "text",
          placeholder: "com.example.myapp",
        },
      ],
    },
    {
      title: "Website & window",
      fields: [
        {
          path: ["forgeApp", "url"],
          label: "Website address",
          help: "The page your app opens.",
          kind: "text",
          placeholder: "https://yoursite.com",
        },
        { path: ["forgeApp", "width"], label: "Width (px)", kind: "number" },
        { path: ["forgeApp", "height"], label: "Height (px)", kind: "number" },
      ],
    },
  ],
  pwa: [
    {
      title: "About your app",
      hint: "What browsers show when visitors install your site.",
      fields: [
        {
          path: ["name"],
          label: "App name",
          kind: "text",
          placeholder: "My App",
        },
        {
          path: ["short_name"],
          label: "Short name",
          help: "Used under the icon when space is tight.",
          kind: "text",
        },
        {
          path: ["start_url"],
          label: "Website address",
          help: "The page the installed app opens.",
          kind: "text",
          placeholder: "https://yoursite.com",
        },
      ],
    },
    {
      title: "Colors",
      fields: [
        {
          path: ["theme_color"],
          label: "Theme color",
          help: "Hex color, e.g. #111111",
          kind: "text",
        },
        {
          path: ["background_color"],
          label: "Background color",
          help: "Hex color shown while the app loads.",
          kind: "text",
        },
      ],
    },
  ],
  "react-native": [
    {
      title: "About your app",
      hint: "Shown on the phone and in the app stores.",
      fields: [
        { path: ["expo", "name"], label: "App name", kind: "text" },
        {
          path: ["expo", "slug"],
          label: "Slug",
          help: "Lowercase name used in URLs and folders.",
          kind: "text",
        },
        {
          path: ["expo", "android", "package"],
          label: "Android package",
          help: "Reverse-domain format, e.g. com.example.myapp",
          kind: "text",
        },
        {
          path: ["expo", "ios", "bundleIdentifier"],
          label: "iOS bundle ID",
          help: "Reverse-domain format, e.g. com.example.myapp",
          kind: "text",
        },
      ],
    },
    {
      title: "Website",
      fields: [
        {
          path: ["expo", "extra", "forgeUrl"],
          label: "Website address",
          help: "The page your app shows.",
          kind: "text",
          placeholder: "https://yoursite.com",
        },
      ],
    },
  ],
}

/**
 * The framework targets of a project, tolerating state written before
 * multi-framework support (falls back to a Tauri target).
 */
export function projectTargets(project: {
  targets?: { framework: string }[]
  tauri_version?: string
}): Framework[] {
  const listed = (project.targets || [])
    .map((t) => t.framework as Framework)
    .filter(Boolean)
  if (listed.length > 0) return listed
  return project.tauri_version ? ["tauri"] : []
}
