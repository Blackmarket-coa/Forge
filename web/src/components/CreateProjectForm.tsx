import React, { useEffect, useMemo, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { useSnackbar } from "notistack"
import { checkEnvironment, createProject, ProjectMeta } from "../api/api"
import Terminal from "./Terminal"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Checkbox } from "./ui/checkbox"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { PageHeader } from "./ui/page-header"
import styles from "./CreateProjectForm.module.scss"

type Step = 1 | 2 | 3 | 4 | 5

const STEP_LABELS = [
  "Basics",
  "Template",
  "Package manager",
  "App config",
  "Create",
]

const TEMPLATES = [
  { label: "React (TypeScript)", value: "react-ts" },
  { label: "Svelte (TypeScript)", value: "svelte-ts" },
  { label: "Vue (TypeScript)", value: "vue-ts" },
  { label: "Vanilla", value: "vanilla" },
  { label: "Angular", value: "angular" },
  { label: "Solid", value: "solid" },
  { label: "Preact", value: "preact" },
]

const MANAGERS = ["npm", "pnpm", "yarn", "bun"] as const
const PLATFORMS = ["macOS", "Linux", "Windows", "iOS", "Android"] as const

export default function CreateProjectForm({
  onCreated,
  onCancel,
}: {
  onCreated: (project: ProjectMeta) => void
  onCancel: () => void
}) {
  const { enqueueSnackbar } = useSnackbar()
  const [step, setStep] = useState<Step>(1)
  const [name, setName] = useState("")
  const [directory, setDirectory] = useState("")
  const [template, setTemplate] = useState("react-ts")
  const [manager, setManager] = useState<typeof MANAGERS[number]>("npm")
  const [availableManagers, setAvailableManagers] = useState<
    Record<string, boolean>
  >({ npm: true, pnpm: false, yarn: false, bun: false })
  const [identifier, setIdentifier] = useState("com.app.app")
  const [windowTitle, setWindowTitle] = useState("")
  const [targets, setTargets] = useState<string[]>([
    "macOS",
    "Linux",
    "Windows",
  ])
  const [creating, setCreating] = useState(false)

  useEffect(() => {
    void (async () => {
      try {
        const env = await checkEnvironment()
        setAvailableManagers((prev) => ({
          ...prev,
          npm: !!env?.node?.installed,
        }))
      } catch {
        /* keep defaults */
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

  const processIdPrefix = useMemo(
    () => (name ? `create:${name}` : "create:"),
    [name]
  )

  const pickDirectory = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") setDirectory(selected)
  }

  const next = () => setStep((s) => Math.min(5, s + 1) as Step)
  const back = () => setStep((s) => Math.max(1, s - 1) as Step)

  const create = async () => {
    setCreating(true)
    setStep(5)
    try {
      const project = await createProject(directory, name, template, manager)
      enqueueSnackbar("Project created", { variant: "success" })
      onCreated(project)
    } catch (e: any) {
      enqueueSnackbar(`Creation failed: ${e?.message || e}`, {
        variant: "error",
      })
    } finally {
      setCreating(false)
    }
  }

  return (
    <div>
      <Button variant="ghost" size="sm" onClick={onCancel}>
        ← Cancel
      </Button>
      <PageHeader title="Create a new project" />

      <div className={styles.steps}>
        {STEP_LABELS.map((label, idx) => {
          const n = idx + 1
          const state = n === step ? "active" : n < step ? "done" : "todo"
          return (
            <div key={label} className={[styles.step, styles[state]].join(" ")}>
              <span className={styles.stepNum}>{n < step ? "✓" : n}</span>
              <span>{label}</span>
            </div>
          )
        })}
      </div>

      <Card>
        {step === 1 && (
          <div className={styles.body}>
            <Field label="Project name">
              <Input
                placeholder="my-app"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </Field>
            <Field
              label="Directory"
              help="Where the project folder is created."
            >
              <div className={styles.dirRow}>
                <Input
                  value={directory}
                  readOnly
                  placeholder="No folder chosen"
                />
                <Button variant="secondary" onClick={pickDirectory}>
                  Pick folder
                </Button>
              </div>
            </Field>
          </div>
        )}

        {step === 2 && (
          <div className={styles.optionGrid}>
            {TEMPLATES.map((t) => (
              <button
                key={t.value}
                className={[
                  styles.option,
                  template === t.value ? styles.optionActive : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                onClick={() => setTemplate(t.value)}
              >
                {t.label}
              </button>
            ))}
          </div>
        )}

        {step === 3 && (
          <div className={styles.optionGrid}>
            {MANAGERS.map((m) => (
              <button
                key={m}
                disabled={!availableManagers[m]}
                className={[
                  styles.option,
                  manager === m ? styles.optionActive : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                onClick={() => setManager(m)}
              >
                {m}
                {!availableManagers[m] && (
                  <Badge tone="neutral">not found</Badge>
                )}
              </button>
            ))}
          </div>
        )}

        {step === 4 && (
          <div className={styles.body}>
            <Field label="App identifier">
              <Input
                value={identifier}
                onChange={(e) => setIdentifier(e.target.value)}
              />
            </Field>
            <Field label="Window title">
              <Input
                value={windowTitle}
                onChange={(e) => setWindowTitle(e.target.value)}
              />
            </Field>
            <Field label="Target platforms">
              <div className={styles.checkRow}>
                {PLATFORMS.map((platform) => (
                  <Checkbox
                    key={platform}
                    label={platform}
                    checked={targets.includes(platform)}
                    onChange={(e) =>
                      setTargets((prev) =>
                        e.target.checked
                          ? prev.includes(platform)
                            ? prev
                            : [...prev, platform]
                          : prev.filter((p) => p !== platform)
                      )
                    }
                  />
                ))}
              </div>
            </Field>
          </div>
        )}

        {step === 5 && (
          <div className={styles.body}>
            <p className={styles.muted}>
              {creating ? "Creating project…" : "Creation finished."}
            </p>
            <Terminal processIdPrefix={processIdPrefix} />
          </div>
        )}
      </Card>

      <div className={styles.footer}>
        {step > 1 && step < 5 && (
          <Button variant="ghost" onClick={back}>
            Back
          </Button>
        )}
        <div className={styles.footerRight}>
          {step < 4 && (
            <Button
              variant="primary"
              onClick={next}
              disabled={step === 1 && (!name || !directory)}
            >
              Next
            </Button>
          )}
          {step === 4 && (
            <Button
              variant="primary"
              onClick={create}
              disabled={!name || !directory || !manager}
            >
              Create project
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
