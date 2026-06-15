import React, { useEffect, useRef } from "react"
import { listen } from "@tauri-apps/api/event"
import { Terminal as XTerm } from "@xterm/xterm"
import { FitAddon } from "@xterm/addon-fit"
import { WebLinksAddon } from "@xterm/addon-web-links"
import "@xterm/xterm/css/xterm.css"
import styles from "./Terminal.module.scss"

interface TerminalProps {
  processIdPrefix?: string
}

export default function Terminal({ processIdPrefix }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const termRef = useRef<XTerm | null>(null)
  const fitRef = useRef<FitAddon | null>(null)

  useEffect(() => {
    const term = new XTerm({
      convertEol: true,
      fontFamily: "monospace",
      fontSize: 12,
      theme: { background: "#111111" },
    })
    const fitAddon = new FitAddon()
    term.loadAddon(fitAddon)
    term.loadAddon(new WebLinksAddon())

    termRef.current = term
    fitRef.current = fitAddon

    if (containerRef.current) {
      term.open(containerRef.current)
      fitAddon.fit()
    }

    const unsubs: Array<() => void> = []

    const matches = (id: string) =>
      !processIdPrefix || id.includes(processIdPrefix)

    const addListener = async (
      event: string,
      handler: (payload: any) => void
    ) => {
      const unlisten = await listen(event, (e: any) => {
        handler(e.payload || {})
      })
      unsubs.push(unlisten)
    }

    // The backend emits a single "process-output" event carrying a stderr flag,
    // plus a "process-exit" event when the process finishes.
    void addListener("process-output", (payload) => {
      if (!matches(String(payload.process_id || ""))) return
      const text = payload.data || ""
      term.writeln(payload.is_stderr ? `\x1b[33m${text}\x1b[0m` : text)
    })

    void addListener("process-exit", (payload) => {
      if (!matches(String(payload.id || ""))) return
      term.writeln(
        `\x1b[31mProcess exited (id=${payload.id}) with code ${payload.code}\x1b[0m`
      )
    })

    const onResize = () => fitAddon.fit()
    window.addEventListener("resize", onResize)

    return () => {
      window.removeEventListener("resize", onResize)
      unsubs.forEach((u) => u())
      term.dispose()
    }
  }, [processIdPrefix])

  return (
    <div className={styles.terminal}>
      <div className={styles.header}>
        <span className={styles.dots} aria-hidden>
          <i />
          <i />
          <i />
        </span>
        <span className={styles.title}>Terminal</span>
        <button
          className={styles.clear}
          onClick={() => termRef.current?.clear()}
        >
          Clear
        </button>
      </div>
      <div ref={containerRef} className={styles.surface} />
    </div>
  )
}
