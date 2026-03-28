import React from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { ProjectMeta } from "../api/api"
import { useAppState } from "../providers/AppStateProvider"

interface LandingScreenProps {
  onSelectProject: (project: ProjectMeta) => void
}

export default function LandingScreen({ onSelectProject }: LandingScreenProps) {
  const { projects, addProject } = useAppState()

  const handleAddProject = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      await addProject(selected)
    }
  }

  const handleCreateNewProject = () => {
    window.alert("Create New Project is coming soon.")
  }

  return (
    <div style={{ minHeight: "100vh", display: "grid", placeItems: "center", padding: 24 }}>
      <div style={{ width: "min(960px, 100%)" }}>
        <h1>Forge</h1>
        <p>The visual project manager for Tauri apps.</p>
        <div style={{ display: "flex", gap: 12, marginBottom: 24 }}>
          <button onClick={handleAddProject}>Add Project</button>
          <button onClick={handleCreateNewProject}>Create New Project</button>
        </div>

        <h2>Recent Projects</h2>
        <ul style={{ listStyle: "none", padding: 0, display: "grid", gap: 8 }}>
          {projects.map((project) => (
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
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <strong>{project.name}</strong>
                <span>{project.status}</span>
              </div>
              <div style={{ opacity: 0.8 }}>{project.path}</div>
            </li>
          ))}
          {projects.length === 0 && <li>No projects yet.</li>}
        </ul>
      </div>
    </div>
  )
}
