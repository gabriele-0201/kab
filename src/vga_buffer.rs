use super::WRITER;
use volatile::Volatile;
use core::fmt;
use super::concurrency::spin_mutex::{ SpinMutex, SpinGuard };
//use lazy_static::lazy_static;
//use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // questo e' come se alla fine la struct fosse tratta come un u8 unico, non wrappato da null
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

impl fmt::Display for ScreenChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "character: {}, ColorCode: {:?}", self.ascii_character as char, self.color_code)
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    //chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    //buffer: &'static mut Buffer,
    buffer: *mut Buffer,
}

impl Writer {

    // This function will init the WRITER static variable
    pub fn init() {

        unsafe {
            WRITER.init(
                SpinMutex::new(Writer {
                    column_position: 0,
                    color_code: ColorCode::new(Color::Yellow, Color::Black),
                    //buffer: &mut *(0xb8000 as *mut Buffer),
                    buffer: 0xb8000 as *mut Buffer,
                })
            );
        }
    }

    // TEST
    pub fn change_ptr_buffer(&mut self, new_buffer_addr: usize) {
        self.buffer = unsafe { &mut *(new_buffer_addr as *mut Buffer) } 
    }

    pub fn get_buffer_addr(&self) -> usize {
        self.buffer as *const Buffer as usize
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                //unsafe{ *self.buffer }.chars[row][col].write(ScreenChar {
                unsafe { 
                    core::ptr::write_volatile(
                        &mut ((*self.buffer).chars[row][col]) as *mut ScreenChar,
                        ScreenChar {
                            ascii_character: byte,
                            color_code,
                        }
                    );
                };
                /*
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
                */
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                /*
                let character = self.buffer.chars[row][col]/*.read()*/;
                //self.buffer.chars[row - 1][col].write(character);
                self.buffer.chars[row - 1][col] = character;
                */
                unsafe {
                    let buffer = &mut *self.buffer;
                    let character = core::ptr::read_volatile(&buffer.chars[row][col] as *const ScreenChar);
                    core::ptr::write_volatile(
                        &mut (buffer.chars[row - 1][col]) as *mut ScreenChar,
                        character
                    );
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            //self.buffer.chars[row][col].write(blank);
            //self.buffer.chars[row][col] = blank;
            unsafe {
                core::ptr::write_volatile(
                    &mut ((*self.buffer).chars[row][col]) as *mut ScreenChar,
                    blank
                );
            }
        }
    }

    pub fn clear_screen(&mut self) {
        for i in 0 .. BUFFER_HEIGHT {
            self.clear_row(i);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }

}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
lazy_static! {
    (pub) static WRITER: SpinMutex<Writer> = SpinMutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        //buffer: 0xb8000 as *const Buffer,
    })
}
*/

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use println;
pub(crate) use print;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // This has to be unsafe until I find a way to have something static but
    // I can initialize on runtime
    unsafe { WRITER.lock().write_fmt(args).unwrap(); }
}

/* OLD
pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}
*/
