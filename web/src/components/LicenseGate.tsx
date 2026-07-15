import React, { useState } from "react"
import { useSnackbar } from "notistack"
import { useAppState } from "../providers/AppStateProvider"
import { isFeatureAvailable } from "../lib/tier"
import { Button } from "./ui/button"
import { ExternalLink } from "./ui/external-link"
import { Input } from "./ui/input"
import styles from "./LicenseGate.module.scss"

interface LicenseGateProps {
  feature: string
  description: string
  children: React.ReactNode
}

export default function LicenseGate({
  feature,
  description,
  children,
}: LicenseGateProps) {
  const { tier, activateLicense } = useAppState()
  const { enqueueSnackbar } = useSnackbar()
  const [showInput, setShowInput] = useState(false)
  const [keyInput, setKeyInput] = useState("")
  const [loading, setLoading] = useState(false)

  if (isFeatureAvailable(feature, tier)) {
    return <>{children}</>
  }

  const onActivate = async () => {
    if (!keyInput.trim()) return
    setLoading(true)
    try {
      const status = await activateLicense(keyInput.trim())
      enqueueSnackbar(
        status.valid
          ? `${status.tier.toUpperCase()} unlocked`
          : "License key is invalid",
        { variant: status.valid ? "success" : "error" }
      )
    } catch (error) {
      enqueueSnackbar(
        error instanceof Error ? error.message : "Validation failed",
        { variant: "error" }
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className={styles.gate}>
      <div className={styles.locked} aria-hidden>
        {children}
      </div>
      <div className={styles.overlay}>
        <div className={styles.panel}>
          <span className={styles.badge}>PRO</span>
          <h3 className={styles.title}>Upgrade to Forge Pro</h3>
          <p className={styles.description}>{description}</p>
          {showInput ? (
            <div className={styles.inputRow}>
              <Input
                placeholder="FORGE-XXXX-XXXX-XXXX"
                value={keyInput}
                onChange={(e) => setKeyInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && onActivate()}
              />
              <Button variant="primary" onClick={onActivate} loading={loading}>
                Activate
              </Button>
            </div>
          ) : (
            <div className={styles.actions}>
              <Button variant="primary" onClick={() => setShowInput(true)}>
                Enter License Key
              </Button>
              <ExternalLink
                href="https://forge.dev/pricing"
                className={styles.link}
              >
                See plans
              </ExternalLink>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
