#![no_std]

#[no_mangle]
pub fn foo(x: u32, y: u32) -> u32 {
    x + y
}
