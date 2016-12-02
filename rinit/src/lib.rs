#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![no_std]

#[macro_use]
extern crate system;
extern crate spin;

#[macro_use]
mod vga_buffer;

use core::ops::{Deref};

/// Decode a code in the PS/2 scan code set 1 (legacy set).
///
/// Difference between set 1 and sets 2 & 3:
///   http://wiki.osdev.org/%228042%22_PS/2_Controller#Translation
///
/// Reference table:
///   http://www.computer-engineering.org/ps2keyboard/scancodes1.html
pub fn from_scancode_printable(code: usize) -> Option<char> {
    let printable = match code {
        0x1e => 'a',
        0x30 => 'b',
        0x2e => 'c',
        0x20 => 'd',
        0x12 => 'e',
        0x21 => 'f',
        0x22 => 'g',
        0x23 => 'h',
        0x17 => 'i',
        0x24 => 'j',
        0x25 => 'k',
        0x26 => 'l',
        0x32 => 'm',
        0x31 => 'n',
        0x18 => 'o',
        0x19 => 'p',
        0x10 => 'q',
        0x13 => 'r',
        0x1f => 's',
        0x14 => 't',
        0x16 => 'u',
        0x2f => 'v',
        0x11 => 'w',
        0x2d => 'x',
        0x15 => 'y',
        0x2c => 'z',
        0x0b => '0',
        0x02 => '1',
        0x03 => '2',
        0x04 => '3',
        0x05 => '4',
        0x06 => '5',
        0x07 => '6',
        0x08 => '7',
        0x09 => '8',
        0x0a => '9',
        0x29 => '`',
        0x0c => '-',
        0x0d => '=',
        0x2b => '\\',
        0x39 => ' ',
        0x1a => '[',
        0x1b => ']',
        0x27 => ';',
        0x28 => '\'',
        0x33 => ',',
        0x34 => '.',
        0x35 => '/',
        0x37 => '*', // Keypad
        0x4a => '-', // Keypad
        0x4e => '+', // Keypad
        _ => return None,
    };

    Some(printable)
}

pub fn from_scancode(code: usize) -> Key {
    if code == 0x1C {
        Key::Enter
    } else {
        let printable = from_scancode_printable(code);
        if printable.is_some() {
            Key::Printable(printable.unwrap())
        } else {
            Key::Nonprintable
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum Key {
    Printable(char),
    Enter,
    Nonprintable
}

#[lang="start"]
#[no_mangle]
fn start(_argc: isize, _argv: *const *const u8) {
    system::set_task_buffer(0x90001000);
    system_print!("hello, world!");
    print!(">>> ");
    let mut lastkey = Key::Nonprintable;
    let mut command = [0u8; 32];
    let mut command_size = 0;
    while true {
        let key = from_scancode(unsafe { vga_buffer::inportb(0x60) } as usize);
        if key == lastkey {
            continue;
        } else {
            lastkey = key.clone();
        }
        match key {
            Key::Printable(c) => {
                print!("{}", c);
                if command_size < 32 {
                    command[command_size] = c as u8;
                    command_size += 1;
                }
            }
            Key::Enter => {
                print!("\n");
                execute_command(::core::str::from_utf8(&command[0..command_size]).unwrap());
                command = [0u8; 32];
                command_size = 0;
            }
            _ => (),
        }
    }
    loop {};
}

fn execute_command(s: &str) {
    if s == "list" {
        print!("Listing task cpool ...\n");
        system::cpool_list_debug();
    } else if s.len() >= 6 && &s[0..4] == "echo" {
        print!("{}\n", &s[5..s.len()]);
    } else if s.len() >= 16 && &s[0..12] == "retype cpool" {
        let st = &s[13..s.len()];
        let mut split = st.split(' ');
        let source: usize = split.next().unwrap().parse().unwrap();
        let target: usize = split.next().unwrap().parse().unwrap();
        system::retype_cpool(source, target);
        print!("Operation finished.\n");
    } else {
        print!("Unknown command.\n");
    }
    print!(">>> ");
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
