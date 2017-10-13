use arch::init;

/// Interrupt handler function type.
pub type HandlerFunc = unsafe extern "C" fn();

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
    pub error_code: Option<u64>,
    pub exception_code: u64
}

#[derive(Debug, Clone)]
pub struct Registers {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            rax: 0, rbx: 0, rcx: 0, rdx: 0, rbp: 0, rsi: 0, rdi: 0,
            r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0
        }
    }
}

pub static mut RSP_AFTER_SAVING_REGISTERS: u64 = 0;

macro_rules! save_registers {
    () => {
        use ::arch::interrupt::switch::RSP_AFTER_SAVING_REGISTERS;

        asm!("push rax
              push rbx
              push rcx
              push rdx
              push rbp
              push rsi
              push rdi
              push r8
              push r9
              push r10
              push r11
              push r12
              push r13
              push r14
              push r15
              push rsp
        " :::: "intel", "volatile");
        asm!("" : "={rsp}"(RSP_AFTER_SAVING_REGISTERS)
             ::: "volatile", "intel");
    }
}

macro_rules! restore_registers {
    () => {
        use ::arch::interrupt::switch::RSP_AFTER_SAVING_REGISTERS;

        asm!(""
             ::
             "{rsp}"(RSP_AFTER_SAVING_REGISTERS)
             :: "volatile", "intel");
        asm!("pop rsp
              pop r15
              pop r14
              pop r13
              pop r12
              pop r11
              pop r10
              pop r9
              pop r8
              pop rdi
              pop rsi
              pop rbp
              pop rdx
              pop rcx
              pop rbx
              pop rax
            " :::: "intel", "volatile");
    }
}

macro_rules! load_usermode_registers {
    () => {
        use ::arch::interrupt::switch::CUR_REGISTERS;

        asm!(""
             ::
             "{rax}"(CUR_REGISTERS.rax), "{rbx}"(CUR_REGISTERS.rbx), "{rcx}"(CUR_REGISTERS.rcx),
             "{rdx}"(CUR_REGISTERS.rdx), "{rsi}"(CUR_REGISTERS.rsi), "{rdi}"(CUR_REGISTERS.rdi),
             "{r8}"(CUR_REGISTERS.r8), "{r9}"(CUR_REGISTERS.r9), "{r10}"(CUR_REGISTERS.r10),
             "{r11}"(CUR_REGISTERS.r11), "{r12}"(CUR_REGISTERS.r12), "{r13}"(CUR_REGISTERS.r13),
             "{r14}"(CUR_REGISTERS.r14), "{r15}"(CUR_REGISTERS.r15)
             :: "volatile", "intel");

        asm!(""
             ::
             "{rbp}"(CUR_REGISTERS.rbp)
             :: "volatile", "intel");
    }
}

macro_rules! save_usermode_registers {
    () => {
        use ::arch::interrupt::switch::CUR_REGISTERS;

        asm!("push rbp" :::: "volatile", "intel");

        asm!("" : "={rax}"(CUR_REGISTERS.rax), "={rbx}"(CUR_REGISTERS.rbx), "={rcx}"(CUR_REGISTERS.rcx),
             "={rdx}"(CUR_REGISTERS.rdx), "={rsi}"(CUR_REGISTERS.rsi),
             "={rdi}"(CUR_REGISTERS.rdi), "={r8}"(CUR_REGISTERS.r8), "={r9}"(CUR_REGISTERS.r9),
             "={r10}"(CUR_REGISTERS.r10), "={r11}"(CUR_REGISTERS.r11), "={r12}"(CUR_REGISTERS.r12),
             "={r13}"(CUR_REGISTERS.r13), "={r14}"(CUR_REGISTERS.r14), "={r15}"(CUR_REGISTERS.r15)
             ::: "volatile", "intel");

        asm!("pop r9" : "={r9}"(CUR_REGISTERS.rbp) ::: "volatile", "intel");
    }
}

unsafe extern "C" fn set_kernel_stack(addr: u64) {
    init::set_kernel_stack(addr);
}

pub unsafe fn switch_to_raw(stack_vaddr: u64, code_start: u64, cpu_flags: u64, code_seg: u64, data_seg: u64) {
    switch_to_raw_naked(stack_vaddr, code_start, cpu_flags, code_seg, data_seg);
}

#[naked]
pub unsafe extern "C" fn switch_to_raw_naked(stack_vaddr: u64, code_start: u64, cpu_flags: u64, code_seg: u64, data_seg: u64) {
    save_registers!();

    asm!("mov rdi, rsp
          call $0

          push rax
          push rbx
          push r8
          push rcx
          push rdx"
         ::
         "i"(set_kernel_stack as unsafe extern "C" fn(u64)),
         "{rax}"(data_seg), "{rbx}"(stack_vaddr), "{rcx}"(code_seg),
         "{rdx}"(code_start), "{r8}"(cpu_flags)
         :: "volatile", "intel");

    load_usermode_registers!();

    asm!("iretq"
         :::: "volatile", "intel");
}

static mut CUR_EXCEPTION_STACK_FRAME: Option<ExceptionStackFrame> = None;
static mut CUR_EXCEPTION_ERROR_CODE: Option<u64> = None;
static mut CUR_EXCEPTION_CODE: Option<u64> = None;
pub static mut CUR_REGISTERS: Registers = Registers {
    rax: 0, rbx: 0, rcx: 0, rdx: 0, rbp: 0, rsi: 0, rdi: 0,
    r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0
};

pub unsafe fn set_cur_registers(registers: Registers) {
    CUR_REGISTERS = registers;
}

pub unsafe fn cur_registers() -> Registers {
    CUR_REGISTERS.clone()
}

pub unsafe extern "C" fn store_exception_stack(exception_raw: *const ExceptionStackFrame, exception_code: u64) {
    let exception = unsafe {&*exception_raw};
    CUR_EXCEPTION_STACK_FRAME = Some(exception.clone());
    CUR_EXCEPTION_ERROR_CODE = None;
    CUR_EXCEPTION_CODE = Some(exception_code);
}

pub unsafe extern "C" fn store_error_exception_stack(exception_raw: *const ExceptionStackFrame, error_code: u64, exception_code: u64) {
    let exception = unsafe {&*exception_raw};
    CUR_EXCEPTION_STACK_FRAME = Some(exception.clone());
    CUR_EXCEPTION_ERROR_CODE = Some(error_code);
    CUR_EXCEPTION_CODE = Some(exception_code);
}

macro_rules! return_to_raw_fn {
    ($name: ident, $exception_code: expr) => (
        #[naked]
        pub unsafe extern "C" fn $name() {
            save_usermode_registers!();

            asm!("mov rdi, rsp
                  sub rsp, 8
                  call $0"
                 :: "i"(::arch::interrupt::switch::store_exception_stack as unsafe extern "C" fn(*const ::arch::interrupt::switch::ExceptionStackFrame, u64)), "{rsi}"($exception_code)
                 :: "volatile", "intel");

            restore_registers!();
        }
    )
}

macro_rules! return_error_to_raw_fn {
    ($name: ident, $exception_code: expr) => (
        #[naked]
        pub unsafe extern "C" fn $name() {
            save_usermode_registers!();

            asm!("pop rsi
                  mov rdi, rsp
                  sub rsp, 8
                  call $0"
                 :: "i"(::arch::interrupt::switch::store_error_exception_stack as unsafe extern "C" fn(*const ::arch::interrupt::switch::ExceptionStackFrame, u64, u64)), "{rdx}"($exception_code)
                 :: "volatile", "intel");

            restore_registers!();
        }
    )
}

pub fn last_exception_return_value() -> Option<ExceptionInfo> {
    unsafe {
        CUR_EXCEPTION_STACK_FRAME.clone().map(|exp| {
            ExceptionInfo {
                instruction_pointer: exp.instruction_pointer,
                code_segment: exp.code_segment,
                cpu_flags: exp.cpu_flags,
                stack_pointer: exp.stack_pointer,
                stack_segment: exp.stack_segment,
                error_code: CUR_EXCEPTION_ERROR_CODE,
                exception_code: CUR_EXCEPTION_CODE.unwrap()
            }
        })
    }
}
