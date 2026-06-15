import React, { useEffect, useMemo, useState } from "react"
import { getDeployStatus } from "../api/api"
import { Badge, Tone } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { EmptyState } from "./ui/empty-state"
import { PageHeader } from "./ui/page-header"
import { Progress } from "./ui/progress"
import { Spinner } from "./ui/spinner"
import styles from "./DeployDashboard.module.scss"

const PLATFORMS = ["macOS", "Linux", "Windows", "iOS", "Android"]

function statusView(value: string): { label: string; tone: Tone } {
  switch (value) {
    case "built":
      return { label: "Built", tone: "success" }
    case "configured":
      return { label: "Configured", tone: "warning" }
    case "not_started":
      return { label: "Missing", tone: "danger" }
    default:
      return { label: "—", tone: "neutral" }
  }
}

export default function DeployDashboard({
  workspaceId,
  onOpenProjectBuild,
}: {
  workspaceId: string
  onOpenProjectBuild: (projectId: string, platform: string) => void
}) {
  const [status, setStatus] = useState<any>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!workspaceId || workspaceId === "all") return
    setLoading(true)
    void (async () => {
      try {
        setStatus(await getDeployStatus(workspaceId))
      } finally {
        setLoading(false)
      }
    })()
  }, [workspaceId])

  const progress = useMemo(
    () => Number(status?.overall_progress || 0),
    [status]
  )
  const matrix = status?.matrix || []
  const blockers = status?.blockers || []
  const checklist = status?.checklist || []

  if (loading) {
    return (
      <div className={styles.loading}>
        <Spinner size={20} /> Checking release readiness…
      </div>
    )
  }

  return (
    <div>
      <PageHeader
        title="Deploy readiness"
        subtitle="Track how close each project is to shipping on every platform."
      />

      <Card>
        <div className={styles.progressRow}>
          <Progress value={progress} />
          <span className={styles.progressLabel}>
            {progress.toFixed(0)}% ready
          </span>
        </div>
      </Card>

      <div className={styles.matrixCard}>
        <Card title="Platform matrix" padded={false}>
          {matrix.length === 0 ? (
            <div className={styles.pad}>
              <EmptyState
                icon="🚀"
                title="No projects in this workspace"
                description="Add projects to the workspace to track deploy readiness."
              />
            </div>
          ) : (
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Project</th>
                  {PLATFORMS.map((p) => (
                    <th key={p}>{p}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {matrix.map((row: any) => (
                  <tr key={row.project_id}>
                    <td className={styles.projectCell}>{row.project_name}</td>
                    {PLATFORMS.map((p) => {
                      const view = statusView(
                        row?.statuses?.[p] || "not_targeted"
                      )
                      return (
                        <td key={p}>
                          <button
                            className={styles.cell}
                            onClick={() =>
                              onOpenProjectBuild(row.project_id, p)
                            }
                          >
                            <Badge tone={view.tone} dot>
                              {view.label}
                            </Badge>
                          </button>
                        </td>
                      )
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </Card>
      </div>

      <div className={styles.columns}>
        <Card title="Checklist">
          {checklist.length === 0 ? (
            <p className={styles.muted}>Nothing to check yet.</p>
          ) : (
            <ul className={styles.list}>
              {checklist.map((item: any, idx: number) => (
                <li key={idx} className={styles.listRow}>
                  <Badge
                    tone={item.tauri_initialized ? "success" : "danger"}
                    dot
                  >
                    {item.tauri_initialized ? "Init" : "No init"}
                  </Badge>
                  <span>{item.project}</span>
                  <Badge tone={item.config_ok ? "success" : "warning"}>
                    {item.config_ok ? "config ok" : "config needs review"}
                  </Badge>
                </li>
              ))}
            </ul>
          )}
        </Card>

        <Card title="Blockers">
          {blockers.length === 0 ? (
            <p className={styles.muted}>
              No blockers. You're clear to ship. 🎉
            </p>
          ) : (
            <ul className={styles.list}>
              {blockers.map((b: any, idx: number) => (
                <li key={idx} className={styles.blocker}>
                  <div className={styles.blockerMsg}>
                    <span className={styles.warnIcon}>⚠️</span>
                    {b.message}
                    {b.affected_project ? ` — ${b.affected_project}` : ""}
                  </div>
                  {b.fix_hint && (
                    <div className={styles.fixHint}>Fix: {b.fix_hint}</div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </Card>
      </div>
    </div>
  )
}
