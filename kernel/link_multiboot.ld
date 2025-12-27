ENTRY(multiboot_entry)

SECTIONS
{
    . = 1M;

    /* Multiboot header MUST be in first 8KB */
    .multiboot ALIGN(4K) : AT(ADDR(.multiboot))
    {
        KEEP(*(.multiboot))
    }

    .text ALIGN(4K) : AT(ADDR(.text))
    {
        *(.text .text.*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata))
    {
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data))
    {
        *(.data .data.*)
    }

    .bss ALIGN(4K) : AT(ADDR(.bss))
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note .note.*)
    }
}
