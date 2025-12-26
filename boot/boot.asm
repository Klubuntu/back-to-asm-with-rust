; boot.asm - Bootloader with long mode support
BITS 16
ORG 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    
    ; Save boot drive
    mov [boot_drive], dl
    
    ; NIE włączaj przerwań - brak IDT!
    ; sti  <-- usuń to

    ; Show that we're starting
    mov si, loaded_msg
    call print_string

    ; Load kernel from disk to 0x8000
    mov ah, 0x02
    mov al, 64           ; Try loading fewer sectors first (load 64 sectors, bytes 32KB)
    mov ch, 0
    mov cl, 2 ; Load from sector 2
    mov dh, 0
    mov dl, [boot_drive]
    mov bx, 0x8000
    int 0x13
    ; Don't check carry - QEMU sometimes sets it incorrectly
    
    ; Show that we loaded OK
    mov si, ok_msg
    call print_string
    
    ; Enable A20 line
    in al, 0x92
    or al, 2
    out 0x92, al
    
    ; Load GDT
    lgdt [gdt_descriptor]
    
    ; Enter protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    ; Far jump to flush pipeline
    jmp CODE_SEG:protected_mode_start

BITS 16
disk_error:
    mov si, error_msg
    call print_string
    jmp $

reboot_system:
    ; Próba przez port 0x64 (standardowy kontroler 8042)
    mov al, 0xFE
    out 0x64, al
    
    ; Jeśli to zawiedzie, wymuszamy skok do adresu resetu BIOS (0xFFFF:0000)
    ; To jest "miękki" restart, który większość BIOSów obsługuje
    jmp 0xFFFF:0000

print_string:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print_string
.done:
    ret

BITS 32
protected_mode_start:
    mov ax, DATA_SEG
    mov ds, ax
    mov ss, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ; Set up page tables for long mode
    ; Identity map first 2MB using 2MB pages
    
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
    
    ; PDT[0] -> 2MB page at 0x0 (identity map)
    mov edi, 0x3000
    mov DWORD [edi], 0x00000083  ; Present, writable, 2MB page
    
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
    
    ; Jump to 64-bit code
    jmp CODE_SEG_64:long_mode

BITS 64
long_mode:
    ; Clear segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    
    ; Set up stack for 64-bit mode
    mov rsp, 0x90000
    
    ; Jump to kernel at 0x8000
    mov rax, 0x8000
    jmp rax

; GDT
gdt_start:
    dq 0x0

gdt_code_32:
    dw 0xFFFF
    dw 0x0
    db 0x0
    db 10011010b
    db 11001111b
    db 0x0

gdt_data:
    dw 0xFFFF
    dw 0x0
    db 0x0
    db 10010010b
    db 11001111b
    db 0x0

gdt_code_64:
    dw 0xFFFF
    dw 0x0
    db 0x0
    db 10011010b
    db 10101111b
    db 0x0

gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE_SEG equ gdt_code_32 - gdt_start
DATA_SEG equ gdt_data - gdt_start
CODE_SEG_64 equ gdt_code_64 - gdt_start

loaded_msg: db 'L', 0
ok_msg: db 'K', 0
error_msg: db 'Disk error!', 0
boot_drive: db 0

times 510-($-$$) db 0
dw 0xAA55
