#![feature(lang_items)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(naked_functions)]
#![feature(associated_consts)]
#![feature(type_ascription)]
#![feature(core_intrinsics)]
#![feature(optin_builtin_traits)]
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
use cap::{UntypedHalf, CPoolHalf, TopPageTableHalf, PageHalf, Capability, CapHalf};

#[no_mangle]
pub fn kmain(archinfo: InitInfo)
{
    log!("archinfo: {:?}", &archinfo);
    let rinit_stack_vaddr = VAddr::from(0x80000000: usize);
    let mut rinit_entry: u64 = 0x0;

    let cpool_cap = {
        let mut region_iter = archinfo.free_regions();
        let cpool_target_region = region_iter.next().unwrap();
        let mut cpool_target_untyped = unsafe {
            UntypedHalf::bootstrap(cpool_target_region.start_paddr(), cpool_target_region.length())
        };

        let mut cpool_cap = CPoolHalf::new(&mut cpool_target_untyped);
        let cpool_cap_cloned = cpool_cap.clone();

        let mut untyped_target = cpool_target_untyped;

        untyped_target = cpool_cap.with_cpool_mut(|cpool| {
            cpool.insert(Capability::CPool(cpool_cap_cloned));

            for region in region_iter {
                let untyped = unsafe {
                    UntypedHalf::bootstrap(region.start_paddr(),
                                           region.length())
                };
                
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
        let rinit_stack_half = PageHalf::new(&mut untyped_target);

        unsafe {
            arch::with_slice(
                archinfo.rinit_region().start_paddr(),
                archinfo.rinit_region().length(),
                |bin_raw: &[u8]| {
                    use elf::{ElfBinary};
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
                            let end_vaddr = VAddr::from(p.vaddr + p.memsz as usize);

                            let mut untyped_for_load = UntypedHalf::new(&mut untyped_target,
                                                                        p.memsz as usize + PageHalf::length(),
                                                                        PageHalf::length());

                            log!("allocating initial page half ...");
                            let initial_page_half = PageHalf::new(&mut untyped_for_load);
                            let rinit_start_paddr = initial_page_half.start_paddr();
                            rinit_pml4_half.map(VAddr::from(p.vaddr), &initial_page_half,
                                                &mut untyped_target, &mut cpool_cap);

                            cpool_cap.with_cpool_mut(|cpool| {
                                cpool.insert(Capability::Page(initial_page_half));
                            });
                            next_page_vaddr += PageHalf::length();

                            while next_page_vaddr <= end_vaddr {
                                log!("mapping from: 0x{:x}", next_page_vaddr);
                                let rinit_page_half = PageHalf::new(&mut untyped_for_load);
                                rinit_pml4_half.map(next_page_vaddr, &rinit_page_half,
                                                    &mut untyped_target, &mut cpool_cap);

                                cpool_cap.with_cpool_mut(|cpool| {
                                    cpool.insert(Capability::Page(rinit_page_half));
                                });
                                next_page_vaddr += PageHalf::length();
                            }

                            untyped_for_load.mark_deleted();

                            log!("initial page half: 0x{:x}", rinit_start_paddr);
                            arch::with_slice_mut(
                                rinit_start_paddr,
                                p.memsz as usize,
                                |page_raw: &mut [u8]| {
                                    for i in p.offset..(p.offset + p.memsz) {
                                        page_raw[(i - p.offset) as usize] = bin_raw[i as usize];
                                    }
                                });
                        }
                    }

                    log!("mapping the rinit stack ...");
                    rinit_pml4_half.map(rinit_stack_vaddr, &rinit_stack_half,
                                        &mut untyped_target, &mut cpool_cap);
                });
        }

        cpool_cap.with_cpool_mut(|cpool| {
            cpool.insert(Capability::Page(rinit_stack_half));
            cpool.insert(Capability::Untyped(untyped_target));
        });

        log!("switching to rinit pml4 ...");
        rinit_pml4_half.switch_to();

        cpool_cap.with_cpool_mut(|cpool| {
            cpool.insert(Capability::TopPageTable(rinit_pml4_half));
        });

        cpool_cap
    };

    // TODO Insert cpool_cap to TCB.

    cpool_cap.with_cpool(|cpool| {
        log!("Current cpool: {:?}", cpool);
    });

    unsafe {
        arch::switch_to_user_mode(VAddr::from(rinit_entry),
                                  (rinit_stack_vaddr + (PageHalf::length() - 4)));
    }
    
    loop {}
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}
