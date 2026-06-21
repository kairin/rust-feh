# Contract: Capabilities Panel UI (008)

**Updated**: 2026-06-22 (aligned with 005 inventory)

## Panel chrome

| Property | Value |
|----------|-------|
| Region | `SidePanel::right("tool_caps")` |
| Resizable | yes, `min_width` 240px |
| Scroll | vertical `ScrollArea` wrapping panel body |
| Title | `Tools & capabilities` |

## Sections (top to bottom)

1. **Dependencies** — feh (required), ImageMagick (optional)
2. **Recheck tools on PATH** button → `refresh_tool_caps()`
3. **Speed / timing** grid — Operation | Handler | Speed (+ detail line)
4. **Format discovery** — legend + extension groups (Scan / View / Resize)
5. **Missing required warning** when feh absent

## Dependency row

| State | Display |
|-------|---------|
| Installed | `✓` + `On PATH: {binary}` |
| Missing required | `✗` + `Not installed` + monospace install cmd + **Copy** |
| Missing optional | `○` + `Not installed` + install cmd + **Copy** |

## Copy action (FR-005)

- Trigger: `Copy##{dep.name}` small button
- Effect: `ctx.copy_text(install_cmd)`
- Status line: `Copied install command for {name}`

## Format discovery legend

Must include inventory-aligned scan definition:

> Scan = native listed or magick-detected (inventory bar); View = feh; Resize = quick resize demo.

Per-group `note` must reflect current `magick_available` / `feh_available` (FR-008).

## Speed / timing rows (minimum)

| Operation | Required |
|-----------|----------|
| Browse / filter / sort list | yes |
| Open / slideshow / navigate | yes |
| Set wallpaper | yes |
| Quick resize (jpg/png/webp) | yes |
| Exotic format view | yes |

## Bottom status region (FR-011)

Short pointer only, e.g. `Tools panel → dependencies, speed tiers, format routing` — no full dependency prose.

## Cross-feature

| Feature | Contract |
|---------|----------|
| 005 | Scan notes match inventory bar field names |
| 009 | Recheck in Tools menu must call same refresh as panel button |