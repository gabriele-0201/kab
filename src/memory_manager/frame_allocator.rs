use super::*;

// Start allocating frame from stack_top + stack_frame_max

pub trait Allocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn deallocate(&mut self, to_deallocate: Frame);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    /// constructor from a physical address
    pub fn from_physical_address(addr: paging::PhysicalAddr) -> Self {
        Self {
            number: addr.get() / FRAME_SIZE + if addr.get() % FRAME_SIZE != 0 { 1 } else { 0 },
        }
    }

    /// constructor from a frame number
    pub fn from_frame_number(frame_number: usize) -> Self {
        Self {
            number: frame_number,
        }
    }

    fn next(&self) -> Self {
        Self {
            number: self.number + 1,
        }
    }

    pub fn get_physical_addr(&self) -> paging::PhysicalAddr {
        paging::PhysicalAddr::new(FRAME_SIZE * self.number)
    }
}

#[derive(Debug)]
struct Stack<T: Default> {
    stack_top: *const T,
    stack_ptr: *mut T,
}

impl<T: Default> Stack<T> {
    fn new(stack_top: *const T) -> Self {
        Self {
            stack_top,
            stack_ptr: stack_top as *mut T,
        }
    }

    fn pop(&mut self) -> Option<T> {
        //crate::println!("stack ptr is: {:?}", self.stack_ptr);
        //crate::println!("something popped");
        if self.stack_top == self.stack_ptr {
            return None;
        }

        self.stack_ptr = ((self.stack_ptr as usize) + core::mem::size_of::<T>()) as *mut T;
        //crate::println!("stack ptr is now: {:?}", self.stack_ptr);
        Some(unsafe { core::mem::take(&mut *self.stack_ptr) })
    }

    // should be manage the overflow???
    fn push(&mut self, val: T) {
        //crate::println!("stack ptr is: {:?}", self.stack_ptr);
        //crate::println!("somethign pushed");
        unsafe { *self.stack_ptr = val };
        self.stack_ptr = ((self.stack_ptr as usize) - core::mem::size_of::<T>()) as *mut T;
        //crate::println!("stack ptr is now: {:?}", self.stack_ptr);
    }
}

// What is needed by the FrameAllocator?
// + current_frame -> pointer to a frame ready to be allocated
// + stack_ptr -> pointer to the stack that store all the free frame
// + total number of avaiable frame
// + reference to boot info?
#[derive(Debug)]
pub struct FrameAllocator {
    max_frame: usize,
    pub first_avaiable_frame: Frame,
    current_frame: Frame,
    stack: Stack<usize>,
}

impl FrameAllocator {
    // create a new frame allocator object and a stack to manage it
    pub fn new(starting_point: usize, boot_info: &BootInfo) -> FrameAllocator {
        // Extract the number of total frame
        // mem_upper and lower are in kilobytes
        let total_memory = 0x100000
            + (boot_info
                .mem_upper
                .expect("Mem Upper not present in multiboot information")
                * 0x400);
        //crate::println!("tot mem: {}", total_memory);
        let max_frame = total_memory / FRAME_SIZE;

        // set up the stack ptr
        // starting from the starting_point we need to reserve the space for a stack
        // this stack will manage all the deallocate frame
        // the dimension of the stack is number of frame and each is described with an usize
        let stack_top =
            unsafe { (starting_point + (max_frame * core::mem::size_of::<usize>())) as *mut usize };

        let stack = Stack::new(stack_top);

        // The frame start from 0
        // Starting from the next frame from the position indicated by the starting point
        // Of course there is some internal framgemntation between starting_point and the next init frame
        let current_frame = Frame::from_physical_address(PhysicalAddr::new(stack_top as usize));
        let first_avaiable_frame = current_frame.clone();

        Self {
            max_frame,
            first_avaiable_frame,
            current_frame,
            stack,
        }
    }
}

impl Allocator for FrameAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        if self.max_frame == self.current_frame.number {
            // the counter is end, search in the the stack
            // panic if all frames are allocated for now
            self.stack.pop().map(|n| Frame::from_frame_number(n))
        } else {
            let new_frame = self.current_frame.clone();
            self.current_frame = self.current_frame.next();
            Some(new_frame)
        }
    }

    fn deallocate(&mut self, to_deallocate: Frame) {
        self.stack.push(to_deallocate.number);
    }
}
