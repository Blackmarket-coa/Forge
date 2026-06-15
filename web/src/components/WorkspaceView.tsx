import React, { useEffect, useState } from "react"
import { useSnackbar } from "notistack"
import {
  addProjectToWorkspace,
  createWorkspace,
  getProjects,
  getWorkspaces,
  ProjectMeta,
  Workspace,
} from "../api/api"
import { isFeatureAvailable } from "../lib/tier"
import { useAppState } from "../providers/AppStateProvider"
import LicenseGate from "./LicenseGate"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Input } from "./ui/input"
import { Select } from "./ui/select"
import styles from "./WorkspaceView.module.scss"

export default function WorkspaceView({
  onSelectProject,
  onWorkspaceChange,
}: {
  onSelectProject: (project: ProjectMeta) => void
  onWorkspaceChange?: (workspaceId: string) => void
}) {
  const { tier } = useAppState()
  const { enqueueSnackbar } = useSnackbar()
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [activeWorkspace, setActiveWorkspace] = useState<string>("all")
  const [newName, setNewName] = useState("")
  const [newColor, setNewColor] = useState("#f5853f")

  const refresh = async () => {
    const [ws, ps] = await Promise.all([getWorkspaces(), getProjects()])
    setWorkspaces(ws)
    setProjects(ps)
  }

  useEffect(() => {
    void refresh()
  }, [])

  const create = async () => {
    if (!newName.trim()) return
    try {
      await createWorkspace(newName.trim())
      setNewName("")
      setNewColor("#f5853f")
      await refresh()
      enqueueSnackbar("Workspace created", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Could not create workspace: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const selectWorkspace = (id: string) => {
    setActiveWorkspace(id)
    onWorkspaceChange?.(id)
  }

  const filtered =
    activeWorkspace === "all"
      ? projects
      : projects.filter((p) => p.workspace_id === activeWorkspace)

  const canManageWorkspaces = isFeatureAvailable("workspaces", tier)

  const workspaceForm = (
    <div className={styles.createRow}>
      <Input
        placeholder="New workspace name"
        value={newName}
        disabled={!canManageWorkspaces}
        onChange={(e) => setNewName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && create()}
      />
      <input
        type="color"
        className={styles.colorInput}
        value={newColor}
        disabled={!canManageWorkspaces}
        onChange={(e) => setNewColor(e.target.value)}
      />
      <Button
        variant="secondary"
        onClick={create}
        disabled={!canManageWorkspaces || !newName.trim()}
      >
        Add
      </Button>
    </div>
  )

  return (
    <div className={styles.root}>
      <div className={styles.toolbar}>
        <div className={styles.pills}>
          <button
            className={[
              styles.pill,
              activeWorkspace === "all" ? styles.pillActive : "",
            ]
              .filter(Boolean)
              .join(" ")}
            onClick={() => selectWorkspace("all")}
          >
            All Projects
          </button>
          {workspaces.map((ws) => (
            <button
              key={ws.id}
              className={[
                styles.pill,
                activeWorkspace === ws.id ? styles.pillActive : "",
              ]
                .filter(Boolean)
                .join(" ")}
              style={{
                borderColor:
                  activeWorkspace === ws.id ? ws.color || undefined : undefined,
              }}
              onClick={() => selectWorkspace(ws.id)}
            >
              <span
                className={styles.dot}
                style={{ background: ws.color || "var(--color-text-subtle)" }}
              />
              {ws.name}
            </button>
          ))}
        </div>

        {canManageWorkspaces ? (
          workspaceForm
        ) : (
          <LicenseGate
            feature="workspaces"
            description="Workspace creation is a Forge Pro feature."
          >
            {workspaceForm}
          </LicenseGate>
        )}
      </div>

      <div className={styles.grid}>
        {filtered.map((project) => (
          <Card key={project.id} interactive padded={false}>
            <div className={styles.cardBody}>
              <button
                className={styles.cardTitle}
                onClick={() => onSelectProject(project)}
              >
                {project.name}
              </button>
              <div className={styles.cardPath} title={project.path}>
                {project.path}
              </div>
              <div className={styles.cardMeta}>
                {project.tauri_version && (
                  <Badge tone="info">Tauri {project.tauri_version}</Badge>
                )}
                {project.git_branch && (
                  <Badge tone="neutral">{project.git_branch}</Badge>
                )}
                {project.git_dirty && <Badge tone="warning">uncommitted</Badge>}
              </div>
              <div className={styles.cardFooter}>
                <Select
                  value={project.workspace_id || ""}
                  onChange={async (e) => {
                    const next = e.target.value
                    if (!next) return
                    await addProjectToWorkspace(next, project.id)
                    await refresh()
                  }}
                >
                  <option value="">Add to workspace…</option>
                  {workspaces.map((ws) => (
                    <option key={ws.id} value={ws.id}>
                      {ws.name}
                    </option>
                  ))}
                </Select>
                <Button size="sm" onClick={() => onSelectProject(project)}>
                  Open
                </Button>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )
}
