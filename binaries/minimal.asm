global _start

section .data

str: db "Hello, world!", 0xA
len: equ $-str

section .text

_start:
	mov rax, 1
	mov rdi, 1
    mov rsi, str
	mov rdx, len
    syscall

    mov rax, 60
    xor rdi, rdi
    syscall
