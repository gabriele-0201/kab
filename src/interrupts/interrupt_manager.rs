use super::{*, idt::IDT };
use crate::print;

pub fn init_drivers() {
    init_keyboard();
}

// pit = programmable interrupt timer
pub fn handle_pit(_idt: &IDT, esp: u32) -> u32 {
    //print!(".");
    esp
}

// this function should only need the data and command port but still get all the idt -> do it
// better in the future
pub fn handle_keyboard_interrupt(idt: &IDT, esp: u32) -> u32 {
    // less efficient ever

    // is this the better option?
    static mut shift: bool = false;

    let data_port = Port8Bit::new(0x60);
    let command_port = Port8Bit::new(0x64);

    let scancode = data_port.read();

    // for now avoid releasing

    unsafe {
        match scancode {
            0x02 => if !shift { print!("1") } else { print!("!") } ,
            0x03 => if !shift { print!("2") } else { print!("@") },
            0x04 => if !shift { print!("3") } else { print!("#") },
            0x05 => if !shift { print!("4") } else { print!("$") },
            0x06 => if !shift { print!("5") } else { print!("%") },
            0x07 => if !shift { print!("6") } else { print!("^") },
            0x08 => if !shift { print!("7") } else { print!("&") },
            0x09 => if !shift { print!("8") } else { print!("*") },
            0x0A => if !shift { print!("9") } else { print!("(") },
            0x0B => if !shift { print!("0") } else { print!(")") },

            0x10 => if !shift { print!("q") } else { print!("Q") },
            0x11 => if !shift { print!("w") } else { print!("W") },
            0x12 => if !shift { print!("e") } else { print!("E") },
            0x13 => if !shift { print!("r") } else { print!("R") },
            0x14 => if !shift { print!("t") } else { print!("T") },
            0x15 => if !shift { print!("z") } else { print!("Z") },
            0x16 => if !shift { print!("u") } else { print!("U") },
            0x17 => if !shift { print!("i") } else { print!("I") },
            0x18 => if !shift { print!("o") } else { print!("O") },
            0x19 => if !shift { print!("p") } else { print!("P") },

            0x1E => if !shift { print!("a") } else { print!("A") },
            0x1F => if !shift { print!("s") } else { print!("S") },
            0x20 => if !shift { print!("d") } else { print!("D") },
            0x21 => if !shift { print!("f") } else { print!("F") },
            0x22 => if !shift { print!("g") } else { print!("G") },
            0x23 => if !shift { print!("h") } else { print!("H") },
            0x24 => if !shift { print!("j") } else { print!("J") },
            0x25 => if !shift { print!("k") } else { print!("K") },
            0x26 => if !shift { print!("l") } else { print!("L") },

            0x2C => if !shift { print!("y") } else { print!("Y") },
            0x2D => if !shift { print!("x") } else { print!("X") },
            0x2E => if !shift { print!("c") } else { print!("C") },
            0x2F => if !shift { print!("v") } else { print!("V") },
            0x30 => if !shift { print!("b") } else { print!("B") },
            0x31 => if !shift { print!("n") } else { print!("N") },
            0x32 => if !shift { print!("m") } else { print!("M") },
            0x33 => if !shift { print!(",") } else { print!("<") },
            0x34 => if !shift { print!(".") } else { print!(">") },
            0x35 => if !shift { print!("-") } else { print!("_") },
            // to be added more signes

            0x1C => print!("\n"),
            0x39 => print!(" "),

            0x2A => shift = true, // press shift
            0xAA => shift = false, // release shift

            _ => {
                // avodi dealing with releas keycode
                if scancode < 0x80 {
                    println!("{:02x}", scancode);
                }
            }

        }
    }

    esp
}

pub fn init_keyboard() {
    let data_port = Port8Bit::new(0x60);
    let command_port = Port8Bit::new(0x64);
    
    while (command_port.read() & 0x1) == 0x1 {
        data_port.read();
    }

    command_port.write(0xAE); // init
    command_port.write(0x20); // give us the current state
    let status = (data_port.read() | 1) & !0x10; // set the right most bit to 1 and clear the 5th bit
    command_port.write(0x60); 
    data_port.write(status);
    data_port.write(0xF4); // activate the keyboard
}
