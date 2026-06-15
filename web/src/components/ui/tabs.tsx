import React from "react"
import styles from "./tabs.module.scss"

interface TabsProps {
  value: string
  onValueChange: (value: string) => void
  tabs: Array<{ value: string; label: string }>
}

export function Tabs({ value, onValueChange, tabs }: TabsProps) {
  return (
    <div className={styles.tabs} role="tablist">
      {tabs.map((tab) => (
        <button
          key={tab.value}
          role="tab"
          aria-selected={value === tab.value}
          className={[styles.tab, value === tab.value ? styles.active : ""]
            .filter(Boolean)
            .join(" ")}
          onClick={() => onValueChange(tab.value)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  )
}
