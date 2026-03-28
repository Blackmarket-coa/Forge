import React from "react"

export function Collapsible({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <details open style={{ border: "1px solid #333", borderRadius: 8, padding: 12 }}>
      <summary style={{ cursor: "pointer", fontWeight: 600 }}>{title}</summary>
      <div style={{ marginTop: 12 }}>{children}</div>
    </details>
  )
}
