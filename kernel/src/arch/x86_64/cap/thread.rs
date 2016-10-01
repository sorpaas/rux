#[derive(Debug)]
pub struct ThreadRuntime {
    instruction_pointer: u64,
    cpu_flags: u64,
    stack_pointer: u64
}

impl ThreadRuntime {
    pub unsafe fn switch_to(&self) {
        let stack_vaddr = self.stack_pointer as usize;
        let code_start = self.instruction_pointer as usize;
        let cpu_flags = self.cpu_flags as usize;
        let code_seg = 0x28 | 0x3;
        let data_seg = 0x30 | 0x3;

        asm!("mov ds, rax
              mov es, rax
              mov fs, rax
              mov gs, rax

              push rax
              push rbx
              push r8
              push rcx
              push rdx
              iretq"
             :: "{rax}"(data_seg), "{rbx}"(stack_vaddr), "{rcx}"(code_seg), "{rdx}"(code_start), "{r8}"(cpu_flags)
             : "memory" : "intel", "volatile");
    }

    pub fn new(instruction_pointer: u64, cpu_flags: u64, stack_pointer: u64) -> ThreadRuntime {
        ThreadRuntime {
            instruction_pointer: instruction_pointer,
            cpu_flags: cpu_flags,
            stack_pointer: stack_pointer
        }
    }

    pub fn update(&mut self, instruction_pointer: u64, cpu_flags: u64, stack_pointer: u64) {
        self.instruction_pointer = instruction_pointer;
        self.cpu_flags = cpu_flags;
        self.stack_pointer = stack_pointer;
    }
}
