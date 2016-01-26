ENTRY(start)

SECTIONS {
	. = 1M;

	.rodata : ALIGN(4K)
	{
		/* ensure that the multiboot header is at the beginning */
		KEEP(*(.multiboot_header))
    *(.rodata .rodata.*)
	}

	.text : ALIGN(4K)
	{
		*(.text .text.*)
	}

  .data : ALIGN(4K)
  {
    *(.data .data.*)
  }

  .data.rel.ro : ALIGN(4K) {
    *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
  }

  .gcc_except_table : ALIGN(4K) {
    *(.gcc_except_table)
  }

}
