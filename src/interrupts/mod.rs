use super::vga_buffer::println;
use super::gdt::GDT;
use super::port::Port8Bit;

pub mod idt;
// not give access to interrupt_manager outside of this module
mod interrupt_manager;
