use super::multiboot::BootInfo;

const FRAME_SIZE: usize = 4096;
const PAGE_SIZE: usize = 4096;

const ENTRIES_PER_PAGE: usize = 1024;

pub mod frame_allocator;
pub mod paging;

use frame_allocator::FrameAllocator;

pub struct MemoryManager {
    frame_allocator: FrameAllocator
}

impl MemoryManager {
    pub fn new(starting_point: usize, boot_info: &BootInfo) -> Self {

        let frame_allocator = FrameAllocator::new(starting_point, boot_info);

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

        Self {
            frame_allocator
        }
    }
}
