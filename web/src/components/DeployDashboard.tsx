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
      return { label: "Ready", tone: "success" }
    case "configured":
      return { label: "Set up", tone: "warning" }
    case "not_started":
      return { label: "Not built", tone: "danger" }
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
        <Spinner size={20} /> Checking what's ready…
      </div>
    )
  }

  return (
    <div>
      <PageHeader
        title="Getting ready to publish"
        subtitle="See how close each app is to being ready on every kind of computer."
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
        <Card title="Where your apps run" padded={false}>
          {matrix.length === 0 ? (
            <div className={styles.pad}>
              <EmptyState
                icon="🚀"
                title="No apps in this group"
                description="Add apps to this group to see how ready they are to publish."
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
        <Card title="Readiness checklist">
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
                    {item.tauri_initialized ? "Set up" : "Not set up"}
                  </Badge>
                  <span>{item.project}</span>
                  <Badge tone={item.config_ok ? "success" : "warning"}>
                    {item.config_ok ? "settings ok" : "check settings"}
                  </Badge>
                </li>
              ))}
            </ul>
          )}
        </Card>

        <Card title="Things to fix">
          {blockers.length === 0 ? (
            <p className={styles.muted}>
              Nothing to fix — you're ready to publish! 🎉
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
