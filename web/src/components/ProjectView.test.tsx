import React from "react"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import ProjectView from "./ProjectView"
import {
  collectArtifacts,
  getBuildHistory,
  getFrameworks,
  ProjectMeta,
  runBuild,
  runDev,
} from "../api/api"

jest.mock("notistack", () => ({
  useSnackbar: () => ({ enqueueSnackbar: jest.fn() }),
}))

// The terminal drags in xterm + Tauri event APIs; the view only needs a stub.
jest.mock("./Terminal", () => () => <div data-testid="terminal" />)

jest.mock("../api/api", () => ({
  addTarget: jest.fn(),
  collectArtifacts: jest.fn(),
  getBuildHistory: jest.fn(),
  getFrameworks: jest.fn(),
  killProcess: jest.fn(),
  openExternal: jest.fn(),
  runBuild: jest.fn(),
  runDev: jest.fn(),
}))

const mockedFrameworks = getFrameworks as jest.MockedFunction<
  typeof getFrameworks
>
const mockedRunBuild = runBuild as jest.MockedFunction<typeof runBuild>
const mockedRunDev = runDev as jest.MockedFunction<typeof runDev>

const FRAMEWORKS = [
  {
    id: "tauri" as const,
    label: "Desktop app (Tauri)",
    tagline: "Small and fast.",
    platforms: ["macOS", "Linux", "Windows"],
    bundle_kinds: [{ id: "deb", label: "Linux app (.deb)", platform: "Linux" }],
    tools: [],
    config_file: "tauri.conf.json",
    dev_label: "Preview app",
    dev_available: true,
  },
  {
    id: "capacitor" as const,
    label: "iPhone & Android app (Capacitor)",
    tagline: "Real mobile apps.",
    platforms: ["Android", "iOS"],
    bundle_kinds: [
      {
        id: "apk",
        label: "Android app (.apk)",
        platform: "Android",
        note: "Needs the Android SDK",
      },
    ],
    tools: [],
    config_file: "capacitor.config.json",
    dev_label: "Preview on Android device",
    dev_available: true,
  },
]

const PROJECT: ProjectMeta = {
  id: "p1",
  name: "Acme",
  path: "/apps/acme",
  platforms: ["macOS", "Linux", "Windows", "Android", "iOS"],
  git_dirty: false,
  status: "ready",
  tags: [],
  source_url: "https://acme.com/",
  targets: [
    { framework: "tauri", dir: ".", version: "2.1.0", status: "ready" },
    { framework: "capacitor", dir: "capacitor", status: "ready" },
  ],
}

describe("ProjectView", () => {
  beforeEach(() => {
    jest.clearAllMocks()
    mockedFrameworks.mockResolvedValue(FRAMEWORKS as any)
    mockedRunBuild.mockResolvedValue({ status: "success", artifacts: [] })
    mockedRunDev.mockResolvedValue("dev:/apps/acme:tauri")
    ;(collectArtifacts as jest.Mock).mockResolvedValue([])
    ;(getBuildHistory as jest.Mock).mockResolvedValue([])
  })

  it("switches app types and builds with the selected framework", async () => {
    render(
      <ProjectView
        project={PROJECT}
        onBack={jest.fn()}
        onOpenConfig={jest.fn()}
      />
    )

    // The desktop target is active first, with its bundle kinds.
    expect(await screen.findByLabelText("Linux app (.deb)")).toBeInTheDocument()

    // Switch to the mobile target.
    fireEvent.click(screen.getByRole("tab", { name: /Capacitor/ }))
    const apk = await screen.findByLabelText("Android app (.apk)")
    expect(screen.getByText("Needs the Android SDK")).toBeInTheDocument()

    fireEvent.click(apk)
    fireEvent.click(screen.getByRole("button", { name: /build installer/i }))

    await waitFor(() =>
      expect(mockedRunBuild).toHaveBeenCalledWith(
        "/apps/acme",
        ["apk"],
        "capacitor"
      )
    )
  })

  it("starts the preview for the active framework", async () => {
    render(
      <ProjectView
        project={PROJECT}
        onBack={jest.fn()}
        onOpenConfig={jest.fn()}
      />
    )

    const previewBtn = await screen.findByRole("button", {
      name: "Preview app",
    })
    fireEvent.click(previewBtn)
    await waitFor(() =>
      expect(mockedRunDev).toHaveBeenCalledWith("/apps/acme", "tauri")
    )
  })
})
