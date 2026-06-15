import React, { useEffect, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { useSnackbar } from "notistack"
import { LIMITS } from "../lib/tier"
import { useAppState } from "../providers/AppStateProvider"
import { ProjectMeta } from "../api/api"
import BuildOrchestrator from "./BuildOrchestrator"
import EnvironmentCheck from "./EnvironmentCheck"
import LicenseGate from "./LicenseGate"
import WorkspaceView from "./WorkspaceView"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { EmptyState } from "./ui/empty-state"
import { PageHeader } from "./ui/page-header"
import styles from "./LandingScreen.module.scss"

interface LandingScreenProps {
  onSelectProject: (project: ProjectMeta) => void
  onOpenCreateWizard: () => void
  onWorkspaceActive?: (workspaceId: string) => void
}

export default function LandingScreen({
  onSelectProject,
  onOpenCreateWizard,
  onWorkspaceActive,
}: LandingScreenProps) {
  const { projects, addProject, refreshProjects, tier } = useAppState()
  const { enqueueSnackbar } = useSnackbar()
  const [activeWorkspace, setActiveWorkspace] = useState<string>("all")
  const [adding, setAdding] = useState(false)

  useEffect(() => {
    void refreshProjects()
  }, [refreshProjects])

  const atLimit = projects.length >= LIMITS[tier].maxProjects

  const handleAddExisting = async () => {
    if (atLimit) return
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected !== "string") return
    setAdding(true)
    try {
      await addProject(selected)
      await refreshProjects()
      enqueueSnackbar("Project added", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Could not add project: ${e?.message || e}`, {
        variant: "error",
      })
    } finally {
      setAdding(false)
    }
  }

  const handleOpenCreate = () => {
    if (atLimit) return
    onOpenCreateWizard()
  }

  const hasProjects = projects.length > 0

  return (
    <div>
      <PageHeader
        title="Projects"
        subtitle="Discover, build, and ship your Tauri apps."
        actions={
          <>
            <Button
              variant="secondary"
              onClick={handleAddExisting}
              loading={adding}
            >
              Add Existing
            </Button>
            <Button variant="primary" onClick={handleOpenCreate}>
              Create New Project
            </Button>
          </>
        }
      />

      {atLimit && (
        <div className={styles.banner}>
          <Banner
            tone="warning"
            title={`You've reached the ${tier} tier limit of ${LIMITS[tier].maxProjects} projects`}
          >
            Upgrade to Forge Pro for unlimited projects.
          </Banner>
        </div>
      )}

      {!hasProjects ? (
        <div className={styles.onboarding}>
          <EmptyState
            icon="📦"
            title="No projects yet"
            description="Create a new Tauri project or add an existing one to get started."
            action={
              <>
                <Button variant="primary" onClick={handleOpenCreate}>
                  Create New
                </Button>
                <Button
                  variant="secondary"
                  onClick={handleAddExisting}
                  loading={adding}
                >
                  Add Existing
                </Button>
              </>
            }
          />
          <Card title="Before you build">
            <p className={styles.onboardingHint}>
              Forge runs your Tauri builds locally. Make sure your toolchain is
              ready:
            </p>
            <EnvironmentCheck />
          </Card>
        </div>
      ) : (
        <div className={styles.stack}>
          <WorkspaceView
            onSelectProject={onSelectProject}
            onWorkspaceChange={(workspaceId) => {
              setActiveWorkspace(workspaceId)
              onWorkspaceActive?.(workspaceId)
            }}
          />
          {activeWorkspace !== "all" && (
            <LicenseGate
              feature="build_presets"
              description="Build presets and orchestration are available on Forge Pro."
            >
              <BuildOrchestrator workspaceId={activeWorkspace} />
            </LicenseGate>
          )}
        </div>
      )}
    </div>
  )
}
