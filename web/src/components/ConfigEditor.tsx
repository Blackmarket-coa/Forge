import React, { useEffect, useMemo, useState } from "react"
import { readConfig, validateConfig, writeConfig } from "../api/api"
import { Button } from "./ui/button"
import { Checkbox } from "./ui/checkbox"
import { Collapsible } from "./ui/collapsible"
import { Input } from "./ui/input"
import { Select } from "./ui/select"
import { Tabs } from "./ui/tabs"

type Mode = "form" | "json"

export default function ConfigEditor({ projectPath }: { projectPath: string }) {
  const [mode, setMode] = useState<Mode>("form")
  const [config, setConfig] = useState<any>({})
  const [rawJson, setRawJson] = useState("{}")
  const [issues, setIssues] = useState<string[]>([])
  const [message, setMessage] = useState("")

  const loadConfig = async () => {
    try {
      const cfg = await readConfig(projectPath)
      setConfig(cfg)
      setRawJson(JSON.stringify(cfg, null, 2))
      setMessage("")
    } catch (e: any) {
      setMessage(`Failed to load config: ${e?.message || e}`)
    }
  }

  useEffect(() => {
    void loadConfig()
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

  const windows0 = useMemo(() => (config?.app?.windows?.[0] ?? {}), [config])

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

  const onRawJsonChange = (value: string) => {
    setRawJson(value)
    try {
      const parsed = JSON.parse(value)
      setConfig(parsed)
    } catch {
      // Keep raw json editing tolerant until save/validate.
    }
  }

  const handleSave = async () => {
    try {
      await writeConfig(projectPath, config)
      setMessage("Saved successfully")
    } catch (e: any) {
      setMessage(`Save failed: ${e?.message || e}`)
    }
  }

  const handleValidate = async () => {
    try {
      const nextIssues = await validateConfig(projectPath, config)
      setIssues(nextIssues)
      setMessage(nextIssues.length ? "Validation completed with issues" : "Validation passed")
    } catch (e: any) {
      setMessage(`Validation failed: ${e?.message || e}`)
    }
  }

  const categoryOptions = [
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

  return (
    <div style={{ padding: 24 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2>Config Editor</h2>
        <Tabs
          value={mode}
          onValueChange={(v) => setMode(v as Mode)}
          tabs={[
            { value: "form", label: "Form" },
            { value: "json", label: "JSON" },
          ]}
        />
      </div>

      {mode === "form" ? (
        <div style={{ display: "grid", gap: 12 }}>
          <Collapsible title="App Identity">
            <label>productName <Input value={config?.productName || ""} onChange={(e) => setField(["productName"], e.target.value)} /></label>
            <label>identifier <Input value={config?.identifier || ""} onChange={(e) => setField(["identifier"], e.target.value)} /></label>
            <div style={{ fontSize: 12, opacity: 0.7 }}>Expected reverse-domain format (e.g. com.example.app)</div>
            <label>version <Input value={config?.version || ""} onChange={(e) => setField(["version"], e.target.value)} /></label>
            <div style={{ fontSize: 12, opacity: 0.7 }}>Semver hint: 1.0.0</div>
          </Collapsible>

          <Collapsible title="Windows">
            <label>title <Input value={windows0?.title || ""} onChange={(e) => setWindowField("title", e.target.value)} /></label>
            <label>width <Input type="number" value={windows0?.width || 0} onChange={(e) => setWindowField("width", Number(e.target.value))} /></label>
            <label>height <Input type="number" value={windows0?.height || 0} onChange={(e) => setWindowField("height", Number(e.target.value))} /></label>
            <label><Checkbox checked={!!windows0?.resizable} onChange={(e) => setWindowField("resizable", e.target.checked)} /> resizable</label>
            <label><Checkbox checked={!!windows0?.fullscreen} onChange={(e) => setWindowField("fullscreen", e.target.checked)} /> fullscreen</label>
            <label><Checkbox checked={!!windows0?.decorations} onChange={(e) => setWindowField("decorations", e.target.checked)} /> decorations</label>
          </Collapsible>

          <Collapsible title="Build">
            <label>devUrl <Input value={build?.devUrl || ""} onChange={(e) => setField(["build", "devUrl"], e.target.value)} /></label>
            <label>frontendDist <Input value={build?.frontendDist || ""} onChange={(e) => setField(["build", "frontendDist"], e.target.value)} /></label>
            <label>beforeDevCommand <Input value={build?.beforeDevCommand || ""} onChange={(e) => setField(["build", "beforeDevCommand"], e.target.value)} /></label>
            <label>beforeBuildCommand <Input value={build?.beforeBuildCommand || ""} onChange={(e) => setField(["build", "beforeBuildCommand"], e.target.value)} /></label>
          </Collapsible>

          <Collapsible title="Bundle">
            <label>icon <Input value={Array.isArray(bundle?.icon) ? bundle.icon.join(",") : bundle?.icon || ""} onChange={(e) => setField(["bundle", "icon"], e.target.value)} /></label>
            <label>copyright <Input value={bundle?.copyright || ""} onChange={(e) => setField(["bundle", "copyright"], e.target.value)} /></label>
            <label>category
              <Select value={bundle?.category || ""} onChange={(e) => setField(["bundle", "category"], e.target.value)}>
                <option value="">Select category</option>
                {categoryOptions.map((c) => <option key={c} value={c}>{c}</option>)}
              </Select>
            </label>
            <div>targets:</div>
            {(["dmg", "appimage", "deb", "nsis", "msi"] as const).map((target) => (
              <label key={target}>
                <Checkbox
                  checked={Array.isArray(bundle?.targets) && bundle.targets.includes(target)}
                  onChange={(e) => {
                    const current = Array.isArray(bundle?.targets) ? [...bundle.targets] : []
                    const next = e.target.checked ? (current.includes(target) ? current : [...current, target]) : current.filter((t: string) => t !== target)
                    setField(["bundle", "targets"], next)
                  }}
                /> {target}
              </label>
            ))}
          </Collapsible>
        </div>
      ) : (
        <textarea
          value={rawJson}
          onChange={(e) => onRawJsonChange(e.target.value)}
          style={{ width: "100%", minHeight: 520, fontFamily: "monospace" }}
        />
      )}

      <div style={{ marginTop: 16, display: "flex", gap: 8 }}>
        <Button onClick={handleSave}>Save</Button>
        <Button onClick={handleValidate}>Validate</Button>
        <Button onClick={() => void loadConfig()}>Reset</Button>
      </div>

      {message && <p style={{ marginTop: 12 }}>{message}</p>}
      {issues.length > 0 && (
        <ul style={{ marginTop: 8 }}>
          {issues.map((issue, idx) => (
            <li key={idx}>{issue}</li>
          ))}
        </ul>
      )}
    </div>
  )
}
