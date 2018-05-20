use common::*;
use core::convert::From;
use util::RwLock;
use util::managed_arc::{ManagedArc, ManagedArcAny};
use abi::ChannelMessage;
use super::{UntypedDescriptor, TaskCap, TaskBufferPageCap};

#[derive(Debug)]
pub enum ChannelValue {
    Raw(u64),
    Cap(ManagedArcAny),
    Payload(TaskBufferPageCap),
}

impl ChannelValue {
    pub fn from_message(message: ChannelMessage, source_root: TaskCap) -> Option<ChannelValue> {
        match message {
            ChannelMessage::Raw(value) => Some(ChannelValue::Raw(value)),
            ChannelMessage::Cap(Some(caddr)) => {
                let source_root = source_root.read().upgrade_cpool().unwrap();
                let obj = source_root.lookup_upgrade_any(caddr);
                if obj.is_some() {
                    Some(ChannelValue::Cap(obj.unwrap()))
                } else {
                    None
                }
            },
            ChannelMessage::Cap(None) => None,
            ChannelMessage::Payload => {
                let source_root = source_root.read().upgrade_buffer().unwrap();
                Some(ChannelValue::Payload(source_root))
            }
        }
    }

    pub fn to_message(value: ChannelValue, target_root: TaskCap) -> ChannelMessage {
        match value {
            ChannelValue::Raw(value) => ChannelMessage::Raw(value),
            ChannelValue::Cap(arc) => {
                let target_root = target_root.read().upgrade_cpool().unwrap();
                let target_desc = target_root.read();
                let index = target_desc.downgrade_any_free(arc);
                ChannelMessage::Cap(index.map(|i| { CAddr::from(i as u8) }))
            },
            ChannelValue::Payload(buffer_cap) => {
                let source_buffer = buffer_cap.read().read();
                let mut target_buffer_cap = target_root.read().upgrade_buffer().unwrap();
                let mut target_buffer = target_buffer_cap.write().write();
                target_buffer.payload_length = source_buffer.payload_length;
                for i in 0..source_buffer.payload_length {
                    target_buffer.payload_data[i] = source_buffer.payload_data[i];
                }
                ChannelMessage::Payload
            }
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
            arc = Some(
                Self::new(paddr, RwLock::new(ChannelDescriptor {
                    value: None,
                    next: next_child,
                }))
            );

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
