#!/usr/bin/env fish
# Helper: build rust-feh and place a copy in the project root (for the plan's "output to root" step)
# Usage: ./build-and-place.sh

cargo build --release
or begin
    echo "Build failed. Make sure Rust is installed (rustup) and you're in the right dir."
    exit 1
end

cp target/release/rust-feh ./rust-feh
or begin
    echo "Copy failed"
    exit 1
end

echo "✅ Binary placed at ./rust-feh (ready for later spec-kit execution)"
ls -l ./rust-feh
