import { diffConfig } from "./diff"

describe("diffConfig", () => {
  it("returns no changes for identical objects", () => {
    const obj = { productName: "App", build: { devUrl: "x" } }
    expect(diffConfig(obj, JSON.parse(JSON.stringify(obj)))).toEqual([])
  })

  it("detects a changed leaf with before/after values", () => {
    const changes = diffConfig({ productName: "Old" }, { productName: "New" })
    expect(changes).toEqual([
      { path: "productName", before: "Old", after: "New" },
    ])
  })

  it("reports nested paths with dot notation", () => {
    const changes = diffConfig(
      { build: { devUrl: "a" } },
      { build: { devUrl: "b" } }
    )
    expect(changes).toEqual([{ path: "build.devUrl", before: "a", after: "b" }])
  })

  it("detects added and removed keys", () => {
    const changes = diffConfig({ a: 1 }, { b: 2 })
    expect(changes).toContainEqual({ path: "a", before: 1, after: undefined })
    expect(changes).toContainEqual({ path: "b", before: undefined, after: 2 })
  })

  it("treats arrays as leaf values", () => {
    const changes = diffConfig(
      { bundle: { targets: ["dmg"] } },
      { bundle: { targets: ["dmg", "deb"] } }
    )
    expect(changes).toEqual([
      {
        path: "bundle.targets",
        before: ["dmg"],
        after: ["dmg", "deb"],
      },
    ])
  })
})
