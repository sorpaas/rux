/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arch/amd64/mod.rs
 * - Top-level file for amd64 architecture
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

// x86 port IO 
#[path = "io.rs"]
mod x86_io;

// Debug output channel (uses serial)
#[path = "debug.rs"]
pub mod debug;

extern {
    pub static multiboot_sig: u32;
    pub static multiboot_ptr: u64;
}
