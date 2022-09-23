#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(pointer_byte_offsets)]

            
//core::arch::global_asm!(core::include_str!("start.s"), options(raw));
//core::arch::global_asm!(include_str!("interrupt_handlers.s"), options(raw));

// src/main.rs

use core::panic::PanicInfo;

mod vga_buffer;
mod init;
mod gdt;
mod port;
mod interrupts;
mod multiboot;
mod memory_manager;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

//static HELLO: &[u8] = b"CIAOOOOOOO";

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_magic_number: usize, multiboot_information_address: usize, stack_kernel_top: usize) -> ! {
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

    let boot_info = multiboot::BootInfo::new(multiboot_magic_number, multiboot_information_address).unwrap();

    /* IDK - for now simply use the stack_top as starting poitn
    println!("{:?}", boot_info);

    for mmap_area in boot_info.mmap.unwrap() {
        if mmap_area.type_mmap != 1 { continue; }
        println!("base: {:X}, length: 0x{:X}", mmap_area.base, mmap_area.length);
    }

    println!("Address multiboot: 0x{:X}", multiboot_information_address);
    println!("Address mmap_addr: {:?}", boot_info.mmap);
    println!("Address TOP stack: 0x{:X}", stack_position);
    */

    let mut memory_manager = memory_manager::MemoryManager::new(stack_kernel_top, &boot_info);

    println!("Paging Enabled!");

    /* this seems to be ok
    println!("page directory[0]: {}", memory_manager.page_directory[0]);
    for i in 0..3 {
        println!("AFTER PTE[{}]: {}", i,  memory_manager.page_directory[0].get_page_table()[i]);
    }
    */

    // TEST paging
    use memory_manager::{ paging::{ PhysicalAddr, VirtualAddr }, frame_allocator::Frame};
    
    // TODO remove all the pub
    use vga_buffer::{ ScreenChar, ColorCode, Color };
    // test mapping 1GB to the frame that contain the vga buffer


    /* THIS cannot work done after the page is enable....
    let virtual_vga_buffer = VirtualAddr::new(0x40000000);
    let physical_vga_buffer = PhysicalAddr::new(0x8B000);

    memory_manager.map_addr(virtual_vga_buffer.clone(), physical_vga_buffer).expect("Impossible address mapping");

    let pde_index = virtual_vga_buffer.get_pd_index();
    println!("page directory[{}]: {}", pde_index,  memory_manager.page_directory[pde_index]);

    let pte_index = virtual_vga_buffer.get_pt_index();
    println!("page table[{}]: {}", pte_index, memory_manager.page_directory[pde_index].get_page_table()[pte_index]);
    */

    /*
    unsafe  {
        use volatile::Volatile;

        let buffer: &mut [[Volatile<ScreenChar>; 80]; 25] = &mut *(0x40000000 as *mut [[Volatile<ScreenChar>; 80]; 25]);

        for i in 0..80 {
            buffer[0][i].write(ScreenChar {
                ascii_character: b'c',
                color_code: ColorCode::new(Color::Blue, Color::Black)
            });
        }
        //println!("first_char = {:?}", first_char);
    }
    */

    loop {}
}
