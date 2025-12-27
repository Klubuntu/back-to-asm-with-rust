#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::{asm,naked_asm};

mod shims;
#[macro_use]
mod macros;
mod kernel_main;


#[unsafe(naked)]
#[unsafe(link_section = ".text._start")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            "mov rsp, 0x90000",
            "sub rsp, 8",       // "Sztuczne" wyrównanie (padding)
            "jmp kernel_main", // Skaczemy do właściwego Rusta
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_write!(0, 0, b'E', 0x4F);
    vga_write!(1, 0, b'R', 0x4F);
    vga_write!(2, 0, b'2', 0x4F);
    vga_write!(3, 0, b'5', 0x4F);
    loop { unsafe { asm!("hlt"); } }
}