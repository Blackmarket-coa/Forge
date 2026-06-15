import React, { useEffect, useState } from "react"
import { useSnackbar } from "notistack"
import {
  collectArtifacts,
  getBuildHistory,
  killProcess,
  ProjectMeta,
  runBuild,
  runDev,
} from "../api/api"
import Terminal from "./Terminal"
import LicenseGate from "./LicenseGate"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Checkbox } from "./ui/checkbox"
import { EmptyState } from "./ui/empty-state"
import { PageHeader } from "./ui/page-header"
import { Tabs } from "./ui/tabs"
import styles from "./ProjectView.module.scss"

interface ProjectViewProps {
  project: ProjectMeta
  onBack: () => void
  onOpenConfig: () => void
}

const BUILD_TARGETS = ["dmg", "appimage", "deb", "nsis", "msi"] as const

export default function ProjectView({
  project,
  onBack,
  onOpenConfig,
}: ProjectViewProps) {
  const { enqueueSnackbar } = useSnackbar()
  const [tab, setTab] = useState("output")
  const [isRunning, setIsRunning] = useState(false)
  const [building, setBuilding] = useState(false)
  const [processId, setProcessId] = useState("")
  const [targets, setTargets] = useState<string[]>([])
  const [buildResult, setBuildResult] = useState<any>(null)
  const [artifacts, setArtifacts] = useState<any[]>([])
  const [history, setHistory] = useState<any[]>([])

  const refreshHistory = async () => {
    try {
      setHistory(await getBuildHistory(project.id, 10))
    } catch {
      /* history is best-effort */
    }
  }

  useEffect(() => {
    void refreshHistory()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project.id])

  // Process ids are like `dev:{path}` and `build:{path}:{target}`, so matching
  // on the project path shows both dev and build output for this project.
  const processIdPrefix = project.path

  const handleDev = async () => {
    try {
      const pid = await runDev(project.path)
      setProcessId(`dev:${project.path}`)
      setTab("output")
      setIsRunning(true)
      enqueueSnackbar(`Dev server started (pid ${pid})`, { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Failed to start dev: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const handleStartBuild = async () => {
    setBuilding(true)
    setTab("output")
    try {
      const result = await runBuild(project.path, targets)
      setBuildResult(result)
      setArtifacts(result?.artifacts || (await collectArtifacts(project.path)))
      await refreshHistory()
      enqueueSnackbar("Build finished", { variant: "success" })
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

  const toggleTarget = (target: string, checked: boolean) => {
    setTargets((prev) =>
      checked
        ? prev.includes(target)
          ? prev
          : [...prev, target]
        : prev.filter((t) => t !== target)
    )
  }

  const meta: Array<[string, React.ReactNode]> = [
    ["Path", project.path],
    ["Identifier", project.identifier || "unknown"],
    ["Framework", project.frontend_framework || "vanilla"],
    ["Role", project.role || "—"],
  ]

  return (
    <div>
      <Button variant="ghost" size="sm" onClick={onBack}>
        ← Back to Projects
      </Button>

      <PageHeader
        title={project.name}
        meta={
          <>
            {project.tauri_version && (
              <Badge tone="info">Tauri {project.tauri_version}</Badge>
            )}
            {project.git_branch && (
              <Badge tone="neutral" dot>
                {project.git_branch}
              </Badge>
            )}
            <Badge tone={project.git_dirty ? "warning" : "success"} dot>
              {project.git_dirty ? "uncommitted changes" : "clean"}
            </Badge>
            {project.status && <Badge tone="neutral">{project.status}</Badge>}
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
                Run Dev
              </Button>
            )}
            <Button variant="secondary" onClick={onOpenConfig}>
              Config
            </Button>
          </>
        }
      />

      <div className={styles.columns}>
        <div className={styles.main}>
          <Card padded={false}>
            <div className={styles.tabBar}>
              <Tabs
                value={tab}
                onValueChange={setTab}
                tabs={[
                  { value: "output", label: "Output" },
                  {
                    value: "artifacts",
                    label: `Artifacts (${artifacts.length})`,
                  },
                  { value: "history", label: "History" },
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
                    title="No artifacts yet"
                    description="Run a build to produce installable artifacts."
                  />
                ) : (
                  <ul className={styles.artifacts}>
                    {artifacts.map((artifact, idx) => (
                      <li key={idx} className={styles.artifactRow}>
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
                  description="Build history and rerun tracking are available on Forge Pro."
                >
                  {history.length === 0 ? (
                    <EmptyState
                      icon="🕓"
                      title="No build history"
                      description="Past builds for this project will appear here."
                    />
                  ) : (
                    <table className={styles.table}>
                      <thead>
                        <tr>
                          <th>Date</th>
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
                                onClick={() => setTargets(row.targets || [])}
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
          <Card title="Build">
            <div className={styles.targets}>
              {BUILD_TARGETS.map((target) => (
                <Checkbox
                  key={target}
                  label={target}
                  checked={targets.includes(target)}
                  onChange={(e) => toggleTarget(target, e.target.checked)}
                />
              ))}
            </div>
            <Button
              variant="primary"
              fullWidth
              loading={building}
              disabled={targets.length === 0}
              onClick={handleStartBuild}
            >
              {building ? "Building…" : "Start Build"}
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
              {(project.platforms || []).map((platform) => (
                <Badge key={platform} tone="neutral">
                  {platform}
                </Badge>
              ))}
              {(project.platforms || []).length === 0 && (
                <span className={styles.muted}>No platforms detected</span>
              )}
            </div>
          </Card>
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
