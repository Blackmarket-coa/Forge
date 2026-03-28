import React from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { registerProject } from "../api/api"

export default function AddExistingProject() {
  const handleAdd = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected === "string") {
      await registerProject(selected)
      window.alert(`Project added: ${selected}`)
    }
  }

  return (
    <div style={{ padding: 24 }}>
      <h2>Add Existing Project</h2>
      <button onClick={handleAdd}>Select Folder</button>
    </div>
  )
}
