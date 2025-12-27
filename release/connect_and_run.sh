cat boot.bin kernel.bin > os.img
ls -lh os.img
qemu-system-x86_64 \
  -machine accel=tcg \
  -drive format=raw,file=os.img \
  -no-shutdown