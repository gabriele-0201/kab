use super::vga_buffer::println;
use super::gdt::GDT;
use super::port::Port8Bit;
use core::arch::asm;

extern {
    pub fn interruptIgnore();
    pub fn handleInterruptRequest0x00();
    pub fn handleInterruptRequest0x01();
    pub fn handleException0x00();
}

/// This gate could be Interrupt Gate or Trap Gate 
#[derive(Clone, Copy, Debug)]
#[repr(C)]
//#[repr(align(16))] this fuck everything, why???
pub/*test*/ struct GateDescritor {
    low_ptr: u16,
    segment_selector: u16,
    reserved: u8,
    access_type: u8, 
    high_ptr: u16,
}

impl GateDescritor {
    /// ptr: pointer to the handler
    /// segment_selector: selector of the correct code segment -> switch of cs
    /// ring: privilage level
    /// descriptor_type: descriptor of the entry
    pub fn new(ptr: unsafe extern "C" fn(), segment_selector: u16, ring: u8, descriptor_type: u8) -> Self {
        GateDescritor {
            low_ptr: ((ptr as *const fn()) as u32 & 0xFFFF) as u16,
            segment_selector,
            reserved: 0,
            access_type: 0x80 /* = IDT_ENTRY_PRESENT*/ | ((ring & 3) << 5) | descriptor_type,
            high_ptr: (((ptr as *const fn()) as u32 >> 16) & 0xFFFF) as u16,
        }
    }

    /* using an array with the default initialization will cause weird stuff
    pub fn update(&mut self, ptr: unsafe extern "C" fn(), segment_selector: u16, ring: u8, descriptor_type: u8) {

        self.low_ptr = ((ptr as *const fn()) as u32 & 0xFFFF) as u16;
        self.segment_selector = segment_selector;
        self.access_type = 0x80 /* = IDT_ENTRY_PRESENT*/ | ((ring & 3) << 5) | descriptor_type;
        self.high_ptr = (((ptr as *const fn()) as u32 >> 16) & 0xFFFF) as u16;
          
    }
    */
}

#[repr(C)]
struct IDTDescriptor {
    size: u16,
    //ptr: *const GateDescritor
    ptr: u32
}

#[repr(C)]
pub struct IDT {
    pub idt: [GateDescritor; 256],
    // used because the IRQ start from 0 but in the cpu the relative entry
    // int he idt is offsetted by a custom value
    pub hw_interrupt_offset: u16, 
    pub pic_master_command: Port8Bit,
    pub pic_master_data: Port8Bit,
    pub pic_slave_command: Port8Bit,
    pub pic_slave_data: Port8Bit,
}


// IDK if is usefull to update the IDT onoging or is fixed after initialization
impl IDT {
    pub fn new(interrupt_offset: u16, gdt: &GDT) /*-> Self*/ {
        let code_segment = gdt.get_kernel_code_segment_offset();

        println!("Before the IDT construct");

        let mut idt_struct = IDT {
            idt: [GateDescritor::new(interruptIgnore, code_segment, 0, 0xE); 256],
            hw_interrupt_offset: interrupt_offset,
            pic_master_command: Port8Bit::new(0x20),
            pic_master_data: Port8Bit::new(0x21),
            pic_slave_command: Port8Bit::new(0xA0),
            pic_slave_data: Port8Bit::new(0xA1)
        };

        for i in 0..255 {
            idt_struct.idt[i] = GateDescritor::new(interruptIgnore, code_segment, 0, 0xE);
        }

        println!("After the IDT construct");

        println!("{}: {:?}", 0, idt_struct.idt[0]);
        //println!("somethig after the print of the first entry");
        //println!("len idt: {}", idt_struct.idt.len());
        //println!("{}: {:?}", 1, idt_struct.idt[1]);
        //println!("{}: {:?}", 2, idt_struct.idt[2]);

        //idt_struct.idt[0x00].update(handleException0x00, code_segment, 0, 0xE);
        //idt_struct.idt[0x00] = GateDescritor::new(handleException0x00, code_segment, 0, 0xE);

        ////idt_struct.idt[(interrupt_offset + 0x00) as usize].update(handleInterruptRequest0x00, code_segment, 0, 0xE);
        //idt_struct.idt[(interrupt_offset + 0x00) as usize]= GateDescritor::new(handleInterruptRequest0x00, code_segment, 0, 0xE);
        ////idt_struct.idt[(interrupt_offset + 0x01) as usize].update(handleInterruptRequest0x01, code_segment, 0, 0xE);
        //idt_struct.idt[(interrupt_offset + 0x01) as usize] = GateDescritor::new(handleInterruptRequest0x01, code_segment, 0, 0xE);

        //// Comunicate with PIC master and slave
        //idt_struct.pic_master_command.write(0x11);
        //idt_struct.pic_slave_command.write(0x11);

        //// remap
        //idt_struct.pic_master_data.write(interrupt_offset as u8);
        //idt_struct.pic_slave_data.write((interrupt_offset+8) as u8);

        //idt_struct.pic_master_data.write(0x04);
        //idt_struct.pic_slave_data.write(0x02);

        //idt_struct.pic_master_data.write(0x01);
        //idt_struct.pic_slave_data.write(0x01);

        //idt_struct.pic_master_data.write(0x00);
        //idt_struct.pic_slave_data.write(0x00);

        // return idt
        //idt_struct
    }

    /*
    pub fn load(&self) {
        
        // Load the idt
        unsafe {
            let idt_descriptor = IDTDescriptor {
                size: ((core::mem::size_of::<GateDescritor>() * self.idt.len()) - 1) as u16,
                //ptr: &self.idt[0] as *const GateDescritor
                ptr: &self.idt[0] as *const _ as u32
            }; 
            asm!("lidt [{}]", in(reg) &idt_descriptor , options(readonly, nostack, preserves_flags));
        }
    }
    */
}

pub fn activate() {
    unsafe {
        // STI sets the interrupt flag (IF) in the EFLAGS register
        asm!("sti" , options(readonly, nostack, preserves_flags));
    }
}

#[no_mangle]
pub extern "C" fn handle_interrupt(interrupt_number: u8, esp: u32) -> u32 {
    println!("Interrupt: {}", interrupt_number);
    esp
}
