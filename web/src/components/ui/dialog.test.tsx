import React from "react"
import { render, screen, fireEvent } from "@testing-library/react"
import { ConfirmDialog } from "./dialog"

describe("ConfirmDialog", () => {
  const baseProps = {
    title: "Remove key?",
    message: "This reverts to Free tier.",
    onConfirm: jest.fn(),
    onCancel: jest.fn(),
  }

  it("does not render when closed", () => {
    render(<ConfirmDialog open={false} {...baseProps} />)
    expect(screen.queryByText("Remove key?")).not.toBeInTheDocument()
  })

  it("renders title and message when open", () => {
    render(<ConfirmDialog open {...baseProps} />)
    expect(screen.getByText("Remove key?")).toBeInTheDocument()
    expect(screen.getByText("This reverts to Free tier.")).toBeInTheDocument()
  })

  it("fires onConfirm and onCancel", () => {
    const onConfirm = jest.fn()
    const onCancel = jest.fn()
    render(
      <ConfirmDialog
        open
        {...baseProps}
        confirmLabel="Remove"
        cancelLabel="Keep"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    )
    fireEvent.click(screen.getByText("Remove"))
    expect(onConfirm).toHaveBeenCalledTimes(1)
    fireEvent.click(screen.getByText("Keep"))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })
})
