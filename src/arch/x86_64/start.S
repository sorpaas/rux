/*
 * Rust BareBones OS
 * - By John Hodge (Mutabah/thePowersGang) 
 *
 * arcm/amd64/start.S
 * - AMD64 Entrypoint
 *
 * == LICENCE ==
 * This code has been put into the public domain, there are no restrictions on
 * its use, and the author takes no liability.
 */

/* The kernel is linked to run at -2GB. This allows efficient addressing */
KERNEL_BASE = 0xFFFFFFFF80000000

/* === Multiboot Header === */
MULTIBOOT_PAGE_ALIGN  =  (1<<0)
MULTIBOOT_MEMORY_INFO =  (1<<1)
MULTIBOOT_REQVIDMODE  =  (1<<2)
MULTIBOOT_HEADER_MAGIC =  0x1BADB002
MULTIBOOT_HEADER_FLAGS = (MULTIBOOT_PAGE_ALIGN | MULTIBOOT_MEMORY_INFO | MULTIBOOT_REQVIDMODE)
MULTIBOOT_CHECKSUM     = -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)
.section .multiboot, "a"
.globl mboot
mboot:
	.long MULTIBOOT_HEADER_MAGIC
	.long MULTIBOOT_HEADER_FLAGS
	.long MULTIBOOT_CHECKSUM
	.long mboot
	/* a.out kludge (not used, the kernel is elf) */
	.long 0, 0, 0, 0	/* load_addr, load_end_addr, bss_end_addr, entry_addr */
	/* Video mode */
	.long 0 	/* Mode type (0: LFB) */
	.long 0 	/* Width (no preference) */
	.long 0 	/* Height (no preference) */
	.long 32	/* Depth (32-bit preferred) */

#define DEBUG(c)	mov $0x3f8, %dx ; mov $c, %al ; outb %al, %dx

/* === Code === */
.section .inittext, "ax"
.globl start
.code32
start:
	/* The kernel starts in protected mode (32-bit mode, we want to switch to long mode) */
	
	/* 1. Save multiboot state */
	mov %eax, mboot_sig - KERNEL_BASE
	mov %ebx, mboot_ptr - KERNEL_BASE
	
	/* 2. Ensure that the CPU support long mode */
	mov $0x80000000, %eax
	cpuid
	/* - Check if CPUID supports the field we want to query */
	cmp $0x80000001, %eax
	jbe not64bitCapable
	/* - Test the IA-32e bit */
	mov $0x80000001, %eax
	cpuid
	test $0x20000000, %edx /* bit 29 = */
	jz not64bitCapable
	
	/* 3. Set up state for long mode */
	/* Enable:
	    PGE (Page Global Enable)
	  + PAE (Physical Address Extension)
	  + PSE (Page Size Extensions)
	*/
	mov %cr4, %eax
	or $(0x80|0x20|0x10), %eax
	mov %eax, %cr4

	/* Load PDP4 */
	mov $(init_pml4 - KERNEL_BASE), %eax
	mov %eax, %cr3

	/* Enable IA-32e mode (Also enables SYSCALL and NX) */
	mov $0xC0000080, %ecx
	rdmsr
	or $(1 << 11)|(1 << 8)|(1 << 0), %eax     /* NXE, LME, SCE */
	wrmsr

	/* Enable paging and enter long mode */
	mov %cr0, %eax
	or $0x80010000, %eax      /* PG & WP */
	mov %eax, %cr0
	lgdt GDTPtr_low - KERNEL_BASE
	ljmp $0x08, $start64


not64bitCapable:
	/* If the CPU isn't 64-bit capable, print a message to serial/b8000 then busy wait */
	mov $0x3f8, %dx
	mov $'N', %al ; outb %al, %dx
	movw $0x100|'N', 0xb8000
	mov $'o', %al ; outb %al, %dx
	movw $0x100|'o', 0xb8002
	mov $'t', %al ; outb %al, %dx
	movw $0x100|'t', 0xb8004
	mov $'6', %al ; outb %al, %dx
	movw $0x100|'6', 0xb8006
	mov $'4', %al ; outb %al, %dx
	movw $0x100|'4', 0xb8008
	
not64bitCapable.loop:
	hlt
	jmp not64bitCapable.loop
.code64
.globl start64
start64:
	/* Running in 64-bit mode, jump to high memory */
	lgdt GDTPtr
	mov $start64_high, %rax
	jmp *%rax

.section .text
.extern kmain
.globl start64_high
start64_high:
	/* and clear low-memory mapping */
	mov $0, %rax
	mov %rax, init_pml4 - KERNEL_BASE + 0
	
	/* Set up segment registers */
	mov $0x10, %ax
	mov %ax, %ss
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
	
	/* Set up stack pointer */
	mov $init_stack, %rsp
	
	/* call the rust code */
	call kmain

	/* and if that returns (it shouldn't) loop forever */
start64.loop:
	hlt
	jmp start64.loop

/*
 RDI = Destination
 RSI = Value
 RDX = Count
*/
.section .text.memset
.globl memset
memset:
	mov %rsi, %rax
	mov %rdx, %rcx
	rep stosb
	ret
/*
 RDI = Destination
 RSI = Source
 RDX = Count
*/
.section .text.memcpy
.globl memcpy
memcpy:
	mov %rdx, %rcx
	rep movsb
	ret


/* === Page-aligned data === */
.section .padata
/* Initial paging structures, four levels */
/* The +3 for sub-pages indicates "present (1) + writable (2)" */
init_pml4:
	.quad low_pdpt - KERNEL_BASE + 3	/* low map for startup, will be cleared before rust code runs */
	.rept 512 - 3
		.quad 0
	.endr
	.quad 0 	/* If you so wish, this is a good place for the "Fractal" mapping */
	.quad init_pdpt - KERNEL_BASE + 3	/* Final mapping */
low_pdpt:
	.quad init_pd - KERNEL_BASE + 3	/* early init identity map */
	.rept 512 - 1
		.quad 0
	.endr
init_pdpt:	/* covers the top 512GB, 1GB each entry */
	.rept 512 - 2
		.quad 0
	.endr
	.quad init_pd - KERNEL_BASE + 3	/* at -2GB, identity map the kernel image */
	.quad 0
init_pd:
	/* 0x80 = Page size extension */
	.quad 0x000000 + 0x80 + 3	/* Map 2MB, enough for a 1MB kernel */
	.quad 0x200000 + 0x80 + 3	/* - give it another 2MB, just in case */
	.rept 512 - 2
		.quad 0
	.endr 
init_stack_base:
	.rept 0x1000 * 2
		.byte 0
	.endr
init_stack:

/* === General Data === */
.section .data
.globl mboot_sig
.globl mboot_ptr
mboot_sig:	.long 0
mboot_ptr:	.long 0

/* Global Descriptor Table */
GDTPtr_low:
	.word GDTEnd - GDT - 1
	.long GDT - KERNEL_BASE
GDTPtr:
	.word GDTEnd - GDT - 1
	.quad GDT
.globl GDT
GDT:
	.long 0, 0
        .long 0x00000000, 0x00209A00	/* 0x08: 64-bit Code */
        .long 0x00000000, 0x00009200    /* 0x10: 64-bit Data */
        .long 0x00000000, 0x0040FA00    /* 0x18: 32-bit User Code */
        .long 0x00000000, 0x0040F200    /* 0x20: User Data        */
        .long 0x00000000, 0x0020FA00    /* 0x28: 64-bit User Code       */
        .long 0x00000000, 0x0000F200    /* 0x30: User Data (64 version) */
GDTEnd:
