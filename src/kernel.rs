#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(pointer_byte_offsets)]

            
//core::arch::global_asm!(core::include_str!("start.s"), options(raw));
//core::arch::global_asm!(include_str!("interrupt_handlers.s"), options(raw));

// src/main.rs


mod vga_buffer;
mod init;
mod gdt;
mod port;
mod interrupts;
mod multiboot;
mod memory_manager;
mod concurrency;
mod runtime_static;

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main(
    multiboot_magic_number: usize, 
    multiboot_information_address: usize, 
    heap_kernel_bottom: usize,
    heap_kernel_top: usize,
    stack_kernel_top: usize
) -> ! {

    // vga_buffer::print_something();
    
    // TODO manage better the lock, could couse dead lock
    //  + example: have something is blocking the WRTIER and a interrupt manager is called
    //  => this cause a dead lock because the interrupt will never find free the WRITER and the
    //  => previous will never finish using it
    
    //vga_buffer::WRITER.lock().clear_screen();
    vga_buffer::Writer::init();

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

    // Init heap
    println!("");
    println!("Heap_Base: 0x{:X}", heap_kernel_bottom);
    println!("Heap_Limit: 0x{:X}", heap_kernel_top);
    let heap_allocator = memory_manager::heap_allocator::HeapAllocator::new(heap_kernel_bottom, heap_kernel_top);

    println!("");

    // Init memory manager (enable paging)
    let memory_manager = memory_manager::MemoryManager::new(stack_kernel_top, &boot_info);

    println!("Paging Enabled!");
    println!("");
    println!("Testing Paging switching vga_buffer pointer!");

    // There is a mapping from 0x40000000 to 0xb8000 inside the memory_manager constructor
    let vga_virtual = memory_manager::paging::VirtualAddr::new(0x40000000);
    let vga_physical = memory_manager::paging::PhysicalAddr::new(0xb8000);

    let switch_vga_buffer = |addr: usize| { unsafe { vga_buffer::WRITER.lock().change_ptr_buffer(addr); } };
    switch_vga_buffer(vga_virtual.get());
    println!("Switched from 0xB8000 to 0x40000000!");
    switch_vga_buffer(vga_physical.get());
    println!("Returned to original pointer");

    loop {}
}
