use core::arch::asm;

/*
macro_rules! porty_type {
    ($name: ident, $type: ty) => {

        struct $name {
            port_number: u16,
        }
        
        impl $name {
        
            pub fn write(&self, val: $type) {
                asm!("out dx, al", in("dx") self.port_number, 
                     ($type: u8) => { in("al") }
                     ($type: u16) => { in("ax") }
                     ($type: u32) => { in("eax") }
                     val
                     , options(nomem, nostack, preserves_flags));
            }
        
            pub fn read(&self) -> $type {
                let val: $type;
                asm!("in al, dx", out("al") val, in("dx") self.port_number, options(nomem, nostack, preserves_flags));
                val
            }
        
        }
    };
}

porty_type!(Port8Bit, u8);
porty_type!(Port16Bit, u16);
porty_type!(Port32Bit, u32);
*/

pub struct Port8Bit {
    port_number: u16,
}

impl Port8Bit {

    pub fn new(port_number: u16) -> Self {
        Port8Bit { port_number }
    }

    pub fn write(&self, val: u8) {
        unsafe { 
            asm!("out dx, al", in("dx") self.port_number, in("al") val, options(nomem, nostack, preserves_flags)); 
        }
    }

    pub fn read(&self) -> u8 {
        let val: u8;
        unsafe { asm!("in al, dx", out("al") val, in("dx") self.port_number, options(nomem, nostack, preserves_flags)); }
        val
    }
}

pub struct Port16Bit {
    port_number: u16,
}

impl Port16Bit {

    pub fn write(&self, val: u16) {
        unsafe { 
            asm!("out dx, al", in("dx") self.port_number, in("ax") val, options(nomem, nostack, preserves_flags)); 
        }
    }

    pub fn read(&self) -> u16 {
        let val: u16;
        unsafe { asm!("in al, dx", out("ax") val, in("dx") self.port_number, options(nomem, nostack, preserves_flags)); }
        val
    }
}

pub struct Port32Bit {
    port_number: u32,
}

impl Port32Bit {

    pub fn write(&self, val: u32) {
        unsafe { 
            asm!("out dx, al", in("dx") self.port_number, in("eax") val, options(nomem, nostack, preserves_flags)); 
        }
    }

    pub fn read(&self) -> u32 {
        let val: u32;
        unsafe { asm!("in al, dx", out("eax") val, in("dx") self.port_number, options(nomem, nostack, preserves_flags)); }
        val
    }
}
