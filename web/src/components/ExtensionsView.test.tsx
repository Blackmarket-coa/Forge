import React from "react"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import ExtensionsView from "./ExtensionsView"
import {
  getExtensionTemplates,
  getFbmStatus,
  packageExtension,
} from "../api/api"

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
  getExtensionTemplates: jest.fn(),
  getFbmStatus: jest.fn(),
  packageExtension: jest.fn(),
  publishExtension: jest.fn(),
  scaffoldExtension: jest.fn(),
  setFbmConfig: jest.fn(),
  validateExtension: jest.fn(),
}))

const mockedStatus = getFbmStatus as jest.MockedFunction<typeof getFbmStatus>
const mockedTemplates = getExtensionTemplates as jest.MockedFunction<
  typeof getExtensionTemplates
>
const mockedPackage = packageExtension as jest.MockedFunction<
  typeof packageExtension
>

describe("ExtensionsView", () => {
  beforeEach(() => {
    mockUseAppState.mockReturnValue({ tier: "free" })
    mockedStatus.mockResolvedValue({ configured: false })
    mockedTemplates.mockResolvedValue([
      {
        id: "featured-vendor-widget",
        label: "Featured Vendor Widget (home card)",
        description: "The end-to-end demo path.",
        artifactKind: "manifest_plugin",
        needsBlob: false,
      },
      {
        id: "theme",
        label: "Theme",
        description: "Packaged as a zip you host at publish.",
        artifactKind: "theme",
        needsBlob: true,
      },
    ])
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
      publishable_key: "pk_test_storefront",
    })
    render(<ExtensionsView />)
    await waitFor(() =>
      expect(screen.getByText("CONNECTED")).toBeInTheDocument()
    )
    expect(screen.getByRole("button", { name: "Browse plugins" })).toBeEnabled()
  })

  it("lists templates from the backend registry", async () => {
    render(<ExtensionsView />)
    await waitFor(() => expect(mockedTemplates).toHaveBeenCalled())
    expect(
      await screen.findByRole("option", { name: "Theme" })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("option", { name: "Featured Vendor Widget (home card)" })
    ).toBeInTheDocument()
  })

  it("asks for the bundle address after packaging an asset-carrying kind", async () => {
    mockUseAppState.mockReturnValue({ tier: "pro" })
    mockedStatus.mockResolvedValue({
      configured: true,
      api_base_url: "https://api.fbm.test",
      publishable_key: "pk_test",
    })
    mockedPackage.mockResolvedValue({
      distDir: "/tmp/night-theme/dist/night-theme-0.1.0",
      digests: { manifestSha256: "a".repeat(64), assetHashes: {} },
      bundlePath: "/tmp/night-theme/dist/night-theme-0.1.0.zip",
      needsBlob: true,
      issues: [],
    })

    render(<ExtensionsView />)
    await waitFor(() => expect(mockedStatus).toHaveBeenCalled())

    fireEvent.change(screen.getByPlaceholderText("/path/to/my-vendor-widget"), {
      target: { value: "/tmp/night-theme" },
    })
    fireEvent.click(screen.getByRole("button", { name: "Package" }))

    // The bundle-address step appears and gates publish until filled.
    const urlInput = await screen.findByPlaceholderText(
      "https://yoursite.com/downloads/my-theme-0.1.0.zip"
    )
    expect(
      screen.getByRole("button", { name: "Publish to registry" })
    ).toBeDisabled()
    fireEvent.change(urlInput, {
      target: { value: "https://example.com/night-theme.zip" },
    })
    expect(
      screen.getByRole("button", { name: "Publish to registry" })
    ).toBeEnabled()
  })
})
