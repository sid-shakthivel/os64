ENTRY(start)
OUTPUT_FORMAT(elf64-x86-64)

SECTIONS
{
    . = 0x19ff000; 

    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.text) *(.text.*)
    }

    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata) *(.rodata.*)
    }

    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data) *(.data.*)
    }

    .data.rel.ro : ALIGN(4K)
    {
         *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
     }

    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss) *(.bss.*)
    }

    /* Beginning of heap */
    . = ALIGN(4k);
    __kernel_end = .;
}