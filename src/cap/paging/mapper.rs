use common::*;

use super::utils;
use spin::{Mutex, MutexGuard};

pub struct Mapper {
    mappable_virtual_start_addr: usize,
    mappable_count: usize,
    is_kernel_bootstrapping: bool,
    mapped_count: usize,
}

// We lock MAPPABLE before borrow_map, borrow_mut_map and switch_mappable. As a
// result, a page table switching cannot happen when a borrow_map is in
// progress.

static ACTIVE_MAPPER: Mutex<Mapper> =
    Mutex::new(Mapper {
        mappable_virtual_start_addr: 0,
        mappable_count: 0,
        is_kernel_bootstrapping: true,
        mapped_count: 0,
    });

pub fn active_mapper<'a>() -> MutexGuard<'a, Mapper> {
    ACTIVE_MAPPER.lock()
}

pub unsafe fn switch_mapper(mappable_virtual_start_addr: usize, mappable_count: usize) {
    let mut mapper = active_mapper();

    assert!(mapper.mapped_count == 0);
    mapper.mappable_virtual_start_addr = mappable_virtual_start_addr;
    mapper.mappable_count = mappable_count;
    mapper.is_kernel_bootstrapping = false;
}

impl Mapper {
    fn borrow_map_addr<F>(&mut self, frame_start_addr: PhysicalAddress, frame_count: usize, f: F)
        where F: FnOnce(usize, &mut Mapper) {
        assert!(frame_start_addr % PAGE_SIZE == 0);

        if self.is_kernel_bootstrapping {
            // Specail conditions when the kernel is bootstrapping because the huge
            // table issue. The first 1GB is identity mapped.

            assert!(frame_start_addr + frame_count * PAGE_SIZE < 0x7ee0000);

            let mapped_addr = frame_start_addr;

            // assert!(sizeof<T>() <= frame.block().size());

            f(mapped_addr, self);

        } else {
            let mapped_addr = self.mappable_virtual_start_addr + self.mapped_count * PAGE_SIZE;

            for i in 0..frame_count {
                assert!(self.mapped_count < self.mappable_count);

                unsafe {
                    utils::map_in_active(frame_start_addr + i * PAGE_SIZE,
                                         self.mappable_virtual_start_addr + self.mapped_count * PAGE_SIZE,
                                         WRITABLE);
                }

                self.mapped_count += 1;
            }

            f(mapped_addr, self);

            for i in (0..frame_count).rev() {
                unsafe {
                    utils::unmap_in_active(self.mappable_virtual_start_addr + self.mapped_count * PAGE_SIZE);
                }

                self.mapped_count -= 1;
            }
        }

    }

    pub unsafe fn borrow_map<T, F>(&mut self, frame_start_addr: PhysicalAddress, frame_count: usize, f: F)
        where F: FnOnce(&T, &mut Mapper) {

        self.borrow_map_addr(frame_start_addr, frame_count, |mapped_addr, mapper| {
            f(&*(mapped_addr as *const T), mapper);
        });
    }

    pub unsafe fn borrow_mut_map<T, F>(&mut self, frame_start_addr: PhysicalAddress, frame_count: usize, f: F)
        where F: FnOnce(&mut T, &mut Mapper) {

        self.borrow_map_addr(frame_start_addr, frame_count, |mapped_addr, mapper| {
            f(&mut *(mapped_addr as *mut T), mapper);
        });
    }
}
