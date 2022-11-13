use super::paging::VirtualAddr;
use crate::concurrency::spin_mutex::SpinMutex;
use crate::runtime_static::RuntimeStatic;
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    mem::size_of,
};

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
    end_heap: VirtualAddr,
}

#[derive(Clone)]
enum Near {
    HeapHead(*mut HeapHead),
    Tail(VirtualAddr),
    Base(VirtualAddr),
}

impl Near {
    fn get_ptr_usize(&self) -> usize {
        match self {
            Near::HeapHead(hh) => *hh as usize,
            Near::Tail(tail) => tail.get(),
            Near::Base(tail) => tail.get(),
        }
    }
}

impl core::fmt::Debug for Near {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Near::Base(start) => write!(f, "Base: 0x{:X}", start.get() as usize),
            Near::Tail(end) => write!(f, "Tail: 0x{:X}", end.get() as usize),
            Near::HeapHead(h) => write!(f, "Near: 0x{:X}", *h as usize),
        }
    }
}

impl Default for Near {
    fn default() -> Self {
        Near::HeapHead(core::ptr::null_mut())
    }
}

/// Keep the pointer to the next HeapHead and the allocated space whith his dimension
#[repr(align(32))]
struct HeapHead {
    prev: Near,
    next: Near,
    allocated_space: *mut u8,
    dim: usize,
}

struct HeapHeadIterator {
    curr: *mut HeapHead,
}

enum From {
    HeapHead(*mut HeapHead),
    VirtualAddr(VirtualAddr, Near),
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
    unsafe fn write_new(
        heap_head: *mut HeapHead,
        prev: Near,
        next: Near,
        allocated_space: *mut u8,
        dim: usize,
    ) -> Result<(), &'static str> {
        if (heap_head as *mut u8).align_offset(size_of::<HeapHead>()) != 0 {
            return Err("HeapHead MUST be aligned");
        }

        heap_head.write_volatile(HeapHead {
            prev,
            next,
            allocated_space,
            dim,
        });

        return Ok(());
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
        let new_next: Near;
        let prev: Near;

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
                prev = Near::HeapHead(heap_head as *const HeapHead as *mut HeapHead);
            }
            From::VirtualAddr(ref addr, ref next) => {
                start_avaiable_space = addr.get();
                end_avaiable_space = next.get_ptr_usize();
                new_next = next.clone();
                // TODO could be possible to avoid passing a ptr but get it from next.prev, the ptr
                // could be useful only for the first insered elemnet
                prev = Near::Base(addr.clone());
            }
        }

        // skip HeapHead needed padding
        //crate::println!("test padding: {}", (start_avaiable_space as *mut u8).align_offset(size_of::<HeapHead>()));
        //crate::println!("test padding: {}", (start_avaiable_space as *mut HeapHead).align_offset(size_of::<HeapHead>()));
        //crate::println!("new heap_head pos: 0x{:X}", start_avaiable_space);
        let new_heap_head_pos: *mut HeapHead =
            super::next_align(start_avaiable_space, size_of::<HeapHead>());
        //crate::println!("PADDED new heap_head pos: 0x{:X}", new_heap_head_pos as usize);

        // skip layout needed padding
        // the minimun padding is size_of::<u8>() (1byte I hope) where
        // I will store the modulo of the offset to the HeapHead that mange this allocated space
        //crate::println!("new allocated_space pos: 0x{:X}", ((new_heap_head_pos as usize) + size_of::<HeapHead>()) as usize);
        let new_allocated_space: *mut u8 = super::next_align(
            (new_heap_head_pos as usize) + size_of::<HeapHead>() + size_of::<u8>(),
            req_align,
        );
        //crate::println!("PADDED new allocated_space pos: 0x{:X}", new_allocated_space as usize);

        // store the offset
        let offset_ptr = new_allocated_space as usize - new_heap_head_pos as usize;
        if offset_ptr > u8::MAX as usize {
            return Err("Too big needed padding");
        }
        let offset_ptr = offset_ptr as u8;
        // this does not require a feature
        *(new_allocated_space.offset(-1)) = offset_ptr;

        //crate::println!("End avaiable space: 0x{:X}", end_avaiable_space);

        //crate::print!("There is enough space: ");
        // PAY ATTENCTION: end_avaiable_space - new_allocated_space as usize could have a negative
        // result because the starting point of space_allocated can be higher than
        // end_avaiable_space
        // SO use short-circuit-evaluation to avoid this problem
        if end_avaiable_space < new_allocated_space as usize
            || (end_avaiable_space - new_allocated_space as usize) < req_dim
        {
            //crate::println!("NO");
            return Err("Not enough space avaiable");
        }
        //crate::println!("YES");

        let current_near_heap_head = Near::HeapHead(new_heap_head_pos);
        match from {
            From::HeapHead(heap_head) => {
                // heap_head = A
                // A <-> B => A <-> NEW <-> B

                // If A point to B than now B.prev have to be the current
                if let Near::HeapHead(hh_ptr) = (*heap_head).next {
                    ((&mut (*hh_ptr).prev) as *mut Near)
                        .write_volatile(current_near_heap_head.clone());
                }

                // for sure A.next now must become equal to the current new HeapHead
                ((&mut (*heap_head).next) as *mut Near).write_volatile(current_near_heap_head);
            }
            From::VirtualAddr(_, ref next) => {
                // next = A
                // BASE <- A => BASE <- NEW <-> A

                if let Near::HeapHead(hh_ptr) = *next {
                    ((&mut (*hh_ptr).prev) as *mut Near).write_volatile(current_near_heap_head);
                }
            }
        }

        Self::write_new(
            new_heap_head_pos,
            prev,
            new_next,
            new_allocated_space,
            req_dim,
        )?;
        Ok(new_heap_head_pos)
    }

    fn get_end_of_allocated_space(&self) -> usize {
        self.allocated_space as usize + self.dim
    }

    /*
    fn get_adiacent_free_space(&self) -> usize {
        // |heap_head|padding|allocated_space__________|___free_space____|next_heap_head
        self.next.get_ptr_usize() - self.get_end_of_allocated_space()
    }
    */

    fn as_mut_ptr(&mut self) -> *mut HeapHead {
        self as *mut HeapHead
    }

    fn as_ptr(&self) -> *const HeapHead {
        self as *const HeapHead
    }
}

impl core::fmt::Debug for HeapHead {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let padding = self.allocated_space as usize
            - (self as *const HeapHead as usize + size_of::<HeapHead>() as usize);
        /*
        write!(
            f,
            "HeapHead pos: 0x{:X}, prev: {:?}, next: {:?} \n\
               Space handled -> dim: {}, pos: 0x{:X}, pad: {}",
            self as *const HeapHead as usize,
            self.prev,
            self.next,
            self.dim,
            self.allocated_space as usize,
            padding
        )
        */
        write!(f, "Space handled -> dim: {}", self.dim)
        /*
        write!(
            f,
            "HeapHead pos: 0x{:X}, prev: {:?}, next: {:?}",
            self as *const HeapHead as usize, self.prev, self.next
        )
        */
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
            return None;
        }

        let curr_heap_head = unsafe { &mut *self.curr };

        match curr_heap_head.next {
            Near::HeapHead(ptr) => self.curr = ptr,
            Near::Tail(_) => self.curr = core::ptr::null_mut(),
            Near::Base(_) => self.curr = core::ptr::null_mut(),
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
            end_heap: VirtualAddr::new(end_heap),
        }
    }
}

/// Let's start creating a bump allocator
unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        macro_rules! try_insert_from_start_heap {
            ($next: expr) => {
                match HeapHead::try_insert(
                    From::VirtualAddr(self.start_heap.clone(), $next),
                    layout,
                ) {
                    Ok(new_head_head) => {
                        //crate::println!("Head of heap heads: {:?}", *self.head_of_heap_head.get());
                        (self.head_of_heap_head.get() as *mut Option<*mut HeapHead>)
                            .write_volatile(Some(new_head_head as *mut HeapHead));
                        //crate::println!("Head of heap heads: {:?}", *self.head_of_heap_head.get());
                        let new_head_head = unsafe { &*new_head_head };
                        return new_head_head.allocated_space;
                    }
                    Err(_) => () // if impossible insert from here no prob, go on
                };
            };
        }
        // layout should not be 0 and must be a power of 2
        if (*self.head_of_heap_head.get()).is_none() {
            //crate::println!("Insert with NO element inside");
            try_insert_from_start_heap!(Near::Tail(self.end_heap.clone()));
        }

        // here now we have to make sure that the first heap_head is at the start,
        // oherwise check if is possible to create a new head_of_heap_heads
        // TODO: test this when also dealloc is done
        let hohh = (*self.head_of_heap_head.get())
            .expect("Something break in HeapAllocator, Head of heap_heads is null");
        if self.start_heap.get() != hohh as usize {
            //panic!("Insert from base_heap should not be possible, not managed dealloc yet");
            //crate::println!("Insert from the beginnign");
            try_insert_from_start_heap!(Near::HeapHead(hohh));
        }

        // now I will loop in the liked list and check for every heap_head
        // if accept the requested layout in fornt of it
        let mut h_iter = unsafe { &mut *hohh }.into_iter();

        while let Some(heap_head) = h_iter.next() {
            let heap_head = unsafe { &mut *heap_head };

            // not sure if is possible to modify the stufff inside the iterator
            //crate::println!("test inserting");
            if let Ok(heap_head) = HeapHead::try_insert(From::HeapHead(heap_head), layout) {
                //crate::println!("Should be insered correctly");
                let heap_head = unsafe { &*heap_head };
                return heap_head.allocated_space;
            }
        }

        return core::ptr::null_mut();
    }

    // How can I manage deallc better than O(n)
    // Knowing the pointer I can go directly to the start of the allocated space
    // so I can store just before it the pointer to the allocated space
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let offset = *ptr.offset(-1) as isize * (-1);
        //crate::println!("{}", offset);
        let heap_head = &mut *(ptr.offset(offset) as *mut HeapHead);

        if layout.size() != heap_head.dim
            || heap_head.allocated_space.align_offset(layout.align()) != 0
        {
            //panic!("LLLOOLLLL");
            crate::handle_alloc_error(layout);
        }

        let to_update_next = heap_head.next.clone();
        //crate::println!("to_update_next: {:?}", to_update_next);
        let to_update_prev = heap_head.prev.clone();
        //crate::println!("to_update_prev: {:?}", to_update_prev);

        if let Near::Base(_) = to_update_prev {
            match to_update_next {
                Near::Tail(_) => (self.head_of_heap_head.get() as *mut Option<*mut HeapHead>)
                    .write_volatile(None),
                Near::HeapHead(hh_ptr) => (self.head_of_heap_head.get()
                    as *mut Option<*mut HeapHead>)
                    .write_volatile(Some(hh_ptr)),
                _ => panic!("to_update_next can't be a Base"),
            }
        }

        if let Near::HeapHead(ref next_heap_head) = to_update_next {
            (**next_heap_head).prev = to_update_prev.clone();
        }

        if let Near::HeapHead(ref next_heap_head) = to_update_prev {
            (**next_heap_head).next = to_update_next.clone();
        }
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

impl RuntimeStatic<SpinMutex<HeapAllocator>> {
    fn get_head_of_heap_heads(&self) -> Option<*mut HeapHead> {
        unsafe { *self.lock().head_of_heap_head.get() }
    }
}

pub mod tests {
    pub fn home_made_test() {
        use crate::println;
        use crate::GLOBAL_ALLOC;
        use core::alloc::{GlobalAlloc, Layout};

        println!("");
        println!("HOME MADE TEST");

        //println!("HeapHead dimension: {}", size_of::<super::HeapHead>());

        let print_allocated_spaces = || {
            let hhof = match GLOBAL_ALLOC.get_head_of_heap_heads() {
                Some(ptr) => unsafe { &mut *ptr },
                None => {
                    println!("EMPTY heap heads list");
                    return;
                }
            };
            for (i, h) in hhof.into_iter().enumerate() {
                let h = unsafe { &*h };
                println!("{} -> {:?}", i, h);
            }
            println!("");
        };

        unsafe {
            // use a differenst scope to drop GLOBAL_ALLOC, otherwise could couse a dead_lock

            let alloc = |size: usize, align: usize| -> *mut u8 {
                let ptr = GLOBAL_ALLOC.alloc(
                    Layout::from_size_align(size, align).expect("This creation should not fail"),
                );
                if ptr.is_null() {
                    panic!("ALLOCATION GONE WRONG");
                }
                //print_allocated_spaces();
                ptr
            };

            // allocate for blocks
            let start_allocation = alloc(3, 2);
            let middle_1_allocation = alloc(5, 4);
            let middle_2_allocation = alloc(6, 4);
            let end_allocation = alloc(9, 8);

            print_allocated_spaces();

            println!("FINISH ALLOCATION, now start deallocation");

            // deallocate the firs one
            GLOBAL_ALLOC.dealloc(
                start_allocation,
                Layout::from_size_align(3, 2).expect("This creation should not fail"),
            );
            print_allocated_spaces();

            let start_allocation = alloc(3, 2);
            print_allocated_spaces();

            GLOBAL_ALLOC.dealloc(
                middle_1_allocation,
                Layout::from_size_align(5, 4).expect("This creation should not fail"),
            );
            print_allocated_spaces();
            GLOBAL_ALLOC.dealloc(
                start_allocation,
                Layout::from_size_align(3, 2).expect("This creation should not fail"),
            );
            print_allocated_spaces();
            GLOBAL_ALLOC.dealloc(
                end_allocation,
                Layout::from_size_align(9, 8).expect("This creation should not fail"),
            );
            print_allocated_spaces();
            GLOBAL_ALLOC.dealloc(
                middle_2_allocation,
                Layout::from_size_align(6, 4).expect("This creation should not fail"),
            );
            print_allocated_spaces();
        }

        println!("Allocation layouts test OK");

        {
            let new_box = alloc::boxed::Box::new(1);
            assert_eq!(1, *new_box);
            println!("Box test OK");

            let n = 1000;
            let mut vec = alloc::vec::Vec::new();
            for i in 0..n {
                vec.push(i);
            }
            assert_eq!(vec.iter().sum::<u32>(), (n - 1) * n / 2);
            println!("Vec test OK");

            crate::print!("test print vec: ");
            vec.iter().for_each(|i| crate::print!("{}", i));
            crate::println!("");

            let str = alloc::string::String::from("Test");
            crate::println!("{}", str);
            let str_2 = alloc::string::String::from(" - format test");
            crate::println!("{}", format!("{}{}", str, str_2));

            print_allocated_spaces();
        }

        print_allocated_spaces();

        println!("Allocation test FINISHED");
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
