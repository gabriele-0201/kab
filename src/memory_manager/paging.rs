use super::{ 
    *, 
    frame_allocator::{ 
        Frame, 
        FrameAllocator, 
        Allocator 
    } 
};

use core::{ fmt, ops::{ Index, IndexMut } };

#[derive(Clone)]
#[repr(transparent)]
pub struct PhysicalAddr(usize);

impl PhysicalAddr {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl fmt::Display for PhysicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "physical: 0x{:X} ", self.0)
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct VirtualAddr(usize);

impl VirtualAddr {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get_offset(&self) -> usize {
        (self.0 & 0xFFF) as usize
    }

    pub fn get_pt_index(&self) -> usize {
        ((self.0 >> 12) & 0x3FF)  as usize
    }

    pub fn get_pd_index(&self) -> usize {
        ((self.0 >> 22) & 0x3FF)  as usize
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

impl fmt::Display for VirtualAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "virtual: 0x{:X} ", self.0)
    }
}

// DIRECTORY
//
// There is problem with the movement of the PageDirectory?
// The move? should not erease the stuff inside the array
#[derive(Clone)]
#[repr(transparent)]
pub struct PageDirectory {
    //pub entries: [PageDirectoryEntry; ENTRIES_PER_PAGE]
    pub entries: *mut [PageDirectoryEntry; ENTRIES_PER_PAGE]
}

impl Index<usize> for PageDirectory {
    type Output = PageDirectoryEntry;

    fn index(&self, index: usize) -> &PageDirectoryEntry {
        unsafe{&(*self.entries)[index]}
    }
}

impl IndexMut<usize> for PageDirectory {
    fn index_mut(&mut self, index: usize) -> &mut PageDirectoryEntry {
        //&mut self.entries[index]
        unsafe{&mut(*self.entries)[index]}
    }
}

impl PageDirectory {

    /// This function use the allocator to create a new frame 
    /// and initialize a new page direcotory inside it
    pub fn new(frame_allocator: &mut FrameAllocator) -> Self {
        unsafe { 

            let new_frame = frame_allocator.allocate().expect("Impossible allocate new frame for the page directory");
            
            // TABLE_PTR has to be cleaned
            // some sort of memset(0)
            let table_ptr = new_frame.get_physical_addr().get() as *mut [PageDirectoryEntry; ENTRIES_PER_PAGE];
            *table_ptr = [PageDirectoryEntry(0); ENTRIES_PER_PAGE];

            Self {
                entries: table_ptr
            }
        }
    }

    pub fn from_physical_address(addr: paging::PhysicalAddr) -> Self {
        Self {
            entries: addr.get() as *mut [PageDirectoryEntry; ENTRIES_PER_PAGE],
        }
    }

    pub fn get_physical_addr(&self) -> paging::PhysicalAddr {
        PhysicalAddr::new(self.entries as usize)
    }

    pub fn alloc_new_page_table(
        &mut self, 
        frame_allocator: &mut FrameAllocator, 
        index: usize,
        flags: u32
    ) -> Result<PageTable, &'static str> {
        // should call the constructor of the page table that will 
        // allocate the new frame, initialize the page table
        // than get the address of that table and enable the entry

        let table = PageTable::new(frame_allocator)?;

        //crate::println!("Allocated page table: {:?}", table.get_physical_addr());

        self[index].add_attribute(flags);
        self[index].set_frame(Frame::from_physical_address(table.get_physical_addr()));

        // hope that the move semantics does not cause errors
        Ok(table)
    }

    pub fn free_page_table(&mut self, index: usize) {
        /*
        let frame = self.entries[index].get_frame();

        self.frame_allocator.deallocate(frame);

        self.entries[index].del_attribute(PageTableFlag::Present as u32);
        */
    }
}

impl fmt::Display for PageDirectory{

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[\n")?;
        for i in 0..ENTRIES_PER_PAGE {
            write!(f, "value: 0x{:X}, is_present: {}, page_table: {}", self[i].get_value(), self[i].is_valid_flag(PageDirectoryFlag::Present as u32), self[i].get_page_table().get_physical_addr())?;
        }
        write!(f, "]\n")
    }
}

#[derive(Clone, Copy)]
pub enum PageDirectoryFlag {
	Present			=	1,		//0000000000000000000000000000001
	Writable		=	2,		//0000000000000000000000000000010
	User			=	4,		//0000000000000000000000000000100
	Writethrough	=	8,		//0000000000000000000000000001000
	NotCacheable	=	0x10,		//0000000000000000000000000010000
	Accessed		=	0x20,		//0000000000000000000000000100000
	Dirty			=	0x40,		//0000000000000000000000001000000
	BigPage			=	0x80,		//0000000000000000000000010000000 4MB page
	CpuGlobal		=	0x100,		//0000000000000000000000100000000
	Lv4Global		=	0x200,		//0000000000000000000001000000000
   	Frame			=	0x7FFFF000 	//1111111111111111111000000000000
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    pub fn add_attribute(&mut self, attribute: u32) {
        self.0 |= attribute;
    }

    pub fn del_attribute(&mut self, attribute: u32) {
        // is this right?
        self.0 &= !attribute;
    }

    pub fn set_frame(&mut self, frame: Frame) {
        // has to be 4K aligned
        /*
        let frame_addr = frame.get_physical_addr().get();
        if (frame_addr & !(PageDirectoryFlag::Frame as usize)) != 0 {
            return Err("Frame address not 4K aligned")
        }
        */
        self.0 |= frame.get_physical_addr().get() as u32;
        //Ok(())
    }

    pub fn get_page_table(&self) -> PageTable {
        // the address should not be shifted, the flag should do everything
        //unsafe { *((self.0 & (PageTableFlag::Frame as u32)) as *mut PageTable) }
        // Now a little update, maybe a possible new solution is to
        // create a new objects every time pointing to the correct tables
        PageTable::from_physical_address(PhysicalAddr::new((self.0 & PageTableFlag::Frame as u32) as usize))
    }

    pub fn is_valid_flag(&self, attribute: u32) -> bool {
        (self.0 & attribute) == attribute
    }

    pub fn get_value(&self) -> u32 {
        self.0 
    }

    // maybe others will be needed, example:
    // extern bool		pd_entry_is_present (pd_entry e);
    // extern bool		pd_entry_is_user (pd_entry);
    // extern bool		pd_entry_is_4mb (pd_entry);
    // extern bool		pd_entry_is_writable (pd_entry e);
    // extern void		pd_entry_enable_global (pd_entry e);
}

impl fmt::Display for PageDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value: 0x{:X}, is_present: {}, page_table: {}", self.get_value(), self.is_valid_flag(PageDirectoryFlag::Present as u32), self.get_page_table().get_physical_addr())
    }
}

// TABLE
#[repr(transparent)]
pub struct PageTable {
    entries: *mut [PageTableEntry; ENTRIES_PER_PAGE]
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        //&self.entries[index]
        unsafe{&(*self.entries)[index]}
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        //&mut self.entries[index]
        unsafe{&mut(*self.entries)[index]}
    }
}

impl PageTable {

    pub fn new(frame_allocator: &mut FrameAllocator) -> Result<Self, &'static str> {
        unsafe { 

            let new_frame = frame_allocator.allocate();

            let new_frame = match new_frame {
                Some(f) => f,
                None => return Err("Impossible alloc a page for the pageTable")
            };
            
            let table_ptr = new_frame.get_physical_addr().get() as *mut [PageTableEntry; ENTRIES_PER_PAGE];
            *table_ptr = [PageTableEntry(0); ENTRIES_PER_PAGE];

            Ok(
                Self {
                    entries: table_ptr
                }
            )
        }
    }

    pub fn from_physical_address(addr: paging::PhysicalAddr) -> Self {
        Self {
            entries: addr.get() as *mut [PageTableEntry; ENTRIES_PER_PAGE] 
        }
    }

    pub fn get_physical_addr(&self) -> paging::PhysicalAddr {
        PhysicalAddr::new(self.entries as usize)
    }

    pub fn alloc_new_page(
        &mut self, 
        frame_allocator: &mut FrameAllocator, 
        index: usize,
        flags: u32
    ) -> Result<(), &'static str> {

        let new_frame = frame_allocator.allocate();

        let new_frame = match new_frame {
            Some(f) => f,
            None => return Err("Impossible alloc a page for the pageTable")
        };

        // if at this index another page is alredy present
        // than a dealloc should be done
        if self[index].is_valid_flag(PageTableFlag::Present as u32) {
            return Err("Entry has a page already allocated, deallocation should be managed")
        }

        self[index].add_attribute(flags);
        self[index].set_frame(Frame::from_physical_address(new_frame.get_physical_addr()));

        Ok(())
    }

    /*
    pub fn free_page(&mut self, index: usize) {
        let frame = self.entries[index].get_frame();

        self.frame_allocator.deallocate(frame);

        self.entries[index].del_attribute(PageTableFlag::Present as u32);
    }
    */
}

pub enum PageTableFlag {
	Present			=	1,		//0000000000000000000000000000001
	Writable		=	2,		//0000000000000000000000000000010
	User			=	4,		//0000000000000000000000000000100
	Writethrough	=	8,		//0000000000000000000000000001000
	NotCacheable	=	0x10,		//0000000000000000000000000010000
	Accessed		=	0x20,		//0000000000000000000000000100000
	Dirty			=	0x40,		//0000000000000000000000001000000
	Pat			    =	0x80,		//0000000000000000000000010000000
	CpuGlobal		=	0x100,		//0000000000000000000000100000000
	Lv4Global		=	0x200,		//0000000000000000000001000000000
   	Frame			=	0x7FFFF000 	//1111111111111111111000000000000 
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub fn add_attribute(&mut self, attribute: u32) {
        self.0 |= attribute;
    }

    pub fn del_attribute(&mut self, attribute: u32) {
        // is this right?
        self.0 &= !attribute;
    }

    pub fn set_frame(&mut self, frame: Frame) {
        // has to be 4K aligned
        /*
        let frame_addr = frame.get_physical_addr().get();
        if (frame_addr & !(PageTableFlag::Frame as usize)) != 0 {
            return Err("Frame address not 4K aligned")
        }
        */
        self.0 |= frame.get_physical_addr().get() as u32;
        //Ok(())
    }

    // maybe is better to call this get_frame? and return a physical addr
    pub fn get_page(&self) -> PhysicalAddr {
        // should be checked that this si not minor than the lower frame
        PhysicalAddr::new((self.0 & (PageTableFlag::Frame as u32)) as usize)
    }

    pub fn is_valid_flag(&self, attribute: u32) -> bool {
        (self.0 & attribute) == attribute
    }

    pub fn get_value(&self) -> u32 {
        self.0
    }

    // maybe others will be needed, example:
    // extern bool 		pt_entry_is_present (pt_entry e);
    // extern bool 		pt_entry_is_writable (pt_entry e);
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value: 0x{:X}, is_present: {}, page_table: {}", self.get_value(), self.is_valid_flag(PageDirectoryFlag::Present as u32), self.get_page())
    }
}
