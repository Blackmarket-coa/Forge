import React, { createContext, useCallback, useContext, useMemo, useState } from "react"
import { getProjects, ProjectMeta, registerProject, Workspace } from "../api/api"

type Theme = "dark" | "light"

interface AppStateValue {
  projects: ProjectMeta[]
  workspaces: Workspace[]
  activeWorkspace: string | null
  theme: Theme
  addProject: (path: string) => Promise<void>
  removeProject: (id: string) => void
  refreshProjects: () => Promise<void>
  setActiveWorkspace: (workspaceId: string | null) => void
  toggleTheme: () => void
}

const AppStateContext = createContext<AppStateValue | undefined>(undefined)

export function AppStateProvider({ children }: { children: React.ReactNode }) {
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [activeWorkspace, setActiveWorkspace] = useState<string | null>(null)
  const [theme, setTheme] = useState<Theme>("dark")

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

  const value = useMemo(
    () => ({
      projects,
      workspaces,
      activeWorkspace,
      theme,
      addProject,
      removeProject,
      refreshProjects,
      setActiveWorkspace,
      toggleTheme,
    }),
    [projects, workspaces, activeWorkspace, theme, addProject, removeProject, refreshProjects, toggleTheme]
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
