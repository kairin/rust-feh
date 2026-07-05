# Integration brief — caption (TagForge) as a rust-feh external tool
_Lead-architect brief for the maintainer's `speckit-specify` session. Facts verified
2026-07-05 against github.com/kairin/caption (private repo). Likely feature number: 015._

## Context
caption is a Python ≥3.12 / Gradio / PyTorch LoRA-dataset captioning toolkit (backends:
JoyCaption Beta One + Alpha Two, ToriiGate v0.4 7B/2B, Qwen2.5-VL 7B/3B; 4-bit via
bitsandbytes; uv-managed; last push 2026-05-01; 0 open issues; **no tags/releases**).
Goal: from the rust-feh browser, select images/folder → dispatch captioning → surface the
resulting `<stem>.txt` sidecars.

## ⚠ Verified correction to the original premise
The headless CLI is NOT local. `batch_caption.py <folder>` calls the public HF Space
`fancyfeast/joy-caption-beta-one` via `gradio_client` (requires `HF_TOKEN` and network);
verified in source: writes `<stem>.txt` next to each image (or `--output-dir`), supports
`--caption-type`, `--caption-length`, `--skip-existing`. Local GPU inference (~3–9 GB VRAM,
~17 GB weights, RTX 3060 12GB recommended) exists ONLY in the Gradio UI path (`app.py`,
localhost:7860). "Fully local/offline" currently holds for the UI path only.

Consequences: rust-feh's positioning ("no network access; entire vulnerability classes
eliminated by design") makes shipping user images to a public Space a privacy/egress event,
even if the egress happens in a child process. Also: user images leave the machine; the
Space is third-party (fancyfeast), rate-limited, and can change/vanish.

## Recommended shape
**Minimal subprocess integration via the existing `tool_caps.rs` capability-matrix pattern —
NOT a general plugin framework.** One data point shouldn't shape an abstraction; generalize
only when a second external-tool consumer (imtools, brief C-2) proves the shape.

Phased:
1. **Phase 015a (interim, zero-risk): "Open in TagForge"** — detect a configured caption
   checkout (configured path + `uv` present), spawn `uv run app.py` (via `direnv exec` or
   configured interpreter), open localhost:7860. rust-feh only launches; no image data flows
   through rust-feh. Surfaces existing `.txt` sidecars in the browser (read-only list/filter).
2. **Phase 015b (the real integration), gated on caption growing a LOCAL batch mode**:
   caption already has local backends in `app.py`; extract them into e.g.
   `batch_caption_local.py` (upstream work in the caption repo, not rust-feh). Then rust-feh
   dispatches "caption these" with progress (parse stdout), local-only by default.
3. **Remote path (current `batch_caption.py`) only as explicit opt-in**: per-run consent UI
   naming the destination Space, HF_TOKEN sourced from the child environment only — never
   stored in rust-feh config.

## Blockers to resolve before/during specify
- **caption has NO LICENSE file** (repo metadata `license: null`). Add one before rust-feh
  documents it as a supported tool.
- Decide 015a-only vs 015a+015b in one feature or two.
- Upstream work lands in the caption repo (local batch CLI) — sequencing dependency outside
  this repo.

## Predicted failure modes (likelihood × damage, high→low)
1. **Privacy/egress surprise** (high × high): user captions a private dataset, images go to
   a public Space. Mitigation: local-first design above; explicit consent for remote.
2. **Argument/path injection** (med × high): filenames with spaces/`;`/unicode passed to a
   spawned shell. Mitigation: `std::process::Command` arg-vector only (never shell string);
   canonicalize + validate paths; `/security-review` gate per GUARDRAILS.
3. **Environment coupling** (high × med): uv/venv/direnv/CUDA drift breaks spawn in ways
   rust-feh can't diagnose. Mitigation: capability probe = run `uv run batch_caption.py
   --help` (or a `--probe` flag upstream) at detection time; degrade gracefully per
   constitution IV; never bundle Python state.
4. **Long-run UX** (high × med): 7B model on 500 images = tens of minutes. Mitigation:
   non-blocking spawn + progress from stdout lines; cancel = kill process group; UI stays
   responsive (constitution V).
5. **HF_TOKEN leakage** (low × high): token must never enter rust-feh config/logs/debug
   panel. Mitigation: inherit from child env only; redact subprocess command lines in logs.
6. **Maintenance burden** (med × med): caption is a moving, unreleased repo. Mitigation:
   pin integration to a caption git tag once one exists (ask maintainer to tag).

## Constitution check
- I Thin-Wrapper: ✔ subprocess delegation, same as feh/magick.
- II Minimal deps: ✔ no new Rust deps needed (reuse `which`, `std::process`).
- III Module separation: ✔ detection in `tool_caps.rs`, spawn/progress logic in a testable
  non-GUI module; keep out of `main.rs`.
- IV Linux-first/graceful degradation: ✔ absent tool → clear status, buttons disabled.
- No-network positioning: ⚠ only if remote path ships — hence local-first + explicit opt-in.

## Open questions for the maintainer
1. Accept the 015a → 015b phasing (and the upstream local-CLI work in caption)?
2. Is the remote HF-Space path wanted at all, even behind consent?
3. Caption sidecar follow-ups (view/edit/filter-by-caption) in 015 or a later feature?
4. Will you add a LICENSE + a version tag to caption?
