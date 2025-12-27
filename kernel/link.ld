ENTRY(_start)

SECTIONS
{
    . = 0x8000;

    .text : AT(0x8000) {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    /* Zmniejszamy wyrównanie do 16 bajtów, aby dane były zaraz po kodzie */
    .rodata : ALIGN(16) { 
        *(.rodata .rodata.*) 
    }
    
    .data : ALIGN(16) { 
        *(.data .data.*) 
    }
    
    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.debug*)    /* To wyrzuci te ścieżki src/main.rs */
        *(.eh_frame)
    }
}