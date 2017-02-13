use common::*;
use core::convert::From;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool3Arc};
use abi::ChannelMessage;
use super::{UntypedDescriptor, CPoolCap};

#[derive(Debug)]
pub enum ChannelValue {
    Raw(u64),
    Cap(ManagedArcAny),
}

impl ChannelValue {
    pub fn from_message(message: ChannelMessage, source_root: CPoolCap) -> Option<ChannelValue> {
        match message {
            ChannelMessage::Raw(value) => Some(ChannelValue::Raw(value)),
            ChannelMessage::Cap(Some(caddr)) => {
                let obj = source_root.lookup_upgrade_any(caddr);
                if obj.is_some() {
                    Some(ChannelValue::Cap(obj.unwrap()))
                } else {
                    None
                }
            },
            ChannelMessage::Cap(None) => None,
        }
    }

    pub fn to_message(value: ChannelValue, target_root: CPoolCap) -> ChannelMessage {
        match value {
            ChannelValue::Raw(value) => ChannelMessage::Raw(value),
            ChannelValue::Cap(arc) => ChannelMessage::Cap(target_root.read().downgrade_any_free(arc)
                                                          .map(|i| { CAddr::from(i as u8) })),
        }
    }
}

/// Channel descriptor.
#[derive(Debug)]
pub struct ChannelDescriptor {
    value: Option<ChannelValue>,
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
    pub fn put(&mut self, value: ChannelValue) {
        self.value = Some(value);
    }

    /// Take a value from the channel. If there's no value in the
    /// channel, `None` is returned.
    pub fn take(&mut self) -> Option<ChannelValue> {
        self.value.take()
    }
}
