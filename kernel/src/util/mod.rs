mod object;
mod guard;
mod streamer;
pub mod managed_arc;

#[macro_use]
pub mod field_offset;

pub use self::object::{ExternMutex, ExternReadonlyObject, MutexGuard, MemoryObject};
pub use self::guard::{UniqueReadGuard, UniqueWriteGuard, SharedReadGuard, SharedWriteGuard,
                      RwLock, RwLockReadGuard, RwLockWriteGuard, IndexedSharedReadGuard, IndexedSharedWriteGuard,
                      RefGuard, RefMutGuard};
pub use self::streamer::{Streamer};
pub use spin::{Mutex};

use common::{PAddr, VAddr};
use core::ops::{Deref, DerefMut};
use core::marker::{PhantomData};
use core::cell::{UnsafeCell};

pub fn align_up(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw + (alignment - (raw % alignment))
    };
    PAddr::from(aligned)
}

pub fn align_down(paddr: PAddr, alignment: usize) -> PAddr {
    let raw = paddr.into(): usize;
    let aligned = if raw % alignment == 0 {
        raw
    } else {
        raw - (raw % alignment)
    };
    PAddr::from(aligned)
}

pub fn block_count(length: usize, block_length: usize) -> usize {
    if length % block_length == 0 {
        length / block_length
    } else {
        length / block_length + 1
    }
}
