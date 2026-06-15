import React, { useEffect, useState } from "react"
import { checkEnvironment } from "../api/api"
import { Badge } from "./ui/badge"
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

const TOOLS: Array<{ key: keyof EnvResult; label: string }> = [
  { key: "rust", label: "Rust" },
  { key: "cargo", label: "Cargo" },
  { key: "node", label: "Node.js" },
  { key: "tauri_cli", label: "Tauri CLI" },
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

  return (
    <div className={styles.root}>
      <div className={styles.header}>
        <span className={styles.heading}>Environment</span>
        <Button size="sm" variant="ghost" onClick={() => void run()}>
          Re-check
        </Button>
      </div>

      {loading && !env ? (
        <div className={styles.loading}>
          <Spinner /> Checking your toolchain…
        </div>
      ) : (
        <ul className={styles.list}>
          {TOOLS.map((tool) => {
            const status = env?.[tool.key] as ToolStatus | undefined
            const ok = !!status?.installed
            return (
              <li key={tool.key} className={styles.row}>
                <span className={styles.name}>{tool.label}</span>
                <span className={styles.version}>
                  {ok ? status?.version : "not found"}
                </span>
                <Badge tone={ok ? "success" : "danger"} dot>
                  {ok ? "Installed" : "Missing"}
                </Badge>
              </li>
            )
          })}
          {(env?.platform_deps || []).map((dep) => (
            <li key={dep.name} className={styles.row}>
              <span className={styles.name}>{dep.name}</span>
              <span className={styles.version} />
              <Badge tone={dep.installed ? "success" : "warning"} dot>
                {dep.installed ? "Installed" : "Missing"}
              </Badge>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
