ENTRY(start)

SECTIONS {
    . = 1M;

     kernel_virtual_start = .;
     kernel_physical_start = . - 0xC0000000;

    .boot :
    {
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data.rel.ro : {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    kernel_virtual_end = .;
    kernel_physical_end = . - 0xC0000000;
}
