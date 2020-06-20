global _start

section .data

str: db "Hello, world!", 0xA
len: equ $-str

section .text

_start:
    mov rax, 0x2100_0004        ; write
    mov	rdi, 1                  ; stdout
    mov rsi, str                ; buf
    mov rdx, len                ; len
    syscall

    mov rax, 1                  ; exit
    xor rdi, rdi                ; status
    syscall
