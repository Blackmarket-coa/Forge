import { useState, useEffect, useRef, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  getProjects,
  detectTauriStatus,
  readConfig,
  writeConfig,
  validateConfig,
  runDev,
  runBuild,
  killProcess,
  getBuildHistory,
  collectArtifacts,
  listenProcessOutput,
} from '../lib/ipc'
import type { ProjectMeta, TauriStatus, BuildRecord, Artifact } from '../lib/types'

type Tab = 'overview' | 'config' | 'build' | 'artifacts' | 'history'

const BUILD_TARGETS = ['deb', 'appimage', 'dmg', 'app', 'msi', 'nsis', 'updater']

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function formatDuration(secs: number) {
  if (secs < 60) return `${secs.toFixed(1)}s`
  const m = Math.floor(secs / 60)
  const s = Math.floor(secs % 60)
  return `${m}m ${s}s`
}

interface TerminalLine {
  text: string
  isStderr: boolean
}

interface TerminalPanelProps {
  lines: TerminalLine[]
}

function TerminalPanel({ lines }: TerminalPanelProps) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [lines])

  return (
    <div className="terminal">
      {lines.length === 0 && <span style={{ color: 'var(--text-2)' }}>No output yet...</span>}
      {lines.map((line, i) => (
        <span key={i} className={line.isStderr ? 'stderr' : undefined}>
          {line.text}
        </span>
      ))}
      <div ref={bottomRef} />
    </div>
  )
}

// Overview tab
interface OverviewTabProps {
  project: ProjectMeta
  tauriStatus: TauriStatus | null
  tauriLoading: boolean
  activeProcessId: string | null
  terminalLines: TerminalLine[]
  onRunDev: () => void
  onRunBuild: () => void
  onKill: () => void
}

function OverviewTab({
  project,
  tauriStatus,
  tauriLoading,
  activeProcessId,
  terminalLines,
  onRunDev,
  onRunBuild,
  onKill,
}: OverviewTabProps) {
  return (
    <div>
      <div className="card" style={{ marginBottom: 16 }}>
        <div style={{ fontWeight: 600, marginBottom: 10, fontSize: 13 }}>Project Info</div>
        <div className="info-row"><span className="info-label">Path</span><span>{project.path}</span></div>
        <div className="info-row"><span className="info-label">Status</span><span>{project.status}</span></div>
        {project.git_branch && (
          <div className="info-row">
            <span className="info-label">Git Branch</span>
            <span>{project.git_branch}{project.git_dirty ? ' (dirty)' : ''}</span>
          </div>
        )}
        {project.identifier && (
          <div className="info-row"><span className="info-label">Identifier</span><span>{project.identifier}</span></div>
        )}
        {project.tauri_version && (
          <div className="info-row"><span className="info-label">Tauri Version</span><span>{project.tauri_version}</span></div>
        )}
        {project.frontend_framework && (
          <div className="info-row"><span className="info-label">Framework</span><span>{project.frontend_framework}</span></div>
        )}
      </div>

      {tauriLoading ? (
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
          <span className="spinner" /><span className="text-muted">Detecting Tauri status...</span>
        </div>
      ) : tauriStatus && (
        <div className="card" style={{ marginBottom: 16 }}>
          <div style={{ fontWeight: 600, marginBottom: 10, fontSize: 13 }}>Tauri Status</div>
          <div className="info-row">
            <span className="info-label">Has tauri.conf.json</span>
            <span>{tauriStatus.has_tauri_conf ? '✓ Yes' : '✗ No'}</span>
          </div>
          {tauriStatus.product_name && (
            <div className="info-row"><span className="info-label">Product Name</span><span>{tauriStatus.product_name}</span></div>
          )}
          {tauriStatus.version && (
            <div className="info-row"><span className="info-label">Version</span><span>{tauriStatus.version}</span></div>
          )}
          <div className="info-row"><span className="info-label">Status</span><span>{tauriStatus.status}</span></div>
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <button className="btn btn-primary" onClick={onRunDev} disabled={!!activeProcessId}>
          ▶ Run Dev
        </button>
        <button className="btn" onClick={onRunBuild} disabled={!!activeProcessId}>
          &#x1F528; Build
        </button>
        {activeProcessId && (
          <button className="btn btn-danger" onClick={onKill}>
            &#x25A0; Kill Process
          </button>
        )}
      </div>

      {terminalLines.length > 0 && (
        <div>
          <div className="label" style={{ marginBottom: 6 }}>Terminal Output</div>
          <TerminalPanel lines={terminalLines} />
        </div>
      )}
    </div>
  )
}

// Config tab
interface ConfigTabProps {
  project: ProjectMeta
}

function ConfigTab({ project }: ConfigTabProps) {
  const [configText, setConfigText] = useState('')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [validating, setValidating] = useState(false)
  const [errors, setErrors] = useState<string[]>([])
  const [saved, setSaved] = useState(false)
  const [loadError, setLoadError] = useState('')

  useEffect(() => {
    setLoading(true)
    readConfig(project.path)
      .then(cfg => setConfigText(JSON.stringify(cfg, null, 2)))
      .catch(err => setLoadError(String(err)))
      .finally(() => setLoading(false))
  }, [project.path])

  async function handleValidate() {
    setValidating(true)
    setErrors([])
    try {
      const parsed = JSON.parse(configText)
      const errs = await validateConfig(project.path, parsed)
      setErrors(errs)
    } catch (err) {
      setErrors([String(err)])
    } finally {
      setValidating(false)
    }
  }

  async function handleSave() {
    setSaving(true)
    setErrors([])
    try {
      const parsed = JSON.parse(configText)
      await writeConfig(project.path, parsed)
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (err) {
      setErrors([String(err)])
    } finally {
      setSaving(false)
    }
  }

  if (loading) return <div style={{ padding: 20, display: 'flex', gap: 8 }}><span className="spinner" /><span className="text-muted">Loading config...</span></div>
  if (loadError) return <div className="error-msg" style={{ padding: 20 }}>{loadError}</div>

  return (
    <div>
      <div style={{ display: 'flex', gap: 8, marginBottom: 10 }}>
        <button className="btn" onClick={handleValidate} disabled={validating}>
          {validating ? <span className="spinner" /> : null}
          Validate
        </button>
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? <span className="spinner" /> : null}
          {saved ? '✓ Saved' : 'Save'}
        </button>
      </div>
      {errors.length > 0 && (
        <div style={{ marginBottom: 10 }}>
          {errors.map((e, i) => <div key={i} className="error-msg">{e}</div>)}
        </div>
      )}
      <textarea
        className="input"
        value={configText}
        onChange={e => setConfigText(e.target.value)}
        style={{ fontFamily: 'monospace', fontSize: 12, minHeight: 400, resize: 'vertical' }}
        spellCheck={false}
      />
    </div>
  )
}

// Build tab
interface BuildTabProps {
  project: ProjectMeta
  activeProcessId: string | null
  terminalLines: TerminalLine[]
  onRunBuild: (targets: string[]) => void
  onKill: () => void
}

function BuildTab({ project: _project, activeProcessId, terminalLines, onRunBuild, onKill }: BuildTabProps) {
  const [selectedTargets, setSelectedTargets] = useState<string[]>([])

  function toggleTarget(t: string) {
    setSelectedTargets(prev =>
      prev.includes(t) ? prev.filter(x => x !== t) : [...prev, t]
    )
  }

  return (
    <div>
      <div className="card" style={{ marginBottom: 16 }}>
        <div style={{ fontWeight: 600, marginBottom: 10, fontSize: 13 }}>Build Targets</div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10, marginBottom: 14 }}>
          {BUILD_TARGETS.map(t => (
            <label key={t} style={{ display: 'flex', alignItems: 'center', gap: 6, cursor: 'pointer', fontSize: 13 }}>
              <input
                type="checkbox"
                checked={selectedTargets.includes(t)}
                onChange={() => toggleTarget(t)}
                style={{ accentColor: 'var(--accent)' }}
              />
              {t}
            </label>
          ))}
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <button
            className="btn btn-primary"
            disabled={!!activeProcessId}
            onClick={() => onRunBuild(selectedTargets)}
          >
            &#x1F528; Run Build
          </button>
          {activeProcessId && (
            <button className="btn btn-danger" onClick={onKill}>
              &#x25A0; Kill
            </button>
          )}
        </div>
      </div>
      <div className="label" style={{ marginBottom: 6 }}>Terminal Output</div>
      <TerminalPanel lines={terminalLines} />
    </div>
  )
}

// Artifacts tab
interface ArtifactsTabProps {
  project: ProjectMeta
}

function ArtifactsTab({ project }: ArtifactsTabProps) {
  const [artifacts, setArtifacts] = useState<Artifact[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    setLoading(true)
    collectArtifacts(project.path)
      .then(setArtifacts)
      .catch(err => setError(String(err)))
      .finally(() => setLoading(false))
  }, [project.path])

  if (loading) return <div style={{ display: 'flex', gap: 8, padding: 20 }}><span className="spinner" /><span className="text-muted">Loading artifacts...</span></div>
  if (error) return <div className="error-msg" style={{ padding: 20 }}>{error}</div>

  if (artifacts.length === 0) {
    return (
      <div className="empty-state">
        <div style={{ fontSize: 28 }}>&#x1F4E6;</div>
        <h3>No artifacts</h3>
        <p>Build the project to generate artifacts.</p>
      </div>
    )
  }

  return (
    <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
      <table className="table">
        <thead>
          <tr>
            <th>File</th>
            <th>Format</th>
            <th>Size</th>
          </tr>
        </thead>
        <tbody>
          {artifacts.map((a, i) => (
            <tr key={i}>
              <td style={{ fontFamily: 'monospace', fontSize: 12, wordBreak: 'break-all' }}>{a.path}</td>
              <td><span className="badge">{a.format}</span></td>
              <td className="text-muted">{formatBytes(a.size_bytes)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

// History tab
interface HistoryTabProps {
  project: ProjectMeta
}

function HistoryTab({ project }: HistoryTabProps) {
  const [history, setHistory] = useState<BuildRecord[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    setLoading(true)
    getBuildHistory(project.id, 50)
      .then(setHistory)
      .catch(err => setError(String(err)))
      .finally(() => setLoading(false))
  }, [project.id])

  if (loading) return <div style={{ display: 'flex', gap: 8, padding: 20 }}><span className="spinner" /><span className="text-muted">Loading history...</span></div>
  if (error) return <div className="error-msg" style={{ padding: 20 }}>{error}</div>

  if (history.length === 0) {
    return (
      <div className="empty-state">
        <div style={{ fontSize: 28 }}>&#x1F4DC;</div>
        <h3>No build history</h3>
        <p>Build records will appear here after builds complete.</p>
      </div>
    )
  }

  return (
    <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
      <table className="table">
        <thead>
          <tr>
            <th>Started</th>
            <th>Status</th>
            <th>Targets</th>
            <th>Duration</th>
            <th>Artifacts</th>
          </tr>
        </thead>
        <tbody>
          {history.map(rec => (
            <tr key={rec.id}>
              <td className="text-muted" style={{ whiteSpace: 'nowrap' }}>{rec.started_at}</td>
              <td>
                <span className={
                  rec.status === 'success' ? 'badge badge-success' :
                  rec.status === 'failed' ? 'badge badge-danger' :
                  'badge badge-warning'
                }>
                  {rec.status}
                </span>
              </td>
              <td>{rec.targets.join(', ') || 'all'}</td>
              <td className="text-muted">{formatDuration(rec.duration_secs)}</td>
              <td className="text-muted">{rec.artifacts.length}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export default function ProjectDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [project, setProject] = useState<ProjectMeta | null>(null)
  const [tauriStatus, setTauriStatus] = useState<TauriStatus | null>(null)
  const [tauriLoading, setTauriLoading] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [activeTab, setActiveTab] = useState<Tab>('overview')
  const [activeProcessId, setActiveProcessId] = useState<string | null>(null)
  const [terminalLines, setTerminalLines] = useState<TerminalLine[]>([])
  const unlistenRef = useRef<(() => void) | null>(null)

  // Set up process output listener
  useEffect(() => {
    let active = true
    listenProcessOutput(ev => {
      if (!active) return
      if (activeProcessId && ev.process_id !== activeProcessId) return
      setTerminalLines(prev => [...prev, { text: ev.data, isStderr: ev.is_stderr }])
    }).then(fn => {
      if (!active) { fn(); return }
      unlistenRef.current = fn
    })
    return () => {
      active = false
      unlistenRef.current?.()
      unlistenRef.current = null
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProcessId])

  const loadProject = useCallback(async () => {
    if (!id) return
    setLoading(true)
    setError('')
    try {
      const projects = await getProjects()
      const found = projects.find(p => p.id === id)
      if (!found) {
        setError('Project not found')
        return
      }
      setProject(found)
      setTauriLoading(true)
      detectTauriStatus(found.path)
        .then(setTauriStatus)
        .catch(() => {})
        .finally(() => setTauriLoading(false))
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => { loadProject() }, [loadProject])

  async function handleRunDev() {
    if (!project) return
    setTerminalLines([])
    try {
      const processId = await runDev(project.path)
      setActiveProcessId(processId)
    } catch (err) {
      setTerminalLines([{ text: String(err), isStderr: true }])
    }
  }

  async function handleRunBuild(targets: string[]) {
    if (!project) return
    setTerminalLines([])
    try {
      await runBuild(project.path, targets)
      setActiveProcessId(null)
    } catch (err) {
      setTerminalLines(prev => [...prev, { text: String(err), isStderr: true }])
      setActiveProcessId(null)
    }
  }

  async function handleKill() {
    if (!activeProcessId) return
    try {
      await killProcess(activeProcessId)
      setActiveProcessId(null)
    } catch (err) {
      setTerminalLines(prev => [...prev, { text: String(err), isStderr: true }])
    }
  }

  if (loading) {
    return (
      <div className="page" style={{ display: 'flex', justifyContent: 'center', paddingTop: 60 }}>
        <span className="spinner" />
      </div>
    )
  }

  if (error || !project) {
    return (
      <div className="page">
        <div className="error-msg" style={{ marginBottom: 12 }}>{error || 'Project not found'}</div>
        <button className="btn" onClick={() => navigate('/')}>Back to Dashboard</button>
      </div>
    )
  }

  const TABS: { key: Tab; label: string }[] = [
    { key: 'overview', label: 'Overview' },
    { key: 'config', label: 'Config' },
    { key: 'build', label: 'Build' },
    { key: 'artifacts', label: 'Artifacts' },
    { key: 'history', label: 'History' },
  ]

  return (
    <div className="page">
      <div style={{ marginBottom: 16 }}>
        <button className="btn btn-ghost" style={{ marginBottom: 10, fontSize: 12 }} onClick={() => navigate('/')}>
          ← Back
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
          <h1 style={{ fontSize: 20, fontWeight: 700, letterSpacing: '-0.02em' }}>{project.name}</h1>
          <span className={
            project.status === 'ready' || project.status === 'ok' ? 'badge badge-success' :
            project.status === 'building' ? 'badge badge-warning' :
            project.status === 'error' ? 'badge badge-danger' : 'badge'
          }>{project.status}</span>
          {project.git_branch && (
            <span className="badge">{project.git_dirty ? '* ' : ''}{project.git_branch}</span>
          )}
          {activeProcessId && <span className="badge badge-warning">Process running</span>}
        </div>
        <div className="text-muted" style={{ marginTop: 4 }}>{project.path}</div>
      </div>

      <div className="tabs">
        {TABS.map(t => (
          <button
            key={t.key}
            className={`tab${activeTab === t.key ? ' active' : ''}`}
            onClick={() => setActiveTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      {activeTab === 'overview' && (
        <OverviewTab
          project={project}
          tauriStatus={tauriStatus}
          tauriLoading={tauriLoading}
          activeProcessId={activeProcessId}
          terminalLines={terminalLines}
          onRunDev={handleRunDev}
          onRunBuild={() => { setActiveTab('build') }}
          onKill={handleKill}
        />
      )}
      {activeTab === 'config' && <ConfigTab project={project} />}
      {activeTab === 'build' && (
        <BuildTab
          project={project}
          activeProcessId={activeProcessId}
          terminalLines={terminalLines}
          onRunBuild={handleRunBuild}
          onKill={handleKill}
        />
      )}
      {activeTab === 'artifacts' && <ArtifactsTab project={project} />}
      {activeTab === 'history' && <HistoryTab project={project} />}
    </div>
  )
}
