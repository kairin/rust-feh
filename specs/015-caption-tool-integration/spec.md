# Feature Specification: Caption Tool Integration (TagForge)

**Feature Branch**: `015-caption-tool-integration`

**Created**: 2026-07-05

**Status**: Draft

**Input**: User description: "Integrate the caption (TagForge) LoRA-dataset captioning toolkit as an external tool: from the rust-feh browser, discover a configured caption installation, launch its UI, and surface per-image caption sidecars. Direction and verified constraints per `.agents/briefs/caption-plugin.md` (2026-07-05): minimal subprocess integration via the existing external-tool capability pattern, local-first; the tool's current batch CLI performs REMOTE inference on a public hosting space, which conflicts with rust-feh's no-network positioning."

## Clarifications

### Session 2026-07-05

- Q: Is batch-captioning dispatch (progress + cancel) in scope for 015? → A: **Deferred** — 015 ships detection, TagForge launch, and read-only sidecar surfacing only. Dispatch becomes its own follow-up feature once the caption tool grows a local batch mode upstream. (Maintainer decision.)
- Q: Is the remote captioning path (public hosting space) permitted behind per-run consent? → A: **Excluded entirely** — rust-feh never invokes the remote path; captioning via rust-feh is local-only. Users wanting the remote CLI run it themselves outside rust-feh. (Maintainer decision.)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch TagForge from the browser (Priority: P1)

While browsing an image collection in rust-feh, I want to open my captioning
toolkit (TagForge) on demand — the same way rust-feh already delegates viewing
to feh — so that rust-feh is the single hub I start all image work from.

**Why this priority**: It is the minimum useful integration, requires no data
to flow through rust-feh, works with the caption tool exactly as it exists
today, and establishes detection/degradation plumbing every later story reuses.

**Independent Test**: With a caption installation configured, one action in
rust-feh opens the TagForge UI; with none configured, the control communicates
unavailability and how to set it up. Fully testable without any other story.

**Acceptance Scenarios**:

1. **Given** a configured caption installation, **When** the user invokes "Open
   TagForge", **Then** the TagForge UI starts and becomes reachable, and the
   activity log records the launch.
2. **Given** no caption installation configured (or its runtime missing),
   **When** the user views captioning controls, **Then** they are presented as
   unavailable with a clear setup hint, and invoking them never crashes the app.
3. **Given** TagForge was launched from rust-feh, **When** the user closes
   rust-feh, **Then** the TagForge session is not silently killed (parity with
   how launched feh viewers outlive rust-feh; recorded in Assumptions).

---

### User Story 2 - See which images already have captions (Priority: P2)

While browsing a dataset folder, I want to see at a glance which images
already have a caption sidecar (`<image-stem>.txt`) and read the caption of
the selected image, so I can tell how much captioning work remains without
opening another tool.

**Why this priority**: Caption sidecars are the tool-agnostic contract of the
captioning workflow (kohya-style, one text file per image). Surfacing them is
pure read-only value on top of data that already exists on disk.

**Independent Test**: Point rust-feh at a folder containing images with and
without `.txt` sidecars; the list distinguishes captioned from uncaptioned
images, and selecting a captioned image shows its caption text read-only.

**Acceptance Scenarios**:

1. **Given** a folder where some images have `<stem>.txt` sidecars, **When**
   the scan completes, **Then** each image row indicates caption presence, and
   the scan inventory reports the captioned/uncaptioned counts.
2. **Given** a captioned image is selected, **When** the user views the
   inspector, **Then** the caption text is shown read-only.
3. **Given** a 10,000-image folder, **When** captions are surfaced, **Then**
   browsing performance is not perceptibly degraded (see SC-004).

---

### Edge Cases

- Image filenames containing spaces, quotes, shell metacharacters, or non-ASCII
  text must launch/caption correctly — never mangled or misinterpreted.
- The configured caption installation exists but its runtime is broken
  (missing interpreter, missing dependencies): controls degrade with a
  diagnostic hint rather than a cryptic failure.
- A sidecar exists but is empty or unreadable: treated as "uncaptioned" with a
  distinct indication, never a crash.
- The caption tool's UI port is already in use by another process: the launch
  failure is reported with the reason, not a silent hang.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST detect a user-configured caption (TagForge)
  installation and report its availability in the existing tool-capabilities
  surface, alongside feh and ImageMagick.
- **FR-002**: Users MUST be able to launch the TagForge UI from rust-feh in a
  single action when the tool is available.
- **FR-003**: When the caption tool is unavailable, captioning controls MUST
  degrade gracefully: visibly unavailable, accompanied by a setup hint, and
  crash-free when invoked (parity with the feh-missing behavior).
- **FR-004**: The system MUST surface caption sidecar presence per image after
  a scan and MUST show the selected image's caption text read-only.
- **FR-005**: Caption surfacing MUST NOT read image file contents or caption
  file contents during listing; caption text is loaded lazily on selection
  (performance parity with existing metadata handling).
- **FR-006**: All interactions with the caption tool MUST handle arbitrary
  valid filenames (spaces, quotes, non-ASCII) without misinterpretation.
- **FR-007**: No rust-feh action may cause image data to leave the machine.
  The caption tool's remote captioning path is EXCLUDED from rust-feh (per
  Clarifications 2026-07-05); rust-feh only launches the tool's local UI.
- **FR-008**: The system MUST NOT store, display, or log captioning-service
  credentials (they belong to the tool's own environment).
- **FR-009**: Every launch and launch failure MUST be recorded in the existing
  activity log with enough detail to diagnose what was attempted.

### Key Entities

- **Caption sidecar**: a per-image text file (`<image-stem>.txt`) next to the
  image (or in a chosen output folder) holding comma-separated caption tags;
  the interchange contract between rust-feh, TagForge, and downstream training
  tools.
- **External captioning tool**: a user-installed TagForge instance (location,
  availability state, diagnostic hint); one entry in the existing external-tool
  capability model.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With a configured caption tool, a user goes from browsing images
  to an open TagForge UI in at most 2 interactions.
- **SC-002**: With the tool absent or broken, 100% of captioning entry points
  communicate unavailability plus a setup hint, and produce zero crashes.
- **SC-003**: In a mixed folder, caption presence indication matches the actual
  sidecar state for 100% of images (verified against a prepared fixture).
- **SC-004**: With 10,000 images, enabling caption surfacing changes scan and
  filter timings by no more than 10% relative to the pre-feature baseline.
- **SC-005**: Filenames with spaces, quotes, and non-ASCII characters work in
  100% of launch/caption operations (prepared adversarial-name fixture).
- **SC-006**: Zero image bytes leave the machine via any rust-feh action,
  verifiable by exercising every captioning surface with networking observed.

## Assumptions

- The caption tool is installed and managed by the user (its own repository,
  runtime, and model weights); rust-feh never bundles or installs it.
- Tools launched from rust-feh outlive it (parity with feh delegation): closing
  rust-feh never silently kills a TagForge session.
- The sidecar naming contract is `<image-stem>.txt` adjacent to the image
  (kohya-style), with an optional distinct output folder for batch runs.
- rust-feh itself remains no-network; anything the caption tool does over the
  network happens in its own process under the user's configuration, subject
  to FR-007's consent rule.
- Editing captions and filtering by caption content are OUT of scope for 015
  (candidate follow-up feature) unless the maintainer says otherwise.
- Batch-captioning dispatch is DEFERRED to a follow-up feature, gated on the
  caption tool gaining a local batch mode upstream (Clarifications 2026-07-05);
  its earlier draft requirements (progress, cancel, no-rescan sidecar refresh,
  single-writer job discipline) are preserved in `.agents/briefs/caption-plugin.md`.
- The caption tool currently has no license file and no tagged release; before
  rust-feh's documentation lists it as a supported tool, the maintainer adds a
  license and a version tag to that repository (tracked in
  `.agents/briefs/caption-plugin.md`).
- A security review of the integration diff is mandatory before merge
  (engagement guardrail: subprocess argument construction and untrusted path
  handling get explicit attention).
