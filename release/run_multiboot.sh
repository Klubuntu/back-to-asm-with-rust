#!/bin/bash

# Run Multiboot kernel directly with QEMU (without ISO)
cd "$(dirname "$0")"

if [ ! -f "kernel_multiboot.elf" ]; then
    echo "Error: kernel_multiboot.elf not found!"
    echo "Run: cd ../kernel && ./make_multiboot.sh"
    exit 1
fi

echo "Running Multiboot kernel with QEMU..."
echo "Press Ctrl+A then X to exit QEMU"
echo ""

qemu-system-x86_64 \
    -kernel kernel_multiboot.elf \
    -m 128M \
    -serial stdio \
    -display gtk
