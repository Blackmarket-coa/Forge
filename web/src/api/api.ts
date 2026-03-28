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
export async function runBuild(projectPath: string, targets: string[]): Promise<string> {
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
