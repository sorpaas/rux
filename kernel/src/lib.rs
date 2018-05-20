#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
#![feature(unique)]
#![feature(naked_functions)]
#![feature(type_ascription)]
#![feature(core_intrinsics)]
#![feature(optin_builtin_traits)]
#![feature(thread_local)]
#![feature(nonzero)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(core_slice_ext)]
#![feature(reflect_marker)]
#![feature(core_slice_ext)]
#![feature(ptr_internals)]
#![no_std]

extern crate spin;
extern crate rlibc;
extern crate abi;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

/// A log macro, used together with architecture-specific logging
/// function that outputs kernel debug messages to I/O ports.
// This mod should load before everything else
#[macro_use]
mod macros;

/// Achitecture-specific modules.
#[cfg(target_arch="x86_64")] #[path="arch/x86_64/mod.rs"]
#[macro_use]
pub mod arch;

/// Exception handling (panic). See also
/// [Unwinding](https://doc.rust-lang.org/nomicon/unwinding.html).
pub mod unwind;

/// Logging writer for use with the log macro.
mod logging;

/// Utils for managed Arc, spinning guard, memory objects and others.
#[macro_use]
mod util;

/// Memory region, virtual address and physical address
/// representation.
mod common;

/// Decoding ELF format, for initializing the user-space rinit program.
mod elf;

/// Capabilities implementation.
#[macro_use]
mod cap;

/// System call handler.
mod system_calls;

use core::slice;
use common::*;
use arch::{InitInfo, Exception};
use cap::{UntypedCap, CPoolCap, RawPageCap, TaskBufferPageCap, TopPageTableCap, TaskCap, TaskStatus, ChannelCap, ChannelValue, PAGE_LENGTH};
use core::ops::DerefMut;
use abi::SystemCall;
use util::MemoryObject;
use core::any::TypeId;

/// Map a stack for the rinit program using the given physical address
/// and stack size.
fn map_rinit_stack(rinit_stack_vaddr: VAddr, rinit_stack_size: usize,
                   cpool: &mut CPoolCap, untyped: &mut UntypedCap, rinit_pml4: &mut TopPageTableCap) {
    for i in 0..rinit_stack_size {
        let mut rinit_stack_page = RawPageCap::retype_from(untyped.write().deref_mut());
        cpool.read().downgrade_free(&rinit_stack_page);
        rinit_pml4.map(rinit_stack_vaddr + i * PAGE_LENGTH, &rinit_stack_page,
                       untyped.write().deref_mut(),
                       cpool.write().deref_mut());
    }
}

/// Map a task buffer for the rinit program.
fn map_rinit_buffer(rinit_buffer_vaddr: VAddr,
                    cpool: &mut CPoolCap, untyped: &mut UntypedCap, rinit_pml4: &mut TopPageTableCap)
                    -> TaskBufferPageCap {
    let rinit_buffer_page = TaskBufferPageCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_buffer_page);
    rinit_pml4.map(rinit_buffer_vaddr, &rinit_buffer_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());
    return rinit_buffer_page;
}

/// Bootstrap paging for the rinit program. This creates stacks and
/// task buffers for both a "parent" and a "child".
fn bootstrap_rinit_paging(archinfo: &InitInfo, cpool: &mut CPoolCap, untyped: &mut UntypedCap) -> (TopPageTableCap, TaskBufferPageCap, VAddr, VAddr) {
    use elf::{ElfBinary};

    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let rinit_child_stack_vaddr = VAddr::from(0x70000000: usize);
    let rinit_stack_size = 4;
    let rinit_buffer_vaddr = VAddr::from(0x90001000: usize);
    let rinit_vga_vaddr = VAddr::from(0x90002000: usize);
    let rinit_child_buffer_vaddr = VAddr::from(0x90003000: usize);

    let mut rinit_pml4 = TopPageTableCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_pml4);

    let slice_object = unsafe { MemoryObject::<u8>::slice(archinfo.rinit_region().start_paddr(),
                                                          archinfo.rinit_region().length()) };
    let bin_raw = unsafe { slice::from_raw_parts(slice_object.as_ptr(),
                                                 archinfo.rinit_region().length()) };
    let bin = ElfBinary::new("rinit", bin_raw).unwrap();

    let rinit_entry = bin.file_header().entry;
    log!("fheader = {:?}", bin.file_header());
    log!("entry = 0x{:x}", rinit_entry);

    for p in bin.program_headers() {
        use elf::{PT_LOAD};

        if p.progtype == PT_LOAD {
            log!("pheader = {}", p);

            let mut next_page_vaddr = VAddr::from(p.vaddr);
            let mut offset = 0x0;
            let end_vaddr = VAddr::from(p.vaddr + p.memsz as usize);

            while next_page_vaddr <= end_vaddr {
                use core::cmp::{min};
                log!("mapping from: 0x{:x}", next_page_vaddr);

                let page_cap = RawPageCap::retype_from(untyped.write().deref_mut());
                cpool.read().downgrade_free(&page_cap);
                rinit_pml4.map(next_page_vaddr, &page_cap,
                               untyped.write().deref_mut(),
                               cpool.write().deref_mut());

                let mut page = page_cap.write();
                let page_length = page.length();
                let mut page_raw = page.write();

                for i in 0..page_length {
                    if i < (p.filesz as usize) - offset {
                        page_raw.0[i] = bin_raw[(p.offset as usize) + offset + i];
                    } else {
                        page_raw.0[i] = 0;
                    }
                }

                offset += page_length;
                next_page_vaddr += page_length;
            }
        }
    }

    log!("mapping the rinit stack ...");
    map_rinit_stack(rinit_stack_vaddr, rinit_stack_size, cpool, untyped, &mut rinit_pml4);

    log!("mapping the child rinit stack ...");
    map_rinit_stack(rinit_child_stack_vaddr, rinit_stack_size, cpool, untyped, &mut rinit_pml4);

    log!("mapping the rinit task buffer ...");
    let rinit_buffer_page = map_rinit_buffer(rinit_buffer_vaddr, cpool, untyped, &mut rinit_pml4);
    let rinit_child_buffer_page = map_rinit_buffer(rinit_child_buffer_vaddr, cpool, untyped, &mut rinit_pml4);

    cpool.read().downgrade_at(&rinit_child_buffer_page, 250);

    log!("mapping the rinit vga buffer ...");
    let rinit_vga_page = unsafe { RawPageCap::bootstrap(PAddr::from(0xb8000: usize), untyped.write().deref_mut()) };
    cpool.read().downgrade_free(&rinit_vga_page);
    rinit_pml4.map(rinit_vga_vaddr, &rinit_vga_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());

    (rinit_pml4, rinit_buffer_page, VAddr::from(rinit_entry), rinit_stack_vaddr + (PAGE_LENGTH * rinit_stack_size - 4))
}

/// The kernel main function. It initialize the rinit program, and
/// then run a loop to switch to all available tasks.
#[no_mangle]
pub fn kmain(archinfo: InitInfo)
{
    log!("archinfo: {:?}", &archinfo);
    let mut region_iter = archinfo.free_regions();

    let (mut cpool_cap, mut untyped_cap) = {
        let cpool_target_region = region_iter.next().unwrap();

        let untyped = unsafe { UntypedCap::bootstrap(cpool_target_region.start_paddr(),
                                                     cpool_target_region.length()) };
        let cpool = CPoolCap::retype_from(untyped.write().deref_mut());

        cpool.read().downgrade_at(&cpool, 0);
        cpool.read().downgrade_free(&untyped);

        let mut untyped_target = untyped;

        for region in region_iter {
            let untyped = unsafe { UntypedCap::bootstrap(region.start_paddr(),
                                                         region.length()) };
            cpool.read().downgrade_free(&untyped);

            if untyped.read().length() > untyped_target.read().length() {
                untyped_target = untyped;
            }
        }

        (cpool, untyped_target)
    };

    log!("CPool: {:?}", cpool_cap);
    log!("Untyped: {:?}", untyped_cap);

    log!("type_id: {:?}", TypeId::of::<CPoolCap>());
    {
        use util::{RwLock};
        use util::managed_arc::{ManagedArc};
        use cap::{CPoolDescriptor};
        log!("type_id: {:?}", TypeId::of::<ManagedArc<RwLock<CPoolDescriptor>>>());
    }

    {
        let (rinit_pml4, rinit_buffer_page, rinit_entry, rinit_stack) =
            bootstrap_rinit_paging(&archinfo, &mut cpool_cap, &mut untyped_cap);
        let rinit_task_cap = TaskCap::retype_from(untyped_cap.write().deref_mut());
        let mut rinit_task = rinit_task_cap.write();
        rinit_task.set_instruction_pointer(rinit_entry);
        rinit_task.set_stack_pointer(rinit_stack);
        rinit_task.set_status(TaskStatus::Active);
        rinit_task.downgrade_cpool(&cpool_cap);
        rinit_task.downgrade_top_page_table(&rinit_pml4);
        rinit_task.downgrade_buffer(&rinit_buffer_page);
    }

    let keyboard_cap = ChannelCap::retype_from(untyped_cap.write().deref_mut());
    cpool_cap.read().downgrade_at(&keyboard_cap, 254);

    let util_chan_cap = ChannelCap::retype_from(untyped_cap.write().deref_mut());
    cpool_cap.read().downgrade_at(&util_chan_cap, 255);

    log!("hello, world!");
    arch::enable_timer();
    loop {
        let mut idle = true;

        for task_cap in cap::task_iter() {
            let status = task_cap.read().status();
            let exception = match status {
                TaskStatus::Inactive => None,
                TaskStatus::Active => {
                    idle = false;
                    Some(task_cap.write().switch_to())
                },
                TaskStatus::ChannelWait(ref chan) => {
                    let value = chan.write().take();
                    if let Some(value) = value {
                        let system_call: SystemCall = {
                            let buffer_cap = task_cap.read().upgrade_buffer().unwrap();
                            let buffer_desc = buffer_cap.read();
                            let buffer = buffer_desc.read();
                            buffer.call.clone().unwrap()
                        };
                        let ret_system_call = match system_call {
                            SystemCall::ChannelTake {
                                request, ..
                            } => {
                                Some(SystemCall::ChannelTake {
                                    request: request,
                                    response: Some(ChannelValue::to_message(
                                        value,
                                        task_cap.clone()))
                                })
                            }
                            _ => panic!(),
                        };
                        if ret_system_call.is_some() {
                            let buffer_cap = task_cap.read().upgrade_buffer().unwrap();
                            let mut buffer_desc = buffer_cap.write();
                            let mut buffer = buffer_desc.write();
                            buffer.call = ret_system_call;
                        }
                        task_cap.write().set_status(TaskStatus::Active);
                        Some(task_cap.write().switch_to())
                    } else {
                        None
                    }
                }
            };
            match exception {
                Some(Exception::SystemCall) => {
                    let cpool_cap = task_cap.read().upgrade_cpool().unwrap();
                    let system_call: SystemCall = {
                        let buffer_cap = task_cap.read().upgrade_buffer().unwrap();
                        let buffer_desc = buffer_cap.read();
                        let buffer = buffer_desc.read();
                        buffer.call.clone().unwrap()
                    };
                    let ret_system_call = system_calls::handle(
                        system_call,
                        task_cap.clone(),
                        cpool_cap.clone());
                    if ret_system_call.is_some() {
                        let buffer_cap = task_cap.read().upgrade_buffer().unwrap();
                        let mut buffer_desc = buffer_cap.write();
                        let mut buffer = buffer_desc.write();
                        buffer.call = ret_system_call;
                    }
                },
                Some(Exception::Keyboard) => {
                    keyboard_cap.write().put(ChannelValue::Raw(unsafe { arch::inportb(0x60) } as u64));
                },
                _ => (),
            }
        }

        if idle {
            let exception = cap::idle();
            match exception {
                Exception::Keyboard => {
                    keyboard_cap.write().put(ChannelValue::Raw(unsafe { arch::inportb(0x60) } as u64));
                },
                _ => (),
            }
        }
    }
}

// fn divide_by_zero() {
//     unsafe {
//         asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
//     }
// }
