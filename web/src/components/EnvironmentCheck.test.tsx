import React from "react"
import { render, screen, waitFor } from "@testing-library/react"
import EnvironmentCheck from "./EnvironmentCheck"
import { checkEnvironment, EnvTool } from "../api/api"

jest.mock("../api/api", () => ({
  checkEnvironment: jest.fn(),
}))

const mockedCheck = checkEnvironment as jest.MockedFunction<
  typeof checkEnvironment
>

function tool(overrides: Partial<EnvTool>): EnvTool {
  return {
    name: "rust",
    label: "Rust",
    installed: true,
    version: "rustc 1.90.0",
    install_hint: "Install Rust",
    needed_by: ["tauri"],
    ...overrides,
  }
}

describe("EnvironmentCheck", () => {
  beforeEach(() => jest.clearAllMocks())

  it("shows every tool with which app types need it", async () => {
    mockedCheck.mockResolvedValue({
      tools: [
        tool({ name: "rust", label: "Rust" }),
        tool({ name: "tauri_cli", label: "Tauri CLI", version: "tauri 2.0" }),
        tool({
          name: "node",
          label: "Node.js",
          installed: false,
          version: "not found",
          install_hint: "Install Node.js (LTS) from https://nodejs.org",
          needed_by: ["capacitor", "electron", "react-native"],
        }),
      ],
    })

    render(<EnvironmentCheck />)

    expect(await screen.findByText("Rust")).toBeInTheDocument()
    expect(screen.getByText("Tauri CLI")).toBeInTheDocument()
    expect(screen.getByText("Node.js")).toBeInTheDocument()

    // Missing tools show the install hint plus who needs them.
    expect(
      screen.getByText(/Install Node\.js \(LTS\) from https:\/\/nodejs\.org/)
    ).toBeInTheDocument()
    expect(
      screen.getByText(/Capacitor, Electron, React Native/)
    ).toBeInTheDocument()
    expect(screen.getAllByText("Installed")).toHaveLength(2)
    expect(screen.getByText("Missing")).toBeInTheDocument()
  })

  it("reports desktop readiness when Rust and the Tauri CLI are present", async () => {
    mockedCheck.mockResolvedValue({
      tools: [
        tool({ name: "rust" }),
        tool({ name: "tauri_cli", label: "Tauri CLI" }),
        tool({ name: "node", label: "Node.js", installed: false }),
      ],
    })

    const onReady = jest.fn()
    render(<EnvironmentCheck onReady={onReady} />)
    await waitFor(() => expect(onReady).toHaveBeenCalledWith(true))
    expect(screen.getByText(/ready to build desktop apps/i)).toBeInTheDocument()
  })
})
