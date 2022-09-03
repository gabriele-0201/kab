use super::{ *, interrupt_manager::*};
use core::arch::asm;

// TODO make a macro to create all the handlers
extern {
    pub fn interruptIgnore();

    pub fn handleException0x00();
    pub fn handleException0x01();
    pub fn handleException0x02();
    pub fn handleException0x03();
    pub fn handleException0x04();
    pub fn handleException0x05();
    pub fn handleException0x06();
    pub fn handleException0x07();
    pub fn handleException0x08();
    pub fn handleException0x09();
    pub fn handleException0x0A();
    pub fn handleException0x0B();
    pub fn handleException0x0C();
    pub fn handleException0x0D();
    pub fn handleException0x0E();
    pub fn handleException0x0F();
    pub fn handleException0x10();
    pub fn handleException0x11();
    pub fn handleException0x12();
    pub fn handleException0x13();
    
    pub fn handleInterruptRequest0x00();
    pub fn handleInterruptRequest0x01();
    pub fn handleInterruptRequest0x02();
    pub fn handleInterruptRequest0x03();
    pub fn handleInterruptRequest0x04();
    pub fn handleInterruptRequest0x05();
    pub fn handleInterruptRequest0x06();
    pub fn handleInterruptRequest0x07();
    pub fn handleInterruptRequest0x08();
    pub fn handleInterruptRequest0x09();
    pub fn handleInterruptRequest0x0A();
    pub fn handleInterruptRequest0x0B();
    pub fn handleInterruptRequest0x0C();
    pub fn handleInterruptRequest0x0D();
    pub fn handleInterruptRequest0x0E();
    pub fn handleInterruptRequest0x0F();
}

// TODO change in a future
static mut INTERRUPT_MANAGER_PTR: Option<*const IDT> = None;

/// This gate could be Interrupt Gate or Trap Gate 
/// Each entry have 64bit
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
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
            low_ptr: ((ptr as *const unsafe extern "C" fn()) as u32 & 0xFFFF) as u16,
            segment_selector,
            reserved: 0,
            access_type: 0x80 /* = IDT_ENTRY_PRESENT*/ | ((ring & 3) << 5) | descriptor_type,
            high_ptr: (((ptr as *const unsafe extern "C" fn()) as u32 >> 16) & 0xFFFF) as u16,
        }
    }

    // using an array with the default initialization will cause weird stuff
    pub fn update(&mut self, ptr: unsafe extern "C" fn(), segment_selector: u16, ring: u8, descriptor_type: u8) {

        self.low_ptr = ((ptr as *const fn()) as u32 & 0xFFFF) as u16;
        self.segment_selector = segment_selector;
        self.access_type = 0x80 /* = IDT_ENTRY_PRESENT*/ | ((ring & 3) << 5) | descriptor_type;
        self.high_ptr = (((ptr as *const fn()) as u32 >> 16) & 0xFFFF) as u16;
          
    }
}

#[repr(C, packed(2))]
struct IDTDescriptor {
    size: u16,
    ptr: *const GateDescritor
}

//#[repr(C)]
#[repr(align(16))]
pub struct IDT {
    idt: [GateDescritor; 256],
    // used because the IRQ start from 0 but in the cpu the relative entry
    // int he idt is offsetted by a custom value
    hw_interrupt_offset: u8, 
    handlers: [Option<fn(&IDT, u32) -> u32>; 256],
    // TODO find a way to make those visible from the manager, pub maybe not the best solution
    pub pic_master_command: Port8Bit,
    pub pic_master_data: Port8Bit,
    pub pic_slave_command: Port8Bit,
    pub pic_slave_data: Port8Bit,
}

// IDK if is usefull to update the IDT onoging or is fixed after initialization
impl IDT {
    pub fn new(interrupt_offset: u8, gdt: &GDT) -> Self {
        let code_segment = gdt.get_kernel_code_segment_offset();

        // TODO check if not use & moves the function
        let mut handlers: [Option<fn(&IDT, u32) -> u32>; 256] = [None; 256];

        // set up handlers
        handlers[(interrupt_offset + 0x00) as usize] = Some(handle_pit);
        handlers[(interrupt_offset + 0x01) as usize] = Some(handle_keyboard_interrupt);

        let mut idt_struct = IDT {
            idt: [GateDescritor::new(interruptIgnore, code_segment, 0, 0xE); 256],
            hw_interrupt_offset: interrupt_offset,
            handlers,
            pic_master_command: Port8Bit::new(0x20),
            pic_master_data: Port8Bit::new(0x21),
            pic_slave_command: Port8Bit::new(0xA0),
            pic_slave_data: Port8Bit::new(0xA1)
        };

        // SET UP THE ENTRY OF THE IDT
        /*
        for i in 0..255 {
            idt_struct.idt[i] = GateDescritor::new(interruptIgnore, code_segment, 0, 0xE);
        }
        */
        
        // maybe update is broken
        // idt_struct.idt[0x00] = GateDescritor::new(handleException0x00, code_segment, 0, 0xE);
        idt_struct.idt[0x00].update(handleException0x00, code_segment, 0, 0xE);
        idt_struct.idt[0x01].update(handleException0x01, code_segment, 0, 0xE);
        idt_struct.idt[0x02].update(handleException0x02, code_segment, 0, 0xE);
        idt_struct.idt[0x03].update(handleException0x03, code_segment, 0, 0xE);
        idt_struct.idt[0x04].update(handleException0x04, code_segment, 0, 0xE);
        idt_struct.idt[0x05].update(handleException0x05, code_segment, 0, 0xE);
        idt_struct.idt[0x06].update(handleException0x06, code_segment, 0, 0xE);
        idt_struct.idt[0x07].update(handleException0x07, code_segment, 0, 0xE);
        idt_struct.idt[0x08].update(handleException0x08, code_segment, 0, 0xE);
        idt_struct.idt[0x09].update(handleException0x09, code_segment, 0, 0xE);
        idt_struct.idt[0x0A].update(handleException0x0A, code_segment, 0, 0xE);
        idt_struct.idt[0x0B].update(handleException0x0B, code_segment, 0, 0xE);
        idt_struct.idt[0x0C].update(handleException0x0C, code_segment, 0, 0xE);
        idt_struct.idt[0x0D].update(handleException0x0D, code_segment, 0, 0xE);
        idt_struct.idt[0x0E].update(handleException0x0E, code_segment, 0, 0xE);
        idt_struct.idt[0x0F].update(handleException0x0F, code_segment, 0, 0xE);
        idt_struct.idt[0x10].update(handleException0x10, code_segment, 0, 0xE);
        idt_struct.idt[0x11].update(handleException0x11, code_segment, 0, 0xE);
        idt_struct.idt[0x12].update(handleException0x12, code_segment, 0, 0xE);
        idt_struct.idt[0x13].update(handleException0x13, code_segment, 0, 0xE);

        // not sure about the correctenss of update -> maybe more correct create a different one
        idt_struct.idt[(interrupt_offset + 0x00) as usize].update(handleInterruptRequest0x00, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x01) as usize].update(handleInterruptRequest0x01, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x02) as usize].update(handleInterruptRequest0x02, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x03) as usize].update(handleInterruptRequest0x03, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x04) as usize].update(handleInterruptRequest0x04, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x05) as usize].update(handleInterruptRequest0x05, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x06) as usize].update(handleInterruptRequest0x06, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x07) as usize].update(handleInterruptRequest0x07, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x08) as usize].update(handleInterruptRequest0x08, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x09) as usize].update(handleInterruptRequest0x09, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0A) as usize].update(handleInterruptRequest0x0A, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0B) as usize].update(handleInterruptRequest0x0B, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0C) as usize].update(handleInterruptRequest0x0C, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0D) as usize].update(handleInterruptRequest0x0D, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0E) as usize].update(handleInterruptRequest0x0E, code_segment, 0, 0xE);
        idt_struct.idt[(interrupt_offset + 0x0F) as usize].update(handleInterruptRequest0x0F, code_segment, 0, 0xE);
        
        //// Comunicate with PIC master and slave
        idt_struct.pic_master_command.write(0x11);
        idt_struct.pic_slave_command.write(0x11);

        //// remap
        // set up the offsets
        idt_struct.pic_master_data.write(interrupt_offset as u8);
        idt_struct.pic_slave_data.write((interrupt_offset+8) as u8);

        // tell to the master that there is a slave and vicecersa
        idt_struct.pic_master_data.write(0x04);
        idt_struct.pic_slave_data.write(0x02);

        // additional information about the environment
        idt_struct.pic_master_data.write(0x01);
        idt_struct.pic_slave_data.write(0x01);

        // restore saved masks ??
        idt_struct.pic_master_data.write(0x00);
        idt_struct.pic_slave_data.write(0x00);

        // return idt
        idt_struct
    }

    pub fn load(&self) {
        
        // Load the idt
        unsafe {
            let idt_descriptor = IDTDescriptor {
                size: ((core::mem::size_of::<GateDescritor>() * self.idt.len()) - 1) as u16,
                ptr: &self.idt[0] as *const GateDescritor
            }; 
            asm!("lidt [{}]", in(reg) &idt_descriptor , options(readonly, nostack, preserves_flags));
        }
    }
    
    /// Enable interrupts.
    /// This is a wrapper around the `sti` instruction.
    #[inline]
    pub fn enable(&self) {
        unsafe {
            asm!("sti", options(nomem, nostack));
            // set up the pointer used to manage the interrupts
            if let None = INTERRUPT_MANAGER_PTR {
                INTERRUPT_MANAGER_PTR = Some(self);
            }
            // init also the drivers => not the best place in the future
            init_drivers();
        }
        println!("Activated interrupts!");
    }
    
    /// Disable interrupts.
    /// This is a wrapper around the `cli` instruction.
    #[inline]
    pub fn disable(&self) {
        unsafe {
            asm!("cli", options(nomem, nostack));
        }
    }

    pub fn do_handle_interrupt(&self, interrupt_number: u8, esp: u32) -> u32 {
    
        let new_esp;

        /*
        if interrupt_number != 0x20 {
            println!("Interrupt 0x{:02x}", interrupt_number);
        }
        */

        if let Some(handler) = self.handlers[interrupt_number as usize] {
                new_esp = handler(self, esp);
        } else {
            println!("Interrupt 0x{:02x} not managed!", interrupt_number);
            new_esp = esp;
        }
    
        if interrupt_number >= self.hw_interrupt_offset && interrupt_number < self.hw_interrupt_offset + 16 {
            self.pic_master_command.write(0x20);
            if interrupt_number >= self.hw_interrupt_offset + 8 {
                self.pic_slave_command.write(0x20);
            }
        }
    
        new_esp
    }
}

#[no_mangle]
pub extern "C" fn handle_interrupts(interrupt_number: u8, esp: u32) -> u32 {
    unsafe {
        if let Some(interrupt_handler) = INTERRUPT_MANAGER_PTR {
           if let Some(interrupt_handler) = <*const IDT>::as_ref(interrupt_handler) {
                return interrupt_handler.do_handle_interrupt(interrupt_number, esp)
           } 
        } 
        esp
    }
}
