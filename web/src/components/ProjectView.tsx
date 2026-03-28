import React, { useEffect, useMemo, useState } from "react"
import { collectArtifacts, getBuildHistory, killProcess, ProjectMeta, runBuild, runDev } from "../api/api"
import Terminal from "./Terminal"
import LicenseGate from "./LicenseGate"

interface ProjectViewProps {
  project: ProjectMeta
  onBack: () => void
  onOpenConfig: () => void
}

const labelStyle: React.CSSProperties = { fontWeight: 600, minWidth: 160 }

export default function ProjectView({ project, onBack, onOpenConfig }: ProjectViewProps) {
  const [log, setLog] = useState("Forge log output will appear here.\n")
  const [showTerminal, setShowTerminal] = useState(false)
  const [isRunning, setIsRunning] = useState(false)
  const [processId, setProcessId] = useState("")
  const [targets, setTargets] = useState<string[]>([])
  const [buildResult, setBuildResult] = useState<any>(null)
  const [artifacts, setArtifacts] = useState<any[]>([])
  const [history, setHistory] = useState<any[]>([])


  const refreshHistory = async () => {
    const rows = await getBuildHistory(project.id, 10)
    setHistory(rows)
  }

  useEffect(() => {
    void refreshHistory()
  }, [project.id])

  const processIdPrefix = useMemo(() => {
    if (processId) return processId
    return `${project.path}`
  }, [processId, project.path])

  const handleDev = async () => {
    const pid = await runDev(project.path)
    setProcessId(`dev:${project.path}`)
    setShowTerminal(true)
    setIsRunning(true)
    setLog((prev) => `${prev}Started dev process PID: ${pid}\n`)
  }

  const handleStartBuild = async () => {
    setShowTerminal(true)
    const result = await runBuild(project.path, targets)
    setBuildResult(result)
    setArtifacts(result?.artifacts || (await collectArtifacts(project.path)))
    await refreshHistory()
  }

  const handleStop = async () => {
    if (!processId) return
    await killProcess(processId)
    setIsRunning(false)
  }

  const toggleTarget = (target: string, checked: boolean) => {
    setTargets((prev) =>
      checked ? (prev.includes(target) ? prev : [...prev, target]) : prev.filter((t) => t !== target)
    )
  }

  return (
    <div style={{ padding: 24 }}>
      <button onClick={onBack}>← Back</button>
      <h1>{project.name}</h1>

      <div style={{ display: "grid", gap: 8, marginBottom: 16 }}>
        <div><span style={labelStyle}>ID:</span> {project.id}</div>
        <div><span style={labelStyle}>Path:</span> {project.path}</div>
        <div><span style={labelStyle}>Workspace:</span> {project.workspace_id || "none"}</div>
        <div><span style={labelStyle}>Tauri Version:</span> {project.tauri_version || "unknown"}</div>
        <div><span style={labelStyle}>Identifier:</span> {project.identifier || "unknown"}</div>
        <div><span style={labelStyle}>Framework:</span> {project.frontend_framework || "vanilla"}</div>
        <div><span style={labelStyle}>Git Branch:</span> {project.git_branch || "unknown"}</div>
        <div><span style={labelStyle}>Git Dirty:</span> {project.git_dirty ? "yes" : "no"}</div>
        <div><span style={labelStyle}>Status:</span> {project.status}</div>
        <div><span style={labelStyle}>Role:</span> {project.role || "none"}</div>
      </div>

      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 16 }}>
        {(project.platforms || []).map((platform) => (
          <span key={platform} style={{ border: "1px solid #666", borderRadius: 12, padding: "2px 10px" }}>
            {platform}
          </span>
        ))}
        {(project.platforms || []).length === 0 && <span>No platforms detected</span>}
      </div>

      <div style={{ display: "flex", gap: 12, marginBottom: 16, alignItems: "center" }}>
        <button onClick={handleDev}>Dev</button>

        <details>
          <summary>Build</summary>
          <div style={{ display: "grid", gap: 4, padding: 8 }}>
            {(["dmg", "appimage", "deb", "nsis", "msi"] as const).map((target) => (
              <label key={target}>
                <input
                  type="checkbox"
                  checked={targets.includes(target)}
                  onChange={(e) => toggleTarget(target, e.target.checked)}
                />
                {target}
              </label>
            ))}
            <button onClick={handleStartBuild} disabled={targets.length === 0}>Start Build</button>
          </div>
        </details>

        <button onClick={onOpenConfig}>Config</button>
        {isRunning && <button onClick={handleStop}>Stop</button>}
      </div>

      {buildResult && (
        <div style={{ marginBottom: 12 }}>
          <strong>Build status:</strong> {buildResult.status} ({buildResult.duration_secs?.toFixed?.(2)}s)
        </div>
      )}

      {artifacts.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <h3>Artifacts</h3>
          <ul>
            {artifacts.map((artifact, idx) => (
              <li key={idx}>
                {artifact.path} — {artifact.size_bytes} bytes
                <a href="#" onClick={(e) => e.preventDefault()}>Open Folder</a>
              </li>
            ))}
          </ul>
        </div>
      )}

      <LicenseGate
        feature="build_history"
        description="Build history and rerun tracking are available on Forge Pro."
      >
        <h3>Build History (last 10)</h3>
        <table style={{ width: "100%", marginBottom: 12 }}>
          <thead>
            <tr>
              <th>Date</th><th>Targets</th><th>Status</th><th>Duration</th><th>Artifacts</th><th></th>
            </tr>
          </thead>
          <tbody>
            {history.map((row) => (
              <tr key={row.id}>
                <td>{row.started_at}</td>
                <td>{(row.targets || []).join(",")}</td>
                <td style={{ color: row.status === "success" ? "#16a34a" : "#dc2626" }}>{row.status}</td>
                <td>{row.duration_secs}s</td>
                <td>{(row.artifacts || []).length}</td>
                <td><button onClick={() => setTargets(row.targets || [])}>Re-run</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      </LicenseGate>

      <details open={showTerminal} onToggle={(e) => setShowTerminal((e.target as HTMLDetailsElement).open)}>
        <summary>Terminal Panel</summary>
        <Terminal processIdPrefix={processIdPrefix} />
      </details>

      <pre style={{ background: "#111", color: "#ddd", padding: 12, minHeight: 120 }}>{log}</pre>
    </div>
  )
}
