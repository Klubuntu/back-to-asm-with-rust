ENTRY(_start)

SECTIONS
{
    /* 0x8000 jest OK dla obrazów ładowanych jako "floppy/raw" */
    . = 0x8000;

    .text : {
        /* Musimy zagwarantować, że _start jest absolutnie pierwszy */
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    /* Wyrównanie do 8 bajtów wystarczy dla danych */
    .rodata : ALIGN(8) { *(.rodata .rodata.*) }
    .data   : ALIGN(8) { *(.data .data.*) }
    
    .bss : ALIGN(8) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
}