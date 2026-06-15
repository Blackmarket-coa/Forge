import React, { useState } from "react"
import { check, Update } from "@tauri-apps/plugin-updater"
import { useSnackbar } from "notistack"
import { Badge } from "./ui/badge"
import { Button } from "./ui/button"
import styles from "./UpdateChecker.module.scss"

type Status = "idle" | "checking" | "available" | "installing" | "uptodate"

export default function UpdateChecker() {
  const { enqueueSnackbar } = useSnackbar()
  const [status, setStatus] = useState<Status>("idle")
  const [update, setUpdate] = useState<Update | null>(null)

  const onCheck = async () => {
    setStatus("checking")
    try {
      const result = await check()
      if (result) {
        setUpdate(result)
        setStatus("available")
      } else {
        setStatus("uptodate")
        enqueueSnackbar("Forge is up to date", { variant: "success" })
      }
    } catch (e: any) {
      setStatus("idle")
      enqueueSnackbar(`Update check failed: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  const onInstall = async () => {
    if (!update) return
    setStatus("installing")
    try {
      await update.downloadAndInstall()
      enqueueSnackbar("Update installed. Restart Forge to apply it.", {
        variant: "success",
      })
    } catch (e: any) {
      setStatus("available")
      enqueueSnackbar(`Install failed: ${e?.message || e}`, {
        variant: "error",
      })
    }
  }

  return (
    <div className={styles.root}>
      <div className={styles.row}>
        <span>Current version</span>
        <Badge tone="neutral">{process.env.REACT_APP_VERSION || "0.1.0"}</Badge>
      </div>

      {(status === "available" || status === "installing") && update ? (
        <div className={styles.available}>
          <div className={styles.row}>
            <span>New version</span>
            <Badge tone="accent">{update.version}</Badge>
          </div>
          {update.body && <p className={styles.notes}>{update.body}</p>}
          <Button
            variant="primary"
            fullWidth
            loading={status === "installing"}
            onClick={onInstall}
          >
            Download &amp; install
          </Button>
        </div>
      ) : (
        <Button
          variant="secondary"
          fullWidth
          loading={status === "checking"}
          onClick={onCheck}
        >
          Check for updates
        </Button>
      )}
    </div>
  )
}
