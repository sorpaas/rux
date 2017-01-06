use common::*;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool3Arc};

use super::{UntypedDescriptor};

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
