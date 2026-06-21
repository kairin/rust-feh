# Feature Specification: Outstanding Issues Master Index

**Feature Branch**: `007-outstanding-roadmap`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Formalize [OUTSTANDING-ISSUES-ROADMAP.md](../OUTSTANDING-ISSUES-ROADMAP.md) as the canonical Spec Kit orchestration document for features 002–006 promoted from feature 001. Include a **multi-agent advisory** requirement when implementers are unsure.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/outstanding-issues.md)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find the Right Feature for an Issue (Priority: P1)

A maintainer or agent finishes feature 001 and sees an open item in `outstanding-issues.md`. They open the master index and immediately know which numbered feature spec owns the work and in what order to implement it.

**Why this priority**: Without a single index, outstanding issues get re-fixed in 001 or lost between sessions.

**Independent Test**: Each row in outstanding-issues Bucket A–D and dogfood table links to exactly one feature spec or explicit "no feature" rationale.

**Acceptance Scenarios**:

1. **Given** outstanding issue A5, **When** reading the master index, **Then** the owner is feature 002 with link to `spec.md`.
2. **Given** closed Bucket B items, **When** reading the index, **Then** they appear under "Not promoted" with reason.
3. **Given** a new dogfood UX gap, **When** triaged, **Then** process says create new `00N-short-name` spec or append to 005/006 — not silent code change.

---

### User Story 2 - Run the Standard Pipeline Per Feature (Priority: P1)

An agent picks the active feature from `.specify/feature.json` and runs plan → tasks → implement without guessing artifact paths.

**Why this priority**: Spec Kit value is repeatable workflows, not one-off markdown.

**Independent Test**: Each feature 002–006 directory contains `spec.md`; roadmap lists commands and expected outputs.

**Acceptance Scenarios**:

1. **Given** feature 003 is active, **When** `/speckit-plan` runs, **Then** artifacts land in `specs/003-gui-performance-validation/`.
2. **Given** implement order in index, **When** 003 completes, **Then** active feature advances to 005 or 006 per roadmap (documented in `feature.json`).

---

### User Story 3 - Consult Advisory Agents When Unsure (Priority: P1)

An agent or human is uncertain about scope, bucket classification, implement order, or whether to patch 001 vs create a new feature. They MUST seek a second opinion before proceeding.

**Why this priority**: Prevents duplicate work, wrong-bucket fixes, and silent shadow features (lesson from feature 001 dogfood).

**Independent Test**: Roadmap and this spec contain an advisory table; clarify sessions document which advisor was consulted for non-obvious decisions.

**Acceptance Scenarios**:

1. **Given** an issue could be Bucket A (docs) or B (code), **When** triaging, **Then** agent consults at least one advisory agent before implementing.
2. **Given** performance validation might be manual or automated, **When** unsure, **Then** agent asks Codex, Grok, Hermes, or DeepSeek 4 Pro (or human) and records decision in feature clarify section.
3. **Given** advisory response conflicts with spec, **When** resolving, **Then** human maintainer arbitrates; spec/clarify updated before `/speckit-implement`.

---

### Edge Cases

- Two features appear to own the same issue — index MUST be updated before either implement starts.
- Advisory agent unavailable — document assumption in target feature `spec.md` Clarifications; do not block P2 work indefinitely.
- Feature 001 reopened — index notes whether work belongs in 001 hygiene vs new 00N.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `specs/OUTSTANDING-ISSUES-ROADMAP.md` MUST remain the **master index** linking buckets, issue IDs, features 002–006, priorities, and implement order.
- **FR-002**: Each promoted feature MUST have its own directory `specs/00N-short-name/` with `spec.md` and `checklists/requirements.md`.
- **FR-003**: `specs/001-persistent-ui-virtual-browsing/outstanding-issues.md` MUST link to the master index and promoted features.
- **FR-004**: `.specify/feature.json` MUST list all features in `features[]` and designate one `feature_directory` as **active** for plan/tasks/implement.
- **FR-005**: Recommended implement order MUST be documented: **003 → 005 → 006 → 004 → 002** (unless advisory consensus changes it with recorded rationale).
- **FR-006**: **Multi-agent advisory** — when uncertain about scope, classification, architecture, or feature boundaries, implementers MUST seek advice from one or more of:
  - **Codex** (OpenAI Codex CLI / coding agent)
  - **Grok** (Grok Build / Cursor agent)
  - **Hermes** (Hermes Agent orchestrator)
  - **DeepSeek 4 Pro** (alternative reasoning model)
  - **Human maintainer** (always valid)
- **FR-007**: Advisory consultations for scope-affecting decisions MUST be summarized in the relevant feature `spec.md` **Clarifications** session (date + question + outcome).
- **FR-008**: Items in "Not promoted" table MUST NOT receive implement tasks without a new `/speckit-specify` pass.
- **FR-009**: Completing feature 003 MUST update feature 001 `gap-audit.md` validated tier for SC-002/SC-004.

### Key Entities

- **MasterIndex**: `OUTSTANDING-ISSUES-ROADMAP.md` — bucket map, order, pipeline, advisory policy.
- **PromotedFeature**: 002–006 specs derived from 001 outstanding issues + dogfood.
- **AdvisoryRecord**: optional clarify bullet — who consulted, question, decision.
- **ActiveFeature**: single `feature_directory` pointer in `feature.json`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of open outstanding issues (post-001 convergence) map to a feature link or explicit not-promoted row.
- **SC-002**: A new contributor can choose the next feature to implement in under 5 minutes using only the master index.
- **SC-003**: At least one documented advisory consultation example exists in a child feature clarify section before that feature's implement phase closes.
- **SC-004**: No duplicate feature specs created for the same issue ID after index is published.

## Assumptions

- Features 002–006 specs already exist from prior `/speckit-specify` pass (2026-06-21).
- This feature (007) governs **process and documentation**, not application runtime code.
- "AGY" in user input interpreted as **agent orchestrators** (Hermes / multi-agent workflows); listed explicitly as Hermes.
- Advisory step is **best-effort** for solo humans; mandatory for autonomous agents when confidence is low.
- Skills live in `~/.grok/skills/` for Grok; Hermes skills in `~/.hermes/skills/` per project setup.

## Master index snapshot (normative)

### Promoted features

| Feature | Outstanding source | Covers |
|---------|-------------------|--------|
| [002-feh-runtime-detection](../002-feh-runtime-detection/spec.md) | A5 | Re-check feh after install without restart |
| [003-gui-performance-validation](../003-gui-performance-validation/spec.md) | Bucket C | SC-002 scroll + SC-004 RSS; closes 001 validated pending |
| [004-scanner-resilience](../004-scanner-resilience/spec.md) | Bucket D | Non–permission-denied walkdir errors |
| [005-image-list-presentation](../005-image-list-presentation/spec.md) | Dogfood | Folder column, sort, path-aware filter |
| [006-window-viewer-stability](../006-window-viewer-stability/spec.md) | Dogfood | Window presets, list fill, feh tiny-image policy |

### Not promoted (closed on 001)

| Issue | Why no new feature |
|-------|-------------------|
| A1–A4 | Doc drift / intentional — sync 001 artifacts only |
| B1, B2 | Fixed T065, T069 |
| T067/T068 automation | Stays on 001; **003** finishes manual tier |

### Pipeline per feature

```
/speckit-plan    → plan.md, research.md, data-model.md, quickstart.md
/speckit-tasks   → tasks.md
/speckit-implement
```

### When to seek advisory (FR-006 triggers)

| Situation | Example | Action |
|-----------|---------|--------|
| Bucket unclear | Artifact drift vs real bug | Consult before code change |
| New issue mid-sprint | UX complaint after 006 | Specify new 00N or extend 005/006 — ask if unsure |
| Perf claim | Is SC-002 pass subjective? | Consult + document in 003 |
| Tooling | Which agent runs validate script | Hermes/Codex/Grok for CI wiring |
| Priority change | Skip 003, do 005 first | Advisory + update roadmap + feature.json |

## Clarifications

### Session 2026-06-21

- Q: What is "AGY" in user request? → A: Treat as **agent orchestrator** coverage; Hermes named explicitly in FR-006.
- Q: Is master index a code feature? → A: **No** — documentation/process feature 007; canonical file remains `OUTSTANDING-ISSUES-ROADMAP.md`.
- Q: Active feature for plan? → A: **003** recommended first; `feature.json` updated when 003 plan completes.