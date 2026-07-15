import React from "react"
import { fireEvent, render, screen } from "@testing-library/react"
import { ExternalLink } from "./external-link"
import { openExternal } from "../../api/api"

jest.mock("../../api/api", () => ({
  openExternal: jest.fn(),
}))

const mockedOpen = openExternal as jest.MockedFunction<typeof openExternal>

describe("ExternalLink", () => {
  afterEach(() => jest.clearAllMocks())

  it("renders an anchor with the href and opens it via openExternal", () => {
    render(
      <ExternalLink href="https://example.com/docs">Read docs</ExternalLink>
    )

    const anchor = screen.getByRole("link", { name: "Read docs" })
    expect(anchor).toHaveAttribute("href", "https://example.com/docs")
    expect(anchor).toHaveAttribute("target", "_blank")

    fireEvent.click(anchor)
    expect(mockedOpen).toHaveBeenCalledWith("https://example.com/docs")
  })
})
