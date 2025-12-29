#![no_std]
#![allow(unused_unsafe)]

use core::panic::PanicInfo;
use core::arch::asm;

#[macro_use]
pub mod macros;
pub mod vga;
pub mod shims;
pub mod kernel_main;
pub mod fs;

pub use crate::kernel_main::kernel_main;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        vga_write!(0, 0, b'E', 0x4F);
        vga_write!(1, 0, b'R', 0x4F);
        vga_write!(2, 0, b'2', 0x4F);
        vga_write!(3, 0, b'5', 0x4F);
        loop { asm!("hlt"); }
    }
}
