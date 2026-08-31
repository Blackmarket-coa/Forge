import { invoke, isTauri } from "@tauri-apps/api/core"
import { openUrl } from "@tauri-apps/plugin-opener"

/** Frameworks Forge can generate and manage. */
export type Framework =
  | "tauri"
  | "capacitor"
  | "electron"
  | "pwa"
  | "react-native"

export interface BundleKind {
  id: string
  label: string
  platform: string
  note?: string
}

export interface ToolInfo {
  name: string
  label: string
  install_hint: string
  platform_only?: string
}

/** Static framework description served by the backend (`get_frameworks`). */
export interface FrameworkInfo {
  id: Framework
  label: string
  tagline: string
  platforms: string[]
  bundle_kinds: BundleKind[]
  tools: ToolInfo[]
  config_file: string
  dev_label: string
  dev_available: boolean
}

/** One framework target inside a project. */
export interface TargetMeta {
  framework: Framework
  dir: string
  version?: string
  status: string
}

export interface ProjectMeta {
  id: string
  name: string
  path: string
  workspace_id?: string
  tauri_version?: string
  identifier?: string
  frontend_framework?: string
  platforms: string[]
  git_branch?: string
  git_dirty: boolean
  status: string
  tags: string[]
  role?: string
  targets: TargetMeta[]
  source_url?: string
}

export interface Workspace {
  id: string
  name: string
  project_ids: string[]
  color?: string
}

export interface BuildStep {
  project_id: string
  framework: Framework
  targets: string[]
  parallel_with_next: boolean
}

export interface BuildPreset {
  id: string
  name: string
  workspace_id: string
  steps: BuildStep[]
}

export interface LicenseStatus {
  tier: "free" | "pro" | "team"
  valid: boolean
  expires_at?: string | null
  key_masked?: string | null
  checked_at?: number | null
}

export interface BuildRecord {
  id: string
  project_id: string
  framework: Framework
  targets: string[]
  status: string
  started_at: string
  duration_secs: number
  artifacts: any[]
  log_path: string
}

/** One tool row from `check_environment`, deduped across frameworks. */
export interface EnvTool {
  name: string
  label: string
  installed: boolean
  version: string
  install_hint: string
  needed_by: Framework[]
}

export interface EnvResult {
  tools: EnvTool[]
}

export async function getFrameworks(): Promise<FrameworkInfo[]> {
  return invoke("get_frameworks")
}

export async function registerProject(path: string): Promise<ProjectMeta> {
  return invoke("register_project", { path })
}
export async function getProjects(
  workspaceId?: string
): Promise<ProjectMeta[]> {
  return invoke("get_projects", { workspaceId })
}
export async function detectProjectStatus(path: string): Promise<any> {
  return invoke("detect_project_status", { path })
}
export async function scanDirectory(path: string): Promise<ProjectMeta[]> {
  return invoke("scan_directory", { path })
}
export async function readConfig(
  projectPath: string,
  framework: Framework = "tauri"
): Promise<any> {
  return invoke("read_config", { projectPath, framework })
}
export async function writeConfig(
  projectPath: string,
  config: any,
  framework: Framework = "tauri"
): Promise<void> {
  return invoke("write_config", { projectPath, framework, config })
}
export async function validateConfig(
  projectPath: string,
  config: any,
  framework: Framework = "tauri"
): Promise<string[]> {
  return invoke("validate_config", { projectPath, framework, config })
}
/** Start a preview/dev run; resolves to the process id used for Stop. */
export async function runDev(
  projectPath: string,
  framework: Framework = "tauri"
): Promise<string> {
  return invoke("run_dev", { projectPath, framework })
}
export async function runBuild(
  projectPath: string,
  targets: string[],
  framework: Framework = "tauri"
): Promise<any> {
  return invoke("run_build", { projectPath, framework, targets })
}
export async function killProcess(processId: string): Promise<void> {
  return invoke("kill_process", { processId })
}
export async function checkEnvironment(): Promise<EnvResult> {
  return invoke("check_environment")
}

/** Open a URL in the user's default browser (Tauri) or a new tab (plain browser dev). */
export async function openExternal(url: string): Promise<void> {
  if (isTauri()) {
    await openUrl(url)
  } else {
    window.open(url, "_blank", "noopener,noreferrer")
  }
}
export async function collectArtifacts(projectPath: string): Promise<any[]> {
  return invoke("collect_artifacts", { projectPath })
}

export async function createProject(
  path: string,
  name: string,
  template: string,
  packageManager: string
): Promise<ProjectMeta> {
  return invoke("create_project", { path, name, template, packageManager })
}

export interface CreateWebAppArgs {
  parentDir: string
  name: string
  url: string
  width?: number
  height?: number
  identifier?: string
  /** Which kinds of app to generate; defaults to a Tauri desktop app. */
  frameworks?: Framework[]
}

/**
 * Generate app(s) that wrap a website URL — one project with a target for
 * each requested framework. Requires only a website address and an app name —
 * no Node.js, package manager, or framework tooling at generation time.
 */
export async function createWebApp(
  args: CreateWebAppArgs
): Promise<ProjectMeta> {
  return invoke("create_web_app", {
    parentDir: args.parentDir,
    name: args.name,
    url: args.url,
    width: args.width,
    height: args.height,
    identifier: args.identifier,
    frameworks: args.frameworks,
  })
}

/** Add another kind of app to an existing project (reuses its website). */
export async function addTarget(
  projectPath: string,
  framework: Framework,
  url?: string
): Promise<ProjectMeta> {
  return invoke("add_target", { projectPath, framework, url })
}

/** A friendly default folder (~/Forge Apps) for saving generated apps. */
export async function getDefaultAppDir(): Promise<string> {
  return invoke("get_default_app_dir")
}

export async function initTauri(projectPath: string): Promise<any> {
  return invoke("init_tauri", { projectPath })
}

export async function createWorkspace(name: string): Promise<Workspace> {
  return invoke("create_workspace", { name })
}

export async function getWorkspaces(): Promise<Workspace[]> {
  return invoke("get_workspaces")
}

export async function updateWorkspace(
  id: string,
  name?: string,
  color?: string
): Promise<Workspace> {
  return invoke("update_workspace", { id, name, color })
}

export async function deleteWorkspace(id: string): Promise<void> {
  return invoke("delete_workspace", { id })
}

export async function addProjectToWorkspace(
  workspaceId: string,
  projectId: string
): Promise<void> {
  return invoke("add_project_to_workspace", { workspaceId, projectId })
}

export async function removeProjectFromWorkspace(
  workspaceId: string,
  projectId: string
): Promise<void> {
  return invoke("remove_project_from_workspace", { workspaceId, projectId })
}

export async function saveBuildPreset(preset: BuildPreset): Promise<void> {
  return invoke("save_build_preset", { preset })
}

export async function getBuildPresets(
  workspaceId: string
): Promise<BuildPreset[]> {
  return invoke("get_build_presets", { workspaceId })
}

export async function runBuildPreset(presetId: string): Promise<any> {
  return invoke("run_build_preset", { presetId })
}

export async function getBuildHistory(
  projectId?: string,
  limit: number = 10
): Promise<BuildRecord[]> {
  return invoke("get_build_history", { projectId, limit })
}

export async function getDeployStatus(workspaceId: string): Promise<any> {
  return invoke("get_deploy_status", { workspaceId })
}

export async function validateLicense(key: string): Promise<LicenseStatus> {
  return invoke("validate_license", { key })
}

export async function getLicenseStatus(): Promise<LicenseStatus> {
  return invoke("get_license_status")
}

export async function clearLicense(): Promise<LicenseStatus> {
  return invoke("clear_license")
}

// ---------------------------------------------------------------------------
// Extensions (W3): scaffold → validate → package → publish → browse.
// ---------------------------------------------------------------------------

export interface ExtensionDigests {
  manifestSha256: string
  /** Hash of the hosted bundle zip (absent for manifest_plugin). */
  codeSha256?: string
  entrySha256?: string
  payloadSha256?: string
  assetHashes: Record<string, string>
}

export interface PackageResult {
  distDir: string
  digests: ExtensionDigests
  /** Path of the deterministic bundle zip, for kinds that need hosting. */
  bundlePath?: string
  /** True when publishing needs the hosted bundle's web address. */
  needsBlob: boolean
  issues: string[]
}

/** One scaffold template from the backend's registry. */
export interface ExtensionTemplate {
  id: string
  label: string
  description: string
  artifactKind: string
  needsBlob: boolean
}

export interface PublishOutcome {
  listingId: string
  pluginSlug?: string | null
  pluginVersion?: string | null
  envelope: unknown
}

export interface FbmStatus {
  configured: boolean
  api_base_url?: string | null
  seller_token_masked?: string | null
  /** Storefront publishable key — public by design, shown in full. */
  publishable_key?: string | null
}

export async function getExtensionTemplates(): Promise<ExtensionTemplate[]> {
  return invoke("get_extension_templates")
}

export async function scaffoldExtension(
  parentDir: string,
  name: string,
  template?: string
): Promise<string> {
  return invoke("scaffold_extension", { parentDir, name, template })
}

export async function validateExtension(
  projectPath: string
): Promise<string[]> {
  return invoke("validate_extension", { projectPath })
}

export async function packageExtension(
  projectPath: string
): Promise<PackageResult> {
  return invoke("package_extension", { projectPath })
}

export async function publishExtension(
  projectPath: string,
  codeBlobUrl?: string
): Promise<PublishOutcome> {
  return invoke("publish_extension", { projectPath, codeBlobUrl })
}

export async function browsePlugins(category?: string): Promise<{
  count: number
  plugins: Array<{
    slug: string
    name: string
    category: string
    description: string
    version: string
    install_count: number
  }>
}> {
  return invoke("browse_plugins", { category })
}

export async function getFbmStatus(): Promise<FbmStatus> {
  return invoke("get_fbm_status")
}

export async function setFbmConfig(
  apiBaseUrl?: string,
  sellerToken?: string,
  publishableKey?: string
): Promise<FbmStatus> {
  return invoke("set_fbm_config", { apiBaseUrl, sellerToken, publishableKey })
}
