import React, { useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { registerProject } from "../api/api"

export default function CreateProjectForm() {
  const [projectName, setProjectName] = useState("")
  const [directory, setDirectory] = useState("")

  const pickDirectory = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      setDirectory(selected)
    }
  }

  const handleCreate = async () => {
    if (!directory) {
      window.alert("Pick a directory first")
      return
    }

    await registerProject(directory)
    window.alert(`Registered project '${projectName || "Unnamed"}' at ${directory}`)
  }

  return (
    <div style={{ padding: 24 }}>
      <h2>Create Project</h2>
      <div style={{ display: "grid", gap: 8, maxWidth: 480 }}>
        <label>
          Project Name
          <input value={projectName} onChange={(e) => setProjectName(e.target.value)} />
        </label>
        <label>
          Directory
          <input value={directory} readOnly />
        </label>
        <button onClick={pickDirectory}>Pick Directory</button>
        <button onClick={handleCreate}>Create</button>
      </div>
    </div>
  )
}
