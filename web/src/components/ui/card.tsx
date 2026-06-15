import React from "react"
import styles from "./card.module.scss"

export interface CardProps
  extends Omit<React.HTMLAttributes<HTMLDivElement>, "title"> {
  title?: React.ReactNode
  subtitle?: React.ReactNode
  actions?: React.ReactNode
  interactive?: boolean
  padded?: boolean
}

export function Card({
  title,
  subtitle,
  actions,
  interactive = false,
  padded = true,
  className,
  children,
  ...props
}: CardProps) {
  const classes = [
    styles.card,
    interactive ? styles.interactive : "",
    className || "",
  ]
    .filter(Boolean)
    .join(" ")
  return (
    <div className={classes} {...props}>
      {(title || actions) && (
        <div className={styles.header}>
          <div>
            {title && <div className={styles.title}>{title}</div>}
            {subtitle && <div className={styles.subtitle}>{subtitle}</div>}
          </div>
          {actions && <div className={styles.actions}>{actions}</div>}
        </div>
      )}
      <div className={padded ? styles.body : ""}>{children}</div>
    </div>
  )
}
