use common::*;
use core::any::{Any, TypeId};
use util::{RwLock, align_up};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool3Arc};
use arch::{TaskRuntime};

use super::{UntypedDescriptor, TopPageTableCap, CPoolCap, TaskBufferPageCap};

#[derive(Debug)]
pub struct TaskDescriptor {
    weak_pool: ManagedWeakPool3Arc,
    runtime: TaskRuntime,
    next: Option<ManagedArcAny>,
}
pub type TaskCap = ManagedArc<RwLock<TaskDescriptor>>;

impl TaskCap {
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let weak_pool = unsafe { ManagedWeakPool3Arc::create(
            untyped.allocate(ManagedWeakPool3Arc::inner_length(),
                             ManagedWeakPool3Arc::inner_alignment())) };

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
        self.weak_pool.read().downgrade_at(cpool, 0)
    }

    pub fn upgrade_cpool(&self) -> Option<CPoolCap> {
        self.weak_pool.read().upgrade(0)
    }

    pub fn downgrade_top_page_table(&self, pml4: &TopPageTableCap) {
        self.weak_pool.read().downgrade_at(pml4, 1)
    }

    pub fn upgrade_top_page_table(&self) -> Option<TopPageTableCap> {
        self.weak_pool.read().upgrade(1)
    }

    pub fn downgrade_buffer(&self, buffer: &TaskBufferPageCap) {
        self.weak_pool.read().downgrade_at(buffer, 2)
    }

    pub fn upgrade_buffer(&self) -> Option<TaskBufferPageCap> {
        self.weak_pool.read().upgrade(2)
    }

    pub fn switch_to(&mut self) {
        if let Some(pml4) = self.upgrade_top_page_table() {
            pml4.write().switch_to();
        }
        unsafe { self.runtime.switch_to() };
    }
}
