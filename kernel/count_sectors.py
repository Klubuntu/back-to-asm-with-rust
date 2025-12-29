import os
size = os.stat('target/target/debug/kernel').st_size
sectors = (size + 511) // 512
print(f"size={size} bytes, sectors={sectors}")
