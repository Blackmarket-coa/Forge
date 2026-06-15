import React, { useState } from "react"
import styles from "./collapsible.module.scss"

export function Collapsible({
  title,
  defaultOpen = true,
  children,
}: {
  title: React.ReactNode
  defaultOpen?: boolean
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <section className={styles.root}>
      <button
        type="button"
        className={styles.header}
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
      >
        <span
          className={[styles.chevron, open ? styles.chevronOpen : ""]
            .filter(Boolean)
            .join(" ")}
          aria-hidden
        >
          ▸
        </span>
        <span className={styles.title}>{title}</span>
      </button>
      {open && <div className={styles.body}>{children}</div>}
    </section>
  )
}
