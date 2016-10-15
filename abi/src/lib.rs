#![feature(lang_items)]
#![feature(asm)]
#![no_std]

#[derive(Debug)]
pub struct CapSystemCall<'a> {
    pub target: &'a [u8],
    pub message: CapSendMessage
}

#[derive(Debug, Clone, Copy)]
pub enum CapSendMessage {
    TCBYield
}
