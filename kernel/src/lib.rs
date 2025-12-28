#![no_std]
#![allow(unused_unsafe)]

#[macro_use]
pub mod macros;
pub mod vga;
pub mod shims;
pub mod kernel_main;

pub use crate::kernel_main::kernel_main;
