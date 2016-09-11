#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(naked_functions)]
#![feature(associated_consts)]
#![feature(type_ascription)]
#![feature(core_intrinsics)]
#![no_std]

extern crate x86;
extern crate spin;
extern crate rlibc;

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
mod utils;
mod common;
mod elf;
mod cap;

use core::mem;
use core::slice;
use common::*;
use arch::{InitInfo};
use cap::{UntypedHalf, CPoolHalf, TopPageTableHalf, PageHalf, Capability};

#[no_mangle]
pub fn kmain(archinfo: InitInfo)
{
    log!("archinfo: {:?}", &archinfo);

    let mut region_iter = archinfo.free_regions();
    let cpool_target_region = region_iter.next().unwrap();
    let mut cpool_target_untyped =
        UntypedHalf::new(cpool_target_region.start_paddr(), cpool_target_region.length());
    let mut cpool_cap = CPoolHalf::new(&mut cpool_target_untyped);
    let cpool_cap_cloned = cpool_cap.clone();

    let mut untyped_target = cpool_target_untyped;

    untyped_target = cpool_cap.with_cpool_mut(|cpool| {
        cpool.insert(Capability::CPool(cpool_cap_cloned));

        for region in region_iter {
            let untyped = UntypedHalf::new(region.start_paddr(),
                                           region.length());
                
            if untyped.length() > untyped_target.length() {
                cpool.insert(Capability::Untyped(untyped_target));
                untyped_target = untyped;
            } else {
                cpool.insert(Capability::Untyped(untyped));
            }
        }
        
        log!("cpool = {:?}", cpool);

        untyped_target
    });

    let mut rinit_pml4_half = TopPageTableHalf::new(&mut untyped_target);
    let rinit_page_half = PageHalf::new(&mut untyped_target);
    let rinit_stack_half = PageHalf::new(&mut untyped_target);
    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);

    // pml4_half.map_page(VAddr::from_usize(0x0), &page_half,
    //                    &mut untyped_target, &mut cpool_cap);

    unsafe {
        arch::with_slice(
            archinfo.rinit_region().start_paddr(),
            archinfo.rinit_region().length(),
            |bin_raw: &[u8]| {
                use elf::{ElfBinary};
                let bin = ElfBinary::new("rinit", bin_raw).unwrap();
                             
                log!("rinit: {:?}", bin);
                log!("fheader = {:?}", bin.file_header());
                for p in bin.program_headers() {
                    use elf::{PT_LOAD};
                                 
                    if p.progtype == PT_LOAD {
                        log!("pheader = {}", p);
                        assert!(p.memsz as usize <= PageHalf::length());
                        assert!(p.filesz == p.memsz);
                        rinit_pml4_half.map(VAddr::from(p.vaddr), &rinit_page_half,
                                            &mut untyped_target, &mut cpool_cap);

                        arch::with_slice_mut(
                            rinit_page_half.start_paddr(),
                            PageHalf::length(),
                            |page_raw: &mut [u8]| {
                                for i in p.offset..(p.offset + p.memsz) {
                                    page_raw[(i - p.offset) as usize] = bin_raw[i as usize];
                                }
                            });
                    }
                }

                rinit_pml4_half.map(rinit_stack_vaddr, &rinit_stack_half,
                                    &mut untyped_target, &mut cpool_cap);
            });
    }

    rinit_pml4_half.switch_to();

    // with_object(kernel_pml4_paddr(), |pml4: &PML4| {
    //     for area in pml4.iter() {
    //         log!("{:?}", area.get_address());
    //     }
    // });

    // for region in archinfo.memory_regions() {
    //     log!("{:?}", region);
    // }

    log!("hello, world!");

    unsafe {
        arch::switch_to_user_mode(VAddr::from(0x0: u64),
                                  (rinit_stack_vaddr + (PageHalf::length() - 4)));
    }
    
	loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
