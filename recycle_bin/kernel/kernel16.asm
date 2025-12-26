; Simple 16-bit kernel
BITS 16
ORG 0x8000

kernel_start:
    ; We're still in 16-bit real mode
    ; Display "123" using BIOS
    mov ax, 0xB800
    mov es, ax
    xor di, di
    
    mov byte [es:di], '1'
    mov byte [es:di+1], 0x0F
    mov byte [es:di+2], '2'
    mov byte [es:di+3], 0x0F
    mov byte [es:di+4], '3'
    mov byte [es:di+5], 0x0F
    
hang:
    hlt
    jmp hang
