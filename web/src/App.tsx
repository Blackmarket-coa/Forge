import React, { useState } from "react"
import LandingScreen from "./components/LandingScreen"
import ProjectView from "./components/ProjectView"
import Settings from "./components/Settings"
import { ProjectMeta } from "./api/api"

export default function App() {
  const [selectedProject, setSelectedProject] = useState<ProjectMeta | null>(null)
  const [showSettings, setShowSettings] = useState(false)

  return (
    <div>
      <header style={{ display: "flex", justifyContent: "flex-end", padding: 12 }}>
        <button onClick={() => setShowSettings((prev) => !prev)}>
          {showSettings ? "Close Settings" : "Settings"}
        </button>
      </header>

      {showSettings && <Settings />}
      {!showSettings && !selectedProject && <LandingScreen onSelectProject={setSelectedProject} />}
      {!showSettings && selectedProject && (
        <ProjectView project={selectedProject} onBack={() => setSelectedProject(null)} />
      )}
    </div>
  )
}
