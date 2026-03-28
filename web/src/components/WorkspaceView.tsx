import React, { useEffect, useState } from "react"
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

export default function WorkspaceView({
  onSelectProject,
  onWorkspaceChange,
}: {
  onSelectProject: (project: ProjectMeta) => void
  onWorkspaceChange?: (workspaceId: string) => void
}) {
  const { tier } = useAppState()
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [activeWorkspace, setActiveWorkspace] = useState<string>("all")
  const [newName, setNewName] = useState("")
  const [newColor, setNewColor] = useState("#64748b")

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
    const ws = await createWorkspace(newName.trim())
    await Promise.resolve(ws)
    setNewName("")
    setNewColor("#64748b")
    await refresh()
  }

  const filtered =
    activeWorkspace === "all"
      ? projects
      : projects.filter((p) => p.workspace_id === activeWorkspace)

  return (
    <div style={{ display: "grid", gap: 12 }}>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
        <button
          style={{ fontWeight: activeWorkspace === "all" ? 700 : 400 }}
          onClick={() => {
            setActiveWorkspace("all")
            onWorkspaceChange?.("all")
          }}
        >
          All Projects
        </button>
        {workspaces.map((ws) => (
          <button
            key={ws.id}
            style={{
              fontWeight: activeWorkspace === ws.id ? 700 : 400,
              borderColor: ws.color || "#666",
            }}
            onClick={() => {
              setActiveWorkspace(ws.id)
              onWorkspaceChange?.(ws.id)
            }}
          >
            {ws.name}
          </button>
        ))}
      </div>

      {isFeatureAvailable("workspaces", tier) ? (
        <div style={{ display: "flex", gap: 8 }}>
          <input placeholder="New workspace" value={newName} onChange={(e) => setNewName(e.target.value)} />
          <input type="color" value={newColor} onChange={(e) => setNewColor(e.target.value)} />
          <button onClick={create}>New Workspace</button>
        </div>
      ) : (
        <LicenseGate feature="workspaces" description="Workspace creation is a Forge Pro feature.">
          <div style={{ display: "flex", gap: 8 }}>
            <input placeholder="New workspace" disabled value={newName} onChange={(e) => setNewName(e.target.value)} />
            <input type="color" disabled value={newColor} onChange={(e) => setNewColor(e.target.value)} />
            <button disabled onClick={create}>New Workspace</button>
          </div>
        </LicenseGate>
      )}

      <div style={{ display: "grid", gap: 8 }}>
        {filtered.map((project) => (
          <div key={project.id} style={{ border: "1px solid #444", borderRadius: 8, padding: 12 }}>
            <div style={{ display: "flex", justifyContent: "space-between", gap: 8 }}>
              <strong onClick={() => onSelectProject(project)} style={{ cursor: "pointer" }}>
                {project.name}
              </strong>
              <select
                value={project.workspace_id || ""}
                onChange={async (e) => {
                  const next = e.target.value
                  if (!next) return
                  await addProjectToWorkspace(next, project.id)
                  await refresh()
                }}
              >
                <option value="">Add to Workspace</option>
                {workspaces.map((ws) => (
                  <option key={ws.id} value={ws.id}>
                    {ws.name}
                  </option>
                ))}
              </select>
            </div>
            <div>{project.path}</div>
          </div>
        ))}
      </div>
    </div>
  )
}
