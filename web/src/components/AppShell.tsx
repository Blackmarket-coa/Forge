import React from "react"
import { Badge } from "./ui/badge"
import { Tier } from "../lib/tier"
import styles from "./AppShell.module.scss"

export interface NavItem {
  id: string
  label: string
  icon: string
  disabled?: boolean
  hint?: string
}

export default function AppShell({
  nav,
  activeNav,
  onNavigate,
  tier,
  theme,
  onToggleTheme,
  children,
}: {
  nav: NavItem[]
  activeNav: string
  onNavigate: (id: string) => void
  tier: Tier
  theme: "dark" | "light"
  onToggleTheme: () => void
  children: React.ReactNode
}) {
  const tierTone = tier === "free" ? "neutral" : "accent"
  return (
    <div className={styles.shell} data-theme={theme}>
      <aside className={styles.sidebar}>
        <div className={styles.brand}>
          <span className={styles.brandMark} aria-hidden>
            🔨
          </span>
          <span className={styles.brandName}>Forge</span>
        </div>

        <nav className={styles.nav}>
          {nav.map((item) => (
            <button
              key={item.id}
              className={[
                styles.navItem,
                activeNav === item.id ? styles.navItemActive : "",
              ]
                .filter(Boolean)
                .join(" ")}
              onClick={() => !item.disabled && onNavigate(item.id)}
              disabled={item.disabled}
              title={item.disabled ? item.hint : undefined}
            >
              <span className={styles.navIcon} aria-hidden>
                {item.icon}
              </span>
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <div className={styles.sidebarFooter}>
          <button
            className={styles.themeToggle}
            onClick={onToggleTheme}
            title="Toggle theme"
          >
            <span aria-hidden>{theme === "dark" ? "🌙" : "☀️"}</span>
            <span>{theme === "dark" ? "Dark" : "Light"}</span>
          </button>
          <button
            className={styles.tierRow}
            onClick={() => onNavigate("settings")}
            title="Manage license"
          >
            <Badge tone={tierTone}>{tier.toUpperCase()}</Badge>
          </button>
        </div>
      </aside>

      <main className={styles.content}>
        <div className={styles.contentInner}>{children}</div>
      </main>
    </div>
  )
}
