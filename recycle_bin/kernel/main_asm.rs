#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        // Write directly to VGA buffer using assembly
        core::arch::asm!(
            "mov rax, 0xb8000",
            "mov word ptr [rax], 0x0F31",      // '1'
            "mov word ptr [rax + 2], 0x0F32",  // '2'  
            "mov word ptr [rax + 4], 0x0F33",  // '3'
            options(nostack)
        );
    }
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
