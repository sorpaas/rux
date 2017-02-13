use common::*;
use core::convert::From;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool3Arc};
use abi::ChannelMessage;
use super::{UntypedDescriptor, CPoolCap};

pub enum ChannelValue {
    Raw(u64),
    Cap(CPoolCap, CAddr),
}

impl ChannelValue {
    pub fn from_message(message: ChannelMessage, cpool: CPoolCap) -> Option<ChannelValue> {
        match message {
            ChannelMessage::Raw(value) => Some(ChannelValue::Raw(value)),
            ChannelMessage::Cap(Some(caddr)) => Some(ChannelValue::Cap(cpool, caddr)),
            ChannelMessage::Cap(None) => None,
        }
    }

    pub fn to_message(value: ChannelValue) -> ChannelMessage {
        match value {
            ChannelValue::Raw(value) => ChannelMessage::Raw(value),
            ChannelValue::Cap(cpool, caddr) => ChannelMessage::Cap(Some(caddr)),
        }
    }
}

/// Channel descriptor.
#[derive(Debug)]
pub struct ChannelDescriptor {
    value: Option<u64>,
    next: Option<ManagedArcAny>,
}
/// Channel capability. Reference-counted smart pointer to channel
/// descriptor.
///
/// Channels are used for inter-process communication of different
/// tasks.
pub type ChannelCap = ManagedArc<RwLock<ChannelDescriptor>>;

impl ChannelCap {
    /// Create a channel capability from an untyped capability.
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        unsafe { untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
            arc = Some(unsafe {
                Self::new(paddr, RwLock::new(ChannelDescriptor {
                    value: None,
                    next: next_child,
                }))
            });

            arc.clone().unwrap().into()
        }) };

        arc.unwrap()
    }
}

impl ChannelDescriptor {
    /// Put a value to the channel.
    pub fn put(&mut self, value: u64) {
        self.value = Some(value);
    }

    /// Take a value from the channel. If there's no value in the
    /// channel, `None` is returned.
    pub fn take(&mut self) -> Option<u64> {
        self.value.take()
    }
}
