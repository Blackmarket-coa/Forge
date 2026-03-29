import { useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { getProjects, getWorkspaces, registerProject, scanDirectory } from '../lib/ipc'
import type { ProjectMeta, Workspace } from '../lib/types'

function statusBadgeClass(status: string) {
  if (status === 'ready' || status === 'ok') return 'badge badge-success'
  if (status === 'building') return 'badge badge-warning'
  if (status === 'error') return 'badge badge-danger'
  return 'badge'
}

interface AddProjectDialogProps {
  onClose: () => void
  onAdded: (p: ProjectMeta) => void
}

function AddProjectDialog({ onClose, onAdded }: AddProjectDialogProps) {
  const [path, setPath] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!path.trim()) return
    setLoading(true)
    setError('')
    try {
      const project = await registerProject(path.trim())
      onAdded(project)
      onClose()
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={e => e.stopPropagation()}>
        <h2>Add Project</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="label">Project Path</label>
            <input
              className="input"
              value={path}
              onChange={e => setPath(e.target.value)}
              placeholder="/path/to/your/tauri-project"
              autoFocus
            />
            {error && <div className="error-msg">{error}</div>}
          </div>
          <div className="dialog-footer">
            <button type="button" className="btn btn-ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn-primary" disabled={loading || !path.trim()}>
              {loading ? <span className="spinner" /> : null}
              Add Project
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

interface ScanDialogProps {
  onClose: () => void
  onFound: (projects: ProjectMeta[]) => void
}

function ScanDialog({ onClose, onFound }: ScanDialogProps) {
  const [path, setPath] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!path.trim()) return
    setLoading(true)
    setError('')
    try {
      const found = await scanDirectory(path.trim())
      onFound(found)
      onClose()
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={e => e.stopPropagation()}>
        <h2>Scan Directory</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="label">Directory to Scan</label>
            <input
              className="input"
              value={path}
              onChange={e => setPath(e.target.value)}
              placeholder="/path/to/scan"
              autoFocus
            />
            {error && <div className="error-msg">{error}</div>}
          </div>
          <div className="dialog-footer">
            <button type="button" className="btn btn-ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn-primary" disabled={loading || !path.trim()}>
              {loading ? <span className="spinner" /> : null}
              Scan
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

interface ProjectCardProps {
  project: ProjectMeta
  workspaces: Workspace[]
  onOpen: () => void
}

function ProjectCard({ project, onOpen }: ProjectCardProps) {
  return (
    <div className="card project-card" style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 8 }}>
        <div>
          <div style={{ fontWeight: 600, fontSize: 15, marginBottom: 2 }}>{project.name}</div>
          <div className="text-muted" style={{ wordBreak: 'break-all', fontSize: 11 }}>{project.path}</div>
        </div>
        <span className={statusBadgeClass(project.status)}>{project.status}</span>
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
        {project.tauri_version && (
          <span className="badge">Tauri {project.tauri_version}</span>
        )}
        {project.frontend_framework && (
          <span className="badge">{project.frontend_framework}</span>
        )}
        {project.git_branch && (
          <span className="badge">
            {project.git_dirty ? '* ' : ''}{project.git_branch}
          </span>
        )}
        {project.platforms.map(p => (
          <span key={p} className="badge">{p}</span>
        ))}
      </div>

      <div style={{ display: 'flex', gap: 6, marginTop: 'auto', paddingTop: 4 }}>
        <button className="btn btn-primary" style={{ fontSize: 12, padding: '4px 10px' }} onClick={onOpen}>
          Open
        </button>
      </div>
    </div>
  )
}

export default function Dashboard() {
  const navigate = useNavigate()
  const [projects, setProjects] = useState<ProjectMeta[]>([])
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showAdd, setShowAdd] = useState(false)
  const [showScan, setShowScan] = useState(false)

  const load = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const [projs, ws] = await Promise.all([getProjects(), getWorkspaces()])
      setProjects(projs)
      setWorkspaces(ws)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  function handleAdded(p: ProjectMeta) {
    setProjects(prev => [...prev.filter(x => x.id !== p.id), p])
  }

  function handleFound(found: ProjectMeta[]) {
    setProjects(prev => {
      const ids = new Set(prev.map(x => x.id))
      const fresh = found.filter(x => !ids.has(x.id))
      return [...prev, ...fresh]
    })
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>Dashboard</h1>
        <div className="page-header-actions">
          <button className="btn" onClick={() => setShowScan(true)}>&#x1F50D; Scan Directory</button>
          <button className="btn btn-primary" onClick={() => setShowAdd(true)}>+ Add Project</button>
        </div>
      </div>

      {error && <div className="error-msg" style={{ marginBottom: 16 }}>{error}</div>}

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: 40 }}>
          <span className="spinner" />
        </div>
      ) : projects.length === 0 ? (
        <div className="empty-state">
          <div style={{ fontSize: 32 }}>&#x1F528;</div>
          <h3>No projects yet</h3>
          <p>Add a Tauri project directory or scan a folder to find existing projects.</p>
          <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
            <button className="btn" onClick={() => setShowScan(true)}>Scan Directory</button>
            <button className="btn btn-primary" onClick={() => setShowAdd(true)}>Add Project</button>
          </div>
        </div>
      ) : (
        <div className="project-grid">
          {projects.map(p => (
            <ProjectCard
              key={p.id}
              project={p}
              workspaces={workspaces}
              onOpen={() => navigate(`/project/${p.id}`)}
            />
          ))}
        </div>
      )}

      {showAdd && (
        <AddProjectDialog onClose={() => setShowAdd(false)} onAdded={handleAdded} />
      )}
      {showScan && (
        <ScanDialog onClose={() => setShowScan(false)} onFound={handleFound} />
      )}
    </div>
  )
}
