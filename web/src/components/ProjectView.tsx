import React, { useState } from "react"
import { ProjectMeta, runDev } from "../api/api"

interface ProjectViewProps {
  project: ProjectMeta
  onBack: () => void
}

export default function ProjectView({ project, onBack }: ProjectViewProps) {
  const [log, setLog] = useState("Forge log output will appear here.\n")

  const handleDev = async () => {
    const pid = await runDev(project.path)
    setLog((prev) => `${prev}Started dev process PID: ${pid}\n`)
  }

  return (
    <div style={{ padding: 24 }}>
      <button onClick={onBack}>← Back</button>
      <h1>{project.name}</h1>
      <p>
        <strong>Path:</strong> {project.path}
      </p>
      <p>
        <strong>Git branch:</strong> {project.git_branch ?? "unknown"}
      </p>
      <p>
        <strong>Status:</strong> {project.status}
      </p>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 16 }}>
        {project.platforms.map((platform) => (
          <span key={platform} style={{ border: "1px solid #666", borderRadius: 12, padding: "2px 10px" }}>
            {platform}
          </span>
        ))}
        {project.platforms.length === 0 && <span>No platforms detected</span>}
      </div>
      <div style={{ display: "flex", gap: 12, marginBottom: 16 }}>
        <button onClick={handleDev}>Dev</button>
        <button onClick={() => window.alert("Build action coming soon")}>Build</button>
        <button onClick={() => window.alert("Config view coming soon")}>Config</button>
      </div>
      <pre style={{ background: "#111", color: "#ddd", padding: 12, minHeight: 220 }}>{log}</pre>
    </div>
  )
}
