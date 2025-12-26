; Simple bootloader - stays in 16-bit mode
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

    ; Show that we're starting
    mov si, loaded_msg
    call print_string

    ; Load kernel from disk to 0x8000
    mov ah, 0x02
    mov al, 5
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    mov bx, 0x8000
    int 0x13
    
    ; Show that we loaded OK
    mov si, ok_msg
    call print_string
    
    ; Jump to kernel (staying in 16-bit mode)
    jmp 0x0000:0x8000

print_string:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print_string
.done:
    ret

loaded_msg: db 'L', 0
ok_msg: db 'K', 0
boot_drive: db 0

times 510-($-$$) db 0
dw 0xAA55
