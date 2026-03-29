import { useState, useEffect, useCallback } from 'react'
import {
  getWorkspaces,
  createWorkspace,
  updateWorkspace,
  deleteWorkspace,
  getProjects,
  addProjectToWorkspace,
  removeProjectFromWorkspace,
} from '../lib/ipc'
import type { Workspace, ProjectMeta } from '../lib/types'

const PRESET_COLORS = [
  '#6366f1', '#8b5cf6', '#ec4899', '#ef4444',
  '#f59e0b', '#22c55e', '#06b6d4', '#3b82f6',
]

interface CreateWorkspaceDialogProps {
  onClose: () => void
  onCreated: (w: Workspace) => void
}

function CreateWorkspaceDialog({ onClose, onCreated }: CreateWorkspaceDialogProps) {
  const [name, setName] = useState('')
  const [color, setColor] = useState(PRESET_COLORS[0])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!name.trim()) return
    setLoading(true)
    setError('')
    try {
      const ws = await createWorkspace(name.trim())
      if (color !== PRESET_COLORS[0]) {
        const updated = await updateWorkspace(ws.id, undefined, color)
        onCreated(updated)
      } else {
        onCreated(ws)
      }
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
        <h2>New Workspace</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="label">Name</label>
            <input
              className="input"
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="My Workspace"
              autoFocus
            />
          </div>
          <div className="form-group">
            <label className="label">Color</label>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              {PRESET_COLORS.map(c => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setColor(c)}
                  style={{
                    width: 28, height: 28,
                    borderRadius: '50%',
                    background: c,
                    border: color === c ? '2px solid white' : '2px solid transparent',
                    cursor: 'pointer',
                    outline: color === c ? `2px solid ${c}` : 'none',
                    outlineOffset: 2,
                  }}
                />
              ))}
            </div>
          </div>
          {error && <div className="error-msg">{error}</div>}
          <div className="dialog-footer">
            <button type="button" className="btn btn-ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn-primary" disabled={loading || !name.trim()}>
              {loading ? <span className="spinner" /> : null}
              Create
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

interface EditWorkspaceDialogProps {
  workspace: Workspace
  onClose: () => void
  onUpdated: (w: Workspace) => void
}

function EditWorkspaceDialog({ workspace, onClose, onUpdated }: EditWorkspaceDialogProps) {
  const [name, setName] = useState(workspace.name)
  const [color, setColor] = useState(workspace.color ?? PRESET_COLORS[0])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      const updated = await updateWorkspace(workspace.id, name.trim(), color)
      onUpdated(updated)
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
        <h2>Edit Workspace</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="label">Name</label>
            <input
              className="input"
              value={name}
              onChange={e => setName(e.target.value)}
              autoFocus
            />
          </div>
          <div className="form-group">
            <label className="label">Color</label>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              {PRESET_COLORS.map(c => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setColor(c)}
                  style={{
                    width: 28, height: 28,
                    borderRadius: '50%',
                    background: c,
                    border: color === c ? '2px solid white' : '2px solid transparent',
                    cursor: 'pointer',
                    outline: color === c ? `2px solid ${c}` : 'none',
                    outlineOffset: 2,
                  }}
                />
              ))}
            </div>
          </div>
          {error && <div className="error-msg">{error}</div>}
          <div className="dialog-footer">
            <button type="button" className="btn btn-ghost" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn btn-primary" disabled={loading}>
              {loading ? <span className="spinner" /> : null}
              Save
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

interface AssignProjectsDialogProps {
  workspace: Workspace
  allProjects: ProjectMeta[]
  onClose: () => void
  onAssigned: (workspaceId: string, projectId: string, add: boolean) => void
}

function AssignProjectsDialog({ workspace, allProjects, onClose, onAssigned }: AssignProjectsDialogProps) {
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState('')

  async function toggle(project: ProjectMeta) {
    setBusy(project.id)
    setError('')
    const assigned = workspace.project_ids.includes(project.id)
    try {
      if (assigned) {
        await removeProjectFromWorkspace(workspace.id, project.id)
        onAssigned(workspace.id, project.id, false)
      } else {
        await addProjectToWorkspace(workspace.id, project.id)
        onAssigned(workspace.id, project.id, true)
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setBusy(null)
    }
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={e => e.stopPropagation()}>
        <h2>Assign Projects — {workspace.name}</h2>
        {error && <div className="error-msg" style={{ marginBottom: 10 }}>{error}</div>}
        {allProjects.length === 0 ? (
          <p className="text-muted">No projects available.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, maxHeight: 300, overflowY: 'auto' }}>
            {allProjects.map(p => {
              const assigned = workspace.project_ids.includes(p.id)
              return (
                <div
                  key={p.id}
                  style={{
                    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                    padding: '8px 10px', background: 'var(--bg-3)',
                    borderRadius: 'var(--radius)', border: '1px solid var(--border)',
                  }}
                >
                  <div>
                    <div style={{ fontWeight: 500, fontSize: 13 }}>{p.name}</div>
                    <div className="text-muted">{p.path}</div>
                  </div>
                  <button
                    className={assigned ? 'btn btn-danger' : 'btn btn-primary'}
                    style={{ fontSize: 12, padding: '4px 10px' }}
                    disabled={busy === p.id}
                    onClick={() => toggle(p)}
                  >
                    {busy === p.id ? <span className="spinner" /> : assigned ? 'Remove' : 'Add'}
                  </button>
                </div>
              )
            })}
          </div>
        )}
        <div className="dialog-footer">
          <button className="btn" onClick={onClose}>Done</button>
        </div>
      </div>
    </div>
  )
}

export default function Workspaces() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([])
  const [allProjects, setAllProjects] = useState<ProjectMeta[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [editing, setEditing] = useState<Workspace | null>(null)
  const [assigning, setAssigning] = useState<Workspace | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const [ws, projs] = await Promise.all([getWorkspaces(), getProjects()])
      setWorkspaces(ws)
      setAllProjects(projs)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  async function handleDelete(ws: Workspace) {
    if (!confirm(`Delete workspace "${ws.name}"?`)) return
    try {
      await deleteWorkspace(ws.id)
      setWorkspaces(prev => prev.filter(w => w.id !== ws.id))
    } catch (err) {
      setError(String(err))
    }
  }

  function handleAssigned(workspaceId: string, projectId: string, add: boolean) {
    setWorkspaces(prev => prev.map(w => {
      if (w.id !== workspaceId) return w
      const ids = add
        ? [...w.project_ids, projectId]
        : w.project_ids.filter(id => id !== projectId)
      return { ...w, project_ids: ids }
    }))
    // Also update the assigning dialog's copy
    setAssigning(prev => {
      if (!prev || prev.id !== workspaceId) return prev
      const ids = add
        ? [...prev.project_ids, projectId]
        : prev.project_ids.filter(id => id !== projectId)
      return { ...prev, project_ids: ids }
    })
  }

  return (
    <div className="page">
      <div className="page-header">
        <h1>Workspaces</h1>
        <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ New Workspace</button>
      </div>

      {error && <div className="error-msg" style={{ marginBottom: 16 }}>{error}</div>}

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: 40 }}>
          <span className="spinner" />
        </div>
      ) : workspaces.length === 0 ? (
        <div className="empty-state">
          <div style={{ fontSize: 32 }}>&#x1F4C1;</div>
          <h3>No workspaces</h3>
          <p>Create a workspace to organize your Tauri projects.</p>
          <button className="btn btn-primary" onClick={() => setShowCreate(true)}>New Workspace</button>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          {workspaces.map(ws => {
            const assignedProjects = allProjects.filter(p => ws.project_ids.includes(p.id))
            return (
              <div key={ws.id} className="card">
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <div style={{
                      width: 14, height: 14, borderRadius: '50%',
                      background: ws.color ?? 'var(--accent)',
                      flexShrink: 0,
                    }} />
                    <span style={{ fontWeight: 600, fontSize: 15 }}>{ws.name}</span>
                    <span className="badge">{ws.project_ids.length} project{ws.project_ids.length !== 1 ? 's' : ''}</span>
                  </div>
                  <div style={{ display: 'flex', gap: 6 }}>
                    <button className="btn btn-ghost" style={{ fontSize: 12 }} onClick={() => setAssigning(ws)}>
                      Assign Projects
                    </button>
                    <button className="btn" style={{ fontSize: 12 }} onClick={() => setEditing(ws)}>
                      Edit
                    </button>
                    <button className="btn btn-danger" style={{ fontSize: 12 }} onClick={() => handleDelete(ws)}>
                      Delete
                    </button>
                  </div>
                </div>
                {assignedProjects.length === 0 ? (
                  <p className="text-muted" style={{ fontSize: 12 }}>No projects assigned yet.</p>
                ) : (
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
                    {assignedProjects.map(p => (
                      <div
                        key={p.id}
                        style={{
                          padding: '4px 10px',
                          background: 'var(--bg-3)',
                          border: '1px solid var(--border)',
                          borderRadius: 'var(--radius)',
                          fontSize: 12,
                        }}
                      >
                        {p.name}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}

      {showCreate && (
        <CreateWorkspaceDialog
          onClose={() => setShowCreate(false)}
          onCreated={ws => setWorkspaces(prev => [...prev, ws])}
        />
      )}
      {editing && (
        <EditWorkspaceDialog
          workspace={editing}
          onClose={() => setEditing(null)}
          onUpdated={ws => setWorkspaces(prev => prev.map(w => w.id === ws.id ? ws : w))}
        />
      )}
      {assigning && (
        <AssignProjectsDialog
          workspace={assigning}
          allProjects={allProjects}
          onClose={() => setAssigning(null)}
          onAssigned={handleAssigned}
        />
      )}
    </div>
  )
}
