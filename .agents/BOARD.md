# .agents/BOARD.md — Class-B task-set index (system of record)

Status: todo | in-progress | blocked | done | verified

| Task set | Owner | Status | Notes |
|----------|-------|--------|-------|
| B-1 .specify upgrade | lead | **done** | PR #145 squash-merged to main (3bdcdff), CI + Codacy green, branch deleted |
| B-2 branch hygiene | lead | **done** | 6 local branches deleted (incl. merged upgrade branch). Remote stale branches deleted 2026-07-05: `014-multi-feh-clipboard`, `20260628-070300-chore-env-dotfiles-example`, `feat/window-viewer-stability-validation`, `fix/local-dev-env-and-jpeg-fallback`; `git ls-remote --heads origin ...` verifies all four absent. |
| B-3 doc hygiene | lead | **done** | Roadmap implement-order refreshed; NEXT-ROUND 2026-07-05 addendum; 002/004 Clarifications + status closed; deferrals recorded |
| B-4 verification baseline | lead | **done** | 122 pass / 0 fail / 2 ignored; release build pass; fmt/clippy local-unavailable (CI-enforced); details in ENV.md |
| A-closure (001/003/006 + 002/004) | lead | **done** | 006 T009 pass (evidence); 003 fully closed; 001 now 69/69. T038/T050/T058 closed 2026-07-05 by waiver-backed evidence: GNOME Wayland blocked synthetic clicks and Xvfb is not installed locally, but code/automated proxies verify behavior; waived residue is click synthesis only. |
| C-1 / C-2 briefs | lead | **done** | C-1 caption: specified as 015 then **deferred** (maintainer 2026-07-05). C-2 image-tools: brief ready, not yet specified. |
| 016 viewer round-trip & staged actions | sonnet-orchestrator + lead | **done** | 26/26 orchestrator tasks + T027 (PASS, zero HIGH/MED) + T028; 186 tests (+64); 3 short manual GUI checks listed in validation-results.md; PR #151 merged to main (2da6097). |
| 017 lazy-folder-scanning | sonnet-orchestrator + lead | **shipped, unmerged** | 6 commits on branch `017-lazy-folder-scanning` (tip `8689292`), never merged to `main` — branch `018-inspector-ux-rework` builds directly on top of it. Retro-specified 2026-07-12 under 018 Batch 0: `specs/017-lazy-folder-scanning/spec.md` + `tasks.md` (handback lists F1-F6 + deferred per-entry image-count cache, picked up as 018 Batch 0 prerequisites). |
| 018 inspector-ux-rework | sonnet-orchestrator | **in-progress (Batch 1 done)** | Batch 0 (pre-work fixes F1/F3-F6 + test isolation + SpecKit scaffolding) complete 2026-07-12 — 11/11 subtasks, `cargo test` 194/0/2 (+5), commits `70d0da0` + `b2aef05`. F1 escalated to opus-specialist (concurrency/data-integrity trigger). Batch 1 (open-state generalization + pin types + width floor + clippy fold-in) complete 2026-07-12 on branch `018-inspector-ux-rework` — `cargo test` 201/0/2 (+7), `cargo clippy --bins`/`--lib -D warnings` both clean, commit `b45b210`. No escalations this batch. Batch 2 (the big move: file list into inspector, central=stage) is next and carries the O-review trigger for nested-virtualization height math. Full batch plan: `specs/018-inspector-ux-rework/tasks.md`. Maintainer decisions: Browse collapsed-by-default, file list moves into inspector (central = stage only), menu duplicates removed, single pinnable Image-actions detached window. |

## Evidence log
- 2026-07-05: Branch audit — feat/window-viewer-stability-validation tree-identical to main
  (squash PR #144); fix/local-dev-env-and-jpeg-fallback = subset; pr143 residual commit
  (test temp-path isolation) superseded by `feh_filelist_temp_path()` src/ui_logic.rs:66;
  014-multi-feh-clipboard + 20260628-070300-chore-env-dotfiles-example merged.
  Tip SHAs at deletion: 014=7a1dbe4, 20260628=cd654c2, feat/wvsv=37f3452,
  fix/ldej=2a9ec4c, pr143=329a7f3, chore/specify-upgrade=6527352 (merged as 3bdcdff).
- 2026-07-05: Remote branch cleanup complete — `git push origin --delete 014-multi-feh-clipboard 20260628-070300-chore-env-dotfiles-example feat/window-viewer-stability-validation fix/local-dev-env-and-jpeg-fallback`; follow-up `git ls-remote --heads origin ...` returned no refs for the four targets.
- 2026-07-05: .specify diff reviewed clean; manifest sha256 re-verified; merged as PR #145.
- 2026-07-05: Scripted GUI session — T009 feh geometry 1280×960 verified (xwininfo +
  screenshot, 5×5 image zoom-max); V1 controls @5k fixture; V8 feh-missing hint;
  SC-005 prefs restore live. Evidence: specs/001-*/evidence/, specs/006-*/evidence/.
  XTEST synthetic input blocked by GNOME Wayland (libei gating); 4 click-only steps later accepted under waiver-backed closure.
- 2026-07-05: 001 click-residue closure — T038/T050/T058 marked complete by waiver-backed evidence after `cargo test --test feature_001_validation` passed 9/9. Xvfb was not installed (`xvfb-run`/`Xvfb` absent), so literal synthetic clicks remained unperformed; code paths and automated proxies verify the underlying behavior.
- 2026-07-05: Dependabot reports 137 vulns on default branch (presumed archive/original-nfeh;
  Codacy excludes it, Dependabot does not) — surfaced to maintainer, no action taken.
- 2026-07-12: 016 BOARD row corrected to reflect PR #151 already merged to `main`
  (`2da6097`); 017/018 rows added; see `specs/017-lazy-folder-scanning/` and
  `specs/018-inspector-ux-rework/` for detail.
