import React, { useState } from "react"
import { ProjectMeta, runDev } from "../api/api"

interface ProjectViewProps {
  project: ProjectMeta
  onBack: () => void
  onOpenConfig: () => void
}

const labelStyle: React.CSSProperties = { fontWeight: 600, minWidth: 160 }

export default function ProjectView({ project, onBack, onOpenConfig }: ProjectViewProps) {
  const [log, setLog] = useState("Forge log output will appear here.\n")

  const handleDev = async () => {
    const pid = await runDev(project.path)
    setLog((prev) => `${prev}Started dev process PID: ${pid}\n`)
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

      <div style={{ display: "flex", gap: 12, marginBottom: 16 }}>
        <button onClick={handleDev}>Dev</button>
        <button onClick={() => window.alert("Build action coming soon")}>Build</button>
        <button onClick={onOpenConfig}>Config</button>
      </div>

      <pre style={{ background: "#111", color: "#ddd", padding: 12, minHeight: 220 }}>{log}</pre>
    </div>
  )
}
