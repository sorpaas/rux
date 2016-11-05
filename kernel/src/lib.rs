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
use arch::{InitInfo};
use cap::{UntypedCap, CPoolCap, RawPageCap, TaskBufferPageCap, TopPageTableCap, TaskCap, PAGE_LENGTH};
use core::ops::{Deref, DerefMut};
use abi::{SystemCall, TaskBuffer};
use util::{MemoryObject};
use core::any::{Any, TypeId};

fn bootstrap_rinit_paging(archinfo: &InitInfo, cpool: &mut CPoolCap, untyped: &mut UntypedCap) -> (TopPageTableCap, TaskBufferPageCap, VAddr, VAddr) {
    use elf::{ElfBinary};

    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let rinit_buffer_vaddr = VAddr::from(0x80001000: usize);
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
    let mut rinit_stack_page = RawPageCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_stack_page);
    rinit_pml4.map(rinit_stack_vaddr, &rinit_stack_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());

    log!("mapping the rinit buffer ...");
    let mut rinit_buffer_page = TaskBufferPageCap::retype_from(untyped.write().deref_mut());
    cpool.read().downgrade_free(&rinit_buffer_page);
    rinit_pml4.map(rinit_buffer_vaddr, &rinit_buffer_page,
                   untyped.write().deref_mut(),
                   cpool.write().deref_mut());

    (rinit_pml4, rinit_buffer_page, VAddr::from(rinit_entry), rinit_stack_vaddr + (PAGE_LENGTH - 4))
}

fn handle_system_call(call: &mut SystemCall) {
    match call {
        &mut SystemCall::Print {
            request: ref request,
            response: ref response,
        } => {
            use core::str;
            let buffer = request.0.clone();
            let slice = &buffer[0..request.1];
            let s = str::from_utf8(slice).unwrap();
            log!("Userspace print: {}", s);
        },
        any => {
            log!("Not yet handled system call: {:?}", any);
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

    let (rinit_pml4, rinit_buffer_page, rinit_entry, rinit_stack) =
        bootstrap_rinit_paging(&archinfo, &mut cpool, &mut untyped);
    let rinit_task_cap = TaskCap::retype_from(untyped.write().deref_mut());
    let mut rinit_task = rinit_task_cap.write();
    rinit_task.set_instruction_pointer(rinit_entry);
    rinit_task.set_stack_pointer(rinit_stack);
    rinit_task.downgrade_cpool(&cpool);
    rinit_task.downgrade_top_page_table(&rinit_pml4);
    rinit_task.downgrade_buffer(&rinit_buffer_page);

    log!("Rinit pml4: {:?}", rinit_pml4);
    log!("Rinit entry: {:?}", rinit_entry);

    log!("hello, world!");
    while true {
        rinit_task.switch_to();
        let buffer = rinit_task.upgrade_buffer();
        handle_system_call(buffer.as_ref().unwrap().write().write().deref_mut().call.as_mut().unwrap());
    }
    
    loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
