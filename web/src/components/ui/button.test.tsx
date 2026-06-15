import React from "react"
import { render, screen, fireEvent } from "@testing-library/react"
import { Button } from "./button"

describe("Button", () => {
  it("renders its label and handles clicks", () => {
    const onClick = jest.fn()
    render(<Button onClick={onClick}>Save</Button>)
    fireEvent.click(screen.getByText("Save"))
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it("is disabled and shows a spinner while loading", () => {
    const onClick = jest.fn()
    render(
      <Button loading onClick={onClick}>
        Save
      </Button>
    )
    const button = screen.getByRole("button")
    expect(button).toBeDisabled()
    fireEvent.click(button)
    expect(onClick).not.toHaveBeenCalled()
    expect(screen.getByRole("status")).toBeInTheDocument()
  })

  it("respects the disabled prop", () => {
    render(<Button disabled>Save</Button>)
    expect(screen.getByRole("button")).toBeDisabled()
  })
})
