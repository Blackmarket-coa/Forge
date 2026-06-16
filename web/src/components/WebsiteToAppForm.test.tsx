import React from "react"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import WebsiteToAppForm, {
  previewFolderName,
  suggestNameFromUrl,
} from "./WebsiteToAppForm"
import { createWebApp, getDefaultAppDir } from "../api/api"

jest.mock("notistack", () => ({
  useSnackbar: () => ({ enqueueSnackbar: jest.fn() }),
}))

jest.mock("@tauri-apps/plugin-dialog", () => ({
  open: jest.fn(),
}))

jest.mock("../api/api", () => ({
  createWebApp: jest.fn(),
  getDefaultAppDir: jest.fn(),
}))

const mockedCreate = createWebApp as jest.MockedFunction<typeof createWebApp>
const mockedDir = getDefaultAppDir as jest.MockedFunction<
  typeof getDefaultAppDir
>

describe("WebsiteToAppForm helpers", () => {
  it("suggests a friendly name from a website address", () => {
    expect(suggestNameFromUrl("https://shop.acme.com")).toBe("Acme")
    expect(suggestNameFromUrl("example.com")).toBe("Example")
    expect(suggestNameFromUrl("")).toBe("")
  })

  it("derives a safe folder name", () => {
    expect(previewFolderName("My Cool Store!!")).toBe("my-cool-store")
    expect(previewFolderName("   ")).toBe("my-app")
  })
})

describe("WebsiteToAppForm", () => {
  beforeEach(() => {
    jest.clearAllMocks()
    mockedDir.mockResolvedValue("/home/user/Forge Apps")
  })

  it("auto-fills the name from the URL and creates the app", async () => {
    const project = {
      id: "1",
      name: "Acme",
      path: "/home/user/Forge Apps/acme",
      platforms: [],
      git_dirty: false,
      status: "ready",
      tags: [],
    }
    mockedCreate.mockResolvedValue(project)
    const onCreated = jest.fn()

    render(<WebsiteToAppForm onCreated={onCreated} onCancel={jest.fn()} />)
    await waitFor(() => expect(mockedDir).toHaveBeenCalled())

    fireEvent.change(screen.getByPlaceholderText("yoursite.com"), {
      target: { value: "acme.com" },
    })

    // The name field is suggested automatically until the user edits it.
    const nameInput = screen.getByPlaceholderText("My Site") as HTMLInputElement
    expect(nameInput.value).toBe("Acme")

    const createBtn = screen.getByRole("button", { name: /create my app/i })
    await waitFor(() => expect(createBtn).not.toBeDisabled())
    fireEvent.click(createBtn)

    // The success state shows an unambiguous "Open my app" action.
    const openBtn = await screen.findByRole("button", { name: /open my app/i })
    expect(mockedCreate).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "Acme",
        url: "acme.com",
        parentDir: "/home/user/Forge Apps",
      })
    )

    fireEvent.click(openBtn)
    expect(onCreated).toHaveBeenCalledWith(project)
  })

  it("surfaces a friendly error when creation fails", async () => {
    mockedCreate.mockRejectedValue(
      new Error('A folder named "acme" already exists here.')
    )

    render(<WebsiteToAppForm onCreated={jest.fn()} onCancel={jest.fn()} />)
    await waitFor(() => expect(mockedDir).toHaveBeenCalled())

    fireEvent.change(screen.getByPlaceholderText("yoursite.com"), {
      target: { value: "acme.com" },
    })
    const createBtn = screen.getByRole("button", { name: /create my app/i })
    await waitFor(() => expect(createBtn).not.toBeDisabled())
    fireEvent.click(createBtn)

    expect(await screen.findByText(/already exists/i)).toBeInTheDocument()
  })
})
