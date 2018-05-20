use common::*;
use core::iter::Iterator;
use util::{RwLock, Mutex};
use util::managed_arc::{ManagedArc, ManagedArcAny, ManagedWeakPool3Arc};
use arch::{TaskRuntime, Exception};

use super::{UntypedDescriptor, TopPageTableCap, CPoolCap, TaskBufferPageCap, ChannelCap};

/// Switch to an idle task that runs in kernel-mode. This is used when
/// no other tasks is runnable. Like normal context switching, this
/// returns only when exceptions (interrupts) happen.
pub fn idle() -> Exception {
    #[naked]
    unsafe fn idle_task() -> ! {
        asm!("hlt");
        ::core::intrinsics::unreachable();
    }

    let mut task_runtime = TaskRuntime::default();
    task_runtime.set_instruction_pointer(VAddr::from(idle_task as *const () as u64));

    unsafe {
        task_runtime.switch_to(false)
    }
}

/// Represent a task status.
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Active,
    ChannelWait(ChannelCap),
    Inactive,
}

/// Task descriptor.
#[derive(Debug)]
pub struct TaskDescriptor {
    weak_pool: ManagedWeakPool3Arc,
    runtime: TaskRuntime,
    next: Option<ManagedArcAny>,
    next_task: Option<TaskCap>,
    status: TaskStatus
}
/// Task capability. Reference-counted smart pointer to task
/// descriptor.
///
/// Tasks represents isolated processes running.
pub type TaskCap = ManagedArc<RwLock<TaskDescriptor>>;

impl TaskCap {
    /// Create a task capability from an untyped capability.
    pub fn retype_from(untyped: &mut UntypedDescriptor) -> Self {
        let mut arc: Option<Self> = None;

        let weak_pool = unsafe { ManagedWeakPool3Arc::create(
            untyped.allocate(ManagedWeakPool3Arc::inner_length(),
                             ManagedWeakPool3Arc::inner_alignment())) };

        unsafe { untyped.derive(Self::inner_length(), Self::inner_alignment(), |paddr, next_child| {
            arc = Some(
                Self::new(paddr, RwLock::new(TaskDescriptor {
                    weak_pool: weak_pool,
                    runtime: TaskRuntime::default(),
                    next: next_child,
                    next_task: None,
                    status: TaskStatus::Inactive,
                }))
            );

            arc.clone().unwrap().into()
        }) };

        register_task(arc.clone().unwrap());

        arc.unwrap()
    }
}

impl TaskDescriptor {
    /// Set the task's instruction pointer.
    pub fn set_instruction_pointer(&mut self, instruction_pointer: VAddr) {
        self.runtime.set_instruction_pointer(instruction_pointer)
    }

    /// Set the task's stack pointer.
    pub fn set_stack_pointer(&mut self, stack_pointer: VAddr) {
        self.runtime.set_stack_pointer(stack_pointer)
    }

    /// Set the task's root capability pool.
    pub fn downgrade_cpool(&self, cpool: &CPoolCap) {
        self.weak_pool.read().downgrade_at(cpool, 0)
    }

    /// Read from the task's root capability pool.
    pub fn upgrade_cpool(&self) -> Option<CPoolCap> {
        self.weak_pool.read().upgrade(0)
    }

    /// Set the task's top page table.
    pub fn downgrade_top_page_table(&self, pml4: &TopPageTableCap) {
        self.weak_pool.read().downgrade_at(pml4, 1)
    }

    /// Read from the task's top page table.
    pub fn upgrade_top_page_table(&self) -> Option<TopPageTableCap> {
        self.weak_pool.read().upgrade(1)
    }

    /// Set the task's buffer.
    pub fn downgrade_buffer(&self, buffer: &TaskBufferPageCap) {
        self.weak_pool.read().downgrade_at(buffer, 2)
    }

    /// Read from the task's buffer.
    pub fn upgrade_buffer(&self) -> Option<TaskBufferPageCap> {
        self.weak_pool.read().upgrade(2)
    }

    /// Current task status.
    pub fn status(&self) -> TaskStatus {
        self.status.clone()
    }

    /// Set the current task status.
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// Switch to the task. The function is returned when exception
    /// happens.
    pub fn switch_to(&mut self) -> Exception {
        if let Some(pml4) = self.upgrade_top_page_table() {
            pml4.write().switch_to();
        }
        unsafe { self.runtime.switch_to(true) }
    }
}

/// The first task initialized by the kernel.
static FIRST_TASK: Mutex<Option<TaskCap>> = Mutex::new(None);

/// Register a new task. Using `FIRST_TASK` static, this forms a
/// linked-list that allows an iterator to iterate over all created
/// tasks.
fn register_task(cap: TaskCap) {
    let mut first_task = FIRST_TASK.lock();
    if first_task.is_none() {
        *first_task = Some(cap);
    } else {
        let mut first = first_task.as_mut().unwrap().write();
        let mut second = cap.write();
        let third_task = first.next_task.take();

        second.next_task = third_task;
        first.next_task = Some(cap.clone());
    }
}

/// A task iterator.
pub struct TaskIterator {
    next: Option<TaskCap>,
}

impl Iterator for TaskIterator {
    type Item = TaskCap;

    fn next(&mut self) -> Option<TaskCap> {
        if let Some(current) = self.next.clone() {
            {
                let current_task = current.read();
                self.next = current_task.next_task.clone();
            }
            return Some(current);
        } else {
            None
        }
    }
}

/// Return a task iterator using `FIRST_TASK`.
pub fn task_iter() -> TaskIterator {
    TaskIterator {
        next: FIRST_TASK.lock().clone(),
    }
}
