import { useState, useEffect } from 'react'
import { checkEnvironment } from '../lib/ipc'
import type { EnvironmentStatus } from '../lib/types'

function CheckIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ color: 'var(--success)', flexShrink: 0 }}>
      <polyline points="20 6 9 17 4 12"/>
    </svg>
  )
}

function XIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ color: 'var(--danger)', flexShrink: 0 }}>
      <line x1="18" y1="6" x2="6" y2="18"/>
      <line x1="6" y1="6" x2="18" y2="18"/>
    </svg>
  )
}

interface CheckRowProps {
  name: string
  installed: boolean
  version?: string
}

function CheckRow({ name, installed, version }: CheckRowProps) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 12,
      padding: '10px 0',
      borderBottom: '1px solid var(--border)',
    }}>
      {installed ? <CheckIcon /> : <XIcon />}
      <div style={{ flex: 1 }}>
        <span style={{ fontWeight: 500, fontSize: 13 }}>{name}</span>
        {version && (
          <span className="text-muted" style={{ marginLeft: 8, fontSize: 12 }}>{version}</span>
        )}
      </div>
      <span className={installed ? 'badge badge-success' : 'badge badge-danger'}>
        {installed ? 'Installed' : 'Missing'}
      </span>
    </div>
  )
}

export default function Environment() {
  const [status, setStatus] = useState<EnvironmentStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  async function load() {
    setLoading(true)
    setError('')
    try {
      const s = await checkEnvironment()
      setStatus(s)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  return (
    <div className="page">
      <div className="page-header">
        <h1>Environment</h1>
        <button className="btn" onClick={load} disabled={loading}>
          {loading ? <span className="spinner" /> : null}
          Re-check
        </button>
      </div>

      {error && <div className="error-msg" style={{ marginBottom: 16 }}>{error}</div>}

      {loading && !status ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: 40 }}>
          <span className="spinner" />
        </div>
      ) : status ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
          <div className="card">
            <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 4 }}>Core Tools</div>
            <CheckRow name="Rust" installed={status.rust.installed} version={status.rust.version} />
            <CheckRow name="Cargo" installed={status.cargo.installed} version={status.cargo.version} />
            <CheckRow name="Node.js" installed={status.node.installed} version={status.node.version} />
            <CheckRow
              name="Tauri CLI"
              installed={status.tauri_cli.installed}
              version={status.tauri_cli.version}
            />
          </div>

          {status.platform_deps.length > 0 && (
            <div className="card">
              <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 4 }}>Platform Dependencies</div>
              {status.platform_deps.map(dep => (
                <CheckRow key={dep.name} name={dep.name} installed={dep.installed} />
              ))}
            </div>
          )}

          <div className="card">
            <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 8 }}>Summary</div>
            {[status.rust, status.cargo, status.node, status.tauri_cli].every(c => c.installed) &&
             status.platform_deps.every(d => d.installed) ? (
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, color: 'var(--success)' }}>
                <CheckIcon />
                <span style={{ fontSize: 13 }}>All requirements satisfied. Ready to build Tauri apps.</span>
              </div>
            ) : (
              <div style={{ fontSize: 13, color: 'var(--text-2)' }}>
                Some tools are missing. Install them to enable full Tauri development.
                {' '}
                <a
                  href="https://tauri.app/start/prerequisites/"
                  target="_blank"
                  rel="noreferrer"
                  style={{ color: 'var(--accent)' }}
                >
                  View prerequisites →
                </a>
              </div>
            )}
          </div>
        </div>
      ) : null}
    </div>
  )
}
