# Contract: Status Bar UI

**Feature**: 012-ui-feedback-polish | **FR**: FR-003–FR-010, FR-014

## Layout (bottom `TopBottomPanel`)

| Zone | Content |
|------|---------|
| Left | Showing count; scan status or idle status; Copy status |
| Right | Speed/timing label; rotating tip + spinner glyph |

## Scanning state

- Distinct visual from idle: pulse fill, blue border, colored count label, `● Scanning...` with animated dots.
- Network scan adds suffix: `(network folder — UI stays responsive)`.
- `request_repaint_after(200ms)` while scanning.

## Idle state

- No scan pulse on count/status.
- Tip rotation may continue at 4s interval when capability tips exist.

## Removed from right panel

- `operation_timings()` carousel MUST NOT appear in Tools panel (format route view-speed lines remain).