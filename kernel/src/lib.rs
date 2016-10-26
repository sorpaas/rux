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
mod util;
mod common;
mod elf;
mod cap;

use core::mem;
use core::slice;
use common::*;
use arch::{InitInfo, ThreadRuntime};
use cap::{UntypedFull, CPoolFull, Capability, TCBHalf, MDB, Cap,
          CapReadObject, CapWriteObject};
use core::ops::{Deref, DerefMut};
use util::{MemoryObject};

#[no_mangle]
pub fn kmain(archinfo: InitInfo)
{
    log!("archinfo: {:?}", &archinfo);
    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let mut rinit_entry: u64 = 0x0;

    let (mut cpool_cap, mut tcb_half) = {
        let mut region_iter = archinfo.free_regions();
        let cpool_target_region = region_iter.next().unwrap();

        let mut cpool_cap_half = unsafe {
            CPoolFull::bootstrap(
                UntypedFull::bootstrap(cpool_target_region.start_paddr(), cpool_target_region.length())
            )
        };

        {
            let cpool_target_untyped_cap = cpool_cap_half.read(0);
            let cpool_target_untyped = match cpool_target_untyped_cap.as_ref().unwrap() {
                &Cap::Untyped(ref untyped) => untyped,
                _ => panic!(),
            };
            for child in cpool_target_untyped.mdb(0).children() {
                log!("child of index 0: {:?}", child.deref());
            }
        }
        log!("CPool index 0: {:?}", cpool_cap_half.read(0).deref());
        log!("CPool index 1: {:?}", cpool_cap_half.read(1).deref());

        ((), ())
    };


    //     let mut untyped_target = {
    //         let mut untyped_target = cpool_target_untyped;

    //         let mut cpool = cpool_cap.write();
    //         cpool.insert(Capability::CPool(cpool_cap_cloned));

    //         for region in region_iter {
    //             let untyped = unsafe {
    //                 UntypedHalf::bootstrap(region.start_paddr(),
    //                                        region.length())
    //             };
                
    //             if untyped.length() > untyped_target.length() {
    //                 cpool.insert(Capability::Untyped(untyped_target));
    //                 untyped_target = untyped;
    //             } else {
    //                 cpool.insert(Capability::Untyped(untyped));
    //             }
    //         }

    //         untyped_target
    //     };

    //     let mut rinit_pml4_half = TopPageTableHalf::new(&mut untyped_target);
    //     let mut rinit_stack_half = PageHalf::new(&mut untyped_target);

    //     {
    //         use elf::{ElfBinary};
    //         let slice_object = unsafe { MemoryObject::<u8>::slice(archinfo.rinit_region().start_paddr(),
    //                                                               archinfo.rinit_region().length()) };
    //         let bin_raw = unsafe { slice::from_raw_parts(*slice_object, archinfo.rinit_region().length()) };
    //         let bin = ElfBinary::new("rinit", bin_raw).unwrap();

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
