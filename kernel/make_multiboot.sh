#!/bin/bash

# Build kernel with Multiboot support
cd "$(dirname "$0")"

echo "Building Multiboot kernel..."

# Create output directories
mkdir -p ../release/multiboot

# Compile Multiboot entry point
nasm -f elf64 multiboot.asm -o multiboot.o || exit 1

# Build Rust kernel as static library
# Using release profile from Cargo.toml which has: opt-level=s, lto=thin, codegen-units=1
# Force the same optimizations via RUSTFLAGS to ensure they apply
export RUSTFLAGS="-C opt-level=s -C lto=thin -C codegen-units=1 -C panic=abort"
cargo +nightly build -Zbuild-std=core,compiler_builtins -Zbuild-std-features=compiler-builtins-mem --release --target=target.json --lib || exit 1

KERNEL_LIB=$(find target/target/release/ -name "libkernel.a" | head -1)

if [ -z "$KERNEL_LIB" ]; then
    echo "Error: libkernel.a not found!"
    echo "Looking for alternatives..."
    find target/target/release/ -name "libkernel*"
    exit 1
fi

echo "Found kernel library: $KERNEL_LIB"

# Link everything together with Multiboot
# multiboot.o has the multiboot header and entry point
# libkernel.a has all the kernel code
ld -n -T link_multiboot.ld \
    multiboot.o \
    "$KERNEL_LIB" \
    -o ../release/multiboot/kernel_multiboot.elf || exit 1

# Create binary from the ELF
objcopy -O binary --strip-all --remove-section=.comment ../release/multiboot/kernel_multiboot.elf ../release/multiboot/kernel_multiboot.bin

# Copy to main release folder
cp ../release/multiboot/kernel_multiboot.elf ../release/kernel_multiboot.elf
cp ../release/multiboot/kernel_multiboot.bin ../release/kernel_multiboot.bin

echo "Multiboot kernel built!"
echo "  - ../release/kernel_multiboot.elf"
echo "  - ../release/kernel_multiboot.bin"
echo ""
echo "To test with GRUB:"
echo "1. Create ISO: ./make_iso.sh"
echo "2. Run: qemu-system-x86_64 -cdrom ../release/rusted.iso"
echo ""
echo "Verify Multiboot2 header:"
echo "  grub-file --is-x86-multiboot2 ../release/kernel_multiboot.elf && echo OK || echo FAIL"
