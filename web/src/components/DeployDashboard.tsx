import React, { useEffect, useMemo, useState } from "react"
import { getDeployStatus } from "../api/api"

export default function DeployDashboard({
  workspaceId,
  onOpenProjectBuild,
}: {
  workspaceId: string
  onOpenProjectBuild: (projectId: string, platform: string) => void
}) {
  const [status, setStatus] = useState<any>(null)

  useEffect(() => {
    if (!workspaceId || workspaceId === "all") return
    void (async () => {
      const data = await getDeployStatus(workspaceId)
      setStatus(data)
    })()
  }, [workspaceId])

  const progress = useMemo(() => Number(status?.overall_progress || 0), [status])
  const matrix = status?.matrix || []
  const blockers = status?.blockers || []
  const checklist = status?.checklist || []
  const platforms = ["macOS", "Linux", "Windows", "iOS", "Android"]

  const labelFor = (value: string) => {
    if (value === "built") return { icon: "✓", label: "Built", color: "#16a34a" }
    if (value === "configured") return { icon: "○", label: "Configured", color: "#f59e0b" }
    if (value === "not_started") return { icon: "✗", label: "Missing config", color: "#dc2626" }
    return { icon: "—", label: "Not targeted", color: "#6b7280" }
  }

  return (
    <div style={{ border: "1px solid #444", borderRadius: 8, padding: 12, marginTop: 12 }}>
      <h3>Deploy Dashboard</h3>
      <div style={{ marginBottom: 12 }}>
        <div style={{ height: 10, background: "#222", borderRadius: 999 }}>
          <div style={{ width: `${progress}%`, height: "100%", background: "#22c55e", borderRadius: 999 }} />
        </div>
        <div>{progress.toFixed(1)}% ready</div>
      </div>

      <table style={{ width: "100%", marginBottom: 12 }}>
        <thead>
          <tr>
            <th>Project</th>
            {platforms.map((p) => (
              <th key={p}>{p}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.map((row: any) => (
            <tr key={row.project_id}>
              <td>{row.project_name}</td>
              {platforms.map((p) => {
                const view = labelFor(row?.statuses?.[p] || "not_targeted")
                return (
                  <td key={p}>
                    <button
                      onClick={() => onOpenProjectBuild(row.project_id, p)}
                      style={{ color: view.color, border: "none", background: "transparent", cursor: "pointer" }}
                    >
                      {view.icon} {view.label}
                    </button>
                  </td>
                )
              })}
            </tr>
          ))}
        </tbody>
      </table>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
        <div>
          <h4>Checklist</h4>
          <ul>
            {checklist.map((item: any, idx: number) => {
              const init = item.tauri_initialized ? "✓" : "✗"
              const cfg = item.config_ok ? "✓" : "○"
              return (
                <li key={idx}>
                  {init} Tauri initialized — {item.project}; {cfg} config status
                </li>
              )
            })}
          </ul>
        </div>

        <div>
          <h4>Blockers</h4>
          <ul>
            {blockers.map((b: any, idx: number) => (
              <li key={idx}>
                ⚠️ {b.message}
                {b.affected_project ? ` — ${b.affected_project}` : ""}
                {b.fix_hint ? <div style={{ opacity: 0.8 }}>Fix: {b.fix_hint}</div> : null}
                {String(b.message).includes("tauri.conf.json") && <button>Open Config Editor</button>}
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}
