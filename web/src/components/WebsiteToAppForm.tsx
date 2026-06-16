import React, { useEffect, useMemo, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { useSnackbar } from "notistack"
import { createWebApp, getDefaultAppDir, ProjectMeta } from "../api/api"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Collapsible } from "./ui/collapsible"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { PageHeader } from "./ui/page-header"
import { Spinner } from "./ui/spinner"
import styles from "./WebsiteToAppForm.module.scss"

interface WebsiteToAppFormProps {
  onCreated: (project: ProjectMeta) => void
  onCancel: () => void
}

/** Turn raw input into a URL the preview can show; never throws. */
function previewUrl(value: string): string {
  const trimmed = value.trim()
  if (!trimmed) return ""
  return trimmed.includes("://") ? trimmed : `https://${trimmed}`
}

/** Suggest a friendly app name from a website address, e.g. "shop.acme.com" → "Acme". */
export function suggestNameFromUrl(value: string): string {
  try {
    const host = new URL(previewUrl(value)).hostname.replace(/^www\./, "")
    const parts = host.split(".").filter(Boolean)
    // Prefer the registrable label (e.g. "acme" in "shop.acme.com").
    const label = parts.length >= 2 ? parts[parts.length - 2] : parts[0] || ""
    return label ? label.charAt(0).toUpperCase() + label.slice(1) : ""
  } catch {
    return ""
  }
}

/** Mirror of the backend folder-name rule, used for the live preview. */
export function previewFolderName(name: string): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
  return slug || "my-app"
}

function joinPath(dir: string, child: string): string {
  if (!dir) return child
  const sep = dir.includes("\\") ? "\\" : "/"
  return `${dir.replace(/[/\\]+$/, "")}${sep}${child}`
}

export default function WebsiteToAppForm({
  onCreated,
  onCancel,
}: WebsiteToAppFormProps) {
  const { enqueueSnackbar } = useSnackbar()
  const [url, setUrl] = useState("")
  const [name, setName] = useState("")
  const [nameTouched, setNameTouched] = useState(false)
  const [location, setLocation] = useState("")
  const [width, setWidth] = useState(1200)
  const [height, setHeight] = useState(800)
  const [status, setStatus] = useState<"form" | "creating" | "done">("form")
  const [error, setError] = useState<string | null>(null)
  const [created, setCreated] = useState<ProjectMeta | null>(null)

  useEffect(() => {
    void getDefaultAppDir()
      .then(setLocation)
      .catch(() => {
        /* keep empty; user can pick a folder */
      })
  }, [])

  const handleUrlChange = (value: string) => {
    setUrl(value)
    // Offer a friendly name automatically until the user types their own.
    if (!nameTouched) {
      setName(suggestNameFromUrl(value))
    }
  }

  const preview = previewUrl(url)
  const folder = useMemo(() => previewFolderName(name), [name])
  const canCreate =
    url.trim().length > 0 && name.trim().length > 0 && location.length > 0

  const pickLocation = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") setLocation(selected)
  }

  const handleCreate = async () => {
    setError(null)
    setStatus("creating")
    try {
      const project = await createWebApp({
        parentDir: location,
        name: name.trim(),
        url: url.trim(),
        width,
        height,
      })
      setCreated(project)
      setStatus("done")
      enqueueSnackbar("Your app is ready!", { variant: "success" })
    } catch (e: any) {
      setError(String(e?.message || e))
      setStatus("form")
    }
  }

  const reset = () => {
    setUrl("")
    setName("")
    setNameTouched(false)
    setError(null)
    setCreated(null)
    setStatus("form")
  }

  if (status === "done" && created) {
    return (
      <div>
        <Card>
          <div className={styles.success}>
            <div className={styles.successIcon} aria-hidden>
              🎉
            </div>
            <h2 className={styles.successTitle}>{created.name} is ready!</h2>
            <p className={styles.muted}>
              We built your app project. It opens{" "}
              <strong>{preview || url}</strong> in its own window.
            </p>
            <div className={styles.pathPill} title={created.path}>
              {created.path}
            </div>

            <div className={styles.nextSteps}>
              <h3 className={styles.nextTitle}>What&rsquo;s next?</h3>
              <ol className={styles.steps}>
                <li>
                  Open your app below to preview it and build an installer you
                  can share.
                </li>
                <li>
                  Building installers uses free tools (Rust and the Tauri CLI).
                  Forge checks for these under{" "}
                  <strong>Settings → Tools on your computer</strong> and tells
                  you how to install anything that&rsquo;s missing.
                </li>
              </ol>
            </div>

            <div className={styles.successActions}>
              <Button variant="primary" onClick={() => onCreated(created)}>
                Open my app
              </Button>
              <Button variant="secondary" onClick={reset}>
                Make another app
              </Button>
            </div>
          </div>
        </Card>
      </div>
    )
  }

  return (
    <div>
      <Button variant="ghost" size="sm" onClick={onCancel}>
        ← Back
      </Button>
      <PageHeader
        title="Turn your website into an app"
        subtitle="Enter your website address and we'll create a desktop app that opens it. No coding required."
      />

      {error && (
        <div className={styles.errorBanner}>
          <Banner tone="danger" title="We couldn't create your app">
            {error}
          </Banner>
        </div>
      )}

      <div className={styles.layout}>
        <Card>
          <div className={styles.body}>
            <Field
              label="Your website address"
              help="The web page your app should open, for example yoursite.com."
            >
              <Input
                placeholder="yoursite.com"
                value={url}
                inputMode="url"
                autoFocus
                onChange={(e) => handleUrlChange(e.target.value)}
              />
            </Field>

            <Field
              label="App name"
              help="What your app is called on your computer."
            >
              <Input
                placeholder="My Site"
                value={name}
                onChange={(e) => {
                  setNameTouched(true)
                  setName(e.target.value)
                }}
              />
            </Field>

            <Collapsible title="More options (optional)" defaultOpen={false}>
              <div className={styles.body}>
                <Field
                  label="Where to save it"
                  help="A new folder for your app is created inside this location."
                >
                  <div className={styles.dirRow}>
                    <Input
                      value={location}
                      readOnly
                      placeholder="Choose a folder"
                    />
                    <Button variant="secondary" onClick={pickLocation}>
                      Change…
                    </Button>
                  </div>
                </Field>
                <Field label="Window size" help="How big the app window opens.">
                  <div className={styles.sizeRow}>
                    <Input
                      type="number"
                      min={400}
                      value={width}
                      aria-label="Window width"
                      onChange={(e) => setWidth(Number(e.target.value) || 1200)}
                    />
                    <span className={styles.times}>×</span>
                    <Input
                      type="number"
                      min={400}
                      value={height}
                      aria-label="Window height"
                      onChange={(e) => setHeight(Number(e.target.value) || 800)}
                    />
                  </div>
                </Field>
              </div>
            </Collapsible>
          </div>
        </Card>

        <Card title="Preview" subtitle="Here's what we'll create.">
          <dl className={styles.preview}>
            <dt>App name</dt>
            <dd>{name.trim() || "Your app"}</dd>
            <dt>Opens</dt>
            <dd className={styles.previewUrl}>{preview || "—"}</dd>
            <dt>Saved as</dt>
            <dd title={joinPath(location, folder)}>
              {location ? joinPath(location, folder) : folder}
            </dd>
            <dt>Icon</dt>
            <dd>Default icon — you can change it later</dd>
          </dl>
        </Card>
      </div>

      <div className={styles.footer}>
        <Button
          variant="primary"
          size="md"
          disabled={!canCreate || status === "creating"}
          loading={status === "creating"}
          onClick={handleCreate}
        >
          {status === "creating" ? "Creating your app…" : "Create my app"}
        </Button>
        {status === "creating" && (
          <span className={styles.creatingHint}>
            <Spinner size={14} /> This only takes a moment.
          </span>
        )}
      </div>
    </div>
  )
}
