# .agents/ROUTING.md — Item → class → source-of-truth map (view over BACKLOG.md)

| Item | Class | Source of truth / board | Yardstick at review |
|------|-------|-------------------------|---------------------|
| B-1 .specify upgrade | B | .agents/tasks/B-1.md · BOARD.md | .agents/SPEC.md §B-1 |
| B-2 branch hygiene | B | .agents/tasks/B-2.md · BOARD.md | .agents/SPEC.md §B-2 |
| B-3 doc hygiene | B | .agents/tasks/B-3.md · BOARD.md | .agents/SPEC.md §B-3 |
| B-4 verification baseline | B | .agents/tasks/B-4.md · BOARD.md | .agents/SPEC.md §B-4 |
| B-5 validation parity (optional) | B | not scheduled | .agents/SPEC.md §B-5 |
| 001 residuals | A | specs/001-persistent-ui-virtual-browsing/tasks.md | that folder (spec/plan/quickstart) via analyze/converge |
| 003 residuals | A | specs/003-gui-performance-validation/tasks.md | that folder via analyze/converge |
| 006 T009 | A | specs/006-window-viewer-stability/tasks.md | that folder via analyze/converge |
| 002 closure record | A | specs/002-feh-runtime-detection/spec.md (Clarifications) | superseded by 009 |
| 004 closure record | A | specs/004-scanner-resilience/spec.md (Clarifications) | absorbed by 011 |
| C-1 caption plugin | C | .agents/briefs/caption-plugin.md → future specs/015-* | future feature folder |
| C-2 image-tools integration | C | .agents/briefs/image-tools-merge.md → future specs/016-* | future feature folder |
| C-3 keyboard navigation | C | BACKLOG.md (deferred) | future feature folder |
| C-4 full config persistence | C | BACKLOG.md (deferred) | future feature folder |

Process rules: class-A boards have a single writer (owning orchestrator). Scope/bucket
changes to anything in specs/OUTSTANDING-ISSUES-ROADMAP.md trigger the 007 FR-006 advisory
rule (see GUARDRAILS.md).
