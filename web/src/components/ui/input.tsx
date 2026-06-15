import React from "react"
import styles from "./control.module.scss"

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  invalid?: boolean
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  function Input({ className, invalid, ...props }, ref) {
    const classes = [
      styles.control,
      invalid ? styles.invalid : "",
      className || "",
    ]
      .filter(Boolean)
      .join(" ")
    return <input ref={ref} className={classes} {...props} />
  }
)
