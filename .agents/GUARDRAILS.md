# .agents/GUARDRAILS.md — Operational guardrails (bind every phase, every agent)

Violating these is a stop-the-line event. Read alongside `.agents/CONSTITUTION.md` and your
task file before doing anything.

## Cost governance
- Before ANY fan-out: lead announces model tier + agent count + rationale.
- Default to cheapest tier that fits: **Haiku** = reading, mechanical coding; **Sonnet** =
  orchestration, review, search; **Opus** = ONLY on a cited escalation trigger (see
  Delegation model below). Never an Opus fleet reflexively.
- Concurrent-agent cap: ≤ 8 live agents unless the maintainer approves more.
- Surface running spend at each phase boundary.

## Git & commit discipline
- ONLY orchestrators (Sonnet) and the lead commit. Haiku implementers return diffs; they do
  not commit or push.
- No push / no PR without maintainer approval (the approved engagement plan of 2026-07-05
  pre-authorizes: the .specify upgrade PR, task-set branches/PRs for B-2..B-4 and class-A
  closure, and branch deletion of the 5 audited stale branches — nothing else).
- **Branch naming ruling (recorded 2026-07-05):** Spec Kit FEATURE branches follow the
  constitution: `###-feature-name`. Non-feature task-set branches (chore/fix/docs) follow
  the datetime convention: `YYYYMMDD-HHMMSS-<kind>-<slug>`. This reconciles the constitution
  with the engagement prompt without amending either; revisit only with the maintainer.
- Commit a GREEN checkpoint (compiles + tests pass) between task sets.

## Parallel isolation
- Concurrent task sets that could touch overlapping files run in separate git worktrees
  (Agent-tool `isolation: "worktree"`). If two sets share files, serialize instead.
- Never two live agents writing the same file. `tasks.md` files have a single writer: the
  owning orchestrator.

## Security checkpoint (non-negotiable for the integration workstreams)
- rust-feh exists because nfeh was retired over CVEs (issue #35). Both integrations expand
  attack surface: caption spawns Python/GPU subprocesses (argument injection, path handling,
  `direnv exec` trust, HF_TOKEN handling); image-tools dlopens libheif at runtime.
- `/security-review` runs on each workstream's diff BEFORE merge, with explicit attention to
  subprocess argument construction, untrusted path handling, and any dlopen loader.
  A failed security review is a hard block.
- Additional standing rule: **caption's `batch_caption.py` ships images to a remote HF Space**
  — any integration of that path is a network-egress + privacy event and must be explicit,
  opt-in, and documented in the feature spec. rust-feh itself stays no-network.

## Repo constraints for all implementers
- Keep logic OUT of `main.rs` (3510 lines — biggest liability). Testable homes: `ui_logic.rs`,
  `image_proc.rs`, `tool_caps.rs`, `scanner.rs`, `types.rs`.
- 50-line functions, complexity caps. Clippy `-D warnings`. `cargo fmt` clean.
- Small branches, green CI; PR discipline as per repo history.

## Delegation model (Phase 3)
- **Sonnet orchestrator** owns one task set / one class-A feature: decompose, dispatch Haiku,
  integrate, keep task file + BOARD.md (or tasks.md checkboxes) current.
- **Haiku implementers**: one well-specified subtask each, small blast radius.
- **Opus specialist** only on cited triggers: (a) Haiku fails acceptance twice; (b) cross-
  cutting / architectural / concurrency / security-sensitive change (image-tools merge design,
  any main.rs restructuring); (c) genuine ambiguity after re-reading the spec; (d) subtask
  touches > 5 files or breaks a public contract.

## Resume durability
- `.agents/{SPEC,BACKLOG,BOARD,ROUTING,GUARDRAILS}.md` + each feature `tasks.md` ARE the
  system of record. On cold resume or compaction: RELOAD them before acting. Keep them
  current — they are the handoff, not your context window.

## Multi-agent advisory rule (007 FR-006)
Before changing scope, bucket classification, or implement order of anything covered by
`specs/OUTSTANDING-ISSUES-ROADMAP.md`: seek advice (Codex / Grok / Hermes / DeepSeek 4 Pro /
human maintainer) and record the outcome in the target feature spec.md **Clarifications**
section (date, question, decision). Maintainer arbitration is final.
