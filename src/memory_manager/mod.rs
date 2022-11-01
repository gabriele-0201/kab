use super::multiboot::BootInfo;

const FRAME_SIZE: usize = 4096;
const PAGE_SIZE: usize = 4096;

const ENTRIES_PER_PAGE: usize = 1024;

pub mod frame_allocator;
pub mod paging;
pub mod global_allocator;
pub mod heap_allocator;

use frame_allocator::{ Frame, FrameAllocator, Allocator };
use paging::*;

extern "C" {
    pub fn change_page_directory(page_direcotry_ptr: usize);
    pub fn enable_paging();
    pub fn flush_tlb_entry(virtual_addr_ptr: usize);
}

pub struct MemoryManager {
    pub page_directory: PageDirectory,
    frame_allocator: FrameAllocator
}

impl MemoryManager {
    pub fn new(starting_point: usize, boot_info: &BootInfo) -> Self {

        let mut frame_allocator = FrameAllocator::new(starting_point, boot_info);

        // Now what should be done?
        // Should be allocated a new PageDirectory
        // Setting up the identity paging for the first 4MiB (one pt)
        // Setting up higher half kernel
        //  + should cover all the space from 0x100000 to stack_top 
        //  and all of this mapped to 0xC00000000
        //
        //  Now we should have a proper intial PageDirectory
        //
        //  + updating CR3
        //  + activate paging in CR0
        //
        //
        //  Now if all goes well it should not crash due to page fault
        //
        //  What is missing now is a proper managing of the page_fault

        let kernel_base = 1024 * 1024;
        let kernel_start_frame = kernel_base / FRAME_SIZE;

        // FOR NOW from 1MiB to 5MiB (dim = 4MiB)
        // should be from 1MiB to stack_top = starting_point (dim = stack_top - 1MiB)
        let kernel_dimension = 4 * 1024 * 1024; let kernel_end = kernel_base + kernel_dimension;
        let kernel_end_frame = kernel_end / FRAME_SIZE;
        
        // create a new page directory
        let mut page_directory = PageDirectory::new(&mut frame_allocator);

        // setting up identity paging -> NOT sure this is working
        let mut identity_table = page_directory.alloc_new_page_table(
            &mut frame_allocator, 
            0,
            PageDirectoryFlag::Present as u32 | PageDirectoryFlag::Writable as u32
        ).expect("Impossible allocate page table");
        
        for i in 0..ENTRIES_PER_PAGE { // loop for 1024 page - frame
            let frame = Frame::from_frame_number(i);
            
            identity_table[i].add_attribute(PageTableFlag::Present as u32);
            identity_table[i].add_attribute(PageTableFlag::Writable as u32);
            identity_table[i].add_attribute(PageTableFlag::NotCacheable as u32);
            identity_table[i].set_frame(frame);

        }
        
        // HHF : 0x10000000 => 0xC0000000
        // SHOULD be done something better to manage kernel bigger that 3MiB
        let virtual_addr_map_kernel = VirtualAddr::new(0xC0000000);
        let mut hhf_table = page_directory.alloc_new_page_table(
            &mut frame_allocator, 
            virtual_addr_map_kernel.get_pd_index(),
            PageDirectoryFlag::Present as u32 | PageDirectoryFlag::Writable as u32
        ).expect("Impossible allocate page table");

        //crate::println!("start frame: 0x{:X}", kernel_start_frame);
        //crate::println!("end frame: 0x{:X}", kernel_start_frame + ENTRIES_PER_PAGE);
        
        for i in kernel_start_frame..(kernel_start_frame + ENTRIES_PER_PAGE) { // loop for 1024 page - frame
            let frame = Frame::from_frame_number(i);
            
            let index_in_the_table = i - kernel_start_frame;
            hhf_table[index_in_the_table].add_attribute(PageTableFlag::Present as u32);
            hhf_table[index_in_the_table].add_attribute(PageTableFlag::Writable as u32);
            hhf_table[index_in_the_table].set_frame(frame);
        }

        // TEST remap the vga buffer
        let mut m = Self {
            page_directory,
            frame_allocator
        };

        let virtual_vga_buffer = VirtualAddr::new(0x40000000);
        let physical_vga_buffer = PhysicalAddr::new(0xb8000);

        m.map_addr_without_paging(
            virtual_vga_buffer.clone(), 
            physical_vga_buffer,
            PageDirectoryFlag::Present as u32 | PageDirectoryFlag::Writable as u32 | PageDirectoryFlag::NotCacheable as u32,
            PageTableFlag::Present as u32 | PageTableFlag::Writable as u32 | PageDirectoryFlag::NotCacheable as u32
        ).expect("Impossible address mapping");
        
        unsafe {
            // Change pd
            change_page_directory(m.page_directory.get_physical_addr().get());

            // enable paging
            enable_paging();
        }

        /*
        Self {
            page_directory,
            frame_allocator
        }
        */
        m

    }

    pub fn switch_page_directory(&mut self, new_pd: *mut PageDirectory) {
        // should call an arm function that change the CR3 with the new pd
        
    }

    /// This function will map a virtual_addr to a specific physical_addr
    /// This mean that when paging is enabled to refer a particular pyshical address we have to
    /// pass throught this virtual addr
    pub fn map_addr_without_paging(
            &mut self, 
            virt_addr: VirtualAddr, 
            physic_addr: PhysicalAddr,
            pd_flag: u32,
            pt_flag: u32
    ) -> Result<(), &'static str> {

        // the pointer is not really pointing to the page directory, only if inside the
        // identity mapping
        // How manage this if paging is enable?
        
        // Get the corret PageDirectoryEntry
        let mut page_table: PageTable;

        // Is this pde valid?
        if !self.page_directory[virt_addr.get_pd_index()].is_valid_flag(PageDirectoryFlag::Present as u32) {
            // if the pde is not present than alloc a new page table 
            // and validate the page directory entry
            page_table = self.page_directory.alloc_new_page_table(&mut self.frame_allocator, virt_addr.get_pd_index(), pd_flag)?;
        } else {
            page_table = self.page_directory[virt_addr.get_pd_index()].get_page_table();
        }

        self.page_directory[virt_addr.get_pd_index()].add_attribute(pd_flag);
        
        //let pte_index = virt_addr.get_pt_index();
        let pte = &mut page_table[virt_addr.get_pt_index()];

        if pte.is_valid_flag(PageTableFlag::Present as u32) {
            return Err("Page already present, should be deallocated and managed")
        }

        pte.add_attribute(pt_flag);
        pte.set_frame(Frame::from_physical_address(physic_addr));

        Ok(())
    }

    // TODO:
    // + map virtual addr to physical addr
    // + flush TLB
    // + enable paging
}

fn next_align<T>(elem: usize, align: usize) -> *mut T {
    let elem = elem as *mut u8;
    let pad = elem.align_offset(align);
    if pad == usize::MAX {
        panic!("Impossibel alignment");
    }
    ((elem as usize) + pad) as *mut T
}
