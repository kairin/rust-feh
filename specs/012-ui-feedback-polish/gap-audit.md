# Gap Audit: UI Feedback & Network Scan Polish

**Date**: 2026-06-22 | **Status**: PASS (implemented; manual SMB GUI pending)

| FR-ID | Status | Evidence |
|-------|--------|----------|
| FR-001 | pass | `scan_magick_enabled` in `ui_logic.rs`; used in `scan_directory` |
| FR-002 | pass | `is_network_mount_path` — GVFS, smb-share, `/nfs/`, `//` |
| FR-003 | pass | `activity_pulse_color` + busy border on bottom panel |
| FR-004 | pass | `render_scanning_label` + `request_repaint_after(200ms)` |
| FR-005 | pass | Network scan label + activity log message on scan start |
| FR-006 | pass | ✅/⚠ dependencies header + per-dep icons |
| FR-007 | pass | `deps_section_open = !has_missing_required()` at startup |
| FR-008 | pass | `refresh_tool_caps` sets `deps_section_open` from required status |
| FR-009 | pass | `rotating_operation_tip` in bottom bar only |
| FR-010 | pass | 4s rotation + spinner glyph |
| FR-011 | pass | `Frame::group` on inventory, list, activity log |
| FR-012 | pass | `activity_log_detached` + `egui::Window` |
| FR-013 | pass | Window `.open()` + Reattach placeholder |
| FR-014 | pass | `render_activity_log_body` in attached and detached modes |

| SC-ID | Status | Evidence |
|-------|--------|----------|
| SC-006 | pass | `cargo test` + `cargo clippy -- -D warnings` |
| SC-001 | manual pending | [validation-results.md](./validation-results.md) V1 |
| SC-002 | manual pending | validation-results V2 |
| SC-003 | pass | Cold-start deps collapse (automated policy); manual quickstart optional |
| SC-004 | pass | Tip rotation logic + unit tests for path policy |
| SC-005 | manual pending | validation-results V5 detach flow |