import React from "react"
import styles from "./spinner.module.scss"

export function Spinner({
  size = 16,
  className,
}: {
  size?: number
  className?: string
}) {
  return (
    <span
      className={[styles.spinner, className || ""].filter(Boolean).join(" ")}
      style={{ width: size, height: size }}
      role="status"
      aria-label="Loading"
    />
  )
}
