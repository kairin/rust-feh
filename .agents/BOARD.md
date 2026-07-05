# .agents/BOARD.md — Class-B task-set index (system of record)

Status: todo | in-progress | blocked | done | verified

| Task set | Owner | Status | Notes |
|----------|-------|--------|-------|
| B-1 .specify upgrade | lead | **done** | PR #145 squash-merged to main (3bdcdff), CI + Codacy green, branch deleted |
| B-2 branch hygiene | lead | **done (local) / blocked (remote)** | 6 local branches deleted (incl. merged upgrade branch). Remote deletion of 4 origin branches blocked by permission classifier — awaiting maintainer: `git push origin --delete 014-multi-feh-clipboard 20260628-070300-chore-env-dotfiles-example feat/window-viewer-stability-validation fix/local-dev-env-and-jpeg-fallback` |
| B-3 doc hygiene | lead | **done** | Roadmap implement-order refreshed; NEXT-ROUND 2026-07-05 addendum; 002/004 Clarifications + status closed; deferrals recorded |
| B-4 verification baseline | lead | **done** | 122 pass / 0 fail / 2 ignored; release build pass; fmt/clippy local-unavailable (CI-enforced); details in ENV.md |
| A-closure (001/003/006 + 002/004) | lead | **done except 4 click-residues** | 006 T009 pass (evidence); 003 fully closed; 001: T026/T051/T055 closed, T038/T050/T058 carry notes — residual GUI clicks (V5, V6, V3 step 4, V8 step 5) pending maintainer choice: Xvfb / manual / waiver |
| C-1 / C-2 briefs | lead | **done** | `.agents/briefs/caption-plugin.md`, `.agents/briefs/image-tools-merge.md` — ready for maintainer `/speckit-specify` |

## Evidence log
- 2026-07-05: Branch audit — feat/window-viewer-stability-validation tree-identical to main
  (squash PR #144); fix/local-dev-env-and-jpeg-fallback = subset; pr143 residual commit
  (test temp-path isolation) superseded by `feh_filelist_temp_path()` src/ui_logic.rs:66;
  014-multi-feh-clipboard + 20260628-070300-chore-env-dotfiles-example merged.
  Tip SHAs at deletion: 014=7a1dbe4, 20260628=cd654c2, feat/wvsv=37f3452,
  fix/ldej=2a9ec4c, pr143=329a7f3, chore/specify-upgrade=6527352 (merged as 3bdcdff).
- 2026-07-05: .specify diff reviewed clean; manifest sha256 re-verified; merged as PR #145.
- 2026-07-05: Scripted GUI session — T009 feh geometry 1280×960 verified (xwininfo +
  screenshot, 5×5 image zoom-max); V1 controls @5k fixture; V8 feh-missing hint;
  SC-005 prefs restore live. Evidence: specs/001-*/evidence/, specs/006-*/evidence/.
  XTEST synthetic input blocked by GNOME Wayland (libei gating) — 4 click-steps residual.
- 2026-07-05: Dependabot reports 137 vulns on default branch (presumed archive/original-nfeh;
  Codacy excludes it, Dependabot does not) — surfaced to maintainer, no action taken.
