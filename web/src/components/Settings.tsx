import React, { useState } from "react"
import { useAppState } from "../providers/AppStateProvider"

export default function Settings() {
  const { theme, toggleTheme, tier, licenseStatus, activateLicense, clearLicenseKey } = useAppState()
  const [draftKey, setDraftKey] = useState("")
  const [message, setMessage] = useState<string | null>(null)

  const onActivate = async () => {
    try {
      const status = await activateLicense(draftKey)
      setMessage(status.valid ? `Activated ${status.tier.toUpperCase()} tier.` : "License key invalid")
      setDraftKey("")
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Activation failed")
    }
  }

  const onRemove = async () => {
    await clearLicenseKey()
    setMessage("License key removed. Free tier active.")
  }

  return (
    <div style={{ padding: 24 }}>
      <h2>Settings</h2>
      <p>Current theme: {theme}</p>
      <button onClick={toggleTheme}>Toggle Theme</button>
      <p style={{ marginTop: 16 }}>State file: ~/.forge/forge.json</p>

      <section style={{ marginTop: 24, display: "grid", gap: 8, maxWidth: 560 }}>
        <h3>License</h3>
        <p>Current tier: <strong>{tier.toUpperCase()}</strong></p>
        <p>License key: {licenseStatus.key_masked || "Not set"}</p>
        <p>Expires at: {licenseStatus.expires_at || "N/A"}</p>
        <div style={{ display: "flex", gap: 8 }}>
          <input
            placeholder="Enter new license key"
            value={draftKey}
            onChange={(event) => setDraftKey(event.target.value)}
          />
          <button onClick={onActivate} disabled={!draftKey.trim()}>
            Change Key
          </button>
          <button onClick={onRemove}>Remove Key</button>
        </div>
        {message && <p>{message}</p>}
        <a href="https://forge.dev/pricing" target="_blank" rel="noreferrer">
          View pricing
        </a>
      </section>
    </div>
  )
}
