import React, { useMemo, useState } from "react"
import AppShell, { NavItem } from "./components/AppShell"
import ConfigEditor from "./components/ConfigEditor"
import CreateProjectForm from "./components/CreateProjectForm"
import DeployDashboard from "./components/DeployDashboard"
import LandingScreen from "./components/LandingScreen"
import LicenseGate from "./components/LicenseGate"
import ProjectView from "./components/ProjectView"
import Settings from "./components/Settings"
import { getProjects, ProjectMeta } from "./api/api"
import { useAppState } from "./providers/AppStateProvider"

type View = "landing" | "project" | "config" | "settings" | "create" | "deploy"

export default function App() {
  const { theme, toggleTheme, tier } = useAppState()
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

  const hasWorkspace = activeWorkspace !== "all"

  const nav: NavItem[] = useMemo(
    () => [
      { id: "projects", label: "Projects", icon: "📁" },
      {
        id: "deploy",
        label: "Deploy",
        icon: "🚀",
        disabled: !hasWorkspace,
        hint: "Select a workspace to view deploy readiness",
      },
      { id: "settings", label: "Settings", icon: "⚙️" },
    ],
    [hasWorkspace]
  )

  const activeNav =
    view === "deploy" ? "deploy" : view === "settings" ? "settings" : "projects"

  const onNavigate = (id: string) => {
    if (id === "projects") setView("landing")
    else if (id === "deploy" && hasWorkspace) setView("deploy")
    else if (id === "settings") setView("settings")
  }

  return (
    <AppShell
      nav={nav}
      activeNav={activeNav}
      onNavigate={onNavigate}
      tier={tier}
      theme={theme}
      onToggleTheme={toggleTheme}
    >
      {view === "landing" && (
        <LandingScreen
          onSelectProject={openProject}
          onOpenCreateWizard={() => setView("create")}
          onWorkspaceActive={(id) => setActiveWorkspace(id)}
        />
      )}

      {view === "settings" && <Settings />}

      {view === "create" && (
        <CreateProjectForm
          onCreated={handleCreated}
          onCancel={() => setView("landing")}
        />
      )}

      {view === "deploy" && hasWorkspace && (
        <LicenseGate
          feature="deploy_dashboard"
          description="Deploy dashboards and release readiness tracking are available on Forge Pro."
        >
          <DeployDashboard
            workspaceId={activeWorkspace}
            onOpenProjectBuild={(projectId) => {
              void (async () => {
                await openProjectById(projectId)
                setView("project")
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
        <ConfigEditor
          projectPath={activeProject.path}
          projectName={activeProject.name}
          onBack={() => setView("project")}
        />
      )}
    </AppShell>
  )
}
