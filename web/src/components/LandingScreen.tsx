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
  onOpenWebsiteWizard: () => void
  onOpenCreateWizard: () => void
  onWorkspaceActive?: (workspaceId: string) => void
}

export default function LandingScreen({
  onSelectProject,
  onOpenWebsiteWizard,
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

  const handleOpenWebsite = () => {
    if (atLimit) return
    onOpenWebsiteWizard()
  }

  const hasProjects = projects.length > 0

  return (
    <div>
      <PageHeader
        title="My Apps"
        subtitle="Turn a website into a desktop app, then build and share it."
        actions={
          <>
            <Button
              variant="ghost"
              onClick={handleAddExisting}
              loading={adding}
            >
              Add existing
            </Button>
            <Button variant="secondary" onClick={handleOpenCreate}>
              New blank project
            </Button>
            <Button variant="primary" onClick={handleOpenWebsite}>
              Turn a website into an app
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
            icon="🌐"
            title="Make your first app"
            description="Have a website? Turn it into a desktop app in two steps — no coding needed. Just enter your web address and a name."
            action={
              <>
                <Button variant="primary" onClick={handleOpenWebsite}>
                  Turn a website into an app
                </Button>
                <Button
                  variant="ghost"
                  onClick={handleAddExisting}
                  loading={adding}
                >
                  Add an existing project
                </Button>
              </>
            }
          />
          <Card
            title="Is your computer ready?"
            subtitle="Forge builds apps right on your computer. Building installers needs these free tools — Forge will tell you how to add anything that's missing."
          >
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
              description="Building several apps at once is a Forge Pro feature."
            >
              <BuildOrchestrator workspaceId={activeWorkspace} />
            </LicenseGate>
          )}
        </div>
      )}
    </div>
  )
}
