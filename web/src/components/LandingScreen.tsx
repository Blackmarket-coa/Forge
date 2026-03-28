import React, { useEffect, useMemo } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { ProjectMeta } from "../api/api"
import { useAppState } from "../providers/AppStateProvider"

interface LandingScreenProps {
  onSelectProject: (project: ProjectMeta) => void
}

function statusColor(status: string) {
  if (status === "ready") return "#2e7d32"
  if (status === "needs_config") return "#f9a825"
  return "#c62828"
}

export default function LandingScreen({ onSelectProject }: LandingScreenProps) {
  const { projects, addProject, refreshProjects } = useAppState()

  useEffect(() => {
    void refreshProjects()
  }, [refreshProjects])

  const sortedProjects = useMemo(
    () => [...projects].sort((a, b) => a.name.localeCompare(b.name)),
    [projects]
  )

  const handleAddProject = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      await addProject(selected)
      await refreshProjects()
    }
  }

  const handleCreateNewProject = () => {
    window.alert("Create New Project is coming soon.")
  }

  return (
    <div style={{ minHeight: "100vh", display: "grid", placeItems: "center", padding: 24 }}>
      <div style={{ width: "min(1024px, 100%)" }}>
        <h1>Forge</h1>
        <p>The visual project manager for Tauri apps.</p>
        <div style={{ display: "flex", gap: 12, marginBottom: 24 }}>
          <button onClick={handleAddProject}>Add Project</button>
          <button onClick={handleCreateNewProject}>Create New Project</button>
        </div>

        <h2>Projects</h2>
        <ul style={{ listStyle: "none", padding: 0, display: "grid", gap: 8 }}>
          {sortedProjects.map((project) => (
            <li
              key={project.id}
              onClick={() => onSelectProject(project)}
              style={{
                border: "1px solid #444",
                borderRadius: 8,
                padding: 12,
                cursor: "pointer",
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", gap: 12, alignItems: "center" }}>
                <strong>{project.name || project.path.split("/").pop() || "Unnamed"}</strong>
                <div style={{ display: "flex", gap: 8 }}>
                  <span
                    style={{
                      border: "1px solid #666",
                      borderRadius: 999,
                      padding: "2px 10px",
                      fontSize: 12,
                    }}
                  >
                    {project.frontend_framework || "vanilla"}
                  </span>
                  <span
                    style={{
                      background: statusColor(project.status),
                      color: "white",
                      borderRadius: 999,
                      padding: "2px 10px",
                      fontSize: 12,
                    }}
                  >
                    {project.status}
                  </span>
                </div>
              </div>
              <div style={{ opacity: 0.85 }}>{project.path}</div>
              <div style={{ opacity: 0.7, marginTop: 4 }}>branch: {project.git_branch || "(no git branch)"}</div>
            </li>
          ))}
          {sortedProjects.length === 0 && <li>No projects yet.</li>}
        </ul>
      </div>
    </div>
  )
}
