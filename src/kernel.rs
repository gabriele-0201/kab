#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

            
//core::arch::global_asm!(core::include_str!("start.s"), options(raw));
//core::arch::global_asm!(include_str!("interrupt_handlers.s"), options(raw));

// src/main.rs

use core::panic::PanicInfo;

mod vga_buffer;
mod init;
mod gdt;
mod port;
mod interrupts;
mod multiboot2;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

//static HELLO: &[u8] = b"CIAOOOOOOO";

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_magic_number: usize, multiboot_information_address: usize) -> ! {
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
    
    // TODO manage better the lock, could couse dead lock
    //  + example: have something is blocking the WRTIER and a interrupt manager is called
    //  => this cause a dead lock because the interrupt will never find free the WRITER and the
    //  => previous will never finish using it
    
    vga_buffer::WRITER.lock().clear_screen();

    println!("Vga Buffer Ready!");
    
    // All of the following code should finish in some init wrapper
    let gdt = gdt::GDT::new(); 
    gdt.load();

    println!("GDT loaded!");

    let idt = interrupts::idt::IDT::new(0x20, &gdt);
    idt.load();

    println!("IDT loaded!");

    println!("Activation interrupts!");
    idt.enable();


    let boot_info = multiboot2::BootInfo::new(multiboot_magic_number, multiboot_information_address).unwrap();
    println!("{:?}", boot_info);

    loop {}
}
