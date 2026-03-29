import { useState, useEffect } from 'react'
import { getLicenseStatus, validateLicense, clearLicense } from '../lib/ipc'
import type { LicenseStatus } from '../lib/types'

export default function Settings() {
  const [license, setLicense] = useState<LicenseStatus | null>(null)
  const [licenseLoading, setLicenseLoading] = useState(true)
  const [licenseError, setLicenseError] = useState('')
  const [keyInput, setKeyInput] = useState('')
  const [validating, setValidating] = useState(false)
  const [clearing, setClearing] = useState(false)
  const [validateMsg, setValidateMsg] = useState('')

  useEffect(() => {
    getLicenseStatus()
      .then(setLicense)
      .catch(err => setLicenseError(String(err)))
      .finally(() => setLicenseLoading(false))
  }, [])

  async function handleValidate(e: React.FormEvent) {
    e.preventDefault()
    if (!keyInput.trim()) return
    setValidating(true)
    setLicenseError('')
    setValidateMsg('')
    try {
      const result = await validateLicense(keyInput.trim())
      setLicense(result)
      setValidateMsg(result.valid ? 'License validated successfully.' : 'License key is not valid.')
      if (result.valid) setKeyInput('')
    } catch (err) {
      setLicenseError(String(err))
    } finally {
      setValidating(false)
    }
  }

  async function handleClear() {
    if (!confirm('Clear the current license?')) return
    setClearing(true)
    setLicenseError('')
    setValidateMsg('')
    try {
      const result = await clearLicense()
      setLicense(result)
    } catch (err) {
      setLicenseError(String(err))
    } finally {
      setClearing(false)
    }
  }

  function tierBadgeClass(tier: string) {
    if (tier === 'pro' || tier === 'enterprise') return 'badge badge-success'
    if (tier === 'trial') return 'badge badge-warning'
    return 'badge'
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>Settings</h1>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 560 }}>
        {/* License section */}
        <div className="card">
          <div style={{ fontWeight: 700, fontSize: 15, marginBottom: 14 }}>License</div>

          {licenseLoading ? (
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <span className="spinner" /><span className="text-muted">Loading license status...</span>
            </div>
          ) : license ? (
            <div style={{ marginBottom: 16 }}>
              <div className="info-row">
                <span className="info-label">Status</span>
                <span className={license.valid ? 'badge badge-success' : 'badge badge-danger'}>
                  {license.valid ? 'Active' : 'Inactive'}
                </span>
              </div>
              <div className="info-row">
                <span className="info-label">Tier</span>
                <span className={tierBadgeClass(license.tier)}>{license.tier}</span>
              </div>
              {license.expires_at && (
                <div className="info-row">
                  <span className="info-label">Expires</span>
                  <span>{license.expires_at}</span>
                </div>
              )}
            </div>
          ) : null}

          {licenseError && <div className="error-msg" style={{ marginBottom: 10 }}>{licenseError}</div>}
          {validateMsg && (
            <div style={{
              fontSize: 12, marginBottom: 10,
              color: validateMsg.includes('successfully') ? 'var(--success)' : 'var(--warning)',
            }}>
              {validateMsg}
            </div>
          )}

          <form onSubmit={handleValidate} style={{ marginBottom: 10 }}>
            <label className="label">Enter License Key</label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                className="input"
                value={keyInput}
                onChange={e => setKeyInput(e.target.value)}
                placeholder="XXXX-XXXX-XXXX-XXXX"
                style={{ flex: 1 }}
              />
              <button
                type="submit"
                className="btn btn-primary"
                disabled={validating || !keyInput.trim()}
              >
                {validating ? <span className="spinner" /> : null}
                Validate
              </button>
            </div>
          </form>

          {license?.valid && (
            <button className="btn btn-danger" onClick={handleClear} disabled={clearing}>
              {clearing ? <span className="spinner" /> : null}
              Clear License
            </button>
          )}
        </div>

        {/* App info */}
        <div className="card">
          <div style={{ fontWeight: 700, fontSize: 15, marginBottom: 14 }}>About Forge</div>
          <div className="info-row">
            <span className="info-label">Application</span>
            <span>Forge</span>
          </div>
          <div className="info-row">
            <span className="info-label">Version</span>
            <span>0.1.0</span>
          </div>
          <div className="info-row">
            <span className="info-label">Description</span>
            <span className="text-muted">Visual project manager for Tauri apps</span>
          </div>
          <div className="info-row">
            <span className="info-label">Built with</span>
            <span className="text-muted">Tauri v2 + React 18 + Vite 5</span>
          </div>
        </div>
      </div>
    </div>
  )
}
