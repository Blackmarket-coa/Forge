import { invoke } from "@tauri-apps/api/core"

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
}

export interface Workspace {
  id: string
  name: string
  project_ids: string[]
  color?: string
}

export interface BuildStep {
  project_id: string
  targets: string[]
  parallel_with_next: boolean
}

export interface BuildPreset {
  id: string
  name: string
  workspace_id: string
  steps: BuildStep[]
}

export interface BuildRecord {
  id: string
  project_id: string
  targets: string[]
  status: string
  started_at: string
  duration_secs: number
  artifacts: any[]
  log_path: string
}

export async function registerProject(path: string): Promise<ProjectMeta> {
  return invoke("register_project", { path })
}
export async function getProjects(workspaceId?: string): Promise<ProjectMeta[]> {
  return invoke("get_projects", { workspaceId })
}
export async function detectTauriStatus(path: string): Promise<any> {
  return invoke("detect_tauri_status", { path })
}
export async function scanDirectory(path: string): Promise<ProjectMeta[]> {
  return invoke("scan_directory", { path })
}
export async function readConfig(projectPath: string): Promise<any> {
  return invoke("read_config", { projectPath })
}
export async function writeConfig(projectPath: string, config: any): Promise<void> {
  return invoke("write_config", { projectPath, config })
}
export async function validateConfig(projectPath: string, config: any): Promise<string[]> {
  return invoke("validate_config", { projectPath, config })
}
export async function runDev(projectPath: string): Promise<number> {
  return invoke("run_dev", { projectPath })
}
export async function runBuild(projectPath: string, targets: string[]): Promise<any> {
  return invoke("run_build", { projectPath, targets })
}
export async function killProcess(processId: string): Promise<void> {
  return invoke("kill_process", { processId })
}
export async function checkEnvironment(): Promise<any> {
  return invoke("check_environment")
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

export async function initTauri(projectPath: string): Promise<any> {
  return invoke("init_tauri", { projectPath })
}

export async function createWorkspace(name: string): Promise<Workspace> {
  return invoke("create_workspace", { name })
}

export async function getWorkspaces(): Promise<Workspace[]> {
  return invoke("get_workspaces")
}

export async function updateWorkspace(id: string, name?: string, color?: string): Promise<Workspace> {
  return invoke("update_workspace", { id, name, color })
}

export async function deleteWorkspace(id: string): Promise<void> {
  return invoke("delete_workspace", { id })
}

export async function addProjectToWorkspace(workspaceId: string, projectId: string): Promise<void> {
  return invoke("add_project_to_workspace", { workspaceId, projectId })
}

export async function removeProjectFromWorkspace(workspaceId: string, projectId: string): Promise<void> {
  return invoke("remove_project_from_workspace", { workspaceId, projectId })
}

export async function saveBuildPreset(preset: BuildPreset): Promise<void> {
  return invoke("save_build_preset", { preset })
}

export async function getBuildPresets(workspaceId: string): Promise<BuildPreset[]> {
  return invoke("get_build_presets", { workspaceId })
}

export async function runBuildPreset(presetId: string): Promise<any> {
  return invoke("run_build_preset", { presetId })
}

export async function getBuildHistory(projectId?: string, limit: number = 10): Promise<BuildRecord[]> {
  return invoke("get_build_history", { projectId, limit })
}
