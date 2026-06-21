# Code & Architecture Requirements Quality Checklist: Persistent UI Layout & Virtual Browsing

**Purpose**: Deep traceability audit — validate that code/architecture requirements in the spec and plan are complete, consistent, and satisfy all Constitution principles. Constitution violations are gated (blocking).
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

---

## Constitution Alignment (⛔ GATED — violations block implementation)

- [ ] CHK001 - Does the spec require any feh feature to be reimplemented in Rust rather than delegated? [Constitution §I, Spec §FR-001–FR-010] ⛔ GATE
- [ ] CHK002 - Does the spec or plan require any new Cargo dependency beyond the existing set (egui 0.30, walkdir 2, rfd 0.15, image 0.25, which 6)? If so, is justification documented? [Constitution §II, Plan §Technical Context] ⛔ GATE
- [ ] CHK003 - Does the spec require GUI code to remain in `src/main.rs` (or separate widget files) without leaking into core modules (scanner, image_proc, types)? Is this constraint stated or merely assumed? [Constitution §III, Spec — no explicit module boundary requirement] ⛔ GATE
- [ ] CHK004 - Does the spec require graceful degradation when `feh` is not installed, or does it assume feh is always present? Constitution §IV mandates degradation but spec §Assumptions states feh is "assumed installed." [Constitution §IV vs Spec §Assumptions] ⛔ GATE
- [ ] CHK005 - Does FR-010 ("UI MUST remain responsive during scan") conflict with the Edge Cases acknowledgment that sync scanning "may briefly block"? Constitution §V requires performance awareness — is this contradiction a violation? [Constitution §V, Spec §FR-010 vs Edge Cases ¶2] ⛔ GATE

## FR-to-Task Traceability

- [ ] CHK006 - Does every functional requirement (FR-001 through FR-015) have at least one corresponding task in tasks.md, and is that mapping documented? [Traceability, Spec §FR-001–FR-015 vs tasks.md]
- [ ] CHK007 - Do all non-polish tasks (T001-T028) trace back to a specific FR or user story acceptance scenario? Are any tasks orphaned (no upstream requirement)? [Traceability, tasks.md]
- [ ] CHK008 - Is FR-006 (status bar counter) explicitly verified by a task? T012 references FR-003 and FR-006 together — is the counter behavior (`Showing X / Y`) independently verifiable or only checked as a side effect? [Traceability, Spec §FR-006 vs T012]
- [ ] CHK009 - Does the gap-audit task (T004) have clear acceptance criteria for what constitutes a "gap"? Is the gap-audit.md format specified, or is the auditor expected to invent one? [Traceability, tasks.md T004]

## Data Model Completeness

- [ ] CHK010 - Does data-model.md document every entity referenced in the spec (ImageEntry, FilteredView, RustFehApp)? Are Selection and SortMode (currently dead code) addressed? [Completeness, data-model.md vs Spec §Key Entities]
- [ ] CHK011 - Are the state transitions described in data-model.md (INITIAL → LOADING → LOADED → FILTERED) consistent with all acceptance scenarios in the spec? For example, does the "no folder loaded" state have a defined transition? [Consistency, data-model.md vs Spec §US1 Acceptance Scenarios]
- [ ] CHK012 - Does the spec define what happens to `selected` when `images` is cleared (e.g., during rescan)? Data-model.md states this invariant but the spec does not encode it as a requirement. [Completeness, Spec — no explicit selection-lifecycle requirement]

## Research-to-Spec Alignment

- [ ] CHK013 - Does the research decision to use `TopBottomPanel::top` (research.md RQ1) have a corresponding spec requirement that controls must be in a non-scrolling region? FR-001 requires "persistent non-scrolling region" — is the alignment explicit or coincidental? [Consistency, research.md RQ1 vs Spec §FR-001]
- [ ] CHK014 - Does the research decision to pre-compute filtered indices (research.md RQ3) have a corresponding spec requirement? FR-005 specifies filter behavior but not the computational approach — is this gap intentional (implementation detail) or a missing requirement? [Consistency, research.md RQ3 vs Spec §FR-005]
- [ ] CHK015 - Does the research decision to keep the debug log as a `CollapsingHeader` in CentralPanel (research.md RQ5) satisfy FR-009's "must not compete with primary controls for persistent screen space"? Is the rationale documented in the spec or only in research.md? [Consistency, research.md RQ5 vs Spec §FR-009]

## Plan Technical Context Completeness

- [ ] CHK016 - Does plan.md's Technical Context constrain all spec requirements? For example, SC-004 requires <150MB RSS — does the plan specify how this will be measured (baseline conditions, tooling)? [Completeness, Plan §Technical Context vs Spec §SC-004]
- [ ] CHK017 - Does the plan address the spec's assumption that "async scanning is deferred to Area 6" by specifying what "responsive" means during sync scan? The plan acknowledges sync scanning is retained but doesn't define acceptable blocking duration. [Completeness, Plan §Technical Context vs Spec §Assumptions]
- [ ] CHK018 - Is the plan's "single crate" decision consistent with the spec's scope? The spec doesn't mention workspace structure, but the plan explicitly defers workspace split to Area 8. Is this documented as a conscious tradeoff? [Consistency, Plan §Structure Decision vs Spec — no structure requirement]

## Error Path & Exception Flow Requirements

- [ ] CHK019 - Are requirements specified for what the application displays when `feh` spawn fails (process not found, permission denied)? The spec's FR-008 handles "no image selected" but not feh launch failure. [Gap, Spec §FR-008]
- [ ] CHK020 - Are requirements specified for what happens when the `image` crate fails to process an image (corrupt file, unsupported subformat)? The spec mentions image processing in FR-002 ("Quick resize") but no error path. [Gap, Spec §FR-002]
- [ ] CHK021 - Are requirements specified for filesystem errors during scanning (permission denied on subdirectory, dangling symlink)? walkdir follows symlinks disabled per Constitution §V, but permission errors are not addressed in the spec. [Gap, Spec — no filesystem-error requirements]

## Constitution Self-Assessment Audit

- [ ] CHK022 - The plan's Constitution Check claims "scanner, image_proc, types modules untouched and independent" (§III PASS). Does the spec require this independence to be preserved, or does it merely assume the current code structure won't regress? [Constitution §III, Plan §Constitution Check]
- [ ] CHK023 - The plan's Constitution Check claims "No new dependencies added" (§II PASS). Is this verified by a task, or is it an assertion with no verification mechanism? If a future gap-fix adds a dependency, is there a gate to catch it? [Constitution §II, Plan §Constitution Check vs tasks.md — no dependency-audit task]
- [ ] CHK024 - Does the spec's scope (US1-US3 only) leave any Constitution principle untested? For example, §V Performance Awareness applies to the whole app — does the spec's scope adequately cover performance requirements, or are deferred areas (async scanning) creating unverified performance assumptions? [Coverage, Constitution §V vs Spec §Assumptions]
- [x] CHK025 - Does `gap-audit.md` exist with a row for every FR-001–FR-015 before user-story implementation begins? [Traceability, tasks.md T004 vs gap-audit.md]

## Notes

- ⛔ GATE items (CHK001-CHK005): These must pass before implementation proceeds. CHK004 resolved in spec via FR-008a. CHK005 resolved via Clarifications 2026-06-21 (adversarial): scanning indicator + brief stall acceptable.
- **2026-06-21 clarify**: CHK025 passes — `gap-audit.md` and `remediation.md` created.
- CHK023 identifies a process gap: the plan's Constitution Check is self-reported with no verification task. If a gap-fix introduces a dependency, nothing catches it.
- CHK008 may reveal a traceability weakness: FR-006 shares a verification task with FR-003, making the counter behavior a side-effect check rather than independently verified.
