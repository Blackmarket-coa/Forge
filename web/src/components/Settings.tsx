import React from "react"
import { useAppState } from "../providers/AppStateProvider"

export default function Settings() {
  const { theme, toggleTheme } = useAppState()

  return (
    <div style={{ padding: 24 }}>
      <h2>Settings</h2>
      <p>Current theme: {theme}</p>
      <button onClick={toggleTheme}>Toggle Theme</button>
      <p style={{ marginTop: 16 }}>State file: ~/.forge/forge.json</p>
    </div>
  )
}
