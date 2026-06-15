import React from "react"
import styles from "./checkbox.module.scss"

interface CheckboxProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type"> {
  checked?: boolean
  label?: React.ReactNode
}

export function Checkbox({
  checked,
  label,
  className,
  ...props
}: CheckboxProps) {
  const input = (
    <input
      type="checkbox"
      checked={checked}
      className={[styles.checkbox, className || ""].filter(Boolean).join(" ")}
      {...props}
    />
  )
  if (label === undefined) return input
  return (
    <label className={styles.wrapper}>
      {input}
      <span>{label}</span>
    </label>
  )
}
