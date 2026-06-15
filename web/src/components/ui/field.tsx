import React from "react"
import styles from "./field.module.scss"

export function Field({
  label,
  help,
  htmlFor,
  children,
}: {
  label?: React.ReactNode
  help?: React.ReactNode
  htmlFor?: string
  children: React.ReactNode
}) {
  return (
    <div className={styles.field}>
      {label && (
        <label className={styles.label} htmlFor={htmlFor}>
          {label}
        </label>
      )}
      {children}
      {help && <div className={styles.help}>{help}</div>}
    </div>
  )
}
