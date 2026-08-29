import React from "react"
import { render, screen, waitFor } from "@testing-library/react"
import ExtensionsView from "./ExtensionsView"
import { getFbmStatus } from "../api/api"

jest.mock("notistack", () => ({
  useSnackbar: () => ({ enqueueSnackbar: jest.fn() }),
}))

const mockUseAppState = jest.fn()
jest.mock("../providers/AppStateProvider", () => ({
  useAppState: () => mockUseAppState(),
}))

jest.mock("../api/api", () => ({
  browsePlugins: jest.fn(),
  getDefaultAppDir: jest.fn(),
  getFbmStatus: jest.fn(),
  packageExtension: jest.fn(),
  publishExtension: jest.fn(),
  scaffoldExtension: jest.fn(),
  setFbmConfig: jest.fn(),
  validateExtension: jest.fn(),
}))

const mockedStatus = getFbmStatus as jest.MockedFunction<typeof getFbmStatus>

describe("ExtensionsView", () => {
  beforeEach(() => {
    mockUseAppState.mockReturnValue({ tier: "free" })
    mockedStatus.mockResolvedValue({ configured: false })
  })

  it("renders the flow cards and reports an unconfigured connection", async () => {
    render(<ExtensionsView />)
    expect(screen.getByText("Extensions")).toBeInTheDocument()
    expect(screen.getByText("FreeBlackMarket connection")).toBeInTheDocument()
    expect(screen.getByText("New extension")).toBeInTheDocument()
    expect(screen.getByText("Package & publish")).toBeInTheDocument()
    await waitFor(() =>
      expect(screen.getByText("NOT CONFIGURED")).toBeInTheDocument()
    )
  })

  it("gates publish and browse behind Pro (plugin_browser finally honored)", async () => {
    render(<ExtensionsView />)
    await waitFor(() => expect(mockedStatus).toHaveBeenCalled())
    expect(
      screen.getByText("Publishing to the registry is a Forge Pro feature.")
    ).toBeInTheDocument()
    expect(
      screen.getByText("Registry browsing is a Forge Pro feature.")
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: "Publish to registry" })
    ).toBeDisabled()
    expect(
      screen.getByRole("button", { name: "Browse plugins" })
    ).toBeDisabled()
  })

  it("enables the gated actions for Pro once FBM is configured", async () => {
    mockUseAppState.mockReturnValue({ tier: "pro" })
    mockedStatus.mockResolvedValue({
      configured: true,
      api_base_url: "https://api.fbm.test",
      seller_token_masked: "seller...3456",
    })
    render(<ExtensionsView />)
    await waitFor(() =>
      expect(screen.getByText("CONNECTED")).toBeInTheDocument()
    )
    expect(screen.getByRole("button", { name: "Browse plugins" })).toBeEnabled()
  })
})
