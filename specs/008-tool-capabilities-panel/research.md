# Research: Tool Capabilities Panel (008)

**Date**: 2026-06-22 (updated post-005)

## R1: Panel placement

**Decision**: Keep `SidePanel::right("tool_caps")` resizable, `default_width(300)`, `min_width(240)`.

**Rationale**: Shipped; fills horizontal dead space; central list stays focused on browsing.

**Alternatives considered**:
- Bottom drawer — rejected (001 layout uses bottom for status counter)
- Collapsible tab — rejected (already shipped as persistent side panel)

## R2: Format routing vs scanner reality

**Decision**: `format_routes()` notes MUST align with **005** scan inventory labels:
- **Native listed** — jpg/jpeg/png/webp/gif/bmp in scanner + inventory `native_listed`
- **Magick-detected (unlisted)** — optional `identify` during scan when ImageMagick on PATH
- **Converted** — `{stem}_processed.*` from Quick resize (005); not a separate panel section

Panel legend (main.rs): *"Scan = native listed or magick-detected (inventory bar)"*.

**Rationale**: POSITIONING requires honest UI; 005 inventory bar is the runtime source of truth for scan counts.

**Alternatives considered**:
- Static brochure text — rejected (FR-008 dynamic)
- Claim all exotic extensions are scanned — rejected until 010 convert bridge

## R3: Install copy commands

**Decision**: Hardcode `sudo apt install feh` and `sudo apt install imagemagick` in `DependencyStatus` builders.

**Rationale**: Linux-first constitution §IV; matches README and POSITIONING.

## R4: Recheck ownership split

**Decision**: **008** verifies panel **Recheck tools on PATH** button; **009** adds Tools menu entry sharing `refresh_tool_caps()`.

**Rationale**: Avoid duplicate spec work; single code path in `main.rs`.

## R5: Test strategy (retroactive)

**Decision**: Extend `tool_caps` unit tests for FR-008 dynamic branches (mock by constructing `ToolCapabilities { feh_available: false, ... }` if needed); manual quickstart for clipboard (egui ctx).

**Rationale**: Clipboard requires egui context — gap-audit documents manual pass for FR-005.