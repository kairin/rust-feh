## [2026-09-13] — GitHub delivery cleanup

### Changed

- Reviewed GitHub delivery configuration for this repository.
- Added minimal protection to the default branch. Direct commits and pushes remain allowed; pull requests, reviews, status checks, and agent approvals are not required.
- Preserved project-specific validation and intentional publishing workflows.

### Removed

- Removed the stale required ci status gate while retaining the actual CI workflow.
