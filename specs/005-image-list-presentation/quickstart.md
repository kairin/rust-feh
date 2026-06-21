# Quickstart: Image List Presentation (005)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Prerequisites

- Built `./rust-feh`
- Test folder with mixed content (see fixture below)
- Optional: `sudo apt install imagemagick` for magick-detect tests

## Fixture layout

```text
/tmp/rust-feh-005-fixture/
├── readme.txt
├── photo.jpg
├── sub/
│   ├── a.png
│   └── sample.heic    # if imagemagick installed
└── sub/deep/
    └── b.webp
```

## V1: Flat list + folder column (US1–US3)

1. Launch `./rust-feh`, choose `/tmp/rust-feh-005-fixture`
2. **Verify**: Folder + Filename columns; `sub` and `sub/deep` distinguish paths
3. Sort **Name** → alphabetical by filename
4. Sort **Folder** → grouped by folder
5. Filter `deep` → only `b.webp` row

**Pass**: SC-001, FR-001–FR-003

## V2: Inventory summary (US5)

1. Rescan fixture with ImageMagick installed
2. **Verify** inventory bar shows:
   - native_listed ≥ 3 (jpg, png, webp)
   - non_image_skipped ≥ 1 (readme.txt)
   - magick_detected ≥ 1 if heic present
3. Run Quick resize on `photo.jpg` → creates `photo_processed.jpg`
4. **Verify** (no rescan required): inventory `converted` increased; source row status `converted`
5. Optional **Rescan** — picks up `photo_processed.jpg` as its own row if present

**Pass**: SC-006

## V3: Folder tree (US4)

1. Toggle **Folder tree**
2. Expand `sub/` → see `a.png`, heic if present
3. Expand `sub/deep/` → see `b.webp`
4. Toggle back to **Flat list** → columns restored, selection preserved

**Pass**: SC-005

## V4: Performance smoke

```fish
./scripts/validate-feature-001.sh
```

Filter 10k test must still pass (SC-003/SC-007).

## Automated

```fish
cargo test feature_005
cargo test -p rust-feh ui_logic
```