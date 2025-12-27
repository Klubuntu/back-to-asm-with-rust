#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::{asm,global_asm};
use crate::vga::{Color, ColorCode};

mod shims;
#[macro_use]
mod macros;
mod vga;
mod kernel_main;


global_asm!(
    r#"
    .intel_syntax noprefix
    .section .text._start
    .global _start
    _start:
        lea rsp, [stack_top]
        call kernel_main      /* call jest bezpieczniejsze niż jmp dla wyrównania stosu */
        
    .section .bss
    .align 16
    stack_bottom:
        .zero 16384           /* 16 KB stosu */
    stack_top:
    "#
);

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