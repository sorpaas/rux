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
use cap::{UntypedCap, CPoolCap, PageCap, TopPageTableCap};
use core::ops::{Deref, DerefMut};
use util::{MemoryObject};
use core::any::{Any, TypeId};

fn bootstrap_rinit_paging(archinfo: &InitInfo, cpool: &mut CPoolCap, untyped: &mut UntypedCap) -> (TopPageTableCap, VAddr) {
    use elf::{ElfBinary};

    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let mut rinit_entry: u64 = 0x0;

    let mut rinit_pml4 = TopPageTableCap::retype_from(untyped.write().deref_mut());
    cpool.downgrade_free(&rinit_pml4);

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

                let page_cap = PageCap::retype_from(untyped.write().deref_mut());
                cpool.downgrade_free(&page_cap);
                rinit_pml4.map(next_page_vaddr, &page_cap, untyped, cpool);

                let mut page = page_cap.write();
                let page_length = page.length();
                let mut page_raw = page.write();

                for i in 0..min(page_length, (p.memsz as usize) - offset) {
                    page_raw[i] = bin_raw[(p.offset as usize) + offset + i];
                }

                offset += page_length;
                next_page_vaddr += page_length;
            }
        }
    }

    log!("mapping the rinit stack ...");
    let mut rinit_stack_page = PageCap::retype_from(untyped.write().deref_mut());
    cpool.downgrade_free(&rinit_stack_page);
    rinit_pml4.map(rinit_stack_vaddr, &rinit_stack_page, untyped, cpool);

    (rinit_pml4, VAddr::from(rinit_entry))
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

        cpool.downgrade_at(&cpool, 0);
        cpool.downgrade_free(&untyped);

        let mut untyped_target = untyped;

        for region in region_iter {
            let untyped = unsafe { UntypedCap::bootstrap(region.start_paddr(),
                                                         region.length()) };
            cpool.downgrade_free(&untyped);

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

    let (rinit_pml4, rinit_entry) = bootstrap_rinit_paging(&archinfo, &mut cpool, &mut untyped);
    rinit_pml4.read().switch_to();

    log!("Rinit pml4: {:?}", rinit_pml4);
    log!("Rinit entry: {:?}", rinit_entry);
    log!("hello, world!");

    //         log!("fheader = {:?}", bin.file_header());
    //         log!("entry = 0x{:x}", bin.file_header().entry);
    //         rinit_entry = bin.file_header().entry;
    //         for p in bin.program_headers() {
    //             use elf::{PT_LOAD};

    //             if p.progtype == PT_LOAD {
    //                 log!("pheader = {}", p);
    //                 assert!(p.filesz == p.memsz);

    //                 let mut next_page_vaddr = VAddr::from(p.vaddr);
    //                 let end_vaddr = VAddr::from(p.vaddr + p.memsz as usize);

    //                 let mut untyped_for_load = UntypedHalf::new(&mut untyped_target,
    //                                                             p.memsz as usize + PageHalf::length(),
    //                                                             PageHalf::length());

    //                 log!("allocating initial page half ...");
    //                 let mut initial_page_half = PageHalf::new(&mut untyped_for_load);
    //                 let rinit_start_paddr = initial_page_half.start_paddr();
    //                 rinit_pml4_half.map(VAddr::from(p.vaddr), &mut initial_page_half,
    //                                     &mut untyped_target, &mut cpool_cap);

    //                 {
    //                     let mut cpool = cpool_cap.write();
    //                     cpool.insert(Capability::Page(initial_page_half));
    //                 }
    //                 next_page_vaddr += PageHalf::length();

    //                 while next_page_vaddr <= end_vaddr {
    //                     log!("mapping from: 0x{:x}", next_page_vaddr);
    //                     let mut rinit_page_half = PageHalf::new(&mut untyped_for_load);
    //                     rinit_pml4_half.map(next_page_vaddr, &mut rinit_page_half,
    //                                         &mut untyped_target, &mut cpool_cap);

    //                     {
    //                         let mut cpool = cpool_cap.write();
    //                         cpool.insert(Capability::Page(rinit_page_half));
    //                     }

    //                     next_page_vaddr += PageHalf::length();
    //                 }

    //                 // untyped_for_load.mark_deleted();

    //                 log!("initial page half: 0x{:x}", rinit_start_paddr);
    //                 {
    //                     let page_object = unsafe { MemoryObject::<u8>::slice(rinit_start_paddr, p.memsz as usize) };
    //                     let page_raw = unsafe { slice::from_raw_parts_mut(*page_object, p.memsz as usize) };

    //                     for i in p.offset..(p.offset + p.memsz) {
    //                         page_raw[(i - p.offset) as usize] = bin_raw[i as usize];
    //                     }
    //                 }
    //             }
    //         }

    //         log!("mapping the rinit stack ...");
    //         rinit_pml4_half.map(rinit_stack_vaddr, &mut rinit_stack_half,
    //                             &mut untyped_target, &mut cpool_cap);
    //     }

    //     log!("switching to rinit pml4 ...");
    //     rinit_pml4_half.switch_to();

    //     log!("creating rinit tcb half ...");
    //     let mut tcb_half = TCBHalf::new(cpool_cap.clone(),
    //                                     &mut untyped_target);
    //     {
    //         let mut tcb = tcb_half.write();
    //         tcb.set_stack_pointer(rinit_stack_vaddr + (PageHalf::length() - 4));
    //         tcb.set_instruction_pointer(VAddr::from(rinit_entry));
    //     }

    //     log!("put everything into rinit cpool ...");
    //     {
    //         let mut cpool = cpool_cap.write();
    //         cpool.insert(Capability::Page(rinit_stack_half));
    //         cpool.insert(Capability::Untyped(untyped_target));
    //         cpool.insert(Capability::TopPageTable(rinit_pml4_half));
    //         cpool.insert(Capability::TCB(tcb_half.clone()));
    //     }

    //     (cpool_cap, tcb_half)
    // };

    // // let mut cpool_full_cap = Capability::CPool(cpool_cap);
    // // for i in 0..32 {
    // //     with_cspace(&cpool_full_cap, &[0, 0, i], |item| {
    // //         log!("cspace item at {} is {:?}", i, item);
    // //     });
    // // }

    // unsafe {
    //     tcb_half.switch_to();
    // }

    // // tcb_half.mark_deleted();
    // // cpool_cap.mark_deleted();
    // // match cpool_full_cap {
    // //     Capability::CPool(ref mut cpool_cap) => {
    // //         cpool_cap.mark_deleted();
    // //     },
    // //     _ => assert!(false)
    // // }
    
    loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
