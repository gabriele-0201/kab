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

#[derive(Clone)]
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

enum From {
    HeapHead(*mut HeapHead),
    VirtualAddr(VirtualAddr, Next)
}

impl HeapHead {
    /// this function is similar to a constructor with the only difference that
    /// does not return a new HeapHead but write it on the passed address with
    /// the specified properties
    ///
    /// Does not return any error, it must be unsafe because you could overwrite 
    /// thigs you wouldn't
    ///
    /// Return error if heap_head pointer is not aligned
    unsafe fn write_new(heap_head: *mut HeapHead, next: Next, allocated_space: *mut u8, dim: usize) -> Result<(), &'static str> {

        if (heap_head as *mut u8).align_offset(core::mem::size_of::<HeapHead>()) != 0 {
            return Err("HeapHead MUST be aligned")
        }

        heap_head.write_volatile(
            HeapHead {
                next,
                allocated_space,
                dim
            }
        );

        return Ok(())
    }

    /// Helper function use to try insert new HeadHead after the argument From
    ///
    /// try to insert a new layout between this HeapHead/prt and the next one,
    /// this function will carry about padding and HeapHead's linked list
    /// management
    unsafe fn try_insert(from: From, layout: Layout) -> Result<*const HeapHead, &'static str> {

        // TODO maybe a simple check at the beginnign could make a lot faster the 
        // iteration through all the stuff

        let req_dim = layout.size(); // req = requested
        let req_align = layout.align();
        let new_next: Next;

        let start_avaiable_space: usize;
        let end_avaiable_space: usize;

        // First thing is evaluate if enough space is avaiable
        // requested_space: 
        // heap_head_padding + heap_head_size + alloc_space_padding + alloc_space_size

        // witch are the information I need from here?
        // start_avaiable_space, end_avaiable_space, next
        match from {
            From::HeapHead(ref heap_head) => {
                let heap_head = &**heap_head;
                start_avaiable_space = heap_head.get_end_of_allocated_space();
                end_avaiable_space = heap_head.next.get_ptr_usize();
                new_next = heap_head.next.clone();
            },
            From::VirtualAddr(ref addr, ref next) => {
                start_avaiable_space = addr.get();
                end_avaiable_space = next.get_ptr_usize();
                new_next = next.clone();
            }
        }

        // skip HeapHead needed padding
        crate::println!("test padding: {}", (start_avaiable_space as *mut u8).align_offset(core::mem::size_of::<HeapHead>()));
        //crate::println!("test padding: {}", (start_avaiable_space as *mut HeapHead).align_offset(core::mem::size_of::<HeapHead>()));
        crate::println!("new heap_head pos: 0x{:X}", start_avaiable_space);
        let new_heap_head_pos: *mut HeapHead = super::next_align(
            start_avaiable_space, 
            core::mem::size_of::<HeapHead>()
        );
        crate::println!("PADDED new heap_head pos: 0x{:X}", new_heap_head_pos as usize);

        // skip layout needed padding
        crate::println!("new allocated_space pos: 0x{:X}", ((new_heap_head_pos as usize) + core::mem::size_of::<HeapHead>()) as usize);
        let new_allocated_space: *mut u8 = super::next_align(
            (new_heap_head_pos as usize) + core::mem::size_of::<HeapHead>(), 
            req_align
        );
        crate::println!("PADDED new allocated_space pos: 0x{:X}", new_allocated_space as usize);

        if (end_avaiable_space - new_allocated_space as usize) < req_dim {
            return Err("Not enough space avaiable")
        }

        // remember to update the From::HeapHead next to the just created HeapHead
        if let From::HeapHead(heap_head) = from {
            (*heap_head).next = Next::HeapHead(new_heap_head_pos);
        }

        Self::write_new(new_heap_head_pos, new_next, new_allocated_space, req_dim)?;
        Ok(new_heap_head_pos)
    }

    /*
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
    */

    fn get_end_of_allocated_space(&self) -> usize {
        self.allocated_space as usize + self.dim
    }

    fn get_adiacent_free_space(&self) -> usize {
        // |heap_head|padding|allocated_space__________|___free_space____|next_heap_head
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
                crate::println!("isert from the beginggin");
                let new_head_head = match HeapHead::try_insert(From::VirtualAddr(self.start_heap.clone(), $next), layout) {
                    Ok(new_head_head) => new_head_head,
                    Err(_) => return core::ptr::null_mut()
                };
                *self.head_of_heap_head.get() = Some(new_head_head as *mut HeapHead);
                let new_head_head = unsafe { &*new_head_head };
                return new_head_head.allocated_space
            }
        }
        // layout should not be 0 and must be a power of 2
        if (*self.head_of_heap_head.get()).is_none() {
            try_insert_from_start_heap!(Next::Tail(self.end_heap.clone()));
        }

        // here now we have to make sure that the first heap_head is at the start,
        // oherwise check if is possible to create a new head_of_heap_heads
        // TODO: test this when also dealloc is done
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
            crate::println!("testing HeapHead");
            if let Ok(heap_head) = HeapHead::try_insert(From::HeapHead(heap_head), layout) {
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
            // This not work
            //alloc_and_print(107, 8);
            //alloc_and_print(1324, 16);
            //alloc_and_print(13839, 32);
        }

        println!("Allocation layouts test OK");

        /*
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
        */

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
