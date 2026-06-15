export interface ConfigChange {
  path: string
  before: unknown
  after: unknown
}

/**
 * Produce a flat list of changed leaf paths between two config objects.
 * Used to preview exactly what a config save will write to disk.
 */
export function diffConfig(
  before: any,
  after: any,
  prefix = ""
): ConfigChange[] {
  const changes: ConfigChange[] = []
  const keys = new Set([
    ...Object.keys(before || {}),
    ...Object.keys(after || {}),
  ])
  keys.forEach((key) => {
    const path = prefix ? `${prefix}.${key}` : key
    const b = before?.[key]
    const a = after?.[key]
    const bothObjects =
      b &&
      a &&
      typeof b === "object" &&
      typeof a === "object" &&
      !Array.isArray(b) &&
      !Array.isArray(a)
    if (bothObjects) {
      changes.push(...diffConfig(b, a, path))
    } else if (JSON.stringify(b) !== JSON.stringify(a)) {
      changes.push({ path, before: b, after: a })
    }
  })
  return changes
}
