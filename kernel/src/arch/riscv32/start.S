#define PAGE_SIZE 4096

.section .text, "ax", %progbits
.globl start
start:
    tail kreset

.section .bss
boot_kstack:
.balign PAGE_SIZE
    .space 16 * PAGE_SIZE

.globl virtq
virtq:
.balign PAGE_SIZE
    .space PAGE_SIZE
