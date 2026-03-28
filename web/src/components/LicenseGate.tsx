import React, { useState } from "react"
import { useAppState } from "../providers/AppStateProvider"
import { isFeatureAvailable } from "../lib/tier"

interface LicenseGateProps {
  feature: string
  description: string
  children: React.ReactNode
}

export default function LicenseGate({ feature, description, children }: LicenseGateProps) {
  const { tier, activateLicense } = useAppState()
  const [showDialog, setShowDialog] = useState(false)
  const [keyInput, setKeyInput] = useState("")
  const [result, setResult] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  if (isFeatureAvailable(feature, tier)) {
    return <>{children}</>
  }

  const onActivate = async () => {
    if (!keyInput.trim()) return
    setLoading(true)
    setResult(null)
    try {
      const status = await activateLicense(keyInput.trim())
      if (status.valid) {
        setResult(`License activated. ${status.tier.toUpperCase()} unlocked.`)
      } else {
        setResult("License key is invalid.")
      }
    } catch (error) {
      setResult(error instanceof Error ? error.message : "Validation failed")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div style={{ position: "relative" }}>
      <div style={{ filter: "blur(1px)", opacity: 0.4, pointerEvents: "none" }}>{children}</div>
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "grid",
          placeItems: "center",
          background: "rgba(12, 12, 12, 0.75)",
          border: "1px solid #fb923c",
          borderRadius: 8,
          padding: 16,
        }}
      >
        <div style={{ textAlign: "center", maxWidth: 520 }}>
          <h3 style={{ marginTop: 0 }}>Upgrade to Forge Pro</h3>
          <p>{description}</p>
          <button onClick={() => setShowDialog((prev) => !prev)}>Enter License Key</button>
          {showDialog && (
            <div style={{ marginTop: 12, display: "grid", gap: 8 }}>
              <input
                placeholder="FORGE-XXXX-XXXX-XXXX"
                value={keyInput}
                onChange={(event) => setKeyInput(event.target.value)}
              />
              <button onClick={onActivate} disabled={loading}>
                {loading ? "Activating..." : "Activate"}
              </button>
              {result && <p style={{ margin: 0 }}>{result}</p>}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
