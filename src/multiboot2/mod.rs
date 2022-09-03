#[derive(Debug)]
pub enum Syms {
    Symbols {
        tabsize: usize, 
        strsize: usize,
        addr: *const usize,
        reserved: usize
    },
    Elfs {
        num: usize,
        size: usize,
        addr: *const usize,
        shndx: usize
    },
}

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo {
    address: *const usize,
    flag: usize, 
    mem_lower: Option<usize>,
    mem_upper: Option<usize>,
    boot_device: Option<usize>,
    cmd_line_address: Option<*const usize>,
    mods_count: Option<usize>,
    mods_address: Option<*const usize>,
    syms: Option<Syms>,
    mmap_length: Option<usize>, 
    mmap_address: Option<*const usize>
    // for now other values is useless I think
}

impl BootInfo {
    pub fn new(magic: usize, address: usize) -> Result<BootInfo, &'static str> {
        if magic != 0x2BADB002 {
            return Err("Magic number wrong, machine state incorrect")
        }

        unsafe {
            let address = address as *const usize;
            let flag = *address;

            macro_rules! check_flag_and_set {
                ($index: expr, $value: expr) => {
                    if (flag & (0x1 << $index)) == 0 {
                        None
                    } else {
                        Some($value)
                    }

                }
            }

            let mem_lower = check_flag_and_set!(0, *address.offset(1));
            let mem_upper = check_flag_and_set!(0, *address.offset(2));
            let boot_device = check_flag_and_set!(1, *address.offset(3));
            let cmd_line_address = check_flag_and_set!(2, *address.offset(4) as *const usize);
            let mods_count = check_flag_and_set!(3, *address.offset(5));
            let mods_address = check_flag_and_set!(3, *address.offset(6) as *const usize);
            
            // construct syms is more complex, 4 a and 5 are mutually exlusive
            let mut syms = check_flag_and_set!(4, Syms::Symbols{
                tabsize: *address.offset(7),
                strsize: *address.offset(8),
                addr: *address.offset(9) as *const usize,
                reserved: *address.offset(10)
            });

            if let None = syms {
                syms = check_flag_and_set!(5, Syms::Elfs{
                    num: *address.offset(7),
                    size: *address.offset(8),
                    addr: *address.offset(9) as *const usize,
                    shndx: *address.offset(10)
                });
            } // TODO understand why is always null

            let mmap_length = check_flag_and_set!(6, *address.offset(11));
            let mmap_address = check_flag_and_set!(6, *address.offset(12) as *const usize);

            Ok(BootInfo {
                address,
                flag, 
                mem_lower,
                mem_upper,
                boot_device,
                cmd_line_address,
                mods_count,
                mods_address,
                syms,
                mmap_length, 
                mmap_address
            })
        }
    }

    /*
    pub fn get_flag(&self) -> usize {
        unsafe{ *self.address }
    }
    */

    /*
    fn check_flag(&self, index: usize) -> Result<(), &'static str> {
        if (self.flag & 0x1 << index) == 0 {
            // this could be implemented as soos as I have an allocator
            //return Err(format_args!("Flag at index {} not setted", index).as_str().unwrap())
            return Err("Flag not setted")
        }
        Ok(())
    }

    // Get lowe and upper mam, following this order
    pub fn get_mem(&self) -> Result<(usize, usize), &'static str> {
        self.check_flag(0)?;
        Ok(unsafe{ (*self.address.offset(1), *self.address.offset(2)) })
    }

    // still not weel formatted, should be divided in 4 part
    pub fn get_boot_device(&self) -> Result<usize, &'static str> {
        self.check_flag(1)?;
        Ok(unsafe{ *self.address.offset(3) })
    }

    /* IDK how read C string like
    pub fn get_cmdline(&self) -> Result<&str, &'static str> {
        self.check_flag(2)?;
        Ok(unsafe{ self.address.offset(4) })
    }
    */

    pub fn get_mods_count(&self) -> Result<usize, &'static str> {
        self.check_flag(3)?;
        Ok(unsafe{ *self.address.offset(5) })
    }

    /*
    pub fn get_mods(&self) -> Result<ModStructure, &'static str> {
        self.check_flag(3)?;
        Ok(unsafe{ *self.address.offset(6) -> from this address should be constructed all the mods })
    }
    */

    pub fn get_flags(&self) -> Result<usize, &'static str> {
        self.check_flag(3)?;
        Ok(unsafe{ *self.address.offset(5) })
    }

    // Flags 4 and 5 are mutually exclusive
    */


}

/*
impl core::fmt::Debug for BootInfo {
    // implement this to be able to use {:?}

    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {

        /*
        println!("Flag: 0x{:b}", boot_info.get_flag()); // now flag: 1001001111
        println!("lower and upper mem (KiB): {:?}", boot_info.get_mem().unwrap()); 
        println!("boot device: 0x{:x}", boot_info.get_boot_device().unwrap()); 
        */

        const ELF_SECTIONS_LIMIT: usize = 17;
        let mut debug = f.debug_struct("Multiboot2 Boot Information");

        debug
            .field("Start Address", &(self.address as usize))
            .field("Flag", &self.get_flag())
            // TODO should be managed better the flag not setted
            .field("Lower and Upper mem (KiB)", &format_args!("{:?}", self.get_mem().unwrap_or((0, 0))))
            .field("Boot Device", &format_args!("0x{:X}", self.get_boot_device().unwrap()))
            .field("Mods Counter", &self.get_mods_count().unwrap_or(0));

        
        debug.finish()
    }
}
*/

#[derive(Debug)]
struct MemoryMap {
    mods_count: usize,
    mods_current: usize,
    mods_address: *const MemoryMapElement,
}

// use copy to avoid moving out datas from memory
#[derive(Clone, Copy)]
#[repr(C)]
struct MemoryMapElement {
    base_addr: *const usize,
    lenght: usize,
    type_map: usize
}

impl Iterator for MemoryMap {
    type Item = MemoryMapElement;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.mods_current >= self.mods_count {
            return None
        }

        unsafe {
            let elem = Some(*self.mods_address);

            let new_address = self.mods_address as usize + *(self.mods_address as *const usize).offset(-1) as usize;
            self.mods_address = new_address as *const MemoryMapElement;
            elem
        }

    }
}
