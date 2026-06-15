import React from "react"
import { render, screen, fireEvent } from "@testing-library/react"
import { Tabs } from "./tabs"

describe("Tabs", () => {
  const tabs = [
    { value: "form", label: "Form" },
    { value: "json", label: "JSON" },
  ]

  it("marks the active tab as selected", () => {
    render(<Tabs value="form" onValueChange={() => {}} tabs={tabs} />)
    expect(screen.getByText("Form")).toHaveAttribute("aria-selected", "true")
    expect(screen.getByText("JSON")).toHaveAttribute("aria-selected", "false")
  })

  it("calls onValueChange when a tab is clicked", () => {
    const onChange = jest.fn()
    render(<Tabs value="form" onValueChange={onChange} tabs={tabs} />)
    fireEvent.click(screen.getByText("JSON"))
    expect(onChange).toHaveBeenCalledWith("json")
  })
})
