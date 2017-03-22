use core::ptr::Unique;
use core::fmt::{self, Write};
use spin::Mutex;

#[cfg(any(target_arch = "x86_64"))]
pub unsafe fn outportb(port: u16, val: u8)
{
    asm!("outb %al, %dx" : : "{dx}"(port), "{al}"(val));
}

#[cfg(any(target_arch = "x86_64"))]
pub unsafe fn inportb(port: u16) -> u8
{
    let ret: u8;
    asm!("inb %dx, %al" : "={ax}"(ret): "{dx}"(port));
    ret
}

fn move_cursor(column: usize, row: usize) {
    let crtc_adr : u16 = 0x3D4;
    let offset : u16 = (column + row * 80) as u16;

    unsafe {
        outportb(crtc_adr + 0, 14);
        outportb(crtc_adr + 1, (offset >> 8) as u8);
        outportb(crtc_adr + 0, 15);
        outportb(crtc_adr + 1, offset as u8);
    }
}

#[allow(dead_code)]
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

#[derive(Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer().chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;

                if (col + 1) >= BUFFER_WIDTH {
                    move_cursor(0, row + 1);
                } else {
                    move_cursor(col + 1, row);
                }
            }
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.get_mut() }
    }

    fn new_line(&mut self) {
        if self.row_position == BUFFER_HEIGHT-1 {
            for row in 0..(BUFFER_HEIGHT-1) {
                let buffer = self.buffer();
                buffer.chars[row] = buffer.chars[row + 1];
            }
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;

        let row = self.row_position;
        self.clear_row(row);

        move_cursor(0, row);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer().chars[row] = [blank; BUFFER_WIDTH];
    }

    fn clear_screen(&mut self) {
        for row in 0..(BUFFER_HEIGHT-1) {
            self.clear_row(row)
        }
    }
}

impl ::core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    row_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { Unique::new(0x90002000 as *mut _) },
});

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

macro_rules! print {
    ($($arg:tt)*) => ({
            use core::fmt::Write;
            $crate::vga_buffer::WRITER.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
