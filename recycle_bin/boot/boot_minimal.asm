; Minimal bootloader for 64-bit Rust kernel
BITS 16
ORG 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [boot_drive], dl
    sti

    ; Load kernel
    mov ah, 0x02
    mov al, 10
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    mov bx, 0x8000
    int 0x13

    ; Enable A20
    in al, 0x92
    or al, 2
    out 0x92, al

    ; Setup minimal identity-mapped page tables
    ; Clear 12KB at 0x1000
    mov di, 0x1000
    xor ax, ax
    mov cx, 0x1800
    rep stosb
    
    ; PML4[0] -> 0x2000
    mov dword [0x1000], 0x2003
    ; PDPT[0] -> 0x3000
    mov dword [0x2000], 0x3003
    ; PD[0] = 0-2MB identity
    mov dword [0x3000], 0x83
    ; PD[1] = 2-4MB identity  
    mov dword [0x3004], 0x200083
    
    ; Load GDT
    lgdt [gdt_desc]
    
    ; Enter protected mode
    mov eax, cr0
    or al, 1
    mov cr0, eax
    jmp 0x08:protected_start

BITS 32
protected_start:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ; Load page tables
    mov eax, 0x1000
    mov cr3, eax
    
    ; Enable PAE
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax
    
    ; Enable long mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr
    
    ; Enable paging
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    
    ; Jump to 64-bit
    jmp 0x18:long_start

BITS 64
long_start:
    ; Clear segments
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ; Jump to kernel
    mov rax, 0x8000
    call rax
    
hang:
    hlt
    jmp hang

; GDT
align 8
gdt_start:
    dq 0                        ; Null descriptor
    dq 0x00CF9A000000FFFF       ; 32-bit code
    dq 0x00CF92000000FFFF       ; 32-bit data
    dq 0x00AF9A000000FFFF       ; 64-bit code
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

boot_drive: db 0

times 510-($-$$) db 0
dw 0xAA55
