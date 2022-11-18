use super::multiboot::BootInfo;

const FRAME_SIZE: usize = 4096;
const PAGE_SIZE: usize = 4096;

const ENTRIES_PER_PAGE: usize = 1024;

pub mod frame_allocator;
pub mod global_allocator;
pub mod heap_allocator;
pub mod paging;

use frame_allocator::{Allocator, Frame, FrameAllocator};
use paging::*;

extern "C" {
    pub fn change_page_directory(page_direcotry_ptr: usize);
    pub fn enable_paging();
    //pub fn flush_tlb_entry(virtual_addr_ptr: usize);
}

// PD(2^10 entry = 1024) -> PT(2^10 entry = 1024) -> offset(2^12)
pub struct MemoryManager {
    page_directory: PageDirectory,
    frame_allocator: FrameAllocator,
}

impl MemoryManager {
    pub fn new(boot_info: &BootInfo) -> Self {
        let mut frame_allocator = FrameAllocator::new(boot_info);

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
        //let kernel_dimension = 4 * 1024 * 1024;
        //let kernel_end = kernel_base + kernel_dimension;
        //let kernel_end_frame = kernel_end / FRAME_SIZE;

        // create a new page directory
        let mut page_directory = PageDirectory::new(&mut frame_allocator);
        /*
        // HHF : 0x10000000 => 0xC0000000
        let virtual_addr_map_kernel = VirtualAddr::new(0xC0000000);
        let mut hhf_table = page_directory
            .alloc_new_page_table(
                &mut frame_allocator,
                virtual_addr_map_kernel.get_pd_index(),
                PageDirectoryFlag::Present as u32 | PageDirectoryFlag::Writable as u32,
            )
            .expect("Impossible allocate page table");

        //crate::println!("start frame: 0x{:X}", kernel_start_frame);
        //crate::println!("end frame: 0x{:X}", kernel_start_frame + ENTRIES_PER_PAGE);

        // Here I'm preparign the HHF but I have to cover from the start of the
        // kernel to the heap_top
        // So I have to prepare n tables where n is = floor((kernel_dim / 4KiB) / ENTRIES_PER_PAGE)
        // Where kernel_dim is in byte
        for i in kernel_start_frame..(kernel_start_frame + ENTRIES_PER_PAGE) {
            // loop for 1024 page - frame
            let frame = Frame::from_frame_number(i);

            let index_in_the_table = i - kernel_start_frame;
            hhf_table[index_in_the_table].add_attribute(PageTableFlag::Present as u32);
            hhf_table[index_in_the_table].add_attribute(PageTableFlag::Writable as u32);
            hhf_table[index_in_the_table].set_frame(frame);
        }
        */

        Self {
            page_directory,
            frame_allocator,
        }
        //m
    }

    pub unsafe fn enable_paging(&self) {
        // Change pd
        change_page_directory(self.page_directory.get_physical_addr().get());
        // enable paging
        enable_paging();
    }

    pub fn set_up_identity_paging(&mut self, to_limit: usize) -> Result<(), &'static str> {
        // TODO evaluation, if to_limit goes over 1GiB LHK is no more respected
        // always identity mapping one page more
        //crate::println!("limit: {}", to_limit);
        let needed_pd = (to_limit / (ENTRIES_PER_PAGE * PAGE_SIZE)) + 1;
        //crate::println!("total pd: {}", needed_pd);

        for i_pd in 0..needed_pd {
            // setting up identity paging
            let mut identity_table = self.page_directory.alloc_new_page_table(
                &mut self.frame_allocator,
                i_pd,
                PageDirectoryFlag::Present as u32 | PageDirectoryFlag::Writable as u32,
            )?;
            //crate::println!("PD {}", i_pd);

            for i_pt in 0..ENTRIES_PER_PAGE {
                //crate::println!("PT {}", i_pt);
                // loop for 1024 page - frame
                let frame = Frame::from_frame_number((i_pd * ENTRIES_PER_PAGE) + i_pt);
                //crate::println!("Used frame {}", (i_pd * ENTRIES_PER_PAGE) + i_pt);

                identity_table[i_pt].add_attribute(PageTableFlag::Present as u32);
                identity_table[i_pt].add_attribute(PageTableFlag::Writable as u32);
                identity_table[i_pt].add_attribute(PageTableFlag::NotCacheable as u32);
                identity_table[i_pt].set_frame(frame);
            }
        }

        Ok(())
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
        pt_flag: u32,
    ) -> Result<(), &'static str> {
        // the pointer is not really pointing to the page directory, only if inside the
        // identity mapping
        // How manage this if paging is enable?

        // Get the corret PageDirectoryEntry
        let mut page_table: PageTable;

        // Is this pde valid?
        if !self.page_directory[virt_addr.get_pd_index()]
            .is_valid_flag(PageDirectoryFlag::Present as u32)
        {
            // if the pde is not present than alloc a new page table
            // and validate the page directory entry
            page_table = self.page_directory.alloc_new_page_table(
                &mut self.frame_allocator,
                virt_addr.get_pd_index(),
                pd_flag,
            )?;
        } else {
            page_table = self.page_directory[virt_addr.get_pd_index()].get_page_table();
        }

        self.page_directory[virt_addr.get_pd_index()].add_attribute(pd_flag);

        //let pte_index = virt_addr.get_pt_index();
        let pte = &mut page_table[virt_addr.get_pt_index()];

        if pte.is_valid_flag(PageTableFlag::Present as u32) {
            return Err("Page already present, should be deallocated and managed");
        }

        pte.add_attribute(pt_flag);
        pte.set_frame(Frame::from_physical_address(physic_addr));

        Ok(())
    }

    // TODO:
    // + map virtual addr to physical addr
    // + flush TLB
    // + switch page_directory
}

fn next_align<T>(elem: usize, align: usize) -> *mut T {
    let elem = elem as *mut u8;
    let pad = elem.align_offset(align);
    if pad == usize::MAX {
        panic!("Impossibel alignment");
    }
    ((elem as usize) + pad) as *mut T
}
