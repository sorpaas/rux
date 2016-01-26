use core::ptr::{self, Unique};
use core::mem;

#[lang = "owned_box"]
pub struct Box<T: ?Sized>(Unique<T>);

