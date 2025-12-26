; Najprostszy test - tylko wyświetla X
BITS 16
ORG 0x7C00

start:
    mov si, msg
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

msg: db 'X', 0

times 510-($-$$) db 0
dw 0xAA55
