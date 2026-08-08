# .agents/ENV.md — Environment & tooling verification

_Verified 2026-07-05 by lead architect. Re-verify before relying on anything time-sensitive._

## Repo
- Remote: `origin = https://github.com/kairin/rust-feh.git` (verified `git remote -v`)
- Default branch: `main`. CI (workflow `CI`) green on main; 0 open PRs at engagement start; 1 open issue (#35 — nfeh CVE rationale, kept open intentionally as audit trail).
- `.specify/feature.json` → `specs/006-window-viewer-stability`.

## Spec Kit
- `specify --version` → **0.12.4** (matches `.specify/init-options.json` after upgrade commit).
- Integration: `ai: hermes`, `invoke_separator: "-"`.
- **Invocation surface**: hermes skills at `~/.hermes/skills/`, hyphen form:
  `speckit-specify`, `speckit-clarify`, `speckit-plan`, `speckit-tasks`, `speckit-analyze`,
  `speckit-implement`, `speckit-converge`, `speckit-checklist`, `speckit-constitution`,
  `speckit-taskstoissues`, `speckit-agent-context-update`.
  These are NOT Claude Code slash commands. From a Claude Code session, run them via the
  `hermes` CLI (Hermes Agent v0.18.0, `~/.local/bin/hermes`) or execute the underlying
  `.specify/scripts/bash/*.sh` + skill markdown manually.
- `.specify/workflows/`: bundled `speckit` workflow (specify → plan → tasks → implement).
- Extension: `agent-context` (refreshes AGENTS.md SPECKIT section after specify/plan).

## Build & test baseline
- Toolchain: Rust stable, edition 2021. Release profile: lto, codegen-units=1, panic=abort, strip.
- Static counts (2026-07-05): 125 `#[test]` fns (52 in src/, 73 in tests/), 0 `#[ignore]`,
  0 TODO/FIXME in src/. Docs cite "2 ignored" from older runs — reconciled by B-4 fresh run
  (results appended below when run).
- Conventions: 50-line function limit, complexity caps; clippy runs `-D warnings`.

## Known environment notes
- GUI available locally (Linux desktop) — manual validations run as scripted GUI sessions
  (xdotool + grim/import), per user decision 2026-07-05.
- GitHub Dependabot reports 137 vulnerabilities on default branch — presumed to be
  `archive/original-nfeh/`. Flagged to maintainer;
  candidate fix: `.github/dependabot.yml` ignore rules or archive removal decision (human call).

## B-4 fresh verification run (2026-07-05, main @ 3bdcdff)
- `cargo test`: **122 passed / 0 failed / 2 ignored** across all suites.
- Ignored-count discrepancy resolved: the two ignored tests use the `#[ignore = "reason"]`
  attribute form, which the earlier static scan's `#[ignore]` pattern missed —
  `tests/feature_005_list.rs:176` (needs ImageMagick + HEIC sample) and
  `src/scanner.rs:330` (manual venice-folder timing).
- `cargo build --release`: pass.
- `cargo fmt --check` / `cargo clippy`: **components not installed locally** (no rustup on
  PATH — matches NEXT-ROUND E1). Canonical fmt/clippy gates run in CI; CI green on main
  (last run: PR #145 merge, pass, plus Codacy pass).
- GUI environment: GNOME Wayland; app runs native Wayland by default; forced XWayland via
  `env -u WAYLAND_DISPLAY` for `import`/`xwininfo` capture. Synthetic input (XTEST) is NOT
  delivered (XWayland→libei portal gating) — click-driven validations need Xvfb or a human.
