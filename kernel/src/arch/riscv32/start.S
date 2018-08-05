#define KERNEL_BASE 0x80000000
#define PAGE_SHIFT 12
#define PAGE_SIZE 4096
#define SATP_SV32 0x10000000
#define PTE_V 1
#define PTE_R 2
#define PTE_W 4
#define PTE_X 8
#define PTE_SIZE 4

.section .text, "ax", %progbits
.globl start
start:
    tail kinit

.section .bss
boot_kstack:
.balign PAGE_SIZE
    .space 16 * PAGE_SIZE

.globl virtq
virtq:
.balign PAGE_SIZE
    .space PAGE_SIZE
