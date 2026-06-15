import React from "react"
import styles from "./badge.module.scss"

export type Tone =
  | "neutral"
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "info"

export function Badge({
  tone = "neutral",
  dot = false,
  children,
}: {
  tone?: Tone
  dot?: boolean
  children: React.ReactNode
}) {
  return (
    <span className={[styles.badge, styles[tone]].join(" ")}>
      {dot && <span className={styles.dot} />}
      {children}
    </span>
  )
}
