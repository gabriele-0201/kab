#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(alloc_error_handler)]
// TODO: remove this feature, used only once for stupid thing
#![feature(pointer_byte_offsets)]

//core::arch::global_asm!(core::include_str!("start.s"), options(raw));
//core::arch::global_asm!(include_str!("interrupt_handlers.s"), options(raw));

// src/main.rs

mod concurrency;
mod gdt;
mod init;
mod interrupts;
mod memory_manager;
mod multiboot;
mod port;
mod runtime_static;
mod vga_buffer;

#[macro_use]
extern crate alloc;

use concurrency::spin_mutex::SpinMutex;
use core::panic::PanicInfo;
use memory_manager::heap_allocator::HeapAllocator;
use runtime_static::RuntimeStatic;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[global_allocator]
static GLOBAL_ALLOC: RuntimeStatic<SpinMutex<HeapAllocator>> = RuntimeStatic::get_uninit();

#[alloc_error_handler]
fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    let _alloc = GLOBAL_ALLOC.lock();

    panic!(
        "Allocator failed to allocate: size: {}, align: {}",
        layout.size(),
        layout.align()
    );
}

#[no_mangle]
pub extern "C" fn kernel_main(
    multiboot_magic_number: usize,
    multiboot_information_address: usize,
    heap_kernel_bottom: usize,
    heap_kernel_top: usize,
    stack_kernel_top: usize,
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

    let boot_info =
        multiboot::BootInfo::new(multiboot_magic_number, multiboot_information_address).unwrap();

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

    GLOBAL_ALLOC.init(SpinMutex::new(HeapAllocator::new(
        heap_kernel_bottom,
        heap_kernel_top,
    )));
    println!("Initialized Heap Allocator!");

    //memory_manager::heap_allocator::tests::home_made_test();

    println!("");

    // Init memory manager (enable paging)
    let mut memory_manager = memory_manager::MemoryManager::new(heap_kernel_top, &boot_info);

    println!("Paging Enabled!");
    println!("");
    println!("Testing Paging switching vga_buffer pointer!");

    use memory_manager::paging::{PageDirectoryFlag, PageTableFlag, PhysicalAddr, VirtualAddr};
    let virtual_vga_buffer = VirtualAddr::new(0x40000000);
    let physical_vga_buffer = PhysicalAddr::new(0xb8000);

    memory_manager
        .map_addr_without_paging(
            virtual_vga_buffer.clone(),
            physical_vga_buffer,
            PageDirectoryFlag::Present as u32
                | PageDirectoryFlag::Writable as u32
                | PageDirectoryFlag::NotCacheable as u32,
            PageTableFlag::Present as u32
                | PageTableFlag::Writable as u32
                | PageTableFlag::NotCacheable as u32,
        )
        .expect("Impossible address mapping");

    // There is a mapping from 0x40000000 to 0xb8000 inside the memory_manager constructor
    let vga_virtual = VirtualAddr::new(0x40000000);
    let vga_physical = PhysicalAddr::new(0xb8000);

    let switch_vga_buffer = |addr: usize| unsafe {
        vga_buffer::WRITER.lock().change_ptr_buffer(addr);
    };
    switch_vga_buffer(vga_virtual.get());
    println!("Switched from 0xB8000 to 0x40000000!");
    switch_vga_buffer(vga_physical.get());
    println!("Returned to original pointer");

    loop {}
}
