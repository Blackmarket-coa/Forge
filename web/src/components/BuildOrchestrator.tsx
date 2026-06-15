import React, { useEffect, useState } from "react"
import { useSnackbar } from "notistack"
import {
  BuildPreset,
  getBuildPresets,
  getProjects,
  ProjectMeta,
  runBuildPreset,
  saveBuildPreset,
} from "../api/api"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Checkbox } from "./ui/checkbox"
import { EmptyState } from "./ui/empty-state"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { Select } from "./ui/select"
import styles from "./BuildOrchestrator.module.scss"

export default function BuildOrchestrator({
  workspaceId,
}: {
  workspaceId: string
}) {
  const { enqueueSnackbar } = useSnackbar()
  const [presets, setPresets] = useState<BuildPreset[]>([])
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [name, setName] = useState("")
  const [selectedProject, setSelectedProject] = useState("")
  const [targets, setTargets] = useState<string[]>(["appimage"])
  const [parallelWithNext, setParallelWithNext] = useState(false)
  const [running, setRunning] = useState<string | null>(null)
  const [timeline, setTimeline] = useState<any[]>([])

  const refresh = async () => {
    const [p, proj] = await Promise.all([
      getBuildPresets(workspaceId),
      getProjects(workspaceId),
    ])
    setPresets(p)
    setProjects(proj)
  }

  useEffect(() => {
    if (workspaceId) void refresh()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [workspaceId])

  const createPreset = async () => {
    if (!name || !selectedProject) return
    try {
      await saveBuildPreset({
        id: "",
        name,
        workspace_id: workspaceId,
        steps: [
          {
            project_id: selectedProject,
            targets,
            parallel_with_next: parallelWithNext,
          },
        ],
      })
      setName("")
      await refresh()
      enqueueSnackbar("Preset saved", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Could not save preset: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const runPreset = async (presetId: string) => {
    setRunning(presetId)
    try {
      const result = await runBuildPreset(presetId)
      setTimeline(result?.timeline || [])
      enqueueSnackbar("Preset finished", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Preset run failed: ${e?.message || e}`, {
        variant: "error",
      })
    } finally {
      setRunning(null)
    }
  }

  return (
    <Card title="Build orchestration" subtitle="Run multi-project builds.">
      <div className={styles.form}>
        <Field label="Preset name">
          <Input
            placeholder="Release all"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
        <Field label="Project">
          <Select
            value={selectedProject}
            onChange={(e) => setSelectedProject(e.target.value)}
          >
            <option value="">Select project</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Targets" help="Comma-separated bundle targets.">
          <Input
            value={targets.join(",")}
            onChange={(e) =>
              setTargets(
                e.target.value
                  .split(",")
                  .map((v) => v.trim())
                  .filter(Boolean)
              )
            }
          />
        </Field>
        <div className={styles.formFooter}>
          <Checkbox
            label="Run parallel with next step"
            checked={parallelWithNext}
            onChange={(e) => setParallelWithNext(e.target.checked)}
          />
          <Button
            variant="secondary"
            onClick={createPreset}
            disabled={!name || !selectedProject}
          >
            Add preset
          </Button>
        </div>
      </div>

      {presets.length === 0 ? (
        <EmptyState
          icon="🧩"
          title="No presets yet"
          description="Create a preset to orchestrate builds across this workspace."
        />
      ) : (
        <ul className={styles.presets}>
          {presets.map((preset) => (
            <li key={preset.id} className={styles.presetRow}>
              <div>
                <strong>{preset.name}</strong>
                <Badge tone="neutral">{preset.steps.length} step(s)</Badge>
              </div>
              <Button
                size="sm"
                variant="primary"
                loading={running === preset.id}
                onClick={() => runPreset(preset.id)}
              >
                Run
              </Button>
            </li>
          ))}
        </ul>
      )}

      {timeline.length > 0 && (
        <div className={styles.timeline}>
          <div className={styles.subLabel}>Timeline</div>
          <ol className={styles.timelineList}>
            {timeline.map((step, idx) => (
              <li key={idx}>
                <Badge
                  tone={step.status === "success" ? "success" : "danger"}
                  dot
                >
                  {step.status}
                </Badge>
                <span className={styles.muted}>{step.duration_secs}s</span>
              </li>
            ))}
          </ol>
        </div>
      )}
    </Card>
  )
}
