section .data
    hello db 'Hello, World!', 0

section .text
    global _start

_start:
    ; Write to stdout
    mov rax, 1          ; syscall number for write (1)
    mov rdi, 1          ; file descriptor (1 = stdout)
    lea rsi, [hello]    ; pointer to string
    mov rdx, 13         ; length of string
    syscall

    ; Exit program
    mov rax, 60         ; syscall number for exit (60)
    xor rdi, rdi        ; exit code 0
    syscall