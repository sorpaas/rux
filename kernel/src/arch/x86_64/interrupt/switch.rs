use arch::init;

pub type HandlerFunc = unsafe extern "C" fn();

#[derive(Debug, Clone)]
#[repr(C)]
struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Debug, Clone)]
pub struct Exception {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
    pub error_code: Option<u64>,
    pub exception_code: u64
}

macro_rules! save_registers {
    () => {
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
    }
}

macro_rules! restore_registers {
    () => {
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

unsafe extern "C" fn set_kernel_stack(addr: u64) {
    init::set_kernel_stack(addr);
}

pub unsafe fn switch_to_raw(stack_vaddr: u64, code_start: u64, cpu_flags: u64) {
    let code_seg: u64 = 0x28 | 0x3;
    let data_seg: u64 = 0x30 | 0x3;

    asm!("call r15" :: "{rdi}"(stack_vaddr), "{rsi}"(code_start), "{rdx}"(cpu_flags), "{rcx}"(code_seg), "{r8}"(data_seg), "{r15}"(switch_to_raw_naked as unsafe extern "C" fn(u64, u64, u64, u64, u64)) :: "volatile", "intel");

    // WARNING: Everything below this before returning will not work.
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn switch_to_raw_naked(stack_vaddr: u64, code_start: u64, cpu_flags: u64, code_seg: u64, data_seg: u64) {
    save_registers!();

    asm!("mov rdi, rsp
          call $0" :: "i"(set_kernel_stack as unsafe extern "C" fn(u64)) : "rdi" : "volatile", "intel");

    asm!("push rax
          push rbx
          push r8
          push rcx
          push rdx
          iretq"
         :: "{rax}"(data_seg), "{rbx}"(stack_vaddr), "{rcx}"(code_seg), "{rdx}"(code_start), "{r8}"(cpu_flags)
         : "rax", "rbx", "rcx", "rdx",
           "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"
         : "volatile", "intel");
}

static mut CUR_EXCEPTION_STACK_FRAME: Option<ExceptionStackFrame> = None;
static mut CUR_EXCEPTION_ERROR_CODE: Option<u64> = None;
static mut CUR_EXCEPTION_CODE: Option<u64> = None;

unsafe extern "C" fn store_exception_stack(exception_raw: *const ExceptionStackFrame, exception_code: u64) {
    let exception = unsafe {&*exception_raw};
    CUR_EXCEPTION_STACK_FRAME = Some(exception.clone());
    CUR_EXCEPTION_ERROR_CODE = None;
    CUR_EXCEPTION_CODE = Some(exception_code);
}

unsafe extern "C" fn store_error_exception_stack(exception_raw: *const ExceptionStackFrame, error_code: u64, exception_code: u64) {
    let exception = unsafe {&*exception_raw};
    CUR_EXCEPTION_STACK_FRAME = Some(exception.clone());
    CUR_EXCEPTION_ERROR_CODE = Some(error_code);
    CUR_EXCEPTION_CODE = Some(exception_code);
}

macro_rules! return_to_raw_fn {
    ($name: ident, $exception_code: expr) => (
        #[no_mangle]
        #[naked]
        pub unsafe extern "C" fn $name() {
            asm!("mov rdi, rsp
                  sub rsp, 8
                  call $0
                  add rsp, 48"
                 :: "i"(store_exception_stack as unsafe extern "C" fn(*const ExceptionStackFrame, u64)), "{rsi}"($exception_code)
                 :: "volatile", "intel");

            restore_registers!();
        }
    )
}

macro_rules! return_error_to_raw_fn {
    ($name: ident, $exception_code: expr) => (
        #[no_mangle]
        #[naked]
        pub unsafe extern "C" fn $name() {
            asm!("pop rsi
                  mov rdi, rsp
                  sub rsp, 8
                  call $0
                  add rsp, 48"
                 :: "i"(store_error_exception_stack as unsafe extern "C" fn(*const ExceptionStackFrame, u64)), "{rdx}"($exception_code)
                 :: "volatile", "intel");

            restore_registers!();
        }
    )
}

return_to_raw_fn!(system_call_return_to_raw, 0x80);
return_to_raw_fn!(debug_call_return_to_raw, 0x81);

pub fn last_exception_return_value() -> Option<Exception> {
    unsafe {
        CUR_EXCEPTION_STACK_FRAME.clone().map(|exp| {
            Exception {
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
