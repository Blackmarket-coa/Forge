import React from "react"
import { Spinner } from "./spinner"
import styles from "./button.module.scss"

type Variant = "primary" | "secondary" | "ghost" | "danger"
type Size = "sm" | "md"

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
  size?: Size
  loading?: boolean
  fullWidth?: boolean
  leftIcon?: React.ReactNode
}

export function Button({
  variant = "secondary",
  size = "md",
  loading = false,
  fullWidth = false,
  leftIcon,
  className,
  children,
  disabled,
  ...props
}: ButtonProps) {
  const classes = [
    styles.button,
    styles[variant],
    styles[size],
    fullWidth ? styles.fullWidth : "",
    className || "",
  ]
    .filter(Boolean)
    .join(" ")

  return (
    <button className={classes} disabled={disabled || loading} {...props}>
      {loading && <Spinner size={size === "sm" ? 12 : 14} />}
      {!loading && leftIcon && <span className={styles.icon}>{leftIcon}</span>}
      {children}
    </button>
  )
}
