import React from "react"
import { openExternal } from "../../api/api"

/**
 * Anchor for URLs outside the app. Inside Tauri a raw target="_blank" anchor
 * navigates the webview in-place (showing an OS error page), so the click is
 * intercepted and routed to the system browser instead. The real href is kept
 * for styling, hover preview, and accessibility.
 */
export function ExternalLink({
  href,
  className,
  children,
}: {
  href: string
  className?: string
  children: React.ReactNode
}) {
  return (
    <a
      href={href}
      className={className}
      target="_blank"
      rel="noreferrer"
      onClick={(e) => {
        e.preventDefault()
        void openExternal(href)
      }}
    >
      {children}
    </a>
  )
}
