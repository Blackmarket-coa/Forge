import React, { useState } from "react"
import LandingScreen from "./components/LandingScreen"
import ProjectView from "./components/ProjectView"
import Settings from "./components/Settings"
import { ProjectMeta } from "./api/api"
import ConfigEditor from "./components/ConfigEditor"

type View = "landing" | "project" | "config" | "settings"

export default function App() {
  const [activeProject, setActiveProject] = useState<ProjectMeta | null>(null)
  const [view, setView] = useState<View>("landing")

  const openProject = (project: ProjectMeta) => {
    setActiveProject(project)
    setView("project")
  }

  return (
    <div>
      <header style={{ display: "flex", justifyContent: "flex-end", padding: 12, gap: 8 }}>
        <button onClick={() => setView("landing")}>Home</button>
        <button onClick={() => setView("settings")}>Settings</button>
      </header>

      {view === "landing" && <LandingScreen onSelectProject={openProject} />}
      {view === "settings" && <Settings />}

      {view === "project" && activeProject && (
        <ProjectView
          project={activeProject}
          onBack={() => setView("landing")}
          onOpenConfig={() => setView("config")}
        />
      )}

      {view === "config" && activeProject && (
        <div>
          <div style={{ padding: "0 24px" }}>
            <button onClick={() => setView("project")}>← Back to Project</button>
          </div>
          <ConfigEditor projectPath={activeProject.path} />
        </div>
      )}
    </div>
  )
}
