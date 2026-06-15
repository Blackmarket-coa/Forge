import React from "react"
import ReactDOM from "react-dom"
import { SnackbarProvider } from "notistack"
import App from "./App"
import ErrorBoundary from "./components/ErrorBoundary"
import { AppStateProvider } from "./providers/AppStateProvider"
import "./index.scss"

ReactDOM.render(
  <ErrorBoundary>
    <SnackbarProvider
      maxSnack={3}
      autoHideDuration={4000}
      anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
    >
      <AppStateProvider>
        <App />
      </AppStateProvider>
    </SnackbarProvider>
  </ErrorBoundary>,
  document.getElementById("root")
)
