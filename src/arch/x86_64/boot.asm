attrib:
    .present              equ 1 << 7
    .ring1                equ 1 << 5
    .ring2                equ 1 << 6
    .ring3                equ 1 << 5 | 1 << 6
    .user                 equ 1 << 4
;user
    .code                 equ 1 << 3
;   code
    .conforming           equ 1 << 2
    .readable             equ 1 << 1
;   data
    .expand_down          equ 1 << 2
    .writable             equ 1 << 1
    .accessed             equ 1 << 0
;system
;   legacy
    .tssAvailabe16        equ 0x1
    .ldt                  equ 0x2
    .tssBusy16            equ 0x3
    .call16               equ 0x4
    .task                 equ 0x5
    .interrupt16          equ 0x6
    .trap16               equ 0x7
    .tssAvailabe32        equ 0x9
    .tssBusy32            equ 0xB
    .call32               equ 0xC
    .interrupt32          equ 0xE
    .trap32               equ 0xF
;   long mode
    .ldt32                equ 0x2
    .tssAvailabe64        equ 0x9
    .tssBusy64            equ 0xB
    .call64               equ 0xC
    .interrupt64          equ 0xE
    .trap64               equ 0xF

flags:
    .granularity equ 1 << 7
    .available equ 1 << 4
;user
    .default_operand_size equ 1 << 6
;   code
    .long_mode equ 1 << 5
;   data
    .reserved equ 1 << 5

struc GDTEntry
  .limitl resw 1
  .basel resw 1
  .basem resb 1
  .attribute resb 1
  .flags__limith resb 1
  .baseh resb 1
endstruc

global start
extern long_mode_start

section .text
bits 32
start:
  mov esp, stack_top
  mov edi, ebx                  ; Move Multiboot info pointer to edi

  call check_multiboot
  call check_cpuid
  call check_long_mode

  call set_up_page_tables
  call enable_paging

  ;; load the 64-bit GDT
  lgdt [gdt64.pointer]

	; update selectors
	mov ax, gdt64.kernel_data
	mov ss, ax  ; stack selector
	mov ds, ax  ; data selector
	mov es, ax  ; extra selector

  call set_up_SSE

  jmp gdt64.kernel_code:long_mode_start

; Prints `ERR: ` and t  he given error code to screen and hangs.
; parameter: error cod  e (in ascii) in al
error:
  mov dword [0xb8000],   0x4f524f45
  mov dword [0xb8004],   0x4f3a4f52
  mov dword [0xb8008],   0x4f204f20
  mov byte  [0xb800a],   al
  hlt

check_multiboot:
  cmp eax, 0x36d76289
  jne .no_multiboot
  ret
.no_multiboot:
  mov al, "0"
  jmp error

check_cpuid:
  ; Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
  ; the FLAGS register. If we can flip it, CPUID is available.

  ; Copy FLAGS in to EAX via stack
  pushfd
  pop eax

  ; Copy to ECX as well for comparing later on
  mov ecx, eax

  ; Flip the ID bit
  xor eax, 1 << 21

  ; Copy EAX to FLAGS via the stack
  push eax
  popfd

  ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
  pushfd
  pop eax

  ; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
  ; back if it was ever flipped).
  push ecx
  popfd

  ; Compare EAX and ECX. If they are equal then that means the bit wasn't
  ; flipped, and CPUID isn't supported.
  xor eax, ecx
  jz .no_cpuid
  ret
.no_cpuid:
  mov al, "1"
  jmp error

check_long_mode:
  mov eax, 0x80000000    ; Set the A-register to 0x80000000.
  cpuid                  ; CPU identification.
  cmp eax, 0x80000001    ; Compare the A-register with 0x80000001.
  jb .no_long_mode       ; It is less, there is no long mode.
  mov eax, 0x80000001    ; Set the A-register to 0x80000001.
  cpuid                  ; CPU identification.
  test edx, 1 << 29      ; Test if the LM-bit is set in the D-register.
  jz .no_long_mode       ; They aren't, there is no long mode.
  ret
.no_long_mode:
  mov al, "2"
  jmp error

set_up_page_tables:
	;; recursive mapping
	mov eax, p4_table
	or eax, 0b11                  ; present + writable
	mov [p4_table + 511 * 8], eax

  ;; map first P4 entry to P3 table
  mov eax, p3_table
  or eax, 0b11                  ; present + writable
  mov [p4_table], eax

  ;; map first P3 entry to P2 table
  mov eax, p2_table
  or eax, 0b11
  mov [p3_table], eax

  ;; map each P2 entry to a hug 2MiB page
  mov ecx, 0

.map_p2_table:
  ;; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
  mov eax, 0x200000             ; 2MiB
  mul ecx                       ; start address of ecx-th page
  or eax, 0b10000011            ; present + writable + hug
  mov [p2_table + ecx * 8], eax ; map ecx-th entry

  inc ecx                       ; increase counter
  cmp ecx, 512                  ; if counter == 512, the whole P2 table is mapped
  jne .map_p2_table             ; else map the next entry

  ret

enable_paging:
  ; load P4 to cr3 register (cpu uses this to access the P4 table)
  mov eax, p4_table
  mov cr3, eax

  ; enable PAE-flag in cr4 (Physical Address Extension)
  mov eax, cr4
  or eax, 1 << 5
  mov cr4, eax

  ; set the long mode bit in the EFER MSR (model specific register)
  mov ecx, 0xC0000080
  rdmsr
  or eax, 1 << 8
  wrmsr

  ; enable paging in the cr0 register
  mov eax, cr0
  or eax, 1 << 31
  mov cr0, eax

  ret

; Check for SSE and enable it. If it's not supported throw error "a".
set_up_SSE:
  ; check for SSE
  mov eax, 0x1
  cpuid
  test edx, 1<<25
  jz .no_SSE

  ; enable SSE
  mov eax, cr0
  and ax, 0xFFFB      ; clear coprocessor emulation CR0.EM
  or ax, 0x2          ; set coprocessor monitoring  CR0.MP
  mov cr0, eax
  mov eax, cr4
  or ax, 3 << 9       ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
  mov cr4, eax

  ret
.no_SSE:
  mov al, "a"
  jmp error

section .bss
align 4096
p4_table:                       ; This will become a guard page to detect stack overflow
  resb 4096
p3_table:                       ; p3 and p2 will be part of the stack after page switching
  resb 4096
p2_table:
  resb 4096
stack_bottom:
  resb 4096 * 4
stack_top:

section .rodata
gdt64:
.null: equ $ - gdt64
  dq 0

.kernel_code: equ $ - gdt64
istruc GDTEntry
  at GDTEntry.limitl, dw 0
  at GDTEntry.basel, dw 0
	at GDTEntry.basem, db 0
	at GDTEntry.attribute, db attrib.present | attrib.user | attrib.code
	at GDTEntry.flags__limith, db flags.long_mode
	at GDTEntry.baseh, db 0
iend

.kernel_data: equ $ - gdt64
istruc GDTEntry
  at GDTEntry.limitl, dw 0
  at GDTEntry.basel, dw 0
  at GDTEntry.basem, db 0
  at GDTEntry.attribute, db attrib.present | attrib.user | attrib.writable
  at GDTEntry.flags__limith, db 0
  at GDTEntry.baseh, db 0
iend

.user_code: equ $ - gdt64
istruc GDTEntry
  at GDTEntry.limitl, dw 0
  at GDTEntry.basel, dw 0
  at GDTEntry.basem, db 0
  at GDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.user | attrib.code
  at GDTEntry.flags__limith, db flags.long_mode
  at GDTEntry.baseh, db 0
iend

.user_data: equ $ - gdt64
istruc GDTEntry
  at GDTEntry.limitl, dw 0
  at GDTEntry.basel, dw 0
  at GDTEntry.basem, db 0
  at GDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.user | attrib.writable
  at GDTEntry.flags__limith, db 0
  at GDTEntry.baseh, db 0
iend

.pointer:
  dw $ - gdt64 - 1
  dq gdt64
