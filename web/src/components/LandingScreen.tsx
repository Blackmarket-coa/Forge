import React, { useEffect, useMemo, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { getProjects, ProjectMeta } from "../api/api"
import { LIMITS } from "../lib/tier"
import { useAppState } from "../providers/AppStateProvider"
import BuildOrchestrator from "./BuildOrchestrator"
import LicenseGate from "./LicenseGate"
import WorkspaceView from "./WorkspaceView"

interface LandingScreenProps {
  onSelectProject: (project: ProjectMeta) => void
  onOpenCreateWizard: () => void
  onWorkspaceActive?: (workspaceId: string) => void
}

export default function LandingScreen({ onSelectProject, onOpenCreateWizard, onWorkspaceActive }: LandingScreenProps) {
  const { projects, addProject, refreshProjects, tier } = useAppState()
  const [activeWorkspace, setActiveWorkspace] = useState<string>("all")
  const [projectCount, setProjectCount] = useState(0)
  const [projectLimitMessage, setProjectLimitMessage] = useState("")

  useEffect(() => {
    void refreshProjects()
    void (async () => {
      const rows = await getProjects()
      setProjectCount(rows.length)
    })()
  }, [refreshProjects])

  const sortedProjects = useMemo(
    () => [...projects].sort((a, b) => a.name.localeCompare(b.name)),
    [projects]
  )

  const enforceProjectLimit = () => {
    const maxProjects = LIMITS[tier].maxProjects
    if (projectCount >= maxProjects) {
      setProjectLimitMessage("You've reached the free tier limit of 2 projects")
      return false
    }
    setProjectLimitMessage("")
    return true
  }

  const handleAddExisting = async () => {
    if (!enforceProjectLimit()) return

    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      await addProject(selected)
      await refreshProjects()
      const rows = await getProjects()
      setProjectCount(rows.length)
    }
  }

  const handleOpenCreate = () => {
    if (!enforceProjectLimit()) return
    onOpenCreateWizard()
  }

  const hasProjects = projectCount > 0 || sortedProjects.length > 0

  return (
    <div style={{ minHeight: "100vh", display: "grid", placeItems: "center", padding: 24 }}>
      <div style={{ width: "min(1100px, 100%)" }}>
        <h1>Forge</h1>
        <p>The visual project manager for Tauri apps.</p>

        <div style={{ display: "flex", gap: 12, marginBottom: 24 }}>
          <button onClick={handleOpenCreate}>Create New Project</button>
          <button onClick={handleAddExisting}>Add Existing Project</button>
        </div>

        {projectLimitMessage && <p style={{ color: "#fca5a5" }}>{projectLimitMessage}</p>}

        {!hasProjects ? (
          <div style={{ border: "1px dashed #555", borderRadius: 8, padding: 24 }}>
            <h2>No projects yet</h2>
            <p>Create a new Tauri project or add an existing one to get started.</p>
            <div style={{ display: "flex", gap: 12 }}>
              <button onClick={handleOpenCreate}>Create New</button>
              <button onClick={handleAddExisting}>Add Existing</button>
            </div>
          </div>
        ) : (
          <div style={{ display: "grid", gap: 16 }}>
            <WorkspaceView
              onSelectProject={onSelectProject}
              onWorkspaceChange={(workspaceId) => {
                setActiveWorkspace(workspaceId)
                onWorkspaceActive?.(workspaceId)
              }}
            />
            {activeWorkspace !== "all" && (
              <LicenseGate feature="build_presets" description="Build presets and orchestration are available on Forge Pro.">
                <BuildOrchestrator workspaceId={activeWorkspace} />
              </LicenseGate>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
