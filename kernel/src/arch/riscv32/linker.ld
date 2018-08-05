ENTRY(start)
OUTPUT_ARCH(riscv)

KERNEL_BASE = 0x80000000;

SECTIONS {
  . = 0x80000000;
  _ftext = .;
  PROVIDE( eprol = . );

  .text :
  {
    *(.text.init)
  }

  .text :
  {
    *(.text)
    *(.text.*)
  }

  .rodata :
  {
    *(.rdata)
    *(.rodata)
    *(.rodata.*)
  }

  PROVIDE( etext = . );
  _etext = .;

  . = ALIGN(16);
  _fdata = .;

  .data :
  {
    *(.data)
    *(.data.*)
    *(.srodata*)
    *(.padata)
    *(.comment)
  }

  . = ALIGN(4);
  PROVIDE( edata = . );
  _edata = .;

  /* Start of uninitialized data segment */
  . = .;
  _fbss = .;

  /* sbss: Uninitialized writeable small data section */
  . = .;

  /* bss: Uninitialized writeable data section */
  . = .;
  _bss_start = .;
  .bss :
  {
    *(.bss)
    *(.bss.*)
    *(.sbss*)
    *(COMMON)
  }

  _end = .;
}
