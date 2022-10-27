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
    fn set_new(heap_head: &VirtualAddr, next: Next, layout: Layout) {
        let dim = layout.size();
        // have to do something about alignent
        unsafe {
            (heap_head.get() as *mut HeapHead).write_volatile(
                HeapHead {
                    next,
                    dim
                }
            );
        }
    }

    /// try to insert a new layout between this HeapHead and the next one,
    /// this function will carry about padding and HeapHead's linked list
    /// management
    ///
    /// TODO: manca ancora da fare che? allora c'e' da fare meglio la roba
    /// del padding e soprattuto ho appena aggiunto a HeapHead anche il pointer
    /// allo spazio allocato cosi da mettere il padding tra l'HeadHead e lo spazio allocato
    /// cosi quando rimuvo un HeadHead in maniera implicita mi libero del padding
    /// e dello spazio allocato
    ///
    fn insert_after(&mut self, layout: Layout) -> bool {
        // TODO: something about alignment

        let padding = self.get_needed_padding(layout);

        if padding + layout.size() + core::mem::size_of::<HeapHead>() <= self.get_adiacent_free_space() {
            // the adiacent free space is enough so I have to insert a new 
            // heap_head in the linked list
            let new_head_head: &mut HeapHead = unsafe { &mut *((self as *const Self as usize + self.dim) as *mut HeapHead) }; 
            
            *new_head_head = HeapHead {
                next: self.next, 
                dim: layout.size()
            }
        }
        false
    }

    /// Return the needed internal pagging to respect the alignment
    ///
    /// HeapHead |--needed_pagging--| start_allocated_space----
    fn get_needed_padding(&self, layout: Layout) -> usize {
        layout.align() - (self as *const Self as usize % layout.align())
    }

    fn get_adiacent_free_space(&self) -> usize {
        // |heap_head|allocated_space__________|___free_space____|next_heap_head
        self.next.get_ptr_usize() - (self as *const HeapHead as usize + core::mem::size_of::<HeapHead>() as usize + self.dim)
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

    /// TODO: understand why here we does not accept &self or &mut self
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

        // layout should not be 0 and must be a power of 2
        
        if self.head_of_heap_head.is_none() {
            // layout must be <= heap size
            HeapHead::set_new(&self.start_heap, Next::Tail(self.end_heap.clone()), layout);

            // TODO this cast could be better
            return (*(self.start_heap.get() as *const HeapHead)).get_allocated_ptr()
        }

        // here now we have to make sure that the first heap_head is at the start,
        // oherwise check if is possible to create a new head_of_heap_heads
        if self.start_heap.get() == self.head_of_heap_head.expect("Something break in HeapAllocator, Head of heap_heads is null") as usize {
            todo!()
        }

        // now I will loop in the liked list and check for every heap_head
        // if accept the requested layout in fornt of it
        
        let mut h_iter = unsafe { &mut *(self.head_of_heap_head.expect("Something break in HeapAllocator, Head of heap_heads is null")) }.into_iter();

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

