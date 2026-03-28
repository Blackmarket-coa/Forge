import React, { useEffect, useState } from "react"
import {
  BuildPreset,
  getBuildPresets,
  getProjects,
  ProjectMeta,
  runBuildPreset,
  saveBuildPreset,
} from "../api/api"
import Terminal from "./Terminal"

export default function BuildOrchestrator({ workspaceId }: { workspaceId: string }) {
  const [presets, setPresets] = useState<BuildPreset[]>([])
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [name, setName] = useState("")
  const [selectedProject, setSelectedProject] = useState("")
  const [targets, setTargets] = useState<string[]>(["appimage"])
  const [parallelWithNext, setParallelWithNext] = useState(false)
  const [timeline, setTimeline] = useState<any[]>([])

  const refresh = async () => {
    const [p, proj] = await Promise.all([getBuildPresets(workspaceId), getProjects(workspaceId)])
    setPresets(p)
    setProjects(proj)
  }

  useEffect(() => {
    if (workspaceId) void refresh()
  }, [workspaceId])

  const createPreset = async () => {
    if (!name || !selectedProject) return
    const preset: BuildPreset = {
      id: "",
      name,
      workspace_id: workspaceId,
      steps: [{ project_id: selectedProject, targets, parallel_with_next: parallelWithNext }],
    }
    await saveBuildPreset(preset)
    setName("")
    await refresh()
  }

  const runPreset = async (presetId: string) => {
    const result = await runBuildPreset(presetId)
    setTimeline(result?.timeline || [])
  }

  return (
    <div style={{ border: "1px solid #444", borderRadius: 8, padding: 12 }}>
      <h3>Build Orchestrator</h3>

      <div style={{ display: "grid", gap: 8, marginBottom: 12 }}>
        <input placeholder="Preset name" value={name} onChange={(e) => setName(e.target.value)} />
        <select value={selectedProject} onChange={(e) => setSelectedProject(e.target.value)}>
          <option value="">Select Project</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        <label>
          Targets (comma-separated)
          <input value={targets.join(",")} onChange={(e) => setTargets(e.target.value.split(",").map((v) => v.trim()).filter(Boolean))} />
        </label>
        <label>
          <input
            type="checkbox"
            checked={parallelWithNext}
            onChange={(e) => setParallelWithNext(e.target.checked)}
          />
          parallel with next
        </label>
        <button onClick={createPreset}>New Preset</button>
      </div>

      <ul>
        {presets.map((preset) => (
          <li key={preset.id}>
            <strong>{preset.name}</strong> ({preset.steps.length} step(s))
            <button onClick={() => runPreset(preset.id)} style={{ marginLeft: 8 }}>
              Run Preset
            </button>
          </li>
        ))}
      </ul>

      {timeline.length > 0 && (
        <div style={{ marginTop: 12 }}>
          <h4>Timeline</h4>
          <ol>
            {timeline.map((step, idx) => (
              <li key={idx}>status={step.status}, duration={step.duration_secs}s</li>
            ))}
          </ol>
        </div>
      )}

      <details>
        <summary>Terminal</summary>
        <Terminal />
      </details>
    </div>
  )
}
