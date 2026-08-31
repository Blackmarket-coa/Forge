import React, { useEffect, useMemo, useState } from "react"
import { useSnackbar } from "notistack"
import {
  addTarget,
  collectArtifacts,
  Framework,
  FrameworkInfo,
  getBuildHistory,
  getFrameworks,
  killProcess,
  openExternal,
  ProjectMeta,
  runBuild,
  runDev,
} from "../api/api"
import {
  FALLBACK_FRAMEWORKS,
  frameworkBadge,
  projectTargets,
} from "../lib/frameworks"
import Terminal from "./Terminal"
import LicenseGate from "./LicenseGate"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Checkbox } from "./ui/checkbox"
import { EmptyState } from "./ui/empty-state"
import { PageHeader } from "./ui/page-header"
import { Select } from "./ui/select"
import { Tabs } from "./ui/tabs"
import styles from "./ProjectView.module.scss"

interface ProjectViewProps {
  project: ProjectMeta
  onBack: () => void
  onOpenConfig: (framework: Framework) => void
}

export default function ProjectView({
  project,
  onBack,
  onOpenConfig,
}: ProjectViewProps) {
  const { enqueueSnackbar } = useSnackbar()
  const [proj, setProj] = useState<ProjectMeta>(project)
  const [frameworkInfos, setFrameworkInfos] =
    useState<FrameworkInfo[]>(FALLBACK_FRAMEWORKS)
  const [tab, setTab] = useState("output")
  const [isRunning, setIsRunning] = useState(false)
  const [building, setBuilding] = useState(false)
  const [addingTarget, setAddingTarget] = useState(false)
  const [processId, setProcessId] = useState("")
  const [targets, setTargets] = useState<string[]>([])
  const [addChoice, setAddChoice] = useState<Framework | "">("")
  const [buildResult, setBuildResult] = useState<any>(null)
  const [artifacts, setArtifacts] = useState<any[]>([])
  const [history, setHistory] = useState<any[]>([])

  const appTypes = useMemo(() => projectTargets(proj), [proj])
  const [activeFramework, setActiveFramework] = useState<Framework>(
    appTypes[0] || "tauri"
  )

  useEffect(() => {
    setProj(project)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project.id])

  useEffect(() => {
    if (appTypes.length > 0 && !appTypes.includes(activeFramework)) {
      setActiveFramework(appTypes[0])
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [appTypes])

  useEffect(() => {
    void getFrameworks()
      .then((list) => {
        if (list.length > 0) setFrameworkInfos(list)
      })
      .catch(() => {
        /* fall back to the static list */
      })
  }, [])

  const activeInfo = useMemo(
    () =>
      frameworkInfos.find((f) => f.id === activeFramework) ||
      FALLBACK_FRAMEWORKS[0],
    [frameworkInfos, activeFramework]
  )

  const refreshHistory = async () => {
    try {
      setHistory(await getBuildHistory(proj.id, 10))
    } catch {
      /* history is best-effort */
    }
  }

  const refreshArtifacts = async () => {
    try {
      setArtifacts(await collectArtifacts(proj.path))
    } catch {
      /* artifacts are best-effort */
    }
  }

  useEffect(() => {
    void refreshHistory()
    void refreshArtifacts()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [proj.id])

  // Reset the bundle-kind selection when switching app type.
  useEffect(() => {
    setTargets([])
    setBuildResult(null)
  }, [activeFramework])

  // Process ids look like `dev:{path}:{framework}` and
  // `build:{path}:{framework}:{kind}`, so matching on the project path shows
  // all activity for this project.
  const processIdPrefix = proj.path

  const handleDev = async () => {
    if (!activeInfo.dev_available) {
      const site = proj.source_url
      if (site) {
        await openExternal(site)
      } else {
        enqueueSnackbar("This app type has no live preview.", {
          variant: "info",
        })
      }
      return
    }
    try {
      const id = await runDev(proj.path, activeFramework)
      setProcessId(id)
      setTab("output")
      setIsRunning(true)
      enqueueSnackbar("Preview started", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Couldn't start preview: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const handleStartBuild = async () => {
    setBuilding(true)
    setTab("output")
    try {
      const result = await runBuild(proj.path, targets, activeFramework)
      setBuildResult(result)
      await refreshArtifacts()
      await refreshHistory()
      enqueueSnackbar(
        result?.status === "success" ? "Build finished" : "Build failed",
        { variant: result?.status === "success" ? "success" : "error" }
      )
    } catch (e: any) {
      enqueueSnackbar(`Build failed: ${e?.message || e}`, { variant: "error" })
    } finally {
      setBuilding(false)
    }
  }

  const handleStop = async () => {
    if (!processId) return
    try {
      await killProcess(processId)
      setIsRunning(false)
      enqueueSnackbar("Process stopped", { variant: "info" })
    } catch (e: any) {
      enqueueSnackbar(`Could not stop process: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const handleAddTarget = async () => {
    if (!addChoice) return
    setAddingTarget(true)
    try {
      const updated = await addTarget(proj.path, addChoice)
      setProj(updated)
      setActiveFramework(addChoice)
      setAddChoice("")
      enqueueSnackbar(
        `Added ${frameworkBadge(addChoice).short} to this project`,
        { variant: "success" }
      )
    } catch (e: any) {
      enqueueSnackbar(`Could not add app type: ${e?.message || e}`, {
        variant: "error",
      })
    } finally {
      setAddingTarget(false)
    }
  }

  const toggleTarget = (target: string, checked: boolean) => {
    setTargets((prev) =>
      checked
        ? prev.includes(target)
          ? prev
          : [...prev, target]
        : prev.filter((t) => t !== target)
    )
  }

  const availableToAdd = frameworkInfos.filter((f) => !appTypes.includes(f.id))

  const meta: Array<[string, React.ReactNode]> = [
    ["Folder", proj.path],
    ["Website", proj.source_url || "—"],
    ["App ID", proj.identifier || "unknown"],
    ["Built with", proj.frontend_framework || "website"],
  ]

  return (
    <div>
      <Button variant="ghost" size="sm" onClick={onBack}>
        ← Back to my apps
      </Button>

      <PageHeader
        title={proj.name}
        meta={
          <>
            {appTypes.map((fw) => {
              const badge = frameworkBadge(fw)
              const version = proj.targets?.find(
                (t) => t.framework === fw
              )?.version
              return (
                <Badge key={fw} tone="info">
                  {badge.emoji} {badge.short}
                  {version ? ` ${version}` : ""}
                </Badge>
              )
            })}
            {proj.git_branch && (
              <Badge tone="neutral" dot>
                {proj.git_branch}
              </Badge>
            )}
            <Badge tone={proj.git_dirty ? "warning" : "success"} dot>
              {proj.git_dirty ? "uncommitted changes" : "clean"}
            </Badge>
            {proj.status && <Badge tone="neutral">{proj.status}</Badge>}
          </>
        }
        actions={
          <>
            {isRunning ? (
              <Button variant="danger" onClick={handleStop}>
                Stop
              </Button>
            ) : (
              <Button variant="primary" onClick={handleDev}>
                {activeInfo.dev_available
                  ? activeInfo.dev_label
                  : "Open site in browser"}
              </Button>
            )}
            <Button
              variant="secondary"
              onClick={() => onOpenConfig(activeFramework)}
            >
              App settings
            </Button>
          </>
        }
      />

      {appTypes.length > 1 && (
        <div className={styles.targetTabs}>
          <Tabs
            value={activeFramework}
            onValueChange={(v) => setActiveFramework(v as Framework)}
            tabs={appTypes.map((fw) => ({
              value: fw,
              label: `${frameworkBadge(fw).emoji} ${frameworkBadge(fw).short}`,
            }))}
          />
        </div>
      )}

      <div className={styles.columns}>
        <div className={styles.main}>
          <Card padded={false}>
            <div className={styles.tabBar}>
              <Tabs
                value={tab}
                onValueChange={setTab}
                tabs={[
                  { value: "output", label: "Activity" },
                  {
                    value: "artifacts",
                    label: `Installers (${artifacts.length})`,
                  },
                  { value: "history", label: "Past builds" },
                ]}
              />
            </div>
            <div className={styles.tabBody}>
              {tab === "output" && (
                <Terminal processIdPrefix={processIdPrefix} />
              )}

              {tab === "artifacts" &&
                (artifacts.length === 0 ? (
                  <EmptyState
                    icon="📦"
                    title="No installers yet"
                    description="Build your app to create an installer you can share."
                  />
                ) : (
                  <ul className={styles.artifacts}>
                    {artifacts.map((artifact, idx) => (
                      <li key={idx} className={styles.artifactRow}>
                        <Badge tone="neutral">
                          {frameworkBadge(artifact.framework || "tauri").short}
                        </Badge>
                        <span className={styles.artifactPath}>
                          {artifact.path}
                        </span>
                        <span className={styles.artifactSize}>
                          {formatBytes(artifact.size_bytes)}
                        </span>
                      </li>
                    ))}
                  </ul>
                ))}

              {tab === "history" && (
                <LicenseGate
                  feature="build_history"
                  description="Seeing and re-running past builds is a Forge Pro feature."
                >
                  {history.length === 0 ? (
                    <EmptyState
                      icon="🕓"
                      title="No builds yet"
                      description="Each time you build your app, it'll show up here."
                    />
                  ) : (
                    <table className={styles.table}>
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>App type</th>
                          <th>Targets</th>
                          <th>Status</th>
                          <th>Duration</th>
                          <th>Artifacts</th>
                          <th />
                        </tr>
                      </thead>
                      <tbody>
                        {history.map((row) => (
                          <tr key={row.id}>
                            <td>{row.started_at}</td>
                            <td>
                              {frameworkBadge(row.framework || "tauri").short}
                            </td>
                            <td>{(row.targets || []).join(", ")}</td>
                            <td>
                              <Badge
                                tone={
                                  row.status === "success"
                                    ? "success"
                                    : "danger"
                                }
                                dot
                              >
                                {row.status}
                              </Badge>
                            </td>
                            <td>{row.duration_secs}s</td>
                            <td>{(row.artifacts || []).length}</td>
                            <td>
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => {
                                  const fw = (row.framework ||
                                    "tauri") as Framework
                                  if (appTypes.includes(fw)) {
                                    setActiveFramework(fw)
                                  }
                                  setTargets(row.targets || [])
                                }}
                              >
                                Re-run
                              </Button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  )}
                </LicenseGate>
              )}
            </div>
          </Card>
        </div>

        <div className={styles.side}>
          <Card
            title={`Build: ${activeInfo.label}`}
            subtitle="Pick which kinds of output to create, then build."
          >
            <div className={styles.targets}>
              {activeInfo.bundle_kinds.length === 0 ? (
                <span className={styles.muted}>
                  No build options available for this app type.
                </span>
              ) : (
                activeInfo.bundle_kinds.map((kind) => (
                  <div key={kind.id} className={styles.targetRow}>
                    <Checkbox
                      label={kind.label}
                      checked={targets.includes(kind.id)}
                      onChange={(e) => toggleTarget(kind.id, e.target.checked)}
                    />
                    {kind.note && (
                      <span className={styles.targetNote}>{kind.note}</span>
                    )}
                  </div>
                ))
              )}
            </div>
            <Button
              variant="primary"
              fullWidth
              loading={building}
              disabled={targets.length === 0}
              onClick={handleStartBuild}
            >
              {building ? "Building…" : "Build installer"}
            </Button>
            {buildResult && (
              <div className={styles.buildResult}>
                <Badge
                  tone={buildResult.status === "success" ? "success" : "danger"}
                >
                  {buildResult.status}
                </Badge>{" "}
                {buildResult.duration_secs?.toFixed?.(2)}s
              </div>
            )}
          </Card>

          <Card title="Details">
            <dl className={styles.details}>
              {meta.map(([label, value]) => (
                <React.Fragment key={label}>
                  <dt>{label}</dt>
                  <dd title={typeof value === "string" ? value : undefined}>
                    {value}
                  </dd>
                </React.Fragment>
              ))}
            </dl>
            <div className={styles.platforms}>
              {(proj.platforms || []).map((platform) => (
                <Badge key={platform} tone="neutral">
                  {platform}
                </Badge>
              ))}
              {(proj.platforms || []).length === 0 && (
                <span className={styles.muted}>No platforms detected</span>
              )}
            </div>
          </Card>

          {availableToAdd.length > 0 && (
            <Card
              title="Add another app type"
              subtitle="Give this website more ways to run — same address, new app."
            >
              <div className={styles.addTargetRow}>
                <Select
                  value={addChoice}
                  onChange={(e) => setAddChoice(e.target.value as Framework)}
                >
                  <option value="">Choose an app type…</option>
                  {availableToAdd.map((f) => (
                    <option key={f.id} value={f.id}>
                      {f.label}
                    </option>
                  ))}
                </Select>
                <Button
                  variant="secondary"
                  disabled={!addChoice}
                  loading={addingTarget}
                  onClick={handleAddTarget}
                >
                  Add
                </Button>
              </div>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}

function formatBytes(bytes?: number): string {
  if (!bytes || bytes <= 0) return "—"
  const units = ["B", "KB", "MB", "GB"]
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit++
  }
  return `${value.toFixed(value < 10 && unit > 0 ? 1 : 0)} ${units[unit]}`
}
