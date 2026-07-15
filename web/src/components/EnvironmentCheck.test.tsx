import React from "react"
import { render, screen, waitFor } from "@testing-library/react"
import EnvironmentCheck from "./EnvironmentCheck"
import { checkEnvironment } from "../api/api"

jest.mock("../api/api", () => ({
  checkEnvironment: jest.fn(),
  openExternal: jest.fn(),
}))

const mockedCheck = checkEnvironment as jest.MockedFunction<
  typeof checkEnvironment
>

describe("EnvironmentCheck", () => {
  afterEach(() => jest.clearAllMocks())

  it("renders tool statuses and reports ready when all installed", async () => {
    mockedCheck.mockResolvedValue({
      rust: { installed: true, version: "rustc 1.0" },
      cargo: { installed: true, version: "cargo 1.0" },
      node: { installed: true, version: "v20" },
      tauri_cli: { installed: true, version: "tauri 2.0" },
      platform_deps: [],
    })
    const onReady = jest.fn()
    render(<EnvironmentCheck onReady={onReady} />)

    await waitFor(() => expect(onReady).toHaveBeenCalledWith(true))
    expect(screen.getByText("rustc 1.0")).toBeInTheDocument()
    expect(screen.getAllByText("Installed").length).toBeGreaterThan(0)
  })

  it("reports not ready when a tool is missing", async () => {
    mockedCheck.mockResolvedValue({
      rust: { installed: false },
      cargo: { installed: true, version: "cargo 1.0" },
      node: { installed: true, version: "v20" },
      tauri_cli: { installed: true, version: "tauri 2.0" },
      platform_deps: [],
    })
    const onReady = jest.fn()
    render(<EnvironmentCheck onReady={onReady} />)

    await waitFor(() => expect(onReady).toHaveBeenCalledWith(false))
    expect(screen.getByText("Missing")).toBeInTheDocument()
  })
})
