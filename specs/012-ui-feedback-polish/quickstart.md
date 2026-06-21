# Quickstart: UI Feedback & Network Scan Polish

## V1 — NAS scan without freeze (SC-001)

1. Choose folder on SMB/GVFS path (e.g. `smb-share:…/AI`).
2. Within 1s of scan start, open **View** menu and resize the window.
3. **Pass**: No window-manager "not responding" dialog; Activity log shows network-optimized scan message.

## V2 — Live status bar (SC-002)

1. Start scan (local or network).
2. Watch bottom status bar for pulsing border/background and animated `● Scanning...` dots.
3. **Pass**: Indicator changes visually at least once per 2s until scan completes; animation stops when idle.

## V3 — Dependencies collapsed when OK (SC-003)

1. Cold start with feh + required tools on PATH.
2. Open right **Tools & capabilities** panel.
3. **Pass**: Header shows ✅ and section is collapsed; click header to expand; Recheck keeps collapsed if still OK.

## V4 — Bottom-bar speed tips (SC-004)

1. Load any folder (tools detected).
2. Read bottom bar right side — speed/timing tip with spinner.
3. Wait 8s.
4. **Pass**: Tip text changes at least once when multiple operations exist.

## V5 — Detach activity log (SC-005)

1. Load folder and expand **Activity log** (bordered panel).
2. Click **Detach window** → separate window opens.
3. Click window **X** to close → main panel shows reattach placeholder.
4. Click **Reattach** (or Detach again).
5. **Pass**: Log content and Copy log work in both modes; flow completes in under 30s.