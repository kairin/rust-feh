#!/usr/bin/env bash
# Manual quickstart hints for specs/009-external-tool-runtime/quickstart.md
# Does NOT run the GUI — prints copy-paste steps for V1–V4.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== 009 External Tool Runtime — Manual Quickstart Hints ==="
echo "Binary: ${ROOT}/rust-feh"
echo "feh:    $(command -v feh || echo MISSING)"
echo "magick: $(command -v magick || echo MISSING)"
echo "convert:$(command -v convert || echo MISSING)"
echo ""

echo "--- V1: Install feh mid-session (panel recheck) ---"
echo "1. Launch WITHOUT feh on PATH:"
echo "   env PATH=\"/bin:/usr/bin\" bash -c 'PATH=\$(echo \"\$PATH\" | tr : \"\\n\" | grep -v \"$(dirname "$(command -v feh 2>/dev/null || echo /usr/bin/feh)")\" | paste -sd: -)\"; export PATH; ./rust-feh'"
echo "   Simpler: env PATH=\"/bin\" DISPLAY=\$DISPLAY ./rust-feh"
echo "2. Verify: feh buttons greyed; panel shows feh ✗"
echo "3. Install or restore PATH, then click Recheck tools on PATH (right panel)"
echo "4. Verify: feh ✓; Open in feh works on a selected image"
echo ""

echo "--- V2: Tools menu recheck ---"
echo "1. Same feh-absent launch as V1"
echo "2. Tools → Recheck tools on PATH (not panel button)"
echo "3. Verify: debug log 'Rechecked tools: feh=true, magick=…'"
echo ""

echo "--- V3: Spawn failure recovery ---"
echo "1. Start normally (feh on PATH)"
echo "2. sudo mv \$(which feh) /tmp/feh.bak"
echo "3. Click Open in feh → buttons disable; panel feh ✗; install status message"
echo "4. sudo mv /tmp/feh.bak \$(dirname \$(which feh 2>/dev/null || echo /usr/bin))/feh"
echo "5. Recheck tools on PATH"
echo ""

echo "--- V4: ImageMagick mid-session ---"
echo "1. Launch without magick:"
echo "   env PATH=\"/bin:\$(dirname \"\$(command -v feh)\")\" DISPLAY=\$DISPLAY ./rust-feh"
echo "2. Panel heic note should mention reduced capability"
echo "3. sudo apt install imagemagick  # or restore PATH"
echo "4. Recheck tools on PATH → ImageMagick ✓; heic note updates"
echo "5. Optional Rescan to refresh inventory counts"
echo ""

echo "--- Automated (already verified in implement) ---"
echo "   cargo test tool_caps && cargo clippy -- -D warnings"
echo ""
echo "Full doc: specs/009-external-tool-runtime/quickstart.md"