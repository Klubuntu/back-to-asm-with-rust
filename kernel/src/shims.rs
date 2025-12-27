#[unsafe(no_mangle)]
pub extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        unsafe {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b { return (a as i32) - (b as i32); }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { *d.add(i) = *s.add(i); } }
    d
}

#[unsafe(no_mangle)]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { *s.add(i) = c as u8; } }
    s
}