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
      setMessage(
        nextIssues.length
          ? "Validation completed with issues"
          : "Validation passed"
      )
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
    setMessage("Added beginner-friendly defaults. Review and Save when ready.")
  }

  const sectionHintStyle: React.CSSProperties = {
    fontSize: 12,
    opacity: 0.75,
    margin: "2px 0 10px 0",
  }
  const rowStyle: React.CSSProperties = {
    display: "grid",
    gridTemplateColumns: "minmax(180px, 240px) minmax(240px, 1fr)",
    alignItems: "center",
    gap: 10,
    marginBottom: 8,
  }
  const helpStyle: React.CSSProperties = {
    fontSize: 12,
    opacity: 0.75,
    marginBottom: 8,
    gridColumn: "2 / -1",
  }
  const labelStyle: React.CSSProperties = { fontWeight: 600 }

  return (
    <div style={{ padding: 24 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
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
      <p style={{ margin: "4px 0 14px 0", opacity: 0.9 }}>
        Use <strong>Form</strong> for guided editing. Use <strong>JSON</strong>{" "}
        only if you need advanced options.
      </p>

      {mode === "form" ? (
        <div style={{ display: "grid", gap: 12 }}>
          <Collapsible title="App Identity">
            <div style={sectionHintStyle}>
              These values appear in installers and app metadata.
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>App name</label>
              <Input
                placeholder="My App"
                value={config?.productName || ""}
                onChange={(e) => setField(["productName"], e.target.value)}
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Bundle ID</label>
              <Input
                placeholder="com.example.myapp"
                value={config?.identifier || ""}
                onChange={(e) => setField(["identifier"], e.target.value)}
              />
            </div>
            <div style={helpStyle}>
              Technical key: <code>identifier</code> (reverse-domain format).
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Version</label>
              <Input
                placeholder="1.0.0"
                value={config?.version || ""}
                onChange={(e) => setField(["version"], e.target.value)}
              />
            </div>
            <div style={helpStyle}>Use semantic versioning, like 1.0.0.</div>
          </Collapsible>

          <Collapsible title="Windows">
            <div style={sectionHintStyle}>
              Controls the first app window users see.
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Window title</label>
              <Input
                value={windows0?.title || ""}
                onChange={(e) => setWindowField("title", e.target.value)}
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Width (px)</label>
              <Input
                type="number"
                value={windows0?.width || 0}
                onChange={(e) =>
                  setWindowField("width", Number(e.target.value))
                }
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Height (px)</label>
              <Input
                type="number"
                value={windows0?.height || 0}
                onChange={(e) =>
                  setWindowField("height", Number(e.target.value))
                }
              />
            </div>
            <div
              style={{
                display: "flex",
                gap: 16,
                flexWrap: "wrap",
                marginTop: 6,
              }}
            >
              <label>
                <Checkbox
                  checked={!!windows0?.resizable}
                  onChange={(e) =>
                    setWindowField("resizable", e.target.checked)
                  }
                />{" "}
                Allow resize
              </label>
              <label>
                <Checkbox
                  checked={!!windows0?.fullscreen}
                  onChange={(e) =>
                    setWindowField("fullscreen", e.target.checked)
                  }
                />{" "}
                Start fullscreen
              </label>
              <label>
                <Checkbox
                  checked={!!windows0?.decorations}
                  onChange={(e) =>
                    setWindowField("decorations", e.target.checked)
                  }
                />{" "}
                Show frame controls
              </label>
            </div>
          </Collapsible>

          <Collapsible title="Build">
            <div style={sectionHintStyle}>
              How Forge runs your app in development and production builds.
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Dev URL</label>
              <Input
                placeholder="http://localhost:3000"
                value={build?.devUrl || ""}
                onChange={(e) => setField(["build", "devUrl"], e.target.value)}
              />
            </div>
            <div style={helpStyle}>
              Where your frontend dev server runs (<code>build.devUrl</code>).
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Built frontend folder</label>
              <Input
                placeholder="../dist"
                value={build?.frontendDist || ""}
                onChange={(e) =>
                  setField(["build", "frontendDist"], e.target.value)
                }
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Before dev command</label>
              <Input
                value={build?.beforeDevCommand || ""}
                onChange={(e) =>
                  setField(["build", "beforeDevCommand"], e.target.value)
                }
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Before build command</label>
              <Input
                value={build?.beforeBuildCommand || ""}
                onChange={(e) =>
                  setField(["build", "beforeBuildCommand"], e.target.value)
                }
              />
            </div>
          </Collapsible>

          <Collapsible title="Bundle">
            <div style={sectionHintStyle}>
              Installer and app-store metadata.
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Icon path(s)</label>
              <Input
                value={
                  Array.isArray(bundle?.icon)
                    ? bundle.icon.join(",")
                    : bundle?.icon || ""
                }
                onChange={(e) => setField(["bundle", "icon"], e.target.value)}
              />
            </div>
            <div style={helpStyle}>
              Use comma-separated paths for multiple icons.
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Copyright</label>
              <Input
                value={bundle?.copyright || ""}
                onChange={(e) =>
                  setField(["bundle", "copyright"], e.target.value)
                }
              />
            </div>
            <div style={rowStyle}>
              <label style={labelStyle}>Category</label>
              <Select
                value={bundle?.category || ""}
                onChange={(e) =>
                  setField(["bundle", "category"], e.target.value)
                }
              >
                <option value="">Select category</option>
                {categoryOptions.map((c) => (
                  <option key={c} value={c}>
                    {c}
                  </option>
                ))}
              </Select>
            </div>
            <div style={{ ...labelStyle, marginBottom: 8 }}>
              Installer targets
            </div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 14 }}>
              {(["dmg", "appimage", "deb", "nsis", "msi"] as const).map(
                (target) => (
                  <label key={target}>
                    <Checkbox
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
                    />{" "}
                    {target}
                  </label>
                )
              )}
            </div>
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
        <Button onClick={useRecommendedDefaults}>
          Use Recommended Defaults
        </Button>
        <Button onClick={handleSave}>Save</Button>
        <Button onClick={handleValidate}>Validate</Button>
        <Button onClick={() => void loadConfig()}>Reset</Button>
      </div>

      {message && (
        <p
          style={{
            marginTop: 12,
            color: message.toLowerCase().includes("failed")
              ? "#ff8f8f"
              : "#9be6b4",
          }}
        >
          {message}
        </p>
      )}
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
