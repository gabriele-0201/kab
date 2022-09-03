use super::vga_buffer::{ println, print };

const KERNEL_CODE_SEGMENT_FLAGS: u8 = 0x9A; // 10011010
const KERNEL_DATA_SEGMENT_FLAGS: u8 = 0x92; // 10010010
const USER_CODE_SEGMENT_FLAGS: u8 = 0xFA; // 11110010
const USER_DATA_SEGMENT_FLAGS: u8 = 0xF2; // 11110010
const TASK_STATE_SEGMENT_FLAGS: u8 = 0xF2; // 11110010
                                           
extern {
    // this is extern "C" unsafe
    fn set_gdt(limit: u32, base: *const GDT);
    fn reloadSegments();
}

#[derive(Debug)]
#[repr(C)]
struct SegmentDescriptor {
    low_limit: u16,
    low_base: u16,
    mid_base: u8,
    // 0: A, Access bit, leave it zero, used by CPU
    // 1: RW, Readeble bit if code segment, Writable bit of data segment
    // 2: DC, Direction bit / Confirming bit
    //          Direction: (for data selectors)
    //              0 -> grows up
    //              1 -> grows down
    //              SO the offset could be greater than the limit
    //          Confirming: (for code selectors)
    //              IDK -> something about how can execute the code, based on ring position 
    // 3: E, Executable bit, this describe a data segment (0) or a code segment (1)
    // 4: S, Descriptor type, (0) system segment or (1) code or data segment
    // 5-6: DPL, Descriptor privilege, (0) highest privilege, kernel or (3) lowest privilage, user
    // application => Rings: 0, 1, 2 e 3 
    // 7: P, Present bit - (1) for any valid segment
    access_type: u8,
    high_limit_and_flags: HighLimitAndFlags,
    high_base: u8,
}

impl SegmentDescriptor {

    /// Constructor of a segment descriptor, for now flags are evalueated based on the limit, not
    /// possible to inser some custom flag
    fn new(base: u32, mut limit: u32, access_type: u8) -> Self {

        // limit is more complex to evaluate it could work with 16 o 32 bit:
        //  16 bit:
        //      the limit is less than 65536
        //  32 bit:
        //      the 12 least significant bit could be discarted ONLY if they are all 1
        //          in this case the 12 bits up to one is implicity
        //      if they are not all 1 than the solution is:
        //          make all the 12 bits to 1 but remove the 13th bit,
        //          this solution does not give more limit than expected but could reduce it,
        //          this create a wasta space
        
        let flags: u8;

        if limit == 0 {
            flags = 0;
        }
        else if limit <= 65536 {
            flags = 0x4; // flags for 16 bit mode
        } else {
            flags = 0xC; // flags for 32 bit mode

            if (limit & 0xFFF) == 0xFFF {
                // all 12 bit are 1
                limit = limit >> 12;
            } else {
                // not all 12 bits are 1
                limit = (limit >> 12) - 1;
            }
        }

        SegmentDescriptor {
            low_limit: (limit & 0xFFFF) as u16,
            low_base: (base & 0xFFFF) as u16,
            mid_base: ((base >> 16) & 0xFF) as u8,
            access_type,
            high_limit_and_flags: HighLimitAndFlags::new(limit, flags),
            high_base: ((base >> 24) & 0xFF) as u8
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct HighLimitAndFlags(u8);

impl HighLimitAndFlags {
    /// Limit is surely under 20 bit
    /// Flags has 4 bit
    pub fn new(limit: u32, flags: u8) -> Self {
        //HighLimitAndFlags((((limit >> 16 & 0xF) | (flags << 4) as u32) & 0xFF) as u8)
        HighLimitAndFlags((((limit >> 16 & 0xF) << 4 | flags as u32) & 0xFF) as u8)
    }
}

// How is used the GDT?
// In Protected mode the Segment Registers will save the index of the segment inside the gdt
// so: Physical address = Segment Base (Found from the descriptor GDT[A]) + B
//
// Notes:
//  + segments can overlap
//  + all segment registers are independent
//  + CS cannot change direclty
//  + C compiler ASSUME flat memory model -> 
#[derive(Debug)]
#[repr(C)]
pub struct GDT {
    // SD = SegmentDescriptor
    null_sd: SegmentDescriptor,
    //unused_sd: SegmentDescriptor,
    k_code_sd: SegmentDescriptor, // k = kernel
    k_data_sd: SegmentDescriptor,
    //u_code_sd: SegmentDescriptor, // u = user
    //u_data_sd: SegmentDescriptor,
    //task_state_sd: SegmentDescriptor,
}

#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const GDT,
}

impl GDT {
    /// Create a simply Global Descriptor Table
    pub fn new() -> Self {
        GDT {
            null_sd: SegmentDescriptor::new(0, 0 ,0),
            //unused_sd: SegmentDescriptor::new(0, 0, 0),
            k_code_sd: SegmentDescriptor::new(0, 0xFFFFFFFF, KERNEL_CODE_SEGMENT_FLAGS), 
            k_data_sd: SegmentDescriptor::new(0, 0xFFFFFFFF, KERNEL_DATA_SEGMENT_FLAGS),
            //u_code_sd: SegmentDescriptor::new(0, 0x00FFFFFF, USER_CODE_SEGMENT_FLAGS), 
            //u_data_sd: SegmentDescriptor::new(0, 0x00FFFFFF, USER_DATA_SEGMENT_FLAGS), 
            //task_state_sd: SegmentDescriptor::new((64*1024*1024 * 4) + 1, 0, TASK_STATE_SEGMENT_FLAGS), // size = 64KiB
        }
    }

    pub fn load(&self) {
        // This function should be load the GDT in Protected and Flat Mode
        
        //unsafe { set_gdt(core::mem::size_of::<GDT>() as u32, self); };
        
        let gdt = DescriptorTablePointer {
            limit: core::mem::size_of::<GDT>() as u16 - 1, // maybe the - 1 is wrong
            base: self
        };
        unsafe {
            core::arch::asm!("lgdt [{}]", in(reg) &gdt, options(readonly, nostack, preserves_flags));
            //Self::print_gdt();
            reloadSegments();
        }
    }

    /// Get the address of the current GDT.
    #[no_mangle]
    pub extern "C" fn print_gdt() {
        let mut gdt: DescriptorTablePointer = DescriptorTablePointer {
            limit: 0,
            base: 0 as *const GDT
        };
        unsafe {
            core::arch::asm!("sgdt [{}]", in(reg) &mut gdt, options(nostack, preserves_flags));


            println!("La base della gdt e': {:x}", gdt.base as u32);
            println!("Il limite dalla gdt e': {}", gdt.limit);
            println!("Raw gdt:");
            let mut counter = 0;
            for i in 0..gdt.limit + 1 {
                print!("{:02x}", *(gdt.base as *const u8).offset(i as isize));

                counter += 1;
                if counter == 8 {
                    counter = 0;
                    println!("");
                }
            }

            let n_segments = 3;
            for index_segment in 0..n_segments {
                let base_segment_offset = index_segment * 8;
                let limit = (*(gdt.base as *const u16) as u32) & 
                    (((*(gdt.base as *const u8).offset(base_segment_offset + 6) & 0xF) as u32) << 16);
                let base = (*(gdt.base as *const u16).offset(base_segment_offset + 1) as u32) & 
                    (*(gdt.base as *const u8).offset(base_segment_offset + 4) as u32) &
                    (*(gdt.base as *const u8).offset(base_segment_offset + 7) as u32);
                let access_type = *(gdt.base as *const u8).offset(base_segment_offset + 5);
                let flag = (*(gdt.base as *const u8).offset(base_segment_offset + 6) & 0xF0) >> 4;

                println!("primi 16 bit: {}", (*(gdt.base as *const u16) as u32));
                println!("altri 8 bit: {}", ((*(gdt.base as *const u8).offset(base_segment_offset + 6) & 0xF) as u32));

                println!("limit: {}", limit);
                println!("base: {}", base);
                println!("access_type: {:02x}", access_type);
                println!("flag: {:02x}", flag);
                
            }

            // test
            //println!("{:02x}", (((0xFFFFF >> 16 & 0xF) | (0xC << 4) as u32) & 0xFF) as u8);
            //println!("{:02x}", (((0xFFFFF >> 16 & 0xF) << 4 | 0xC as u32) & 0xFF) as u8);

        }
        //gdt
    }

    /// return the offset of the kernel code segment inside the table
    pub fn get_kernel_code_segment_offset(&self) -> u16 {
        ((&self.k_code_sd as *const SegmentDescriptor) as u32 - (self as *const GDT) as u32) as u16
    }
    pub fn get_kernel_data_segment_offset(&self) -> u16 {
        (&self.k_data_sd as *const SegmentDescriptor) as u16 - (self as *const GDT) as u16
    }
    /*
    pub fn get_user_code_segment_offset(&self) -> u16 {
        (&self.u_code_sd as *const SegmentDescriptor) as u16 - (self as *const GDT) as u16
    }
    pub fn get_user_data_segment_offset(&self) -> u16 {
        (&self.u_code_sd as *const SegmentDescriptor) as u16 - (self as *const GDT) as u16
    }
    */
}
