import React from "react"

interface State {
  error: Error | null
}

/**
 * Top-level error boundary so an unexpected render error shows a recoverable
 * screen instead of a blank window.
 */
export default class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    // eslint-disable-next-line no-console
    console.error("Forge UI error:", error, info)
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            display: "grid",
            placeItems: "center",
            minHeight: "100vh",
            padding: 24,
            textAlign: "center",
          }}
        >
          <div style={{ maxWidth: 480 }}>
            <h1 style={{ marginBottom: 12 }}>Something went wrong</h1>
            <p style={{ color: "var(--color-text-muted)", marginBottom: 20 }}>
              Forge hit an unexpected error. Reloading usually resolves it.
            </p>
            <pre
              style={{
                textAlign: "left",
                background: "var(--color-bg-inset)",
                border: "1px solid var(--color-border)",
                borderRadius: 10,
                padding: 12,
                overflow: "auto",
                fontSize: 12,
                color: "var(--color-danger)",
                marginBottom: 20,
              }}
            >
              {this.state.error.message}
            </pre>
            <button
              onClick={() => window.location.reload()}
              style={{
                background: "var(--color-accent)",
                color: "var(--color-accent-contrast)",
                border: "none",
                borderRadius: 6,
                padding: "8px 16px",
                fontWeight: 600,
                cursor: "pointer",
              }}
            >
              Reload Forge
            </button>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}
