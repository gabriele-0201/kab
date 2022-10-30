use super::paging::VirtualAddr;
use crate::runtime_static::RuntimeStatic;
use crate::concurrency::spin_mutex::SpinMutex;
use core::{ cell::UnsafeCell, alloc::{ GlobalAlloc, Layout } };

// TODO: alloc inital function is an unsafe function, I can make
// all other function usafe to make it easier to read with deferentation

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
    head_of_heap_head: UnsafeCell<Option<*mut HeapHead>>,
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

impl core::fmt::Debug for Next {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Next::Tail(end) => write!(f, "Tail: 0x{:X}", end.get() as usize),
            Next::HeapHead(h) => write!(f, "Next: 0x{:X}", *h as usize)
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
        crate::println!("Saving new Head");
        crate::println!("ALign missing: {}", (heap_head as *mut u8).align_offset(core::mem::size_of::<HeapHead>()));
        //crate::println!("is aligned: {}", heap_head.is_aligned());
        heap_head.write_volatile(
            HeapHead {
                next,
                allocated_space,
                dim
            }
        );
        crate::println!("Saving new Head - DONE");
    }

    /// this function is used to create a new HeapHead from a base address
    /// using only a next element
    ///
    /// Example: the first HeapHead that must be allocated
    fn insert_from(heap_head: *mut HeapHead, layout: Layout, next: Next) -> Result<*const HeapHead, ()> {
        
        // Ensure that the layout dimension is less that the avaiable space
        if layout.size() > (next.get_ptr_usize() - heap_head as usize) {
            return Err(());
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

        Ok(heap_head)
    }

    /// try to insert a new layout between this HeapHead and the next one,
    /// this function will carry about padding and HeapHead's linked list
    /// management
    fn insert_after(&mut self, layout: Layout) -> Result<*const HeapHead, ()> {

        // TODO refactor, start using something like : next_align
        // Stare molto attenti al fatto che anche la base dell'heap_head puo' richiedere un
        // alignment, questo signigica che bisogna valutarlo fin da subito anche nel calcolo dello
        // spazio adiacente -> merita un bel refactor un po' tutto

        let offset_for_allocated_space = HeapHead::get_needed_padding(self.get_end_of_allocated_space() as *const HeapHead, layout) + core::mem::size_of::<HeapHead>();
        crate::println!("offset requested: {}", offset_for_allocated_space);

        if offset_for_allocated_space + layout.size() > self.get_adiacent_free_space() {
            return Err(())
        }

        crate::println!("There is enough space");

        // the adiacent free space is enough so I have to insert a new 
        // heap_head in the linked list
        let new_head_head = self.get_end_of_allocated_space() as *mut HeapHead; 

        // align if needed

        let new_allocated_space = (new_head_head as usize + offset_for_allocated_space) as *mut u8;
        crate::println!("new heap_head: 0x{:X}", new_head_head as usize);
        crate::println!("new base allocated space: 0x{:X}", new_allocated_space as usize);

        // now we have a new heap head that point to the next
        unsafe { HeapHead::write_new(new_head_head, core::mem::take(&mut self.next), new_allocated_space, layout.size()) };

        //now the current heap head must point to the just created one
        self.next = Next::HeapHead(new_head_head);

        crate::println!("DONE");
        Ok(new_head_head)
    }

    /// Return the needed internal pagging to respect the alignment
    /// Given a pointer to the HeapHead and a layout 
    ///
    /// HeapHead |--needed_pagging--| start_allocated_space----
    fn get_needed_padding(heap_head: *const HeapHead, layout: Layout) -> usize {
        let base = (core::mem::size_of::<HeapHead>() + (heap_head as usize)) as *const u8;
        crate::println!("ptr {}, align {}", base as usize, layout.align());
        let al = base.align_offset(layout.align());
        crate::println!("{}", al);
        al
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

impl core::fmt::Debug for HeapHead {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let padding = self.allocated_space as usize - (self as *const HeapHead as usize + core::mem::size_of::<HeapHead>() as usize);
        write!(f, "HeapHead pos: 0x{:X}, next: {:?} \nSpace handled -> dim: {}, pos: 0x{:X}, pad: {}", self as *const HeapHead as usize, self.next, self.dim, self.allocated_space as usize, padding)
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

        if self.curr.is_null() {
            return None 
        }

        let curr_heap_head = unsafe { &mut *self.curr };

        match curr_heap_head.next {
            Next::HeapHead(ptr) => {
                self.curr = ptr
            }, 
            Next::Tail(_) => {
                self.curr = core::ptr::null_mut()
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
            head_of_heap_head: UnsafeCell::new(None),
            end_heap: VirtualAddr::new(end_heap)
        }
    }
}

/// Let's start creating a bump allocator
unsafe impl GlobalAlloc for HeapAllocator {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        macro_rules! try_insert_from_start_heap {
            ($next: expr) => {
                let new_head_head = match HeapHead::insert_from(self.start_heap.get() as *mut HeapHead, layout, $next) {
                    Ok(new_head_head) => new_head_head,
                    Err(()) => return core::ptr::null_mut()
                };
                let new_head_head = unsafe { &*new_head_head };
                *self.head_of_heap_head.get() = Some(self.start_heap.get() as *mut HeapHead);
                return new_head_head.allocated_space
            }
        }
        // layout should not be 0 and must be a power of 2
        if (*self.head_of_heap_head.get()).is_none() {
            try_insert_from_start_heap!(Next::Tail(self.end_heap.clone()));
        }

        // here now we have to make sure that the first heap_head is at the start,
        // oherwise check if is possible to create a new head_of_heap_heads
        let hohh = (*self.head_of_heap_head.get()).expect("Something break in HeapAllocator, Head of heap_heads is null");
        if self.start_heap.get() !=  hohh as usize {
            try_insert_from_start_heap!(Next::HeapHead(hohh));
        }

        // now I will loop in the liked list and check for every heap_head
        // if accept the requested layout in fornt of it
        let mut h_iter = unsafe { &mut *hohh }.into_iter();

        while let Some(heap_head) = h_iter.next() {

            let heap_head = unsafe { &mut *heap_head };

            // not sure if is possible to modify the stufff inside the iterator
            if let Ok(heap_head) = heap_head.insert_after(layout) {
                crate::println!("Should be insered correctly");
                let heap_head = unsafe { &*heap_head };
                return heap_head.get_allocated_ptr()
            }

        }

        return core::ptr::null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
       todo!() 
    }

}

unsafe impl GlobalAlloc for RuntimeStatic<SpinMutex<HeapAllocator>> {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().dealloc(ptr, layout)
    }

}

pub mod tests {
    pub fn home_made_test() {

        use crate::println;
        use alloc::{ boxed::Box, vec::Vec };
        use crate::GLOBAL_ALLOC;
        use core::alloc::GlobalAlloc;

        println!("");
        println!("HOME MADE TEST");

        println!("HeapHead dimension: {}", core::mem::size_of::<super::HeapHead>());

        unsafe { // use a differenst scope to drop GLOBAL_ALLOC, otherwise could couse a dead_lock

            let alloc_and_print = |size: usize, align: usize| {
                GLOBAL_ALLOC.alloc(core::alloc::Layout::from_size_align(size, align).expect("This creation should not fail"));
                let hhof = &mut*(*GLOBAL_ALLOC.lock().head_of_heap_head.get()).expect("Head of HeapHead must be defined");
                for (i, h) in hhof.into_iter().enumerate() {
                    println!("{} -> {:?}", i, *h);
                }
            };

            alloc_and_print(7, 2);
            alloc_and_print(34, 4);
            //alloc_and_print(523, 8);
            //alloc_and_print(1324, 16);
            //alloc_and_print(13839, 32);
        }

        println!("Allocation layouts test OK");

        let new_box = Box::new(1);
        assert_eq!(1, *new_box);
        println!("Box test OK");

        let n = 1000;
        let mut vec = alloc::vec::Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u64>(), (n-1)*n/2);
        println!("Box test OK");

    }
}

/* TODO find a way to use cargo TEST 
#[cfg(test)]
mod tests {
    use alloc::{ boxed::Box, vec::Vec };

    #[test]
    fn box_allocation_test() {
        let new_box = Box::new(1);
        assert_eq!(1, *new_box);
    }

    #[test]
    fn large_vec() {
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u64>(), (n-1)*n/2);
    }
}
*/
