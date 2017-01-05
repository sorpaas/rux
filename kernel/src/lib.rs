#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(naked_functions)]
#![feature(associated_consts)]
#![feature(type_ascription)]
#![feature(core_intrinsics)]
#![feature(optin_builtin_traits)]
#![feature(drop_types_in_const)]
#![feature(thread_local)]
#![feature(nonzero)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(core_slice_ext)]
#![feature(reflect_marker)]
#![feature(relaxed_adts)]
#![no_std]

extern crate x86;
extern crate spin;
extern crate rlibc;
extern crate abi;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

/// Achitecture-specific modules
#[cfg(target_arch="x86_64")] #[path="arch/x86_64/mod.rs"]
pub mod arch;

/// Exception handling (panic)
pub mod unwind;

/// Logging code
mod logging;

#[macro_use]
mod util;
mod common;
mod elf;
mod cap;

use core::mem;
use core::slice;
use common::*;
use arch::{InitInfo, inportb, outportb, Exception};
use cap::{UntypedCap, CPoolCap, CPoolDescriptor, RawPageCap, TaskBufferPageCap, TopPageTableCap, TaskCap, TaskDescriptor, TaskStatus, ChannelCap, ChannelDescriptor, PAGE_LENGTH};
use core::ops::{Deref, DerefMut};
use abi::{SystemCall, TaskBuffer};
use util::{MemoryObject};
use core::any::{Any, TypeId};

fn bootstrap_rinit_paging(archinfo: &InitInfo, cpool: &mut CPoolCap, untyped: &mut UntypedCap) -> (TopPageTableCap, TaskBufferPageCap, VAddr, VAddr) {
    use elf::{ElfBinary};

    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let rinit_stack_size = 4;
    let rinit_buffer_vaddr = VAddr::from(0x90001000: usize);
    let rinit_vga_vaddr = VAddr::from(0x90002000: usize);
    let mut rinit_entry: u64 = 0x0;

    let mut rinit_pml4 = TopPageTableCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_pml4);

    let slice_object = unsafe { MemoryObject::<u8>::slice(archinfo.rinit_region().start_paddr(),
                                                          archinfo.rinit_region().length()) };
    let bin_raw = unsafe { slice::from_raw_parts(*slice_object,
                                                 archinfo.rinit_region().length()) };
    let bin = ElfBinary::new("rinit", bin_raw).unwrap();

    log!("fheader = {:?}", bin.file_header());
    log!("entry = 0x{:x}", bin.file_header().entry);
    rinit_entry = bin.file_header().entry;

    for p in bin.program_headers() {
        use elf::{PT_LOAD};

        if p.progtype == PT_LOAD {
            log!("pheader = {}", p);
            assert!(p.filesz == p.memsz);

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

                for i in 0..min(page_length, (p.memsz as usize) - offset) {
                    page_raw.0[i] = bin_raw[(p.offset as usize) + offset + i];
                }

                offset += page_length;
                next_page_vaddr += page_length;
            }
        }
    }

    log!("mapping the rinit stack ...");
    for i in 0..rinit_stack_size {
        let mut rinit_stack_page = RawPageCap::retype_from(untyped.write().deref_mut());
        cpool.read().downgrade_free(&rinit_stack_page);
        rinit_pml4.map(rinit_stack_vaddr + i * PAGE_LENGTH, &rinit_stack_page,
                       untyped.write().deref_mut(),
                       cpool.write().deref_mut());
    }

    log!("mapping the rinit task buffer ...");
    let mut rinit_buffer_page = TaskBufferPageCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_buffer_page);
    rinit_pml4.map(rinit_buffer_vaddr, &rinit_buffer_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());

    log!("mapping the rinit vga buffer ...");
    let mut rinit_vga_page = unsafe { RawPageCap::bootstrap(PAddr::from(0xb8000: usize), untyped.write().deref_mut()) };
    cpool.read().downgrade_free(&rinit_vga_page);
    rinit_pml4.map(rinit_vga_vaddr, &rinit_vga_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());

    (rinit_pml4, rinit_buffer_page, VAddr::from(rinit_entry), rinit_stack_vaddr + (PAGE_LENGTH * rinit_stack_size - 4))
}

fn handle_system_call(call: &mut SystemCall, task: &mut TaskDescriptor, cpool: &CPoolDescriptor) {
    match call {
        &mut SystemCall::Print {
            request: ref request
        } => {
            use core::str;
            let buffer = request.0.clone();
            let slice = &buffer[0..request.1];
            let s = str::from_utf8(slice).unwrap();
            log!("Userspace print: {}", s);
        },
        &mut SystemCall::CPoolListDebug => {
            for i in 0..256 {
                let arc = cpool.upgrade_any(i);
                if arc.is_some() {
                    let arc = arc.unwrap();
                    if arc.is::<CPoolCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): CPoolCap);
                    } else if arc.is::<UntypedCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): UntypedCap);
                    } else if arc.is::<TaskCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TaskCap);
                    } else if arc.is::<RawPageCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): RawPageCap);
                    } else if arc.is::<TaskBufferPageCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TaskBufferPageCap);
                    } else if arc.is::<TopPageTableCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): TopPageTableCap);
                    } else if arc.is::<ChannelCap>() {
                        log!("CPool index {} => {:?}", i, arc.into(): ChannelCap);
                    } else {
                        log!("CPool index {} (arch specific) => {:?}", i, arc);
                        cap::drop_any(arc);
                    }
                }
            }
        },
        &mut SystemCall::RetypeCPool {
            request: ref request,
        } => {
            let source: Option<UntypedCap> = cpool.upgrade(request.0);
            if source.is_some() {
                let source = source.unwrap();
                let target = CPoolCap::retype_from(source.write().deref_mut());
                let result = cpool.downgrade_at(&target, request.1);
            }
        },
        &mut SystemCall::ChannelTake {
            request: ref request,
            response: ref mut response,
        } => {
            let mut chan_option: Option<ChannelCap> = cpool.upgrade(*request);
            if let Some(chan) = chan_option {
                task.set_status(TaskStatus::ChannelWait(chan))
            }
        },
        &mut SystemCall::ChannelPut {
            request: ref request,
        } => {
            let chan_option: Option<ChannelCap> = cpool.upgrade(request.0);
            if let Some(chan) = chan_option {
                chan.write().put(request.1);
            }
        }
    }
}

#[no_mangle]
pub fn kmain(archinfo: InitInfo)
{
    log!("archinfo: {:?}", &archinfo);
    let mut region_iter = archinfo.free_regions();

    let (mut cpool, mut untyped) = {
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

    log!("CPool: {:?}", cpool);
    log!("Untyped: {:?}", untyped);

    log!("type_id: {:?}", TypeId::of::<CPoolCap>());
    {
        use util::{RwLock};
        use util::managed_arc::{ManagedArc};
        use cap::{CPoolDescriptor};
        log!("type_id: {:?}", TypeId::of::<ManagedArc<RwLock<CPoolDescriptor>>>());
    }

    {
        let (rinit_pml4, rinit_buffer_page, rinit_entry, rinit_stack) =
            bootstrap_rinit_paging(&archinfo, &mut cpool, &mut untyped);
        let rinit_task_cap = TaskCap::retype_from(untyped.write().deref_mut());
        let mut rinit_task = rinit_task_cap.write();
        rinit_task.set_instruction_pointer(rinit_entry);
        rinit_task.set_stack_pointer(rinit_stack);
        rinit_task.downgrade_cpool(&cpool);
        rinit_task.downgrade_top_page_table(&rinit_pml4);
        rinit_task.downgrade_buffer(&rinit_buffer_page);
    }

    let mut keyboard_cap = ChannelCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_at(&keyboard_cap, 254);

    let mut util_chan_cap = ChannelCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_at(&util_chan_cap, 255);

    util_chan_cap.write().put(0);

    log!("hello, world!");
    arch::enable_timer();
    while true {
        let mut idle = true;

        for task_cap in cap::task_iter() {
            let mut task = task_cap.write();
            let exception = match task.status() {
                TaskStatus::Active => {
                    idle = false;
                    Some(task.switch_to())
                },
                TaskStatus::ChannelWait(ref chan) => {
                    let value = chan.write().take();
                    if let Some(value) = value {
                        let buffer = task.upgrade_buffer();
                        let mut buffer_desc = buffer.as_ref().unwrap().write().write();
                        let system_call = buffer_desc.deref_mut().call.as_mut().unwrap();
                        match system_call {
                            &mut SystemCall::ChannelTake {
                                request: ref request,
                                response: ref mut response,
                            } => {
                                idle = false;
                                *response = Some(value);
                                task.set_status(TaskStatus::Active);
                                Some(task.switch_to())
                            }
                            _ => panic!(),
                        }
                    } else {
                        None
                    }
                }
            };
            match exception {
                Some(Exception::SystemCall) => {
                    let cpool = task.upgrade_cpool();
                    let buffer = task.upgrade_buffer();
                    handle_system_call(buffer.as_ref().unwrap().write().write().deref_mut().call.as_mut().unwrap(),
                                       task.deref_mut(),
                                       cpool.as_ref().unwrap().read().deref());
                },
                Some(Exception::Keyboard) => {
                    keyboard_cap.write().put(unsafe { arch::inportb(0x60) } as u64);
                },
                _ => (),
            }
        }

        if idle {
            let exception = cap::idle();
            match exception {
                Exception::Keyboard => {
                    keyboard_cap.write().put(unsafe { arch::inportb(0x60) } as u64);
                },
                _ => (),
            }
        }
    }
    
    loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
