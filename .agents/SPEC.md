# .agents/SPEC.md — Definition of done for CLASS-B work (frozen on approval 2026-07-05)

**Scope: class-B (non–Spec Kit) items ONLY.** Class-A items are judged against their own
`specs/<feature>/` folder via speckit analyze/converge — never against this file. Class-C
items get their own spec via `speckit-specify` (human checkpoint). Cross-cutting rules live
in `.agents/CONSTITUTION.md` and `.agents/GUARDRAILS.md`.

Derived from: README.md, docs/POSITIONING.md, docs/NFEH-COMPARISON-AND-MIGRATION.md,
docs/MAGICK-CACHE-SETUP.md, specs/OUTSTANDING-ISSUES-ROADMAP.md,
specs/NEXT-ROUND-CONSOLIDATED.md, open issue #35, existing tests, AGENTS.md.

## Product yardstick (context all class-B work must preserve)
- rust-feh is a Linux-first, lightweight, no-network feh ORCHESTRATOR: the GUI owns
  scan/filter/sort/select/launch; feh owns viewing. Single native binary, egui/eframe 0.30
  glow. It is NOT an image editor, NOT a thumbnail picker, NOT ImageMagick-dependent.
- Health invariants: `cargo test` green; `cargo clippy -- -D warnings` clean;
  `cargo fmt --check` clean; release build succeeds; CI green.

## B-1 — .specify 0.12.4 upgrade (IN FLIGHT: PR #145)
DONE when: the 7-file diff (reviewed clean 2026-07-05) is merged to main via PR with green
CI; working tree clean; no other change rides along.

## B-2 — Branch hygiene
DONE when these 5 branches are deleted locally AND on origin, with the audit evidence
recorded in BOARD.md:
`014-multi-feh-clipboard` (merged), `20260628-070300-chore-env-dotfiles-example` (merged),
`feat/window-viewer-stability-validation` (squash-merged as PR #144; tree-identical to main),
`fix/local-dev-env-and-jpeg-fallback` (strict subset of the above),
`pr143` (single residual commit superseded by pid-isolated `feh_filelist_temp_path()`,
src/ui_logic.rs:66). No other branches touched.

## B-3 — Doc hygiene / roadmap reconciliation
DONE when:
- `specs/OUTSTANDING-ISSUES-ROADMAP.md` implement-order reflects reality (005/008/009/011/
  012/013/014 shipped; 006 shipped incl. persistence via PR #144; 003 residual = manual
  session outcome).
- `specs/NEXT-ROUND-CONSOLIDATED.md` gets a dated addendum marking A1/C1/C2 done (PR #144),
  D1/D2 closed, 002/004 closure recorded.
- 002 and 004 spec.md each gain a Clarifications entry (date, question, decision:
  superseded-by-009 / absorbed-by-011, maintainer-confirmed 2026-07-05).
- Deferred decisions recorded: keyboard navigation + full config persistence = class-C,
  deferred until after integration workstreams (maintainer decision 2026-07-05).
- No scope/bucket RE-classification happens (advisory rule untriggered); only outcomes of
  maintainer-approved decisions are recorded.

## B-4 — Fresh verification baseline
DONE when: full suite (`cargo test`, `clippy -D warnings`, `fmt --check`, release build) run
on current main; results + counts appended to `.agents/ENV.md`; the "2 ignored" vs
"0 #[ignore] markers" discrepancy explained in one sentence there.

## B-5 — validation-results.md parity for 005/008/009/013/014 (OPTIONAL)
Not committed to; do only if maintainer asks. DONE when each listed feature has a
validation-results.md consistent with its checked board.

## Class-A closure companion (governed by feature tasks.md, listed for routing only)
- 001: T026/T038/T050/T051/T058 validated via scripted GUI session or waived-with-note;
  T055 closed as superseded by T068.
- 003: T037 closed as shipped-via-011; T026 fires only if session inconclusive.
- 006: T009 validated via scripted screenshot capture or waived-with-note.
Every closure = checkbox update + note in the owning tasks.md (single writer) and, where
the 007 advisory rule applies, a Clarifications entry.
