import React, { useEffect, useState } from "react"
import { checkEnvironment, EnvTool } from "../api/api"
import { frameworkBadge } from "../lib/frameworks"
import { Badge } from "./ui/badge"
import { Banner } from "./ui/banner"
import { Button } from "./ui/button"
import { Spinner } from "./ui/spinner"
import styles from "./EnvironmentCheck.module.scss"

export default function EnvironmentCheck({
  onReady,
}: {
  onReady?: (ready: boolean) => void
}) {
  const [tools, setTools] = useState<EnvTool[] | null>(null)
  const [loading, setLoading] = useState(true)

  const run = async () => {
    setLoading(true)
    try {
      const result = await checkEnvironment()
      setTools(result.tools || [])
      onReady?.(canBuildDesktop(result.tools || []))
    } catch {
      setTools(null)
      onReady?.(false)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void run()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // The default desktop app (Tauri) needs Rust + the Tauri CLI specifically.
  const canBuild = canBuildDesktop(tools || [])
  const missingCount = (tools || []).filter((t) => !t.installed).length

  return (
    <div className={styles.root}>
      <div className={styles.header}>
        <span className={styles.heading}>Tools on your computer</span>
        <Button size="sm" variant="ghost" onClick={() => void run()}>
          Check again
        </Button>
      </div>

      {loading && !tools ? (
        <div className={styles.loading}>
          <Spinner /> Checking your computer…
        </div>
      ) : (
        <>
          {tools &&
            (missingCount === 0 ? (
              <Banner tone="success" title="You're ready to build apps">
                All the tools Forge looks for are installed.
              </Banner>
            ) : canBuild ? (
              <Banner tone="success" title="You're ready to build desktop apps">
                Some app types need extra tools — each row says which kinds of
                app use it.
              </Banner>
            ) : (
              <Banner tone="info" title="You can make apps right now">
                Creating apps needs nothing extra. To build installers, add the
                tools marked below for the kinds of app you want.
              </Banner>
            ))}

          <ul className={styles.list}>
            {(tools || []).map((tool) => (
              <li key={tool.name} className={styles.row}>
                <div className={styles.toolInfo}>
                  <span className={styles.name}>{tool.label}</span>
                  <span className={styles.desc}>
                    {tool.installed ? (
                      <>
                        For:{" "}
                        {tool.needed_by
                          .map((fw) => frameworkBadge(fw).short)
                          .join(", ")}
                      </>
                    ) : (
                      <>
                        {tool.install_hint} (needed for{" "}
                        {tool.needed_by
                          .map((fw) => frameworkBadge(fw).short)
                          .join(", ")}
                        )
                      </>
                    )}
                  </span>
                </div>
                <span className={styles.version}>
                  {tool.installed ? tool.version : ""}
                </span>
                <Badge tone={tool.installed ? "success" : "warning"} dot>
                  {tool.installed ? "Installed" : "Missing"}
                </Badge>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  )
}

function canBuildDesktop(tools: EnvTool[]): boolean {
  const installed = (name: string) =>
    tools.some((t) => t.name === name && t.installed)
  return installed("rust") && installed("tauri_cli")
}
