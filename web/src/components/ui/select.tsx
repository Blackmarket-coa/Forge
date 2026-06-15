import React from "react"
import styles from "./control.module.scss"

export function Select({
  className,
  ...props
}: React.SelectHTMLAttributes<HTMLSelectElement>) {
  const classes = [styles.control, styles.select, className || ""]
    .filter(Boolean)
    .join(" ")
  return <select className={classes} {...props} />
}
