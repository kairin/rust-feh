# UX & Performance Requirements Quality Checklist: Persistent UI Layout & Virtual Browsing

**Purpose**: Validate the quality, clarity, completeness, and consistency of UX and performance requirements before implementation
**Created**: 2026-06-21
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are the exact control elements required in the persistent top region explicitly enumerated, or does "folder controls (...) search/filter input, recursive checkbox, and rescan button" leave room for omission? [Completeness, Spec §FR-001]
- [ ] CHK002 - Are the exact primary action buttons required in the persistent region explicitly enumerated? FR-002 lists three but US1 mentions only "Open in feh" and "Set as wallpaper" while the existing code also has "Quick resize" — is the spec's list exhaustive? [Completeness, Spec §FR-002]
- [ ] CHK003 - Are requirements defined for what happens to the persistent controls when no folder is loaded? FR-001 through FR-003 assume `current_dir.is_some()` — is the empty-state layout specified? [Completeness, Gap]
- [ ] CHK004 - Are requirements specified for what the debug log viewer displays when there are zero log entries? FR-009 covers placement but not content of the empty state. [Completeness, Spec §FR-009]

## Requirement Clarity

- [ ] CHK005 - Is "persistent non-scrolling region" defined precisely enough to determine whether a collapsible panel qualifies, or must the controls be always-expanded? [Clarity, Spec §FR-001]
- [ ] CHK006 - Is "render only the items currently visible in the scrollable viewport" unambiguous about how many buffer items above/below the viewport are acceptable? [Clarity, Spec §FR-004]
- [ ] CHK007 - Is "the UI MUST remain responsive (controls accessible)" during a sync scan sufficiently defined? The spec acknowledges sync scanning blocks in Edge Cases but FR-010 mandates responsiveness — is this contradiction resolved? [Clarity/Conflict, Spec §FR-010 vs Edge Cases ¶2]
- [ ] CHK008 - Is the "Showing X / Y images" counter format specified for the edge case where no images match the filter but images exist (X=0, Y=N)? The spec mentions this in Edge Cases but FR-006 only gives the populated example. [Clarity, Spec §FR-006]

## Requirement Consistency

- [ ] CHK009 - Do the persistent-control requirements (FR-001, FR-002, FR-003) consistently agree with the "debug log must not compete for persistent screen space" constraint (FR-009)? If the debug log is in a collapsible CentralPanel element, is that consistently "non-competing"? [Consistency, Spec §FR-001/FR-009]
- [ ] CHK010 - Are the selection-model requirements (FR-007: "first image MUST be selected") consistent with the edge case of loading a directory with zero images, where no selection is possible? [Consistency, Spec §FR-007 vs Edge Cases ¶1]

## Acceptance Criteria Quality

- [ ] CHK011 - Can "the user can access and interact with the filter/search box and action buttons without scrolling" be objectively measured when the buttons are technically reachable but the user must know they exist? Does "access" imply discoverability or just physical reachability? [Measurability, Spec §SC-001]
- [ ] CHK012 - Is "no visible stutter, frame drops below 30fps are imperceptible in normal use" sufficiently objective? The "imperceptible" qualifier introduces subjectivity — is a 30fps threshold the actual acceptance criterion? [Measurability, Spec §SC-002]
- [ ] CHK013 - Is the 200ms filter response time (SC-003) measured from the last keystroke or from the first? The spec says "within 200ms of the last keystroke" but this implies debounce behavior that isn't specified elsewhere. [Measurability, Spec §SC-003]
- [ ] CHK014 - Can "no external feh window appears until the user explicitly clicks Open in feh" be verified when feh might have been launched by a prior session or another process? Is the verification scoped to the application's own behavior? [Measurability, Spec §SC-005]

## Scenario Coverage

- [ ] CHK015 - Are requirements defined for what the user sees during a folder scan (loading state)? The spec acknowledges sync scanning blocks briefly but no visual feedback requirement exists for the scanning period. [Coverage, Gap]
- [ ] CHK016 - Are requirements specified for what happens when the window is resized while a large collection is loaded? The persistent panels and virtualized list should adapt, but no resize behavior is specified. [Coverage, Gap]
- [ ] CHK017 - Are requirements defined for the transition when a user changes folders while a previous folder's images are still displayed? Should the list clear immediately, show a loading state, or keep stale data? [Coverage, Gap]
- [ ] CHK018 - Are requirements specified for the case where `feh` is not installed? The Assumptions section states feh is "assumed installed" but the Constitution mandates graceful degradation — does the spec need a requirement for this? [Coverage, Spec §Assumptions vs Constitution §IV]

## Edge Case Coverage

- [ ] CHK019 - Are edge cases defined for filenames containing characters that affect substring matching (regex metacharacters, Unicode normalization, leading/trailing whitespace)? The spec says "case-insensitive substring match" but doesn't address encoding or special characters. [Edge Case, Spec §FR-005]
- [ ] CHK020 - Is the behavior specified when a loaded image is deleted from disk between scans? The scan results are a snapshot, but the spec doesn't address stale entries in the image list. [Edge Case, Gap]

## Deferral Boundary Quality

- [ ] CHK021 - Are the deferred Areas (4-8) explicitly listed with clear triggers for when each becomes in-scope? The Assumptions section defers specific items but the roadmap from ISSUES_AND_CORE_AREAS.md is not referenced in the spec. [Completeness, Spec §Assumptions]
- [ ] CHK022 - Does the spec avoid leaking deferred requirements into Phase 1 scope? E.g., US2 mentions "this is a blocker for thumbnails (Area 4)" — is this forward reference clear that thumbnails are out of scope, or does it create ambiguity about whether partial thumbnail work is expected? [Clarity, Spec §US2 Why This Priority]

## Notes

- **2026-06-21 clarify (adversarial)**: CHK007 resolved — FR-010 requires visible "Scanning…" indicator and panel visibility; brief render stall acceptable. Async deferred to Area 6.
- The spec's FR-010 ("UI MUST remain responsive during scan") directly conflicts with the Edge Cases acknowledgment that sync scanning "may briefly block." This is the highest-impact ambiguity — CHK007 flags it, and resolution may require either relaxing FR-010 or adding a "loading state" requirement.
- Several items (CHK015, CHK016, CHK017) identify missing scenario coverage that was likely deferred to Area 8 (Polish) but could cause implementation confusion if the implementer expects complete coverage in the current spec.
- CHK013 (filter timing measurement) is subtle but important — if implementers measure from first keystroke instead of last, the spec and implementation will disagree.
