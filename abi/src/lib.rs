#![feature(lang_items)]
#![feature(asm)]
#![no_std]

pub trait SetDefault {
    fn set_default(&mut self);
}

#[derive(Debug)]
pub struct CapSystemCall<'a> {
    pub target: &'a [u8],
    pub message: CapSendMessage
}

#[derive(Debug, Clone, Copy)]
pub enum CapSendMessage {
    TCBYield
}

#[derive(Debug)]
pub enum SystemCallRequest {
    CPoolDebug(usize),
    Print([u8; 32], usize),
}

#[derive(Debug)]
pub struct TaskBuffer {
    pub request: Option<SystemCallRequest>
}

impl SetDefault for TaskBuffer {
    fn set_default(&mut self) {
        self.request = None;
    }
}
