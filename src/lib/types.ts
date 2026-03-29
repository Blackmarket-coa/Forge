export interface ProjectMeta {
  id: string;
  name: string;
  path: string;
  workspace_id?: string;
  tauri_version?: string;
  identifier?: string;
  frontend_framework?: string;
  platforms: string[];
  git_branch?: string;
  git_dirty: boolean;
  status: string;
  tags: string[];
  role?: string;
}

export interface Workspace {
  id: string;
  name: string;
  project_ids: string[];
  color?: string;
}

export interface BuildPreset {
  id: string;
  name: string;
  workspace_id: string;
  steps: BuildStep[];
}

export interface BuildStep {
  project_id: string;
  targets: string[];
  parallel_with_next: boolean;
}

export interface Artifact {
  path: string;
  size_bytes: number;
  format: string;
  created_at: number;
}

export interface BuildRecord {
  id: string;
  project_id: string;
  targets: string[];
  status: string;
  started_at: string;
  duration_secs: number;
  artifacts: Artifact[];
  log_path: string;
}

export interface LicenseStatus {
  valid: boolean;
  tier: string;
  expires_at?: string;
}

export interface TauriStatus {
  has_tauri_conf: boolean;
  product_name?: string;
  identifier?: string;
  version?: string;
  tauri_version?: string;
  frontend_framework?: string;
  status: string;
}

export interface EnvironmentCheck {
  installed: boolean;
  version: string;
}

export interface EnvironmentStatus {
  rust: EnvironmentCheck;
  cargo: EnvironmentCheck;
  node: EnvironmentCheck;
  tauri_cli: EnvironmentCheck;
  platform_deps: { name: string; installed: boolean }[];
}

export interface BuildResult {
  status: string;
  duration_secs: number;
  artifacts: Artifact[];
}

export interface DeployStatus {
  workspace_id: string;
  overall_progress: number;
  matrix: unknown[];
  checklist: unknown[];
  blockers: unknown[];
}

export interface ProcessOutputEvent {
  process_id: string;
  data: string;
  is_stderr: boolean;
}
