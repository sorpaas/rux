#![feature(lang_items)]
#![feature(asm)]
#![no_std]

mod caddr;

pub use caddr::CAddr;

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
        request: (CAddr, CAddr),
    },
    ChannelTake {
        request: CAddr,
        response: Option<ChannelMessage>,
    },
    ChannelPut {
        request: (CAddr, ChannelMessage),
    },
    RetypeTask {
        request: (CAddr, CAddr),
    },
    TaskSetInstructionPointer {
        request: (CAddr, u64),
    },
    TaskSetStackPointer {
        request: (CAddr, u64),
    },
    TaskSetCPool {
        request: (CAddr, CAddr),
    },
    TaskSetTopPageTable {
        request: (CAddr, CAddr),
    },
    TaskSetBuffer {
        request: (CAddr, CAddr),
    },
    TaskSetActive {
        request: CAddr
    },
    TaskSetInactive {
        request: CAddr
    },
}

/// Represents a task buffer used for system calls.
pub struct TaskBuffer {
    pub call: Option<SystemCall>,
    pub payload_length: usize,
    pub payload_data: [u8; 1024],
}

impl SetDefault for TaskBuffer {
    fn set_default(&mut self) {
        self.call = None;
    }
}

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    Raw(u64),
    Cap(Option<CAddr>),
    Payload,
}
