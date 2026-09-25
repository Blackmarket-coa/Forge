import React from "react"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import LicenseGate from "./LicenseGate"

const mockEnqueue = jest.fn()
jest.mock("notistack", () => ({
  useSnackbar: () => ({ enqueueSnackbar: mockEnqueue }),
}))

jest.mock("./ui/external-link", () => ({
  ExternalLink: ({ children }: { children: React.ReactNode }) => (
    <span>{children}</span>
  ),
}))

const mockUseAppState = jest.fn()
jest.mock("../providers/AppStateProvider", () => ({
  useAppState: () => mockUseAppState(),
}))

describe("LicenseGate", () => {
  beforeEach(() => {
    mockEnqueue.mockReset()
  })

  it("shows the backend's 'not configured' message instead of a generic error", async () => {
    const notConfigured =
      "Licensing is not configured in this build of Forge, so license keys cannot be validated."
    mockUseAppState.mockReturnValue({
      tier: "free",
      activateLicense: jest.fn().mockRejectedValue(notConfigured),
    })

    render(
      <LicenseGate feature="extension_publish" description="Publish">
        <div>child</div>
      </LicenseGate>
    )
    fireEvent.click(screen.getByText("Enter License Key"))
    fireEvent.change(screen.getByPlaceholderText("FORGE-XXXX-XXXX-XXXX"), {
      target: { value: "FORGE-ABCD-1234-WXYZ" },
    })
    fireEvent.click(screen.getByText("Activate"))

    await waitFor(() =>
      expect(mockEnqueue).toHaveBeenCalledWith(notConfigured, {
        variant: "error",
      })
    )
  })

  it("reports a rejected key as invalid", async () => {
    mockUseAppState.mockReturnValue({
      tier: "free",
      activateLicense: jest
        .fn()
        .mockResolvedValue({ tier: "free", valid: false }),
    })

    render(
      <LicenseGate feature="extension_publish" description="Publish">
        <div>child</div>
      </LicenseGate>
    )
    fireEvent.click(screen.getByText("Enter License Key"))
    fireEvent.change(screen.getByPlaceholderText("FORGE-XXXX-XXXX-XXXX"), {
      target: { value: "FORGE-ABCD-1234-WXYZ" },
    })
    fireEvent.click(screen.getByText("Activate"))

    await waitFor(() =>
      expect(mockEnqueue).toHaveBeenCalledWith("License key is invalid", {
        variant: "error",
      })
    )
  })
})
