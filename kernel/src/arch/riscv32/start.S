#define RISCV_PGSHIFT 12
#define RISCV_PGSIZE (1 << RISCV_PGSHIFT)
#define LOG_REGBYTES 2
#define REGBYTES (1 << LOG_REGBYTES)
#define INTEGER_CONTEXT_SIZE (32 * REGBYTES)
#define MENTRY_FRAME_SIZE INTEGER_CONTEXT_SIZE
#define MAX_HARTS 1
#define STORE sw
#define LOAD lw

.option norvc
.section .text.init, "ax", @progbits
.globl reset_vector

reset_vector:
    j do_reset

trap_vector:
    csrrw sp, mscratch, sp
    beqz sp, .Ltrap_from_machine_mode
    ebreak

# We expect all traps from machine mode
.Ltrap_from_machine_mode:
    csrr sp, mscratch
    addi sp, sp, -INTEGER_CONTEXT_SIZE

    STORE a0, 10*REGBYTES(sp)
    STORE a1, 11*REGBYTES(sp)
    STORE a2, 12*REGBYTES(sp)

    mv a0, sp                        # a0 <- regs
    csrr a1, mcause                  # a1 <- mcause
    csrr a2, mepc                    # a2 <- mepc

    # Save registers.
    STORE ra, 1*REGBYTES(sp)
    STORE gp, 3*REGBYTES(sp)
    STORE tp, 4*REGBYTES(sp)
    STORE t0, 5*REGBYTES(sp)
    STORE t1, 6*REGBYTES(sp)
    STORE t2, 7*REGBYTES(sp)
    STORE s0, 8*REGBYTES(sp)
    STORE s1, 9*REGBYTES(sp)
    STORE a3,13*REGBYTES(sp)
    STORE a4,14*REGBYTES(sp)
    STORE a5,15*REGBYTES(sp)
    STORE a6,16*REGBYTES(sp)
    STORE a7,17*REGBYTES(sp)
    STORE s2,18*REGBYTES(sp)
    STORE s3,19*REGBYTES(sp)
    STORE s4,20*REGBYTES(sp)
    STORE s5,21*REGBYTES(sp)
    STORE s6,22*REGBYTES(sp)
    STORE s7,23*REGBYTES(sp)
    STORE s8,24*REGBYTES(sp)
    STORE s9,25*REGBYTES(sp)
    STORE s10,26*REGBYTES(sp)
    STORE s11,27*REGBYTES(sp)
    STORE t3,28*REGBYTES(sp)
    STORE t4,29*REGBYTES(sp)
    STORE t5,30*REGBYTES(sp)
    STORE t6,31*REGBYTES(sp)

    # Invoke the handler.
    jal trap_handler

    # Restore registers.
    LOAD ra, 1*REGBYTES(sp)
    LOAD gp, 3*REGBYTES(sp)
    LOAD tp, 4*REGBYTES(sp)
    LOAD t0, 5*REGBYTES(sp)
    LOAD t1, 6*REGBYTES(sp)
    LOAD t2, 7*REGBYTES(sp)
    LOAD s0, 8*REGBYTES(sp)
    LOAD s1, 9*REGBYTES(sp)
    LOAD a0,10*REGBYTES(sp)
    LOAD a1,11*REGBYTES(sp)
    LOAD a2,12*REGBYTES(sp)
    LOAD a3,13*REGBYTES(sp)
    LOAD a4,14*REGBYTES(sp)
    LOAD a5,15*REGBYTES(sp)
    LOAD a6,16*REGBYTES(sp)
    LOAD a7,17*REGBYTES(sp)
    LOAD s2,18*REGBYTES(sp)
    LOAD s3,19*REGBYTES(sp)
    LOAD s4,20*REGBYTES(sp)
    LOAD s5,21*REGBYTES(sp)
    LOAD s6,22*REGBYTES(sp)
    LOAD s7,23*REGBYTES(sp)
    LOAD s8,24*REGBYTES(sp)
    LOAD s9,25*REGBYTES(sp)
    LOAD s10,26*REGBYTES(sp)
    LOAD s11,27*REGBYTES(sp)
    LOAD t3,28*REGBYTES(sp)
    LOAD t4,29*REGBYTES(sp)
    LOAD t5,30*REGBYTES(sp)
    LOAD t6,31*REGBYTES(sp)

    csrrw sp, mscratch, x0
    mret

do_reset:
    li x1, 0
    li x2, 0
    li x3, 0
    li x4, 0
    li x5, 0
    li x6, 0
    li x7, 0
    li x8, 0
    li x9, 0
    li x10, 0
    li x11, 0
    li x12, 0
    li x13, 0
    li x14, 0
    li x15, 0
    li x16, 0
    li x17, 0
    li x18, 0
    li x19, 0
    li x20, 0
    li x21, 0
    li x22, 0
    li x23, 0
    li x24, 0
    li x25, 0
    li x26, 0
    li x27, 0
    li x28, 0
    li x29, 0
    li x30, 0
    li x31, 0
    csrw mscratch, x0

    # write mtvec and make sure it sticks
    la t0, trap_vector
    csrw mtvec, t0
    csrr t1, mtvec
1:  bne t0, t1, 1b

    la sp, stacks + RISCV_PGSIZE - MENTRY_FRAME_SIZE
    j kinit

.bss
.align RISCV_PGSHIFT
stacks:
    .skip RISCV_PGSIZE * MAX_HARTS * 16
