#define KERNEL_BASE 0xF0000000
#define PAGE_SHIFT 12
#define PAGE_SIZE 4096
#define SATP_SV32 0x10000000
#define PTE_V 1
#define PTE_R 2
#define PTE_W 4
#define PTE_X 8
#define PTE_SIZE 4

/* === Multiboot Header === */
MULTIBOOT_PAGE_ALIGN  =  (1<<0)
MULTIBOOT_MEMORY_INFO =  (1<<1)
/* REQVIDMODE is not used, because it is not supported by qemu */
MULTIBOOT_REQVIDMODE  =  (1<<2)
MULTIBOOT_HEADER_MAGIC =  0x1BADB002
MULTIBOOT_HEADER_FLAGS = (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO)
MULTIBOOT_CHECKSUM     = -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
.section .multiboot, "a"
.globl multiboot
multiboot:
    .long MULTIBOOT_HEADER_MAGIC
    .long MULTIBOOT_HEADER_FLAGS
    .long MULTIBOOT_CHECKSUM
    .long multiboot
    /* a.out kludge (not used, the kernel is elf) */
    .long 0, 0, 0, 0    /* load_addr, load_end_addr, bss_end_addr, entry_addr */
    /* Video mode */
    .long 0     /* Mode type (0: LFB) */
    .long 0     /* Width (no preference) */
    .long 0     /* Height (no preference) */
    .long 32    /* Depth (32-bit preferred) */

.section .text, "ax", %progbits
.globl start
start:
    # setup paging
    la t0, boot_pd
    srli t0, t0, PAGE_SHIFT
    li t1, SATP_SV32
    or t0, t0, t1
    csrw satp, t0

    # reallocate
    la t0, reallocated
    li t1, KERNEL_BASE
    add t0, t0, t1
    add a1, a1, t1
    jr t0

reallocated:
    # setup sp
    mv sp, a0
    addi sp, sp, 1
    slli sp, sp, PAGE_SHIFT
    la t0, boot_kstack
    add sp, sp, t0

    # a0 = hartid
    # a1 = fdt
    tail kinit

.section .data
.balign PAGE_SIZE
.globl boot_pd
boot_pd:
    # map va 0 + 4mb to pa 0 + 4mb
    .long 0x00000000 + (PTE_V | PTE_R | PTE_W | PTE_X)
    .space (KERNEL_BASE >> 22) * PTE_SIZE - (. - boot_pd)
    # map va KERNEL_BASE + 4mb to pa 0 + 4mb
    .long 0x00000000 + (PTE_V | PTE_R | PTE_W | PTE_X)
    .space PAGE_SIZE - (. - boot_pd)

.section .bss
boot_kstack:
.balign PAGE_SIZE
    .space 16 * PAGE_SIZE

.globl virtq
virtq:
.balign PAGE_SIZE
    .space PAGE_SIZE
