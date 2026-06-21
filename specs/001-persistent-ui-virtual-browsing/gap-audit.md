# Gap Audit: Persistent UI Layout & Virtual Browsing

**Feature**: `001-persistent-ui-virtual-browsing`
**Updated**: 2026-06-22 (SC-004 RSS audit via `scripts/measure-resources.sh`; SC-002 scroll pending)

## Summary

| Code | Validated (auto) | Validated (manual) |
|------|------------------|------------------|
| 15 pass | 12 pass | SC-004 **pass** (RSS); SC-002 **pending** (scroll) |

Run: `./scripts/validate-feature-001.sh` — see `validation-results.md`.

---

## FR Gap Table

| FR-ID | Code | Validated | Method |
|-------|------|-----------|--------|
| FR-001 | pass | auto | static TopBottomPanel checks |
| FR-002 | pass | auto | static action buttons in top panel |
| FR-003 | pass | auto | bottom status panel |
| FR-004 | pass | auto | show_rows + scan_10k test |
| FR-005 | pass | auto | sc003_filter_10k_under_200ms |
| FR-006 | pass | auto | showing_count_label + v4 test |
| FR-007 | pass | auto | pick_folder grep + v3 status test |
| FR-008 | pass | auto | logic tests |
| FR-008a | pass | auto | v8 + static feh_available |
| FR-008b | pass | auto | code review |
| FR-009 | pass | auto | static (eprintln startup policy) |
| FR-010 | pass | auto | scan_10k <10s + scanning static |
| FR-011 | pass | auto | static pick_folder/menu |
| FR-012 | pass | auto | v9 empty scan |
| FR-013 | pass | auto | scanning static |
| FR-014 | pass | auto | code review |
| FR-015 | pass | auto | t068_permission_denied_warning |

## Success criteria

| SC | Validated | Notes |
|----|-----------|-------|
| SC-001 | auto | static layout checks |
| SC-002 | **manual** | 60fps scroll — requires GUI session ([003 validation](../003-gui-performance-validation/validation-results.md) pending) |
| SC-003 | auto | filter 10k <200ms test |
| SC-004 | **pass** | RSS &lt;150MB @10k — **pass** 2026-06-22: peak ~126 MB @10k, ~124 MB @1k ([003 RSS audit](../003-gui-performance-validation/validation-results.md)) |
| SC-005 | auto | v3 status test |
| SC-006 | auto | v4 counter test |
| SC-007 | auto | v8 feh missing test |