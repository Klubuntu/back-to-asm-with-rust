set -euo pipefail

# Ensure nightly toolchain and rust-src for build-std
if command -v rustup >/dev/null 2>&1; then
    rustup toolchain list | grep -q '^nightly' || rustup toolchain install nightly
    rustup component list --toolchain nightly --installed | grep -q '^rust-src' || rustup component add rust-src --toolchain nightly
fi

# Clean to avoid stale binaries
cargo +nightly clean || true

# Build kernel for custom target with core only
cargo +nightly build --release --target target.json -Z build-std=core

# Produce flat binary
llvm-objcopy -I elf64-x86-64 -O binary \
        --strip-all \
        --remove-section=.comment \
        './target/target/release/kernel' ../release/kernel.bin