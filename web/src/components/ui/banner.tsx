import React from "react"
import styles from "./banner.module.scss"

type Tone = "info" | "success" | "warning" | "danger"

export function Banner({
  tone = "info",
  title,
  children,
  action,
}: {
  tone?: Tone
  title?: React.ReactNode
  children?: React.ReactNode
  action?: React.ReactNode
}) {
  return (
    <div className={[styles.banner, styles[tone]].join(" ")} role="status">
      <div className={styles.content}>
        {title && <div className={styles.title}>{title}</div>}
        {children && <div className={styles.body}>{children}</div>}
      </div>
      {action && <div className={styles.action}>{action}</div>}
    </div>
  )
}
