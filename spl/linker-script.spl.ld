OUTPUT_ARCH("riscv")
ENTRY(all_harts_entry)

/*
SRAM: 0xC080_0000 - 0xC084_0000 (256KB 0x4_0000)
    FSBL: 0xC080_0000 - 0xC083_4000 (208KB 0x3_4000)
	    HEADER: 0xC080_0000 - 0xC080_1000 (4KB 0x1000)
	    SPL: 0xC080_1000 - 0xC083_4000 (204KB 0x3_3000)
		    .text
			.rodata
			.data
    BSS: 0xC083_7000 - 0xC083_9000 (8KB 0x2000)
	STACK: 0xC083_9000 - 0xC084_0000 (28KB 0x7000)
*/
MEMORY {
    SPL_SRAM (rwx) : ORIGIN = 0xC0801000, LENGTH = 0x33000
    BSS_SRAM (rw)  : ORIGIN = 0xC0837000, LENGTH = 0x2000
}

PROVIDE(STACK_TOP = 0xC0840000);

SECTIONS {
	.text : ALIGN(4) {
		*(.text.entry)
		*(.text.*)
        *(.text)
	} > SPL_SRAM

	.rodata : ALIGN(4) {
		*(SORT_BY_ALIGNMENT(SORT_BY_NAME(.rodata*)))
	} > SPL_SRAM

	.data : ALIGN(4) {
		__data_start = .;
		*(.data*)
		*(.sdata*)
		__data_end = .;
	} > SPL_SRAM

    . = ALIGN(4);
    _image_end = .;

    .bss (NOLOAD) : ALIGN(8) {
		__bss_start = .;
		*(.bss*)
		*(.sbss*)
		. = ALIGN(8);
		__bss_end = .;
	} > BSS_SRAM

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
        *(.note*)
    }
}