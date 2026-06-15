import React, { useEffect, useMemo, useState } from "react"
import { useSnackbar } from "notistack"
import { readConfig, validateConfig, writeConfig } from "../api/api"
import { diffConfig } from "../lib/diff"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Checkbox } from "./ui/checkbox"
import { Collapsible } from "./ui/collapsible"
import { ConfirmDialog } from "./ui/dialog"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { PageHeader } from "./ui/page-header"
import { Select } from "./ui/select"
import { Tabs } from "./ui/tabs"
import styles from "./ConfigEditor.module.scss"

type Mode = "form" | "json"

const CATEGORY_OPTIONS = [
  "Business",
  "Developer Tool",
  "Education",
  "Entertainment",
  "Finance",
  "Game",
  "Graphics",
  "Lifestyle",
  "Music",
  "Productivity",
  "Utilities",
]

const BUNDLE_TARGETS = ["dmg", "appimage", "deb", "nsis", "msi"] as const

export default function ConfigEditor({
  projectPath,
  projectName,
  onBack,
}: {
  projectPath: string
  projectName?: string
  onBack?: () => void
}) {
  const { enqueueSnackbar } = useSnackbar()
  const [mode, setMode] = useState<Mode>("form")
  const [config, setConfig] = useState<any>({})
  const [original, setOriginal] = useState<any>({})
  const [rawJson, setRawJson] = useState("{}")
  const [issues, setIssues] = useState<string[]>([])
  const [jsonError, setJsonError] = useState("")
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [saving, setSaving] = useState(false)

  const loadConfig = async () => {
    try {
      const cfg = await readConfig(projectPath)
      setConfig(cfg)
      setOriginal(cfg)
      setRawJson(JSON.stringify(cfg, null, 2))
      setJsonError("")
    } catch (e: any) {
      enqueueSnackbar(`Failed to load config: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  useEffect(() => {
    void loadConfig()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectPath])

  const updateConfig = (next: any) => {
    setConfig(next)
    setRawJson(JSON.stringify(next, null, 2))
  }

  const setField = (path: string[], value: any) => {
    const next = JSON.parse(JSON.stringify(config || {}))
    let cur = next
    for (let i = 0; i < path.length - 1; i++) {
      const key = path[i]
      if (cur[key] == null) cur[key] = {}
      cur = cur[key]
    }
    cur[path[path.length - 1]] = value
    updateConfig(next)
  }

  const windows0 = useMemo(() => config?.app?.windows?.[0] ?? {}, [config])

  const setWindowField = (key: string, value: any) => {
    const next = JSON.parse(JSON.stringify(config || {}))
    if (!next.app) next.app = {}
    if (!Array.isArray(next.app.windows)) next.app.windows = [{}]
    if (!next.app.windows[0]) next.app.windows[0] = {}
    next.app.windows[0][key] = value
    updateConfig(next)
  }

  const build = config?.build ?? {}
  const bundle = config?.bundle ?? {}

  const changes = useMemo(
    () => diffConfig(original, config),
    [original, config]
  )

  const onRawJsonChange = (value: string) => {
    setRawJson(value)
    try {
      setConfig(JSON.parse(value))
      setJsonError("")
    } catch (e: any) {
      setJsonError(e?.message || "Invalid JSON")
    }
  }

  const requestSave = () => {
    if (jsonError) {
      enqueueSnackbar("Fix the JSON errors before saving.", {
        variant: "error",
      })
      return
    }
    if (changes.length === 0) {
      enqueueSnackbar("No changes to save.", { variant: "info" })
      return
    }
    setConfirmOpen(true)
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      await writeConfig(projectPath, config)
      setOriginal(config)
      setConfirmOpen(false)
      enqueueSnackbar("Configuration saved", { variant: "success" })
    } catch (e: any) {
      enqueueSnackbar(`Save failed: ${e?.message || e}`, { variant: "error" })
    } finally {
      setSaving(false)
    }
  }

  const handleValidate = async () => {
    try {
      const nextIssues = await validateConfig(projectPath, config)
      setIssues(nextIssues)
      enqueueSnackbar(
        nextIssues.length
          ? `Found ${nextIssues.length} issue(s)`
          : "Validation passed",
        { variant: nextIssues.length ? "warning" : "success" }
      )
    } catch (e: any) {
      enqueueSnackbar(`Validation failed: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const useRecommendedDefaults = () => {
    const next = JSON.parse(JSON.stringify(config || {}))
    next.productName = next.productName || "My App"
    next.identifier = next.identifier || "com.example.myapp"
    next.version = next.version || "0.1.0"
    if (!next.app) next.app = {}
    if (!Array.isArray(next.app.windows) || !next.app.windows[0])
      next.app.windows = [{}]
    next.app.windows[0].title = next.app.windows[0].title || next.productName
    next.app.windows[0].width = next.app.windows[0].width || 1200
    next.app.windows[0].height = next.app.windows[0].height || 800
    if (next.app.windows[0].resizable == null)
      next.app.windows[0].resizable = true
    if (next.app.windows[0].decorations == null)
      next.app.windows[0].decorations = true
    if (next.app.windows[0].fullscreen == null)
      next.app.windows[0].fullscreen = false
    if (!next.build) next.build = {}
    next.build.devUrl = next.build.devUrl || "http://localhost:3000"
    next.build.frontendDist = next.build.frontendDist || "../dist"
    updateConfig(next)
    enqueueSnackbar("Added beginner-friendly defaults. Review and Save.", {
      variant: "info",
    })
  }

  return (
    <div>
      {onBack && (
        <Button variant="ghost" size="sm" onClick={onBack}>
          ← Back to Project
        </Button>
      )}

      <PageHeader
        title="Configuration"
        subtitle={
          projectName
            ? `Editing tauri.conf.json for ${projectName}`
            : "Editing tauri.conf.json"
        }
        actions={
          <Tabs
            value={mode}
            onValueChange={(v) => setMode(v as Mode)}
            tabs={[
              { value: "form", label: "Form" },
              { value: "json", label: "JSON" },
            ]}
          />
        }
      />

      {mode === "form" ? (
        <div className={styles.sections}>
          <Collapsible title="App Identity">
            <p className={styles.hint}>
              These values appear in installers and app metadata.
            </p>
            <div className={styles.rows}>
              <Field label="App name">
                <Input
                  placeholder="My App"
                  value={config?.productName || ""}
                  onChange={(e) => setField(["productName"], e.target.value)}
                />
              </Field>
              <Field
                label="Bundle ID"
                help="Reverse-domain format, e.g. com.example.myapp"
              >
                <Input
                  placeholder="com.example.myapp"
                  value={config?.identifier || ""}
                  onChange={(e) => setField(["identifier"], e.target.value)}
                />
              </Field>
              <Field
                label="Version"
                help="Use semantic versioning, like 1.0.0."
              >
                <Input
                  placeholder="1.0.0"
                  value={config?.version || ""}
                  onChange={(e) => setField(["version"], e.target.value)}
                />
              </Field>
            </div>
          </Collapsible>

          <Collapsible title="Window">
            <p className={styles.hint}>
              Controls the first app window users see.
            </p>
            <div className={styles.rows}>
              <Field label="Window title">
                <Input
                  value={windows0?.title || ""}
                  onChange={(e) => setWindowField("title", e.target.value)}
                />
              </Field>
              <Field label="Width (px)">
                <Input
                  type="number"
                  value={windows0?.width || 0}
                  onChange={(e) =>
                    setWindowField("width", Number(e.target.value))
                  }
                />
              </Field>
              <Field label="Height (px)">
                <Input
                  type="number"
                  value={windows0?.height || 0}
                  onChange={(e) =>
                    setWindowField("height", Number(e.target.value))
                  }
                />
              </Field>
            </div>
            <div className={styles.checkRow}>
              <Checkbox
                label="Allow resize"
                checked={!!windows0?.resizable}
                onChange={(e) => setWindowField("resizable", e.target.checked)}
              />
              <Checkbox
                label="Start fullscreen"
                checked={!!windows0?.fullscreen}
                onChange={(e) => setWindowField("fullscreen", e.target.checked)}
              />
              <Checkbox
                label="Show frame controls"
                checked={!!windows0?.decorations}
                onChange={(e) =>
                  setWindowField("decorations", e.target.checked)
                }
              />
            </div>
          </Collapsible>

          <Collapsible title="Build">
            <p className={styles.hint}>
              How Forge runs your app in development and production builds.
            </p>
            <div className={styles.rows}>
              <Field
                label="Dev URL"
                help="Where your frontend dev server runs (build.devUrl)."
              >
                <Input
                  placeholder="http://localhost:3000"
                  value={build?.devUrl || ""}
                  onChange={(e) =>
                    setField(["build", "devUrl"], e.target.value)
                  }
                />
              </Field>
              <Field label="Built frontend folder">
                <Input
                  placeholder="../dist"
                  value={build?.frontendDist || ""}
                  onChange={(e) =>
                    setField(["build", "frontendDist"], e.target.value)
                  }
                />
              </Field>
              <Field label="Before dev command">
                <Input
                  value={build?.beforeDevCommand || ""}
                  onChange={(e) =>
                    setField(["build", "beforeDevCommand"], e.target.value)
                  }
                />
              </Field>
              <Field label="Before build command">
                <Input
                  value={build?.beforeBuildCommand || ""}
                  onChange={(e) =>
                    setField(["build", "beforeBuildCommand"], e.target.value)
                  }
                />
              </Field>
            </div>
          </Collapsible>

          <Collapsible title="Bundle">
            <p className={styles.hint}>Installer and app-store metadata.</p>
            <div className={styles.rows}>
              <Field
                label="Icon path(s)"
                help="Use comma-separated paths for multiple icons."
              >
                <Input
                  value={
                    Array.isArray(bundle?.icon)
                      ? bundle.icon.join(",")
                      : bundle?.icon || ""
                  }
                  onChange={(e) => setField(["bundle", "icon"], e.target.value)}
                />
              </Field>
              <Field label="Copyright">
                <Input
                  value={bundle?.copyright || ""}
                  onChange={(e) =>
                    setField(["bundle", "copyright"], e.target.value)
                  }
                />
              </Field>
              <Field label="Category">
                <Select
                  value={bundle?.category || ""}
                  onChange={(e) =>
                    setField(["bundle", "category"], e.target.value)
                  }
                >
                  <option value="">Select category</option>
                  {CATEGORY_OPTIONS.map((c) => (
                    <option key={c} value={c}>
                      {c}
                    </option>
                  ))}
                </Select>
              </Field>
            </div>
            <div className={styles.subLabel}>Installer targets</div>
            <div className={styles.checkRow}>
              {BUNDLE_TARGETS.map((target) => (
                <Checkbox
                  key={target}
                  label={target}
                  checked={
                    Array.isArray(bundle?.targets) &&
                    bundle.targets.includes(target)
                  }
                  onChange={(e) => {
                    const current = Array.isArray(bundle?.targets)
                      ? [...bundle.targets]
                      : []
                    const next = e.target.checked
                      ? current.includes(target)
                        ? current
                        : [...current, target]
                      : current.filter((t: string) => t !== target)
                    setField(["bundle", "targets"], next)
                  }}
                />
              ))}
            </div>
          </Collapsible>
        </div>
      ) : (
        <div className={styles.jsonWrap}>
          {jsonError && <Banner tone="danger">{jsonError}</Banner>}
          <textarea
            className={styles.textarea}
            value={rawJson}
            onChange={(e) => onRawJsonChange(e.target.value)}
            spellCheck={false}
          />
        </div>
      )}

      {issues.length > 0 && (
        <div className={styles.issues}>
          <Banner tone="warning" title="Validation issues">
            <ul className={styles.issueList}>
              {issues.map((issue, idx) => (
                <li key={idx}>{issue}</li>
              ))}
            </ul>
          </Banner>
        </div>
      )}

      <div className={styles.actions}>
        <Button variant="ghost" onClick={useRecommendedDefaults}>
          Use Recommended Defaults
        </Button>
        <Button variant="secondary" onClick={handleValidate}>
          Validate
        </Button>
        <Button variant="ghost" onClick={() => void loadConfig()}>
          Reset
        </Button>
        <Button variant="primary" onClick={requestSave} disabled={!!jsonError}>
          Save
          {changes.length > 0 ? ` (${changes.length})` : ""}
        </Button>
      </div>

      <ConfirmDialog
        open={confirmOpen}
        title="Review changes"
        confirmLabel="Save changes"
        loading={saving}
        onConfirm={handleSave}
        onCancel={() => setConfirmOpen(false)}
        message={
          <div>
            <p className={styles.confirmIntro}>
              {changes.length} field(s) will be written to tauri.conf.json. A
              backup (.bak) is created automatically.
            </p>
            <ul className={styles.diffList}>
              {changes.map((c) => (
                <li key={c.path} className={styles.diffRow}>
                  <code className={styles.diffPath}>{c.path}</code>
                  <span className={styles.diffBefore}>{format(c.before)}</span>
                  <span className={styles.diffArrow}>→</span>
                  <span className={styles.diffAfter}>{format(c.after)}</span>
                </li>
              ))}
            </ul>
          </div>
        }
      />
    </div>
  )
}

function format(value: unknown): string {
  if (value === undefined) return "—"
  if (typeof value === "string") return value || '""'
  return JSON.stringify(value)
}
