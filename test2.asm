section .data
    hello db 'Hello, World!', 0
    newline db 10, 0  ; Newline character
    message db 'This is a test of multiple strings in our assembler!', 0

section .text
    global _start

_start:
    ; Write first message
    mov rax, 1          ; syscall number for write (1)
    mov rdi, 1          ; file descriptor (1 = stdout)
    lea rsi, [hello]    ; pointer to string
    mov rdx, 13         ; length of string
    syscall

    ; Write newline
    mov rax, 1
    mov rdi, 1
    lea rsi, [newline]
    mov rdx, 1
    syscall

    ; Write second message 
    mov rax, 1
    mov rdi, 1
    lea rsi, [message]
    mov rdx, 52         ; length of string (exactly 52 characters)
    syscall

    ; Write newline again
    mov rax, 1
    mov rdi, 1
    lea rsi, [newline]
    mov rdx, 1
    syscall

    ; Exit program
    mov rax, 60         ; syscall number for exit (60)
    xor rdi, rdi        ; exit code 0
    syscall 