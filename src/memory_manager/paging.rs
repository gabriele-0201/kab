use super::{ 
    *, 
    frame_allocator::{ 
        Frame, 
        FrameAllocator, 
        Allocator 
    } 
};

#[derive(Debug)]
#[repr(transparent)]
pub struct PhysicalAddr(usize);

impl PhysicalAddr {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct VirtualAddr(usize);

impl VirtualAddr {
    fn get_offset(&self) -> usize {
        (self.0 & 0xFFF) as usize
    }

    fn get_pt_index(&self) -> usize {
        ((self.0 >> 12) & 0x3FF)  as usize
    }

    fn get_pd_index(&self) -> usize {
        ((self.0 >> 22) & 0x3FF)  as usize
    }
}

// DIRECTORY
struct PageDirectory {
    pub entries: [PageDirectoryEntry; ENTRIES_PER_PAGE]
}

enum PageEntryFlag {
	Present			=	1,		//0000000000000000000000000000001
	Writable		=	2,		//0000000000000000000000000000010
	User			=	4,		//0000000000000000000000000000100
	Pwt			    =	8,		//0000000000000000000000000001000
	Pcd			    =	0x10,		//0000000000000000000000000010000
	Accessed		=	0x20,		//0000000000000000000000000100000
	Dirty			=	0x40,		//0000000000000000000000001000000
	BigPage			=	0x80,		//0000000000000000000000010000000 4MB page
	CpuGlobal		=	0x100,		//0000000000000000000000100000000
	Lv4Global		=	0x200,		//0000000000000000000001000000000
   	Frame			=	0x7FFFF000 	//1111111111111111111000000000000
}

#[derive(Debug)]
#[repr(transparent)]
struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    fn add_attribute(&mut self, attribute: u32) {
        self.0 |= attribute;
    }

    fn del_attribute(&mut self, attribute: u32) {
        // is this right?
        self.0 &= !attribute;
    }

    fn set_frame(&mut self, frame: Frame) {
        self.0 |= frame.get_physical_addr().get() as u32;
    }

    fn get_frame(&mut self) -> PhysicalAddr {
        PhysicalAddr((self.0 & (PageTableFlag::Frame as u32)) as usize)
    }

    // maybe others will be needed, example:
    // extern bool		pd_entry_is_present (pd_entry e);
    // extern bool		pd_entry_is_user (pd_entry);
    // extern bool		pd_entry_is_4mb (pd_entry);
    // extern bool		pd_entry_is_writable (pd_entry e);
    // extern void		pd_entry_enable_global (pd_entry e);
}

// TABLE
struct PageTable<'a> {
    frame_allocator: &'a mut FrameAllocator,
    pub entries: [PageTableEntry; ENTRIES_PER_PAGE]
}

impl<'a> PageTable<'a> {

    pub fn new(frame_allocator: &'a mut FrameAllocator) -> Self {
        Self {
            frame_allocator,
            entries: [PageTableEntry(0); ENTRIES_PER_PAGE]
        }
    }

    pub fn alloc_new_page(&mut self, index: usize) -> bool {
        let frame = self.frame_allocator.allocate();

        let frame = match frame {
            Some(f) => f,
            None => return false
        };

        self.entries[index].add_attribute(PageTableFlag::Present as u32);
        self.entries[index].set_frame(frame);

        true
    }

    pub fn free_page(&mut self, index: usize) {
        let frame = self.entries[index].get_frame();

        self.frame_allocator.deallocate(frame);

        self.entries[index].del_attribute(PageTableFlag::Present as u32);
    }
}

enum PageTableFlag {
	Present			=	1,		//0000000000000000000000000000001
	Writable		=	2,		//0000000000000000000000000000010
	User			=	4,		//0000000000000000000000000000100
	Writethough		=	8,		//0000000000000000000000000001000
	NotCacheable	=	0x10,		//0000000000000000000000000010000
	Accessed		=	0x20,		//0000000000000000000000000100000
	Dirty			=	0x40,		//0000000000000000000000001000000
	Pat			    =	0x80,		//0000000000000000000000010000000
	CpuGlobal		=	0x100,		//0000000000000000000000100000000
	Lv4Global		=	0x200,		//0000000000000000000001000000000
   	Frame			=	0x7FFFF000 	//1111111111111111111000000000000 WHY??
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct PageTableEntry(u32);

impl PageTableEntry {
    fn add_attribute(&mut self, attribute: u32) {
        self.0 |= attribute;
    }

    fn del_attribute(&mut self, attribute: u32) {
        // is this right?
        self.0 &= !attribute;
    }

    fn set_frame(&mut self, frame: Frame) {
        self.0 |= frame.get_physical_addr().get() as u32;
    }

    fn get_frame(&mut self) -> Frame {
        // should be checked that this si not minor than the lower frame
        Frame::new((self.0 & (PageTableFlag::Frame as u32)) as usize)
    }

    // maybe others will be needed, example:
    // extern bool 		pt_entry_is_present (pt_entry e);
    // extern bool 		pt_entry_is_writable (pt_entry e);
}
