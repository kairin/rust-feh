# Remediation Plan: Decomposed Implementation Instructions

**Feature**: `001-persistent-ui-virtual-browsing`
**Created**: 2026-06-21 via `/speckit-clarify` (adversarial-review integration)
**Purpose**: Step-by-step instructions to close every `gap` and `partial` row in `gap-audit.md`
**Prerequisite**: Read `gap-audit.md`, `data-model.md`, `spec.md` Clarifications session 2026-06-21 (adversarial)

Execute steps in order. Each step lists exact files, code changes, verification, and task IDs.

---

## Phase A: Shared State Fields (blocks B–E)

### A1 — Add `feh_available` (FR-008a)

**File**: `src/main.rs`

1. Add field to `RustFehApp`:
   ```rust
   feh_available: bool,
   ```
2. In `run_native` closure init:
   ```rust
   feh_available: which::which("feh").is_ok(),
   ```
3. If `!feh_available` at startup, set:
   ```rust
   status: "feh not found — install with `sudo apt install feh`".to_string(),
   ```
4. **Verify**: `rg 'feh_available' src/main.rs` — ≥3 hits (field, init, usage)

**Tasks**: T010 | **Quickstart**: V8

---

### A2 — Add `scanning` (FR-010, FR-013)

**File**: `src/main.rs`

1. Add field: `scanning: bool,` default `false`
2. At start of `scan_directory` (before `images.clear()`):
   ```rust
   self.scanning = true;
   self.status = "Scanning…".to_string();
   ```
3. At end of `scan_directory` (after images assigned):
   ```rust
   self.scanning = false;
   ```
4. In bottom panel: when `scanning`, prefer showing "Scanning…" (or keep status string)
5. **Verify**: Toggle recursive → status flashes "Scanning…"

**Tasks**: T011, T031, T034 | **Quickstart**: V9

---

### A3 — Add scroll reset fields (FR-005)

**File**: `src/main.rs`

1. Add fields:
   ```rust
   list_scroll_offset: f32,
   prior_search: String,
   ```
2. In `update()`, before rendering list, after computing `filtered`:
   ```rust
   if self.search != self.prior_search {
       self.prior_search = self.search.clone();
       self.list_scroll_offset = 0.0;
   }
   ```
3. Apply offset to `ScrollArea` — use `ui.scroll_to_cursor` or store `ScrollArea` state via `egui::Id` and reset offset when search changes (egui 0.30: `ui.scroll_to_rect` with zero rect, or `ScrollArea::vertical().id_source("image_list").show_rows(...)` and reset stored offset)
4. **Verify**: V10 — scroll bottom, type filter, list jumps to top

**Tasks**: T012, T033, T032 | **Quickstart**: V10

---

## Phase B: `scan_directory` Lifecycle (FR-007, FR-012, FR-013)

### B1 — Centralize selection in `scan_directory`

**File**: `src/main.rs`

**Current problem**: Folder-pick handler (L113–121) auto-selects; `scan_directory` clears but never re-selects.

1. Rewrite `scan_directory`:
   ```rust
   fn scan_directory(&mut self, dir: &Path) {
       self.scanning = true;
       self.status = "Scanning…".to_string();
       self.selected = None;
       self.images.clear();

       let (entries, warnings) = scan_images(dir, self.recursive);
       self.images = entries;
       self.scanning = false;

       for w in warnings {
           self.log(w);
       }

       self.log(format!(
           "Scanned '{}' (recursive={}), found {} supported images",
           dir.display(), self.recursive, self.images.len()
       ));

       if self.images.is_empty() {
           self.status = "No images found".to_string();
           self.selected = None;
       } else {
           let p = self.images[0].path.clone();
           self.selected = Some(p.clone());
           self.status = format!(
               "Loaded {} images. First selected (click Open in feh to view).",
               self.images.len()
           );
           self.log(format!("Auto-selected first image: {}", p.display()));
       }
   }
   ```

2. **Remove** duplicate auto-select block from folder-pick handler (L113–121) — keep only:
   ```rust
   self.current_dir = Some(dir.clone());
   self.scan_directory(&dir);
   ```

3. **Verify**: Rescan + recursive toggle → first image re-selected (V9 ST-04, ST-05)

**Tasks**: T044, T049, T041, T042, T035 | **Quickstart**: V3, V9

---

## Phase C: Scanner Warnings (FR-015)

### C1 — Return warnings from scanner

**File**: `src/scanner.rs`

1. Change signature:
   ```rust
   pub fn scan_images(dir: &Path, recursive: bool) -> (Vec<ImageEntry>, Vec<String>)
   ```
2. Replace `.filter_map(Result::ok)` with iteration that:
   - On `Ok(entry)` — process as today
   - On `Err(e)` — if `io::ErrorKind::PermissionDenied`, push warning string; continue
3. Return `(entries, warnings)` sorted entries unchanged

**File**: `src/main.rs`

4. Destructure in `scan_directory` and `self.log(w)` each warning

**Verify**: Create unreadable subdir, scan parent — warning in debug log (ST-11)

**Tasks**: T013, T014, T055

---

## Phase D: Layout Fixes (FR-006, FR-001)

### D1 — Move counter to bottom bar

**File**: `src/main.rs`

1. Compute `total` and `shown` (filtered.len()) **once** at start of `update()`, before panels
2. **Delete** `ui.label(format!("Showing {} / {} images", shown, total));` from CentralPanel (~L241)
3. **Add** to bottom panel (before or after status):
   ```rust
   ui.label(format!("Showing {} / {} images", shown, total));
   ui.label(&self.status);
   ```
4. **Verify**: `rg -n 'Showing.*images' src/main.rs` — exactly 1 hit, inside bottom panel closure

**Tasks**: T021, T018, T027 | **Quickstart**: V1, V4

---

### D2 — Empty-state controls (FR-001 partial)

**File**: `src/main.rs`

1. Confirm filter/actions/recursive/rescan only render inside `if self.current_dir.is_some()` — already mostly true
2. Menu bar (Choose folder) stays always visible — already true
3. **Verify**: Launch with no folder — only Choose folder + menu visible (V1 step 1)

**Tasks**: T019

---

## Phase E: feh Degradation (FR-008a)

### E1 — Disable feh buttons

**File**: `src/main.rs`

1. Wrap feh buttons:
   ```rust
   ui.add_enabled(self.feh_available, egui::Button::new("Open in feh"))
   ```
   Or disable via `ui.disable()` when `!feh_available`
2. Same for "Set as wallpaper" and Tools→Open in feh menu item
3. On click when disabled, set status to feh-not-found message (no spawn)
4. Guard `open_in_feh` / `set_wallpaper`: early-return if `!feh_available`

**Tasks**: T045, T047, T048, T025 | **Quickstart**: V8

---

## Phase F: Menu Bar (FR-011)

### F1 — Extract `pick_folder()`

**File**: `src/main.rs`

```rust
fn pick_folder(&mut self) {
    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
        self.log(format!("User chose folder: {}", dir.display()));
        self.current_dir = Some(dir.clone());
        self.scan_directory(&dir);
    }
}
```

1. Toolbar "Choose folder" calls `self.pick_folder()`
2. File→"Choose folder..." calls `self.pick_folder()` — **delete stub comment**
3. File→"Rescan" — already calls `scan_directory` — OK
4. View→"Include subfolders":
   ```rust
   let changed = ui.checkbox(&mut self.recursive, "Include subfolders").changed();
   if changed {
       ui.close_menu();
       if let Some(d) = self.current_dir.clone() {
           self.scan_directory(&d);
       }
   }
   ```
5. Tools→"Open in feh" — add `feh_available` guard like toolbar

**Verify**: `rg 'for now note|trigger the logic' src/main.rs` — zero hits

**Tasks**: T020, T022, T023, T024, T025 | **Quickstart**: V7

---

## Phase G: Debug Log (FR-009 partial)

### G1 — Startup message outside debug_logs

**File**: `src/main.rs`

1. **Remove** auto-log block at L66–68 that pushes to `debug_logs` on empty
2. Replace with one-time `eprintln!` only, or set a `startup_notice_shown: bool` flag
3. Empty `debug_logs` → collapsing panel shows "(no debug messages yet)"

**Tasks**: T053, T058 | **Quickstart**: V6

---

## Phase H: Polish & Validation

### H1 — Dead code (optional this phase)

**File**: `src/types.rs` — remove or use `Selection`/`SortMode` (T056)

### H2 — Remove stale comments

**File**: `src/main.rs` — delete "next iteration for Area 3" comment at L205

**Tasks**: T057

### H3 — Final validation suite

```bash
cargo build --release
cargo clippy -- -D warnings
cargo test
./build-and-place.sh
```

Run quickstart V1–V10 manually. Update `gap-audit.md` rows to `pass`.

**Tasks**: T001–T003, T026–T040, T050–T052, T058–T064

---

## Execution Order DAG

```text
A1, A2, A3 (parallel)
    ↓
B1 (depends A2)
    ↓
C1 (parallel with B1 if careful — prefer B1 first for compile)
    ↓
D1, D2 (parallel)
    ↓
E1 (depends A1)
    ↓
F1 (depends B1, E1)
    ↓
G1 (parallel)
    ↓
H1–H3
```

---

## Per-FR Closure Checklist

| FR | Remediation steps | Verify command / scenario |
|----|-------------------|---------------------------|
| FR-001 | D2 | V1 |
| FR-002 | (pass) | T017 |
| FR-003 | D1 | V1 |
| FR-004 | (pass) | V2 |
| FR-005 | A3 | V10 |
| FR-006 | D1 | `rg 'Showing' src/main.rs` |
| FR-007 | B1 | V3 |
| FR-008 | (pass) | — |
| FR-008a | A1, E1 | V8 |
| FR-008b | (pass) | — |
| FR-009 | G1 | V6 |
| FR-010 | A2, B1 | V9 |
| FR-011 | F1 | V7 |
| FR-012 | B1 | V9 |
| FR-013 | A2, B1 | V9 |
| FR-014 | (pass) | — |
| FR-015 | C1 | ST-11 |

---

## Suggested Next Command

After completing remediation steps in code:

```
/speckit-implement
```

Reference: `remediation.md` + `gap-audit.md` for ordered execution.