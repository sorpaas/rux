use common::*;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool2Arc};
use arch::{TaskRuntime};

use super::{UntypedDescriptor, TopPageTableCap, CPoolCap};

#[derive(Debug)]
pub struct TaskDescriptor {
    weak_pool: ManagedWeakPool2Arc,
    runtime: TaskRuntime,
    next: Option<ManagedArcAny>,
}
pub type TaskCap = ManagedArc<RwLock<TaskDescriptor>>;

impl TaskCap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let weak_pool = unsafe { ManagedWeakPool2Arc::create(
            untyped.allocate(ManagedWeakPool2Arc::inner_length(),
                             ManagedWeakPool2Arc::inner_alignment())) };

        unsafe { untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
            arc = Some(unsafe {
                Self::new(paddr, RwLock::new(TaskDescriptor {
                    weak_pool: weak_pool,
                    runtime: TaskRuntime::default(),
                    next: next_child,
                }))
            });

            arc.clone().unwrap().into()
        }) };

        arc.unwrap()
    }
}

impl TaskDescriptor {
    pub fn set_instruction_pointer(&mut self, instruction_pointer: VAddr) {
        self.runtime.set_instruction_pointer(instruction_pointer)
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: VAddr) {
        self.runtime.set_stack_pointer(stack_pointer)
    }

    pub fn downgrade_cpool(&self, cpool: &CPoolCap) {
        self.weak_pool.downgrade_at(cpool, 0)
    }

    pub fn upgrade_cpool(&self) -> Option<CPoolCap> {
        self.weak_pool.upgrade(0)
    }

    pub fn downgrade_top_page_table(&self, pml4: &TopPageTableCap) {
        self.weak_pool.downgrade_at(pml4, 1)
    }

    pub fn upgrade_top_page_table(&self) -> Option<TopPageTableCap> {
        self.weak_pool.upgrade(1)
    }

    pub fn switch_to(&mut self) {
        if let Some(pml4) = self.upgrade_top_page_table() {
            pml4.write().switch_to();
        }
        unsafe { self.runtime.switch_to() };
    }
}
