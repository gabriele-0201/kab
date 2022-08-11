// src/main.rs

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use core::arch::global_asm;

global_asm!(include_str!("start.s"), options(raw));
global_asm!(include_str!("interrupt_handlers.s"), options(raw));

mod vga_buffer;
mod init;
mod gdt;
mod port;
mod interrupts;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

//static HELLO: &[u8] = b"CIAOOOOOOO";

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    /* OLD
    let vga_buffer = 0xb8000 as *mut u8;

    for y in 0..25 {
        for x in 0..80 {
            unsafe {
                *vga_buffer.offset(y as isize * 80 + x + 1) = 0x00;
            }
        }
    }

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xF;
        }
    }
    */

    // vga_buffer::print_something();
    

    vga_buffer::WRITER.lock().clear_screen();

    println!("Vga Buffer Ready!");
    
    // All of the following code should finish in some init wrapper
    let gdt = gdt::GDT::new(); 
    gdt.load();

    println!("GDT loaded!");

    //let idt = interrupts::IDT::new(0x20, &gdt);
    //interrupts::IDT::new(0x20, &gdt);
    //idt.load();

    //TEST
    let mut idt_struct = interrupts::IDT {
        idt: [interrupts::GateDescritor::new(interrupts::interruptIgnore, gdt.get_kernel_code_segment_offset(), 0, 0xE); 256],
        hw_interrupt_offset: 0x20,
        pic_master_command: port::Port8Bit::new(0x20),
        pic_master_data: port::Port8Bit::new(0x21),
        pic_slave_command: port::Port8Bit::new(0xA0),
        pic_slave_data: port::Port8Bit::new(0xA1)
    };

    println!("After the IDT construct");

    println!("{}: {:?}", 0, idt_struct.idt[0]);
    println!("somethig after the print of the first entry");
    //println!("len idt: {}", idt_struct.idt.len());
    //println!("{}: {:?}", 1, idt_struct.idt[1]);

    println!("IDT loaded!");

    //println!("Activation interrupts!");
    //interrupts::activate();

    loop {}
}
