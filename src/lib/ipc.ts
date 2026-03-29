import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  LicenseStatus,
  ProjectMeta,
  TauriStatus,
  BuildResult,
  Workspace,
  BuildPreset,
  BuildRecord,
  Artifact,
  DeployStatus,
  EnvironmentStatus,
  ProcessOutputEvent,
} from './types'

// License
export const validateLicense = (key: string) =>
  invoke<LicenseStatus>('validate_license', { key })

export const getLicenseStatus = () =>
  invoke<LicenseStatus>('get_license_status')

export const clearLicense = () =>
  invoke<LicenseStatus>('clear_license')

// Projects
export const registerProject = (path: string) =>
  invoke<ProjectMeta>('register_project', { path })

export const getProjects = (workspaceId?: string) =>
  invoke<ProjectMeta[]>('get_projects', { workspace_id: workspaceId })

export const detectTauriStatus = (path: string) =>
  invoke<TauriStatus>('detect_tauri_status', { path })

export const scanDirectory = (path: string) =>
  invoke<ProjectMeta[]>('scan_directory', { path })

// Config
export const readConfig = (projectPath: string) =>
  invoke<unknown>('read_config', { project_path: projectPath })

export const writeConfig = (projectPath: string, config: unknown) =>
  invoke<void>('write_config', { project_path: projectPath, config })

export const validateConfig = (projectPath: string, config: unknown) =>
  invoke<string[]>('validate_config', { project_path: projectPath, config })

// Process management
export const runDev = (projectPath: string) =>
  invoke<number>('run_dev', { project_path: projectPath })

export const runBuild = (projectPath: string, targets: string[]) =>
  invoke<BuildResult>('run_build', { project_path: projectPath, targets })

export const killProcess = (processId: string) =>
  invoke<void>('kill_process', { process_id: processId })

// Project creation
export const createProject = (
  path: string,
  name: string,
  template: string,
  packageManager: string
) =>
  invoke<ProjectMeta>('create_project', {
    path,
    name,
    template,
    package_manager: packageManager,
  })

export const initTauri = (projectPath: string) =>
  invoke<TauriStatus>('init_tauri', { project_path: projectPath })

// Workspaces
export const createWorkspace = (name: string) =>
  invoke<Workspace>('create_workspace', { name })

export const getWorkspaces = () =>
  invoke<Workspace[]>('get_workspaces')

export const updateWorkspace = (id: string, name?: string, color?: string) =>
  invoke<Workspace>('update_workspace', { id, name, color })

export const deleteWorkspace = (id: string) =>
  invoke<void>('delete_workspace', { id })

export const addProjectToWorkspace = (workspaceId: string, projectId: string) =>
  invoke<void>('add_project_to_workspace', {
    workspace_id: workspaceId,
    project_id: projectId,
  })

export const removeProjectFromWorkspace = (workspaceId: string, projectId: string) =>
  invoke<void>('remove_project_from_workspace', {
    workspace_id: workspaceId,
    project_id: projectId,
  })

// Build presets
export const saveBuildPreset = (preset: BuildPreset) =>
  invoke<void>('save_build_preset', { preset })

export const getBuildPresets = (workspaceId: string) =>
  invoke<BuildPreset[]>('get_build_presets', { workspace_id: workspaceId })

export const runBuildPreset = (presetId: string) =>
  invoke<unknown>('run_build_preset', { preset_id: presetId })

// Build history / artifacts
export const getBuildHistory = (projectId?: string, limit = 20) =>
  invoke<BuildRecord[]>('get_build_history', { project_id: projectId, limit })

export const collectArtifacts = (projectPath: string) =>
  invoke<Artifact[]>('collect_artifacts', { project_path: projectPath })

// Deploy / environment
export const getDeployStatus = (workspaceId: string) =>
  invoke<DeployStatus>('get_deploy_status', { workspace_id: workspaceId })

export const checkEnvironment = () =>
  invoke<EnvironmentStatus>('check_environment')

// Process output streaming
export const listenProcessOutput = (
  cb: (event: ProcessOutputEvent) => void
) =>
  listen<ProcessOutputEvent>('forge://process-output', (e) => cb(e.payload))
