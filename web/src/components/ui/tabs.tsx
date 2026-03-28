import React from "react"

interface TabsProps {
  value: string
  onValueChange: (value: string) => void
  tabs: Array<{ value: string; label: string }>
}

export function Tabs({ value, onValueChange, tabs }: TabsProps) {
  return (
    <div style={{ display: "flex", gap: 8 }}>
      {tabs.map((tab) => (
        <button
          key={tab.value}
          onClick={() => onValueChange(tab.value)}
          style={{
            border: "1px solid #666",
            background: value === tab.value ? "#444" : "transparent",
            color: "white",
            borderRadius: 6,
            padding: "4px 10px",
          }}
        >
          {tab.label}
        </button>
      ))}
    </div>
  )
}
