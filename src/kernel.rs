// src/main.rs

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

mod vga_buffer;
mod init;
mod gdt;

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
    gdt.set();

    loop {}
}
