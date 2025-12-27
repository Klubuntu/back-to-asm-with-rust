; multiboot.asm - Multiboot2 header for GRUB (64-bit support)
BITS 32

section .multiboot
align 8
multiboot2_header:
    dd 0xE85250D6                ; Multiboot2 magic number
    dd 0                         ; Architecture: 0 (i386)
    dd multiboot2_header_end - multiboot2_header  ; Header length
    dd -(0xE85250D6 + 0 + (multiboot2_header_end - multiboot2_header)) ; Checksum

    ; End tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
multiboot2_header_end:

section .text
global multiboot_entry
extern kernel_main

multiboot_entry:
    ; GRUB uruchamia nas w trybie Protected Mode (32-bit)
    ; Musimy przejść do Long Mode (64-bit) aby uruchomić kernel Rust
    
    cli
    
    ; Ustaw stos
    mov esp, stack_top
    
    ; Zero out 16KB for page tables at 0x1000
    mov edi, 0x1000
    mov cr3, edi
    xor eax, eax
    mov ecx, 0x1000
    rep stosd
    
    ; Build page tables
    mov edi, cr3
    
    ; PML4[0] -> PDPT at 0x2000
    mov DWORD [edi], 0x2003
    
    ; PDPT[0] -> PDT at 0x3000  
    mov edi, 0x2000
    mov DWORD [edi], 0x3003
    
    ; PDT[0] -> 2MB page at 0x0 (identity map first 6MB)
    mov edi, 0x3000
    mov DWORD [edi], 0x00000083        ; 0-2MB
    mov DWORD [edi + 8], 0x00200083    ; 2MB-4MB
    mov DWORD [edi + 16], 0x00400083   ; 4MB-6MB
    
    ; Enable PAE
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax
    
    ; Set long mode bit in EFER MSR
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr
    
    ; Enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    
    ; Load 64-bit GDT
    lgdt [gdt64_descriptor]
    
    ; Jump to 64-bit code
    jmp 0x08:long_mode_start

BITS 64
long_mode_start:
    ; Clear segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ; Set up stack for 64-bit mode
    mov rsp, stack_top
    
    ; Call Rust kernel
    call kernel_main
    
    ; Jeśli kernel_main zwróci (nie powinien), zatrzymaj się
    cli
.hang:
    hlt
    jmp .hang

; 64-bit GDT
section .rodata
align 8
gdt64:
    dq 0                        ; Null descriptor
    dq 0x00AF9A000000FFFF      ; Code segment (64-bit)
    dq 0x00CF92000000FFFF      ; Data segment

gdt64_descriptor:
    dw gdt64_descriptor - gdt64 - 1
    dq gdt64

; Stack
section .bss
align 16
stack_bottom:
    resb 16384  ; 16 KB stack
stack_top:

; Mark stack as non-executable (remove linker warning)
section .note.GNU-stack noalloc noexec nowrite progbits
