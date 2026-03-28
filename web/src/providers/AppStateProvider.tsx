import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react"
import {
  clearLicense,
  getLicenseStatus,
  getProjects,
  LicenseStatus,
  ProjectMeta,
  registerProject,
  validateLicense,
  Workspace,
} from "../api/api"
import { Tier } from "../lib/tier"

type Theme = "dark" | "light"

interface AppStateValue {
  projects: ProjectMeta[]
  workspaces: Workspace[]
  activeWorkspace: string | null
  theme: Theme
  tier: Tier
  licenseStatus: LicenseStatus
  addProject: (path: string) => Promise<void>
  removeProject: (id: string) => void
  refreshProjects: () => Promise<void>
  setActiveWorkspace: (workspaceId: string | null) => void
  toggleTheme: () => void
  activateLicense: (key: string) => Promise<LicenseStatus>
  clearLicenseKey: () => Promise<LicenseStatus>
  refreshLicenseStatus: () => Promise<void>
}

const defaultLicenseStatus: LicenseStatus = {
  tier: "free",
  valid: false,
  expires_at: null,
  key_masked: null,
  checked_at: null,
}

const AppStateContext = createContext<AppStateValue | undefined>(undefined)

export function AppStateProvider({ children }: { children: React.ReactNode }) {
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [activeWorkspace, setActiveWorkspace] = useState<string | null>(null)
  const [theme, setTheme] = useState<Theme>("dark")
  const [tier, setTier] = useState<Tier>("free")
  const [licenseStatus, setLicenseStatus] = useState<LicenseStatus>(defaultLicenseStatus)

  const addProject = useCallback(async (path: string) => {
    const project = await registerProject(path)
    setProjects((prev) => {
      if (prev.some((p) => p.id === project.id)) {
        return prev
      }
      return [...prev, project]
    })
  }, [])

  const removeProject = useCallback((id: string) => {
    setProjects((prev) => prev.filter((p) => p.id !== id))
    setWorkspaces((prev) =>
      prev.map((w) => ({ ...w, project_ids: w.project_ids.filter((pid) => pid !== id) }))
    )
  }, [])

  const refreshProjects = useCallback(async () => {
    const loaded = await getProjects(activeWorkspace ?? undefined)
    setProjects(loaded)
  }, [activeWorkspace])

  const toggleTheme = useCallback(() => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"))
  }, [])

  const refreshLicenseStatus = useCallback(async () => {
    const status = await getLicenseStatus()
    setTier(status.tier)
    setLicenseStatus(status)
  }, [])

  const activateLicense = useCallback(async (key: string) => {
    const status = await validateLicense(key)
    setTier(status.tier)
    setLicenseStatus(status)
    return status
  }, [])

  const clearLicenseKey = useCallback(async () => {
    const status = await clearLicense()
    setTier(status.tier)
    setLicenseStatus(status)
    return status
  }, [])

  useEffect(() => {
    void refreshLicenseStatus()
  }, [refreshLicenseStatus])

  const value = useMemo(
    () => ({
      projects,
      workspaces,
      activeWorkspace,
      theme,
      tier,
      licenseStatus,
      addProject,
      removeProject,
      refreshProjects,
      setActiveWorkspace,
      toggleTheme,
      activateLicense,
      clearLicenseKey,
      refreshLicenseStatus,
    }),
    [
      projects,
      workspaces,
      activeWorkspace,
      theme,
      tier,
      licenseStatus,
      addProject,
      removeProject,
      refreshProjects,
      toggleTheme,
      activateLicense,
      clearLicenseKey,
      refreshLicenseStatus,
    ]
  )

  return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>
}

export function useAppState() {
  const context = useContext(AppStateContext)
  if (!context) {
    throw new Error("useAppState must be used within AppStateProvider")
  }
  return context
}
