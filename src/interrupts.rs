use super::gdt::GDT;
use super::port::Port8Bit;

extern {
    fn interruptIgnore();
    fn handleInterruptRequest0x00();
}

/// This gate could be Interrupt Gate or Trap Gate 
#[derive(Clone, Copy)]
#[repr(C)]
struct GateDescritor {
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

    pub fn update(&mut self, ptr: unsafe extern "C" fn(), segment_selector: u16, ring: u8, descriptor_type: u8) {

        self.low_ptr = ((ptr as *const fn()) as u32 & 0xFFFF) as u16;
        self.segment_selector = segment_selector;
        self.access_type = 0x80 /* = IDT_ENTRY_PRESENT*/ | ((ring & 3) << 5) | descriptor_type;
        self.high_ptr = (((ptr as *const fn()) as u32 >> 16) & 0xFFFF) as u16;
          
    }
}

struct IDT {
    idt: [GateDescritor; 256],
    // used because the IRQ start from 0 but in the cpu the relative entry
    // int he idt is offsetted by a custom value
    hw_interrupt_offset: u32, 
    pic_master_command: Port8Bit,
    pic_master_data: Port8Bit,
    pic_slave_command: Port8Bit,
    pic_slave_data: Port8Bit,
}

// IDK if is usefull to update the IDT onoging or is fixed after initialization
impl IDT {
    pub fn new(interrupt_offset: u16, gdt: &GDT) -> Self {
        let code_segment = gdt.get_kernel_code_segment_offset();
        let mut idt_struct = IDT {
            idt: [GateDescritor::new(interruptIgnore, code_segment, 0, 0xE); 256],
            hw_interrupt_offset: interrupt_offset,
            pic_master_command: Port8Bit::new(0x20),
            pic_master_data: Port8Bit::new(0x21),
            pic_slave_command: Port8Bit::new(0xA0),
            pic_slave_data: Port8Bit::new(0xA1)
        };

        idt_struct.idt[0].update(handleInterruptRequest0x00, code_segment, 0, 0xE);

        // Comunicate with PIC master and slave
        idt_struct.pic_master_command.write(0x11);
        idt_struct.pic_slave_command.write(0x11);

        // remap
        idt_struct.pic_master_data.write(interrupt_offset as u8);
        idt_struct.pic_slave_data.write((interrupt_offset+8) as u8);

        idt_struct.pic_master_data.write(0x04);
        idt_struct.pic_slave_data.write(0x02);

        idt_struct.pic_master_data.write(0x01);
        idt_struct.pic_slave_data.write(0x01);

        idt_struct.pic_master_data.write(0x00);
        idt_struct.pic_slave_data.write(0x00);

        
        // Load the idt
        // AAAHHHH INLINE ASSMBLY
        

        // return idt
        idt_struct
    }
}

extern "C" fn handle_interrupt(interrupt_number: u8, esp: u32) {
    
}
