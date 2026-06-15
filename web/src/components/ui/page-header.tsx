import React from "react"
import styles from "./page-header.module.scss"

export function PageHeader({
  title,
  subtitle,
  actions,
  meta,
}: {
  title: React.ReactNode
  subtitle?: React.ReactNode
  actions?: React.ReactNode
  meta?: React.ReactNode
}) {
  return (
    <header className={styles.header}>
      <div className={styles.headline}>
        <div>
          <h1 className={styles.title}>{title}</h1>
          {subtitle && <p className={styles.subtitle}>{subtitle}</p>}
        </div>
        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
      {meta && <div className={styles.meta}>{meta}</div>}
    </header>
  )
}
