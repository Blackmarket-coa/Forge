import React, { useEffect, useState } from "react"
import { checkEnvironment } from "../api/api"
import { Badge } from "./ui/badge"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Spinner } from "./ui/spinner"
import styles from "./EnvironmentCheck.module.scss"

interface ToolStatus {
  installed: boolean
  version?: string
}

interface EnvResult {
  rust?: ToolStatus
  cargo?: ToolStatus
  node?: ToolStatus
  tauri_cli?: ToolStatus
  platform_deps?: Array<{ name: string; installed: boolean }>
}

interface ToolInfo {
  key: keyof EnvResult
  label: string
  /** Plain-language explanation of what this tool is for. */
  description: string
  /** What to do when it's missing. */
  fix: React.ReactNode
}

const TOOLS: ToolInfo[] = [
  {
    key: "rust",
    label: "Rust",
    description: "Builds your app into a real program.",
    fix: (
      <>
        Install it from{" "}
        <a
          href="https://www.rust-lang.org/tools/install"
          target="_blank"
          rel="noreferrer"
        >
          rust-lang.org
        </a>
        .
      </>
    ),
  },
  {
    key: "cargo",
    label: "Cargo",
    description: "Comes with Rust — manages the build.",
    fix: <>Installed automatically with Rust.</>,
  },
  {
    key: "node",
    label: "Node.js",
    description: "Only needed for blank projects, not for website apps.",
    fix: (
      <>
        Optional. Install from{" "}
        <a href="https://nodejs.org" target="_blank" rel="noreferrer">
          nodejs.org
        </a>{" "}
        if you build a blank project.
      </>
    ),
  },
  {
    key: "tauri_cli",
    label: "Tauri CLI",
    description: "Packages your app into an installer you can share.",
    fix: (
      <>
        Install by running{" "}
        <code>cargo install tauri-cli --version &quot;^2&quot;</code> in a
        terminal.
      </>
    ),
  },
]

export default function EnvironmentCheck({
  onReady,
}: {
  onReady?: (ready: boolean) => void
}) {
  const [env, setEnv] = useState<EnvResult | null>(null)
  const [loading, setLoading] = useState(true)

  const run = async () => {
    setLoading(true)
    try {
      const result = (await checkEnvironment()) as EnvResult
      setEnv(result)
      const ready = TOOLS.every((t) => (result[t.key] as ToolStatus)?.installed)
      onReady?.(ready)
    } catch {
      setEnv(null)
      onReady?.(false)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void run()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Building installers needs Rust + the Tauri CLI specifically.
  const canBuild = !!(env?.rust?.installed && env?.tauri_cli?.installed)

  return (
    <div className={styles.root}>
      <div className={styles.header}>
        <span className={styles.heading}>Tools on your computer</span>
        <Button size="sm" variant="ghost" onClick={() => void run()}>
          Check again
        </Button>
      </div>

      {loading && !env ? (
        <div className={styles.loading}>
          <Spinner /> Checking your computer…
        </div>
      ) : (
        <>
          {env &&
            (canBuild ? (
              <Banner tone="success" title="You're ready to build apps">
                All the tools needed to build and share installers are
                installed.
              </Banner>
            ) : (
              <Banner tone="info" title="You can make apps right now">
                Creating a website app needs nothing extra. To build an
                installer you can share, add the items marked below.
              </Banner>
            ))}

          <ul className={styles.list}>
            {TOOLS.map((tool) => {
              const status = env?.[tool.key] as ToolStatus | undefined
              const ok = !!status?.installed
              return (
                <li key={tool.key} className={styles.row}>
                  <div className={styles.toolInfo}>
                    <span className={styles.name}>{tool.label}</span>
                    <span className={styles.desc}>
                      {ok ? tool.description : tool.fix}
                    </span>
                  </div>
                  <span className={styles.version}>
                    {ok ? status?.version : ""}
                  </span>
                  <Badge tone={ok ? "success" : "warning"} dot>
                    {ok ? "Installed" : "Missing"}
                  </Badge>
                </li>
              )
            })}
            {(env?.platform_deps || []).map((dep) => (
              <li key={dep.name} className={styles.row}>
                <div className={styles.toolInfo}>
                  <span className={styles.name}>System library</span>
                  <span className={styles.desc}>{dep.name}</span>
                </div>
                <span className={styles.version} />
                <Badge tone={dep.installed ? "success" : "warning"} dot>
                  {dep.installed ? "Installed" : "Missing"}
                </Badge>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  )
}
