; Prosty test bootloadera - tylko wyświetla OK jeśli załaduje dane
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

    ; Try to load
    mov ah, 0x02
    mov al, 1
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    mov bx, 0x8000
    int 0x13
    jc disk_error
    
    ; Success
    mov si, success_msg
    call print_string
    jmp $

disk_error:
    mov si, error_msg
    call print_string
    jmp $

print_string:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    int 0x10
    jmp print_string
.done:
    ret

success_msg: db 'OK!', 0
error_msg: db 'ERROR!', 0
boot_drive: db 0

times 510-($-$$) db 0
dw 0xAA55
