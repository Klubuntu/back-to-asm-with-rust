; Simple 16-bit kernel that displays "123"
BITS 16
ORG 0x8000

start:
    ; Display "123" using VGA text mode
    mov ax, 0xB800      ; VGA text mode segment
    mov es, ax
    xor di, di
    
    mov byte [es:di], '1'
    mov byte [es:di+1], 0x0F    ; White on black
    mov byte [es:di+2], '2'
    mov byte [es:di+3], 0x0F
    mov byte [es:di+4], '3'
    mov byte [es:di+5], 0x0F
    
hang:
    hlt
    jmp hang
