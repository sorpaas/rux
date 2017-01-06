/// External readonly object helpers.
mod object;

/// Lock guard helpers.
mod guard;

/// Streaming iterator
mod streamer;

/// Managed reference-counted pointers that erases all weak pointers
/// when the last strong pointer goes out.
pub mod managed_arc;

/// Get the offset of a struct field.
#[macro_use]
pub mod field_offset;

pub use self::object::{ExternMutex, ExternReadonlyObject, MutexGuard, MemoryObject};
pub use self::guard::{UniqueReadGuard, UniqueWriteGuard};
pub use self::streamer::{Streamer};
pub use spin::{Mutex, RwLock};

use common::{PAddr, VAddr};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};
use core::cell::{UnsafeCell};

/// Align the physical address up using the alignment.
pub fn align_up(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw + (alignment - (raw % alignment))
    };
    PAddr::from(aligned)
}

/// Align the physical address down using the alignment.
pub fn align_down(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw - (raw % alignment)
    };
    PAddr::from(aligned)
}

/// Count blocks needed for the length.
pub fn block_count(length: usize, block_length: usize) -> usize {
    if length % block_length == 0 {
        length / block_length
    } else {
        length / block_length + 1
    }
}
