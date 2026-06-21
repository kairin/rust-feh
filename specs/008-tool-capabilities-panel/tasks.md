# Tasks: Tool Capabilities Panel

**Input**: `specs/008-tool-capabilities-panel/`  
**Generated**: 2026-06-22 | **Type**: Retroactive gap-audit  
**Prerequisites**: plan.md, spec.md — code largely shipped in `src/tool_caps.rs`, `src/main.rs`

## Phase 1: Setup

- [x] T001 Run `cargo test tool_caps` — baseline green
- [x] T002 Run `cargo clippy -- -D warnings`

## Phase 2: Gap Audit (US1–US4)

- [x] T003 [US1] Audit FR-001–FR-005 deps + install copy in `src/main.rs` `render_tool_caps_panel` → `gap-audit.md`
- [x] T004 [US2] Verify Copy sets clipboard + status per FR-005 in `src/main.rs`
- [x] T005 [US3] Audit speed grid + format routes in `src/tool_caps.rs` vs FR-006–FR-008
- [x] T006 [US4] Verify SidePanel resizable scroll per FR-012–FR-013 in `src/main.rs`
- [x] T007 Fix any gaps from T003–T006 in `src/tool_caps.rs` or `src/main.rs`

## Phase 3: Validate

- [x] T008 Run `specs/008-tool-capabilities-panel/quickstart.md` manual checklist
- [x] T009 [P] Cross-check `docs/POSITIONING.md` claims vs panel per SC-005
- [x] T010 [P] After **005** lands, verify format notes align with inventory labels in `src/tool_caps.rs`

**Total**: 10 tasks | **MVP**: T001–T008