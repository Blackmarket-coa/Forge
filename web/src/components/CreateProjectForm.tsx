import React, { useEffect, useMemo, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { checkEnvironment, createProject, ProjectMeta } from "../api/api"
import Terminal from "./Terminal"

type Step = 1 | 2 | 3 | 4 | 5

const templates = [
  { label: "React (TypeScript)", value: "react-ts" },
  { label: "Svelte (TypeScript)", value: "svelte-ts" },
  { label: "Vue (TypeScript)", value: "vue-ts" },
  { label: "Vanilla", value: "vanilla" },
  { label: "Angular", value: "angular" },
  { label: "Solid", value: "solid" },
  { label: "Preact", value: "preact" },
]

const managers = ["npm", "pnpm", "yarn", "bun"] as const

export default function CreateProjectForm({
  onCreated,
  onCancel,
}: {
  onCreated: (project: ProjectMeta) => void
  onCancel: () => void
}) {
  const [step, setStep] = useState<Step>(1)
  const [name, setName] = useState("")
  const [directory, setDirectory] = useState("")
  const [template, setTemplate] = useState("react-ts")
  const [manager, setManager] = useState<(typeof managers)[number]>("npm")
  const [availableManagers, setAvailableManagers] = useState<Record<string, boolean>>({
    npm: true,
    pnpm: false,
    yarn: false,
    bun: false,
  })
  const [identifier, setIdentifier] = useState("com.app.app")
  const [windowTitle, setWindowTitle] = useState("")
  const [targets, setTargets] = useState<string[]>(["macOS", "Linux", "Windows"])
  const [creating, setCreating] = useState(false)
  const [error, setError] = useState("")

  useEffect(() => {
    void (async () => {
      try {
        const env = await checkEnvironment()
        const npmAvailable = !!env?.node?.installed
        setAvailableManagers((prev) => ({ ...prev, npm: npmAvailable }))
      } catch {
        // keep defaults
      }
    })()
  }, [])

  useEffect(() => {
    if (name.trim()) {
      const cleaned = name.trim().toLowerCase().replace(/\s+/g, "-")
      setIdentifier(`com.${cleaned}.app`)
      setWindowTitle(name.trim())
    }
  }, [name])

  const processIdPrefix = useMemo(() => (name ? `create:${name}` : "create:"), [name])

  const pickDirectory = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      setDirectory(selected)
    }
  }
const next = () => setStep((s) => Math.min(5, s + 1) as Step)
const back = () => setStep((s) => Math.max(1, s - 1) as Step)
  const create = async () => {
    setCreating(true)
    setStep(5)
    setError("")

    try {
      const project = await createProject(directory, name, template, manager)
      onCreated(project)
    } catch (e: any) {
      setError(e?.message || String(e))
    } finally {
      setCreating(false)
    }
  }

  return (
    <div style={{ padding: 24 }}>
      <h2>Create New Project</h2>
      <p>Step {step} of 5</p>

      {step === 1 && (
        <div style={{ display: "grid", gap: 8, maxWidth: 640 }}>
          <label>
            Project name
            <input value={name} onChange={(e) => setName(e.target.value)} />
          </label>
          <label>
            Directory
            <input value={directory} readOnly />
          </label>
          <button onClick={pickDirectory}>Pick Folder</button>
        </div>
      )}

      {step === 2 && (
        <div style={{ display: "grid", gap: 6 }}>
          {templates.map((t) => (
            <label key={t.value}>
              <input type="radio" checked={template === t.value} onChange={() => setTemplate(t.value)} /> {t.label}
            </label>
          ))}
        </div>
      )}

      {step === 3 && (
        <div style={{ display: "grid", gap: 6 }}>
          {managers.map((m) => (
            <label key={m} style={{ opacity: availableManagers[m] ? 1 : 0.5 }}>
              <input
                type="radio"
                disabled={!availableManagers[m]}
                checked={manager === m}
                onChange={() => setManager(m)}
              />
              {m}
            </label>
          ))}
        </div>
      )}

      {step === 4 && (
        <div style={{ display: "grid", gap: 8, maxWidth: 640 }}>
          <label>
            App identifier
            <input value={identifier} onChange={(e) => setIdentifier(e.target.value)} />
          </label>
          <label>
            Window title
            <input value={windowTitle} onChange={(e) => setWindowTitle(e.target.value)} />
          </label>
          <div>
            Target platforms:
            {(["macOS", "Linux", "Windows", "iOS", "Android"] as const).map((platform) => (
              <label key={platform} style={{ display: "block" }}>
                <input
                  type="checkbox"
                  checked={targets.includes(platform)}
                  onChange={(e) =>
                    setTargets((prev) =>
                      e.target.checked
                        ? (prev.includes(platform) ? prev : [...prev, platform])
                        : prev.filter((p) => p !== platform)
                    )
                  }
                />
                {platform}
              </label>
            ))}
          </div>
        </div>
      )}

      {step === 5 && (
        <div style={{ display: "grid", gap: 10 }}>
          <p>{creating ? "Creating project..." : "Creation finished."}</p>
          <Terminal processIdPrefix={processIdPrefix} />
          {error && <p style={{ color: "#ff6b6b" }}>{error}</p>}
        </div>
      )}

      <div style={{ marginTop: 12, display: "flex", gap: 8 }}>
        <button onClick={onCancel}>Cancel</button>
        {step > 1 && step < 5 && <button onClick={back}>Back</button>}
        {step < 4 && <button onClick={next} disabled={(step === 1 && (!name || !directory))}>Next</button>}
        {step === 4 && (
          <button onClick={create} disabled={!name || !directory || !manager}>
            Create
          </button>
        )}
      </div>
    </div>
  )
}
