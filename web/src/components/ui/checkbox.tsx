import React from "react"

interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type"> {
  checked?: boolean
}

export function Checkbox({ checked, ...props }: CheckboxProps) {
  return <input type="checkbox" checked={checked} {...props} />
}
