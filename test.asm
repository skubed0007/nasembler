section .data
    hello db 'Hello, World!', 0x0A, 0

section .text
    global _start

_start:
    ; Write the message to stdout
    mov rax, 1          ; sys_write
    mov rdi, 1          ; stdout
    lea rsi, [hello]    ; RIP-relative addressing (automatic)
    mov rdx, 14         ; message length
    syscall

    ; Exit
    mov rax, 60         ; sys_exit
    xor rdi, rdi        ; exit code 0
    syscall