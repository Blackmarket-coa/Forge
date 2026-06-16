import React, { useState } from "react"
import { useSnackbar } from "notistack"
import { useAppState } from "../providers/AppStateProvider"
import EnvironmentCheck from "./EnvironmentCheck"
import UpdateChecker from "./UpdateChecker"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { ConfirmDialog } from "./ui/dialog"
import { Field } from "./ui/field"
import { Input } from "./ui/input"
import { PageHeader } from "./ui/page-header"
import styles from "./Settings.module.scss"

export default function Settings() {
  const { tier, licenseStatus, activateLicense, clearLicenseKey } =
    useAppState()
  const { enqueueSnackbar } = useSnackbar()
  const [draftKey, setDraftKey] = useState("")
  const [activating, setActivating] = useState(false)
  const [confirmRemove, setConfirmRemove] = useState(false)

  const onActivate = async () => {
    if (!draftKey.trim()) return
    setActivating(true)
    try {
      const status = await activateLicense(draftKey.trim())
      enqueueSnackbar(
        status.valid
          ? `Activated ${status.tier.toUpperCase()} tier`
          : "License key invalid",
        { variant: status.valid ? "success" : "error" }
      )
      if (status.valid) setDraftKey("")
    } catch (error) {
      enqueueSnackbar(
        error instanceof Error ? error.message : "Activation failed",
        { variant: "error" }
      )
    } finally {
      setActivating(false)
    }
  }

  const onRemove = async () => {
    await clearLicenseKey()
    setConfirmRemove(false)
    enqueueSnackbar("License removed. Free tier active.", { variant: "info" })
  }

  return (
    <div>
      <PageHeader
        title="Settings"
        subtitle="Manage your license and check the tools on your computer."
      />

      <div className={styles.grid}>
        <Card
          title="License"
          subtitle="Unlock app groups, multi-app builds, and the publish dashboard."
          actions={
            <Badge tone={tier === "free" ? "neutral" : "accent"}>
              {tier.toUpperCase()}
            </Badge>
          }
        >
          <dl className={styles.summary}>
            <dt>Key</dt>
            <dd>{licenseStatus.key_masked || "Not set"}</dd>
            <dt>Expires</dt>
            <dd>{licenseStatus.expires_at || "N/A"}</dd>
          </dl>

          <Field
            label="License key"
            help="Enter a Forge Pro or Team key to upgrade."
          >
            <div className={styles.keyRow}>
              <Input
                placeholder="FORGE-XXXX-XXXX-XXXX"
                value={draftKey}
                onChange={(e) => setDraftKey(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && onActivate()}
              />
              <Button
                variant="primary"
                onClick={onActivate}
                loading={activating}
                disabled={!draftKey.trim()}
              >
                Activate
              </Button>
            </div>
          </Field>

          <div className={styles.licenseFooter}>
            <a
              href="https://forge.dev/pricing"
              target="_blank"
              rel="noreferrer"
            >
              View pricing
            </a>
            {licenseStatus.key_masked && (
              <Button
                variant="danger"
                size="sm"
                onClick={() => setConfirmRemove(true)}
              >
                Remove key
              </Button>
            )}
          </div>
        </Card>

        <Card
          title="Build tools"
          subtitle="Forge builds apps right on your computer. Here's what's installed."
        >
          <EnvironmentCheck />
        </Card>

        <Card
          title="Updates"
          subtitle="Forge can update itself from signed releases."
        >
          <UpdateChecker />
        </Card>

        <Card title="About">
          <dl className={styles.summary}>
            <dt>Saved data</dt>
            <dd>~/.forge/forge.json</dd>
            <dt>License file</dt>
            <dd>~/.forge/license.json</dd>
          </dl>
        </Card>
      </div>

      <ConfirmDialog
        open={confirmRemove}
        title="Remove license key?"
        message="Forge will revert to the Free tier and Pro features will be locked."
        confirmLabel="Remove key"
        destructive
        onConfirm={onRemove}
        onCancel={() => setConfirmRemove(false)}
      />
    </div>
  )
}
