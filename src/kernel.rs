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
use runtime_static::RuntimeStatic;
use concurrency::spin_mutex::SpinMutex;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// should I make it mutable?
// done like this ONLY for now
static mut WRITER: RuntimeStatic<SpinMutex<vga_buffer::Writer>> = RuntimeStatic::get_uninit();

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_magic_number: usize, multiboot_information_address: usize, stack_kernel_top: usize) -> ! {

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

    unsafe {
        let p = PhysicalAddr::new(0xb8000);
        let ptr = p.get() as *mut u16;
        core::ptr::write_volatile(ptr, 231);

        for i in 0..5 {
            let res = core::ptr::read_volatile(ptr);
            println!("elem at the virtual address MODIFIED: 0x{:X}", res);
        }
    }

    let mut memory_manager = memory_manager::MemoryManager::new(stack_kernel_top, &boot_info);

    println!("Paging Enabled!");
    println!("");

    /* this seems to be ok
    println!("page directory[0]: {}", memory_manager.page_directory[0]);
    for i in 0..3 {
        println!("AFTER PTE[{}]: {}", i,  memory_manager.page_directory[0].get_page_table()[i]);
    }
    */

    // TEST paging
    use memory_manager::{ paging::{ PhysicalAddr, VirtualAddr }, frame_allocator::Frame};
    use vga_buffer::{ ScreenChar, ColorCode, Color };

    let vga_virtual = VirtualAddr::new(0x40000000);
    let vga_physical = PhysicalAddr::new(0xb8000);
    let virtual_test_address = VirtualAddr::new(0x40001000);
    let physical_test_address = PhysicalAddr::new(0x300000);

    let prev_buff_addr;
    let current_buff_addr;

    {
        let mut wr = unsafe { WRITER.lock() };
        prev_buff_addr = wr.get_buffer_addr();
        wr.change_ptr_buffer(vga_virtual.get());
        current_buff_addr = wr.get_buffer_addr();
    }

    println!("Ptr changed, test writing!");

    println!("prev buffer ptr: 0x{:X}", prev_buff_addr);
    println!("curr buffer ptr: 0x{:X}", current_buff_addr);
    
    unsafe  {

        //let first_cell = &mut *(vga_physical.get() as *mut ScreenChar);
        //crate::println!("The value should be: {}", first_cell);

        /* uselss, the mapping is not the problem
        let pde_index = vga_virtual.get_pd_index();
        let pde = memory_manager.page_directory[pde_index];
        crate::println!("page directory[{}]: 0x{:X}", pde_index, pde.get_value());

        let page_table = pde.get_page_table();
        crate::println!("page table address {}", page_table.get_physical_addr());

        let pte_index = vga_virtual.get_pt_index();
        let pte = page_table[pte_index];
        crate::println!("page directory[{}] -> page table[{}]: 0x{:X}", pde_index, pte_index, pte.get_value());
        */
    
        // TODO remove all the pub
        
         
        let test_write_and_read = |v: VirtualAddr, p: PhysicalAddr| {
            println!("0x{:X} -pointing-> 0x{:X}", v.get(), p.get());

            //let ptr = &mut *(v.get() as *mut u16);
            //println!("elem at the virtual address: 0x{:X}", *ptr);

            //core::ptr::write_volatile(v.get() as *mut u16, 231);

            //let res = core::ptr::read_volatile(v.get() as *mut u16);
            //println!("elem at the virtual address MODIFIED: 0x{:X}", res);

            //memory_manager::flush_tlb_entry(v.get() as usize);
            //memory_manager::flush_tlb_entry(p.get() as usize);

            //let ptr = &mut *(p.get() as *mut u16);
            //println!("elem at the phyisical address PREV: 0x{:X}", *ptr);

            /*
            let ptr = p.get() as *mut u16;
            loop {
                for _ in 1..1000000 {}
                let res = core::ptr::read_volatile(ptr);
                print!("0x{:X} ", res);
            }
            */

            let ptr = p.get() as *mut u16;
            core::ptr::write_volatile(ptr, 231);

            for i in 0..5 {
                let res = core::ptr::read_volatile(ptr);
                println!("elem at the virtual address MODIFIED: 0x{:X}", res);
            }
            
        };

        println!("");
        println!("Test modify vga_buffer[0] in virtual address and see from the physical address");
        //test_write_and_read(vga_virtual, vga_physical);

        println!("");
        println!("Test modify something in virtual address and see from the physical address");
        //test_write_and_read(virtual_test_address, physical_test_address);
    }

    loop {}
}
