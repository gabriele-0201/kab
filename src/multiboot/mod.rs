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
    pub mem_lower: Option<usize>,
    pub mem_upper: Option<usize>,
    pub boot_device: Option<usize>,
    pub cmd_line_address: Option<*const usize>,
    pub mods_count: Option<usize>,
    pub mods_address: Option<*const usize>,
    pub syms: Option<Syms>,
    pub mmap: Option<MemoryMap>
    /*
    mmap_length: Option<usize>, 
    mmap_address: Option<*const usize>
    */
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

            let mmap = check_flag_and_set!(6, 
                MemoryMap {
                    length: check_flag_and_set!(6, *address.offset(11)).unwrap(),
                    start_address: check_flag_and_set!(6, *address.offset(12) as *const MemoryMapElement).unwrap(),
                });

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
                mmap, 
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

#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    length: usize, // length of the buffer, I think in bytes
    pub start_address: *const MemoryMapElement,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMapIterator {
    current: *const MemoryMapElement,
    end: *const MemoryMapElement
}

// use copy to avoid moving out datas from memory
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MemoryMapElement {
    size: usize,
    base_addr: usize,
    reserved_addr: usize,
    lenght: usize,
    reserved_length: usize,
    type_mmap: usize
}

pub struct MemoryMapArea {
    pub base: usize,
    pub length: usize,
    pub type_mmap: usize
}

impl IntoIterator for MemoryMap {
    type Item = MemoryMapArea;
    type IntoIter = MemoryMapIterator;

    fn into_iter(self) -> Self::IntoIter {
        MemoryMapIterator {
            current: self.start_address,
            end: unsafe { self.start_address.byte_add(self.length) }
        }
    }
}

impl Iterator for MemoryMapIterator {
    type Item = MemoryMapArea;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None
        }

        unsafe {
            let elem = *self.current;

            // should be equal to: self.current = self.current.offset(2);
            self.current = self.current.add(1);

            Some(MemoryMapArea {
                    base: elem.base_addr, 
                    length: elem.lenght, 
                    type_mmap: elem.type_mmap
            })
        }

    }
}
