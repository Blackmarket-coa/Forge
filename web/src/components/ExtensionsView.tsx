import React, { useCallback, useEffect, useState } from "react"
import { useSnackbar } from "notistack"
import { useAppState } from "../providers/AppStateProvider"
import { isFeatureAvailable } from "../lib/tier"
import {
  browsePlugins,
  getDefaultAppDir,
  getFbmStatus,
  packageExtension,
  publishExtension,
  scaffoldExtension,
  setFbmConfig,
  validateExtension,
  FbmStatus,
  PackageResult,
  PublishOutcome,
} from "../api/api"
import { Badge } from "./ui/badge"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { PageHeader } from "./ui/page-header"
import { Select } from "./ui/select"

/**
 * Extensions (W3): author a BMC extension, package it, and publish it into
 * the FreeBlackMarket registry (build → sign → publish — signing happens on
 * FBM's side at publish). Browsing uses the public registry list route and
 * finally honors the `plugin_browser` tier key; publishing is Pro-gated with
 * its own `extension_publish` key.
 */
export default function ExtensionsView() {
  const { tier } = useAppState()
  const { enqueueSnackbar } = useSnackbar()

  const [fbm, setFbm] = useState<FbmStatus | null>(null)
  const [draftBase, setDraftBase] = useState("")
  const [draftToken, setDraftToken] = useState("")

  const [name, setName] = useState("")
  const [template, setTemplate] = useState("featured-vendor-widget")
  const [projectPath, setProjectPath] = useState("")

  const [busy, setBusy] = useState<string | null>(null)
  const [issues, setIssues] = useState<string[]>([])
  const [packaged, setPackaged] = useState<PackageResult | null>(null)
  const [published, setPublished] = useState<PublishOutcome | null>(null)
  const [plugins, setPlugins] = useState<
    Array<{
      slug: string
      name: string
      category: string
      version: string
      install_count: number
    }>
  >([])

  const canBrowse = isFeatureAvailable("plugin_browser", tier)
  const canPublish = isFeatureAvailable("extension_publish", tier)

  const refreshFbm = useCallback(async () => {
    try {
      const status = await getFbmStatus()
      setFbm(status)
      setDraftBase(status.api_base_url ?? "")
    } catch {
      setFbm(null)
    }
  }, [])

  useEffect(() => {
    void refreshFbm()
  }, [refreshFbm])

  const run = async (label: string, action: () => Promise<void>) => {
    setBusy(label)
    try {
      await action()
    } catch (error) {
      enqueueSnackbar(error instanceof Error ? error.message : String(error), {
        variant: "error",
      })
    } finally {
      setBusy(null)
    }
  }

  const onSaveFbm = () =>
    run("settings", async () => {
      const status = await setFbmConfig(draftBase, draftToken)
      setFbm(status)
      setDraftToken("")
      enqueueSnackbar(
        status.configured
          ? "FreeBlackMarket connected"
          : "Saved (not fully configured yet)",
        { variant: status.configured ? "success" : "info" }
      )
    })

  const onScaffold = () =>
    run("scaffold", async () => {
      const parent = await getDefaultAppDir()
      const path = await scaffoldExtension(parent, name, template)
      setProjectPath(path)
      setIssues([])
      setPackaged(null)
      setPublished(null)
      enqueueSnackbar(`Created ${path}`, { variant: "success" })
    })

  const onValidate = () =>
    run("validate", async () => {
      const result = await validateExtension(projectPath)
      setIssues(result)
      enqueueSnackbar(
        result.length === 0 ? "Manifest is valid" : "Manifest has issues",
        {
          variant: result.length === 0 ? "success" : "warning",
        }
      )
    })

  const onPackage = () =>
    run("package", async () => {
      const result = await packageExtension(projectPath)
      setPackaged(result)
      setIssues(result.issues)
      enqueueSnackbar(
        result.issues.length === 0
          ? "Packaged with digests"
          : "Packaged with issues",
        { variant: result.issues.length === 0 ? "success" : "warning" }
      )
    })

  const onPublish = () =>
    run("publish", async () => {
      const outcome = await publishExtension(projectPath)
      setPublished(outcome)
      enqueueSnackbar(
        outcome.pluginSlug
          ? `Published ${outcome.pluginSlug}@${
              outcome.pluginVersion ?? "?"
            } to the registry`
          : "Published (listing signed)",
        { variant: "success" }
      )
    })

  const onBrowse = () =>
    run("browse", async () => {
      const result = await browsePlugins()
      setPlugins(result.plugins ?? [])
    })

  return (
    <div>
      <PageHeader
        title="Extensions"
        subtitle="Build BMC extensions and publish them to the FreeBlackMarket registry — FBM signs at publish; Forge never holds keys."
      />

      <Card
        title="FreeBlackMarket connection"
        subtitle="Stored on this computer only (~/.forge/fbm.json). The seller token is masked everywhere."
        actions={
          <Badge tone={fbm?.configured ? "accent" : "neutral"}>
            {fbm?.configured ? "CONNECTED" : "NOT CONFIGURED"}
          </Badge>
        }
      >
        <Field
          label="API address"
          help="Your FreeBlackMarket backend, e.g. https://api.freeblackmarket.com"
        >
          <Input
            value={draftBase}
            onChange={(e) => setDraftBase(e.target.value)}
            placeholder="https://api.freeblackmarket.com"
          />
        </Field>
        <Field
          label="Seller token"
          help={
            fbm?.seller_token_masked
              ? `Current: ${fbm.seller_token_masked} — leave blank to keep it.`
              : "A seller bearer token with listing publish access."
          }
        >
          <Input
            type="password"
            value={draftToken}
            onChange={(e) => setDraftToken(e.target.value)}
            placeholder="paste token"
          />
        </Field>
        <Button onClick={onSaveFbm} disabled={busy !== null}>
          {busy === "settings" ? "Saving…" : "Save connection"}
        </Button>
      </Card>

      <Card
        title="New extension"
        subtitle="Start from a template; the Featured Vendor Widget is the end-to-end demo path."
      >
        <Field label="Name">
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Vendor Widget"
          />
        </Field>
        <Field label="Template">
          <Select
            value={template}
            onChange={(e) => setTemplate(e.target.value)}
          >
            <option value="featured-vendor-widget">
              Featured Vendor Widget
            </option>
            <option value="blank">Blank manifest plugin</option>
          </Select>
        </Field>
        <Button onClick={onScaffold} disabled={busy !== null || !name.trim()}>
          {busy === "scaffold" ? "Creating…" : "Create extension"}
        </Button>
      </Card>

      <Card
        title="Package & publish"
        subtitle="Validate the manifest, compute digests, then publish through your seller account."
      >
        <Field
          label="Extension folder"
          help="The folder holding manifest.json."
        >
          <Input
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            placeholder="/path/to/my-vendor-widget"
          />
        </Field>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          <Button
            variant="secondary"
            onClick={onValidate}
            disabled={busy !== null || !projectPath}
          >
            {busy === "validate" ? "Checking…" : "Validate"}
          </Button>
          <Button
            variant="secondary"
            onClick={onPackage}
            disabled={busy !== null || !projectPath}
          >
            {busy === "package" ? "Packaging…" : "Package"}
          </Button>
          <Button
            onClick={onPublish}
            disabled={
              busy !== null || !projectPath || !canPublish || !fbm?.configured
            }
          >
            {busy === "publish" ? "Publishing…" : "Publish to registry"}
          </Button>
        </div>
        {!canPublish && (
          <Banner tone="info">
            Publishing to the registry is a Forge Pro feature.
          </Banner>
        )}
        {issues.length > 0 && (
          <Banner tone="warning">
            <strong>Manifest issues:</strong>
            <ul>
              {issues.map((issue) => (
                <li key={issue}>{issue}</li>
              ))}
            </ul>
          </Banner>
        )}
        {packaged && issues.length === 0 && (
          <p>
            Packaged to <code>{packaged.distDir}</code> (manifest{" "}
            <code>{packaged.digests.manifestSha256.slice(0, 12)}…</code>)
          </p>
        )}
        {published && (
          <Banner tone="success">
            Published listing <code>{published.listingId}</code>
            {published.pluginSlug
              ? ` → registry ${published.pluginSlug}@${
                  published.pluginVersion ?? "?"
                }`
              : ""}
          </Banner>
        )}
      </Card>

      <Card
        title="Registry"
        subtitle="Browse what's installable on the marketplace (public list)."
        actions={
          <Button
            variant="secondary"
            onClick={onBrowse}
            disabled={busy !== null || !canBrowse}
          >
            {busy === "browse" ? "Loading…" : "Browse plugins"}
          </Button>
        }
      >
        {!canBrowse && (
          <Banner tone="info">Registry browsing is a Forge Pro feature.</Banner>
        )}
        {plugins.length > 0 && (
          <ul data-testid="plugin-list">
            {plugins.map((plugin) => (
              <li key={plugin.slug}>
                <strong>{plugin.name}</strong> <code>{plugin.slug}</code> v
                {plugin.version} · {plugin.category} · {plugin.install_count}{" "}
                installs
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  )
}
