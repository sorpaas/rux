#![feature(lang_items)]
#![feature(asm)]
#![no_std]

/// A trait that allows setting a struct back to its default value.
pub trait SetDefault {
    /// Set this struct back to its default value.
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

#[derive(Debug, Clone)]
pub enum SystemCall {
    CPoolListDebug,
    Print {
        request: ([u8; 32], usize)
    },
    RetypeCPool {
        request: (usize, usize),
    },
    ChannelTake {
        request: usize,
        response: Option<u64>,
    },
    ChannelPut {
        request: (usize, u64),
    },
    RetypeTask {
        request: (usize, usize),
    },
    TaskSetInstructionPointer {
        request: (usize, u64),
    },
    TaskSetStackPointer {
        request: (usize, u64),
    },
    TaskSetCPool {
        request: (usize, usize),
    },
    TaskSetTopPageTable {
        request: (usize, usize),
    },
    TaskSetBuffer {
        request: (usize, usize),
    },
    TaskSetActive {
        request: usize
    },
    TaskSetInactive {
        request: usize
    },
}

/// Represents a task buffer used for system calls.
#[derive(Debug)]
pub struct TaskBuffer {
    pub call: Option<SystemCall>
}

impl SetDefault for TaskBuffer {
    fn set_default(&mut self) {
        self.call = None;
    }
}
