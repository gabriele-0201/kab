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
    use vga_buffer::{ ScreenChar, ColorCode, Color };

    unsafe  {
        let cell = ScreenChar{
            ascii_character: b' ',
            color_code: ColorCode::new(Color::Yellow, Color::Black)
        };

        crate::println!("The value should be: 0x{:X}", *((&cell as *const ScreenChar) as *const u16));

        let vga_virtual = VirtualAddr::new(0x40000000);
        //let vga_physical = PhysicalAddr::new(0xb8000);
        let vga_physical = PhysicalAddr::new(0x300000);

        let pde_index = vga_virtual.get_pd_index();
        let pde = memory_manager.page_directory[pde_index];
        crate::println!("page directory[{}]: 0x{:X}", pde_index, pde.get_value());

        let page_table = pde.get_page_table();
        crate::println!("page table address {}", page_table.get_physical_addr());

        let pte_index = vga_virtual.get_pt_index();
        let pte = page_table[pte_index];
        crate::println!("page directory[{}] -> page table[{}]: 0x{:X}", pde_index, pte_index, pte.get_value());
    
        // TODO remove all the pub

        let ptr = &mut *(vga_virtual.get() as *mut u16);
        println!("elem at the virtual address: 0x{:X}", *ptr);
        *ptr = 231;
        println!("elem at the virtual address MODIFIED: 0x{:X}", *ptr);

        //memory_manager::flush_tlb_entry(vga_physical.get() as usize);

        let ptr = &mut *(vga_physical.get() as *mut u16);
        println!("elem at the phyisical address: 0x{:X}", *ptr);
        /*
        *ptr = 231;
        println!("elem at the virtual address MODIFIED: 0x{:X}", ptr);

        //memory_manager::flush_tlb_entry(vga_physical.get() as usize);

        let ptr = &mut *(vga_physical.get() as *mut u16);
        println!("elem at the phyisical address after modified: 0x{:X}", *ptr);
        
        // Test using volatile
        use volatile::Volatile;
        let ptr = &mut *(vga_virtual.get() as *mut Volatile<u16>);
        println!("elem at the phyisical address: {:?}", ptr);
        ptr.write(231);
        println!("elem at the virtual address MODIFIED: {:?}", ptr);

        memory_manager::flush_tlb_entry(vga_virtual.get() as usize);

        let ptr = &mut *(vga_physical.get() as *mut Volatile<u16>);
        println!("elem at the phyisical address after modified: {:?}", ptr);
        */

        /*
        struct Buffer {
            chars: [[Volatile<ScreenChar>; 80]; 25],
        }

        let buffer = &mut *( vga_physical.get() as *mut Buffer);

        for i in 0..80 {
            buffer.chars[0][i].write(ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::Blue, Color::Black)
            });
        }
        */
    }

    loop {}
}
