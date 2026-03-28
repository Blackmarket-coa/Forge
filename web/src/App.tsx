import React, { useState } from "react"
import ConfigEditor from "./components/ConfigEditor"
import CreateProjectForm from "./components/CreateProjectForm"
import DeployDashboard from "./components/DeployDashboard"
import LandingScreen from "./components/LandingScreen"
import LicenseGate from "./components/LicenseGate"
import ProjectView from "./components/ProjectView"
import Settings from "./components/Settings"
import { getProjects, ProjectMeta } from "./api/api"

type View = "landing" | "project" | "config" | "settings" | "create" | "deploy"

export default function App() {
  const [activeProject, setActiveProject] = useState<ProjectMeta | null>(null)
  const [activeWorkspace, setActiveWorkspace] = useState<string>("all")
  const [view, setView] = useState<View>("landing")

  const openProject = (project: ProjectMeta) => {
    setActiveProject(project)
    setView("project")
  }

  const handleCreated = (project: ProjectMeta) => {
    setActiveProject(project)
    setView("project")
  }

  const openProjectById = async (projectId: string) => {
    const projects = await getProjects()
    const project = projects.find((p) => p.id === projectId)
    if (project) openProject(project)
  }

  return (
    <div>
      <header style={{ display: "flex", justifyContent: "flex-end", padding: 12, gap: 8 }}>
        <button onClick={() => setView("landing")}>Home</button>
        {activeWorkspace !== "all" && <button onClick={() => setView("deploy")}>Deploy</button>}
        <button onClick={() => setView("settings")}>Settings</button>
      </header>

      {view === "landing" && (
        <LandingScreen
          onSelectProject={openProject}
          onOpenCreateWizard={() => setView("create")}
          onWorkspaceActive={(id) => setActiveWorkspace(id)}
        />
      )}
      {view === "settings" && <Settings />}
      {view === "create" && (
        <CreateProjectForm onCreated={handleCreated} onCancel={() => setView("landing")} />
      )}
      {view === "deploy" && activeWorkspace !== "all" && (
        <LicenseGate
          feature="deploy_dashboard"
          description="Deploy dashboards and release readiness tracking are available on Forge Pro."
        >
          <DeployDashboard
            workspaceId={activeWorkspace}
            onOpenProjectBuild={(projectId, platform) => {
              void (async () => {
                await openProjectById(projectId)
                setView("project")
                void platform
              })()
            }}
          />
        </LicenseGate>
      )}

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
