use common::*;
use core::mem::size_of;
use core::marker::PhantomData;

mod frame;
mod untyped;
mod pool;
mod paging;

// TODO There seems to be a bug in Rust compiler and as a result,
// VGABufferCapability is not used.
mod vga;
mod utils;

pub use self::untyped::{MemoryBlock, UntypedCapability};
pub use self::frame::{FrameCapability, GuardedFrameCapability};
pub use self::pool::{CapabilityUnion, CapabilityPool, CapabilityMove};
pub use self::vga::{VGABufferCapability};
pub use self::paging::{PageTableCapability, VirtualAddress};

pub trait Capability { }
