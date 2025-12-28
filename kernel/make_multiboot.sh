#!/bin/bash

# Build kernel with Multiboot support
cd "$(dirname "$0")"

echo "Building Multiboot kernel..."

# Create output directories
mkdir -p ../release/multiboot

# Compile Multiboot entry point
nasm -f elf64 multiboot.asm -o multiboot.o || exit 1

# Size-focused flags (no LTO to avoid embed-bitcode conflict)
export RUSTFLAGS="-C opt-level=z -C codegen-units=1 -C panic=abort -C strip=symbols -C link-arg=--gc-sections -C link-arg=-zseparate-code"

# Build Rust kernel as static library
cargo +nightly build -Zbuild-std=core,compiler_builtins -Zbuild-std-features=compiler-builtins-mem --release --target=target.json --lib || exit 1

# Find the static library
KERNEL_LIB=$(find target/target/release/ -name "libkernel.a" | head -1)

if [ -z "$KERNEL_LIB" ]; then
    echo "Error: libkernel.a not found!"
    echo "Contents of target/target/release/:"
    ls -la target/target/release/
    exit 1
fi

echo "Found kernel library: $KERNEL_LIB"

# Link everything together with Multiboot
ld -n -T link_multiboot.ld --gc-sections -o ../release/multiboot/kernel_multiboot.elf \
    multiboot.o \
    "$KERNEL_LIB" || exit 1

# Create binary
objcopy -O binary --strip-all --remove-section=.comment ../release/multiboot/kernel_multiboot.elf ../release/multiboot/kernel_multiboot.bin

# Copy to main release folder
cp ../release/multiboot/kernel_multiboot.elf ../release/kernel_multiboot.elf
cp ../release/multiboot/kernel_multiboot.bin ../release/kernel_multiboot.bin

echo "Multiboot kernel built!"
echo "  - ../release/kernel_multiboot.elf"
echo "  - ../release/kernel_multiboot.bin"
echo "  - ../release/multiboot/kernel_multiboot.elf"
echo "  - ../release/multiboot/kernel_multiboot.bin"
echo ""
echo "To test with GRUB:"
echo "1. Create ISO: ./make_iso.sh"
echo "2. Run: qemu-system-x86_64 -cdrom ../release/rusted.iso"
