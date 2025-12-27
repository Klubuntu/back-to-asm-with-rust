cargo +nightly build --target target.json -Z build-std=core
#llvm-objcopy -O binary --only-section=.text '/home/coder/Documents/Projects/back-to-asm-with-rust/kernel/target/target/debug/kernel' ../release/kernel.bin
llvm-objcopy -I elf64-x86-64 -O binary \
    --strip-all \
    --remove-section=.comment \
    './target/target/debug/kernel' ../release/kernel.bin