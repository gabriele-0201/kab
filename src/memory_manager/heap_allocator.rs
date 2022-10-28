use super::{ paging::VirtualAddr };
use core::alloc::{ GlobalAlloc, Layout };

/// HeapAllocator will use some sort of linked list
/// where each HeapHead will contain the dim of the allocated space and
/// a ptr to the next allocated space (another HeapHead)
///
/// Algorithm:
///
/// At the first allocation the heap is totally free, in fact current
/// head_of_heap_head is None so the thing to do is: set a new HeapHead
/// at the beginning of the heap (always with a dimension check)
///
///
pub struct HeapAllocator {
    start_heap: VirtualAddr,
    head_of_heap_head: Option<*mut HeapHead>,
    end_heap: VirtualAddr
}

enum Next {
    HeapHead(*mut HeapHead),
    Tail(VirtualAddr)
}

impl Next {
    fn get_ptr_usize(&self) -> usize {
        match self {
            Next::HeapHead(hh) => *hh as usize,
            Next::Tail(tail) => tail.get()
        }
    }
}

impl Default for Next {
    fn default() -> Self {
        Next::HeapHead(core::ptr::null_mut())
    }
}
/// Keep the pointer to the next HeapHead and the allocated space whith his dimension
struct HeapHead {
    next: Next,
    allocated_space: *mut u8,
    dim: usize
}

struct HeapHeadIterator {
    curr: *mut HeapHead
}

impl HeapHead {
    /// this function is similar to a constructor with the only difference that
    /// does not return a new HeapHead but write it on the passed address with
    /// the specified properties
    ///
    /// Does not return any error, it must be unsafe because you could overwrite 
    /// thigs you wouldn't
    unsafe fn write_new(heap_head: *mut HeapHead, next: Next, allocated_space: *mut u8, dim: usize) {
        heap_head.write_volatile(
            HeapHead {
                next,
                allocated_space,
                dim
            }
        );
    }

    /// this function is used to create a new HeapHead from a base address
    /// using only a next element
    ///
    /// Example: the first HeapHead that must be allocated
    fn insert_from(heap_head: *mut HeapHead, layout: Layout, next: Next) -> bool {
        
        // Ensure that the layout dimension is less that the avaiable space
        if layout.size() > (next.get_ptr_usize() - heap_head as usize) {
            return false;
        }

        // This could be of course generalized and mixed with the insert new implementation
        // but I'm relly tired now so I will not do that, hope it at least works
        let end_heap = heap_head as usize + core::mem::size_of::<HeapHead>() as usize;
        let required_padding = (end_heap as *mut u8).align_offset(layout.align());

        unsafe {
            HeapHead::write_new(
                heap_head,
                next,
                (end_heap + required_padding) as *mut u8,
                layout.size()
            );
        }

        true
    }

    /// try to insert a new layout between this HeapHead and the next one,
    /// this function will carry about padding and HeapHead's linked list
    /// management
    fn insert_after(&mut self, layout: Layout) -> bool {

        let offset_for_allocated_space = self.get_needed_padding(layout) + core::mem::size_of::<HeapHead>();

        if offset_for_allocated_space + layout.size() > self.get_adiacent_free_space() {
            return false
        }

        // the adiacent free space is enough so I have to insert a new 
        // heap_head in the linked list
        let new_head_head = self.get_end_of_allocated_space() as *mut HeapHead; 
        let new_allocated_space = (new_head_head as usize + offset_for_allocated_space) as *mut u8;

        // now we have a new heap head that point to the next
        unsafe { HeapHead::write_new(new_head_head, core::mem::take(&mut self.next), new_allocated_space, layout.size()) };

        //now the current heap head must point to the just created one
        self.next = Next::HeapHead(new_head_head);

        true
    }

    /// Return the needed internal pagging to respect the alignment
    ///
    /// HeapHead |--needed_pagging--| start_allocated_space----
    fn get_needed_padding(&self, layout: Layout) -> usize {
        (self.as_ptr() as *mut u8).align_offset(layout.align())
    }

    fn get_end_of_allocated_space(&self) -> usize {
        self.allocated_space as usize + self.dim
    }

    fn get_adiacent_free_space(&self) -> usize {
        // |heap_head|allocated_space__________|___free_space____|next_heap_head
        self.next.get_ptr_usize() - self.get_end_of_allocated_space()
    }

    fn get_allocated_ptr(&self) -> *mut u8 {
        unsafe  {
            (self as *const HeapHead).offset(core::mem::size_of::<HeapHead>() as isize) as *mut u8
        }
    }

    fn as_mut_ptr(&mut self) -> *mut HeapHead {
        self as *mut HeapHead
    }

    fn as_ptr(&self) -> *const HeapHead {
        self as *const HeapHead
    }
     
}

impl IntoIterator for &mut HeapHead {
    type Item = *mut HeapHead;
    type IntoIter = HeapHeadIterator;

    fn into_iter(self) -> Self::IntoIter {
        HeapHeadIterator {
            curr: self as *mut HeapHead,
        }
    }
}

impl Iterator for HeapHeadIterator {
    type Item = *mut HeapHead;

    fn next(&mut self) -> Option<Self::Item> {

        let curr_heap_head = unsafe { &mut *self.curr };

        match curr_heap_head.next {
            Next::HeapHead(ptr) => {
                self.curr = ptr
            }, 
            Next::Tail(_) => {
                return None
            }
        }

        Some(curr_heap_head.as_mut_ptr())
    }
}

/// 
impl HeapAllocator {
    pub fn new(start_heap: usize, end_heap: usize) -> HeapAllocator {
        HeapAllocator {
            start_heap: VirtualAddr::new(start_heap), 
            head_of_heap_head: None,
            end_heap: VirtualAddr::new(end_heap)
        }
    }
}

/// Let's start creating a bump allocator
unsafe impl GlobalAlloc for HeapAllocator {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        macro_rules! try_insert_from_start_heap {
            ($next: expr) => {
                if !HeapHead::insert_from(self.start_heap.get() as *mut HeapHead, layout, $next) {
                    return core::ptr::null_mut();
                }
                return self.start_heap.get() as *mut u8
            }
        }
        // layout should not be 0 and must be a power of 2
        if self.head_of_heap_head.is_none() {
            try_insert_from_start_heap!(Next::Tail(self.end_heap.clone()));
        }

        // here now we have to make sure that the first heap_head is at the start,
        // oherwise check if is possible to create a new head_of_heap_heads
        let hohh = self.head_of_heap_head.expect("Something break in HeapAllocator, Head of heap_heads is null");
        if self.start_heap.get() ==  hohh as usize {
            try_insert_from_start_heap!(Next::HeapHead(hohh));
        }

        // now I will loop in the liked list and check for every heap_head
        // if accept the requested layout in fornt of it
        
        let mut h_iter = unsafe { &mut *hohh }.into_iter();

        while let Some(heap_head) = h_iter.next() {

            let heap_head = unsafe { &mut *heap_head };

            // not sure if is possible to modify the stufff inside the iterator
            if heap_head.insert_after(layout) {

                let heap_head = unsafe { &*h_iter.next().expect("Impossible unwrap just insered HeapHead") };

                return heap_head.get_allocated_ptr()
            }

        }

        return core::ptr::null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
       todo!() 
    }

}

