; This is a test file with multiple errors

section .text
    global _start

_start:
    ; Error 1: Undefined label
    jmp missing_label

    ; Error 2: Duplicate label
duplicate:
    mov rax, 1
    mov rdi, 1
    mov rsi, message
    mov rdx, 13
    syscall

; Error 3: Another duplicate label
duplicate:
    mov rax, 60
    mov rdi, 0
    syscall

    ; Error 4: Invalid instruction
    invalidinstr rax, rbx

    ; Error 5: Missing operand
    mov rax

    ; Error 6: Invalid register
    mov xyz, 42

    ; Error 7: Invalid section name
section 123invalid

    ; Error 8: Invalid memory reference
    mov rax, [rbx+*4]

section .data
    ; Error 9: Unclosed string
    message db "Hello, World 