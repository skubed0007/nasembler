# NASembler Example Programs

This document provides a collection of example programs demonstrating common patterns and techniques when using NASembler.

## Table of Contents

- [Hello World Example](#hello-world-example)
- [File Operations](#file-operations)
- [Arithmetic Examples](#arithmetic-examples)
- [Control Flow Examples](#control-flow-examples)
- [Memory Management](#memory-management)
- [Function Calls](#function-calls)
- [Using Syscalls](#using-syscalls)

## Hello World Example

A basic "Hello, World!" program in x86-64 assembly:

```asm
section .data
    message db "Hello, World!", 10, 0  ; String with newline and null terminator

section .text
    global _start

_start:
    ; Write the message to stdout
    mov rax, 1          ; syscall number for sys_write
    mov rdi, 1          ; file descriptor 1 (stdout)
    mov rsi, message    ; pointer to message
    mov rdx, 14         ; message length (including newline)
    syscall

    ; Exit the program
    mov rax, 60         ; syscall number for sys_exit
    xor rdi, rdi        ; exit code 0
    syscall
```

## File Operations

### Reading from a File

```asm
section .data
    filename db "input.txt", 0
    
section .bss
    buffer resb 1024    ; Reserve 1024 bytes for file data

section .text
    global _start

_start:
    ; Open the file
    mov rax, 2          ; syscall number for sys_open
    mov rdi, filename   ; pointer to filename
    mov rsi, 0          ; flags (O_RDONLY)
    mov rdx, 0          ; mode (ignored for O_RDONLY)
    syscall
    
    ; Save file descriptor
    mov r8, rax         ; save file descriptor in r8
    
    ; Read from the file
    mov rax, 0          ; syscall number for sys_read
    mov rdi, r8         ; file descriptor
    mov rsi, buffer     ; buffer pointer
    mov rdx, 1024       ; buffer size
    syscall
    
    ; rax now contains the number of bytes read
    
    ; Close the file
    mov rax, 3          ; syscall number for sys_close
    mov rdi, r8         ; file descriptor
    syscall
    
    ; Exit the program
    mov rax, 60         ; syscall number for sys_exit
    xor rdi, rdi        ; exit code 0
    syscall
```

### Writing to a File

```asm
section .data
    filename db "output.txt", 0
    message db "Hello, File!", 10, 0
    msglen equ $ - message - 1  ; Length of the message
    
section .text
    global _start

_start:
    ; Open the file (create if it doesn't exist)
    mov rax, 2          ; syscall number for sys_open
    mov rdi, filename   ; pointer to filename
    mov rsi, 65         ; flags (O_WRONLY | O_CREAT)
    mov rdx, 0644o      ; mode (permissions)
    syscall
    
    ; Save file descriptor
    mov r8, rax         ; save file descriptor in r8
    
    ; Write to the file
    mov rax, 1          ; syscall number for sys_write
    mov rdi, r8         ; file descriptor
    mov rsi, message    ; buffer pointer
    mov rdx, msglen     ; buffer size
    syscall
    
    ; Close the file
    mov rax, 3          ; syscall number for sys_close
    mov rdi, r8         ; file descriptor
    syscall
    
    ; Exit the program
    mov rax, 60         ; syscall number for sys_exit
    xor rdi, rdi        ; exit code 0
    syscall
```

## Arithmetic Examples

### Basic Math Operations

```asm
section .text
    global _start

_start:
    ; Addition
    mov rax, 5
    add rax, 10         ; rax = 5 + 10 = 15
    
    ; Subtraction
    mov rbx, 20
    sub rbx, 7          ; rbx = 20 - 7 = 13
    
    ; Multiplication
    mov rax, 6
    mov rcx, 7
    mul rcx             ; rax = 6 * 7 = 42 (unsigned multiplication)
    
    ; Signed multiplication
    mov rax, -4
    mov rcx, 5
    imul rcx            ; rax = -4 * 5 = -20 (signed multiplication)
    
    ; Division (64-bit by 32-bit)
    mov rax, 100
    mov rcx, 8
    xor rdx, rdx        ; Clear rdx for division
    div ecx             ; rax = 100 / 8 = 12, rdx = 100 % 8 = 4
    
    ; Exit
    mov rax, 60
    xor rdi, rdi
    syscall
```

### Bit Manipulation

```asm
section .text
    global _start

_start:
    ; Bitwise AND
    mov rax, 0b1010
    and rax, 0b1100     ; rax = 0b1000 (8 in decimal)
    
    ; Bitwise OR
    mov rbx, 0b1010
    or rbx, 0b0101      ; rbx = 0b1111 (15 in decimal)
    
    ; Bitwise XOR
    mov rcx, 0b1010
    xor rcx, 0b1111     ; rcx = 0b0101 (5 in decimal)
    
    ; Clear register using XOR
    xor rdx, rdx        ; rdx = 0 (more efficient than mov rdx, 0)
    
    ; Bit shifting
    mov r8, 1
    shl r8, 3           ; r8 = 1 << 3 = 8 (left shift, multiply by 2^3)
    
    mov r9, 16
    shr r9, 2           ; r9 = 16 >> 2 = 4 (right shift, divide by 2^2)
    
    ; Exit
    mov rax, 60
    xor rdi, rdi
    syscall
```

## Control Flow Examples

### Conditional Jumps

```asm
section .data
    result db 0

section .text
    global _start

_start:
    mov rax, 10
    mov rbx, 20
    
    ; Compare rax and rbx
    cmp rax, rbx
    jg greater          ; Jump if rax > rbx
    je equal            ; Jump if rax = rbx
    jl less             ; Jump if rax < rbx
    
greater:
    mov byte [result], 1
    jmp done
    
equal:
    mov byte [result], 0
    jmp done
    
less:
    mov byte [result], -1
    
done:
    ; Exit
    mov rax, 60
    movsx rdi, byte [result]  ; Sign extend the result as exit code
    syscall
```

### Loops

```asm
section .data
    counter db 10

section .text
    global _start

_start:
    ; Loop using a counter
    mov rcx, [counter]
    
loop_start:
    ; Loop body
    ; ... (do something)
    
    ; Decrement counter and check if done
    dec rcx
    cmp rcx, 0
    jg loop_start       ; Jump if rcx > 0
    
    ; Alternate loop using jnz
    mov rcx, [counter]
    
loop2_start:
    ; Loop body
    ; ... (do something else)
    
    dec rcx
    test rcx, rcx       ; Test sets ZF if rcx is zero
    jnz loop2_start     ; Jump if rcx is not zero
    
    ; Exit
    mov rax, 60
    xor rdi, rdi
    syscall
```

## Memory Management

### Array Access

```asm
section .data
    array dd 10, 20, 30, 40, 50    ; Array of 5 dwords (32-bit integers)

section .text
    global _start

_start:
    ; Access array elements
    mov eax, [array]              ; First element (10)
    mov ebx, [array + 4]          ; Second element (20)
    mov ecx, [array + 8]          ; Third element (30)
    
    ; Array indexing with a variable
    mov rdx, 3                    ; Index
    mov r8d, [array + rdx*4]      ; Fourth element (40)
    
    ; Exit
    mov rax, 60
    xor rdi, rdi
    syscall
```

### Allocating Memory on the Stack

```asm
section .text
    global _start

_start:
    ; Allocate 16 bytes on the stack
    sub rsp, 16
    
    ; Use the stack memory
    mov qword [rsp], 42           ; Store value at the beginning
    mov qword [rsp+8], 100        ; Store another value 8 bytes in
    
    ; Work with allocated memory
    mov rax, [rsp]                ; Load first value
    add rax, [rsp+8]              ; Add second value
    
    ; Deallocate memory
    add rsp, 16
    
    ; Exit with the calculated sum
    mov rax, 60
    mov rdi, rax
    syscall
```

## Function Calls

### Simple Function

```asm
section .text
    global _start

_start:
    ; Call the function
    call add_numbers
    
    ; Exit with result
    mov rax, 60
    mov rdi, rcx       ; Use function result as exit code
    syscall

; Function to add two numbers
; Input: None (uses hard-coded values)
; Output: rcx = sum
add_numbers:
    mov rax, 25
    mov rbx, 17
    mov rcx, rax
    add rcx, rbx       ; rcx = rax + rbx
    ret
```

### Function with Parameters on the Stack

```asm
section .text
    global _start

_start:
    ; Prepare function arguments (push in reverse order)
    push 17            ; Second argument
    push 25            ; First argument
    
    ; Call the function
    call add_numbers
    
    ; Clean up the stack after the call
    add rsp, 16        ; Deallocate 16 bytes (2 arguments × 8 bytes)
    
    ; Exit with result
    mov rax, 60
    mov rdi, rcx       ; Use function result as exit code
    syscall

; Function to add two numbers
; Input:  [rsp+8] = first number
;         [rsp+16] = second number
; Output: rcx = sum
add_numbers:
    ; Function prologue
    push rbp           ; Save old base pointer
    mov rbp, rsp       ; Set new base pointer
    
    ; Access parameters using base pointer
    mov rax, [rbp+16]  ; First argument
    mov rbx, [rbp+24]  ; Second argument
    
    ; Perform addition
    mov rcx, rax
    add rcx, rbx       ; rcx = rax + rbx
    
    ; Function epilogue
    pop rbp            ; Restore old base pointer
    ret
```

## Using Syscalls

The x86-64 Linux syscall convention:

1. Syscall number goes in `rax`
2. Arguments go in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`
3. Return value comes in `rax`

### Common Syscalls

```asm
section .data
    message db "Example syscalls", 10, 0
    msglen equ $ - message - 1

section .text
    global _start

_start:
    ; sys_write (write to stdout)
    mov rax, 1          ; sys_write
    mov rdi, 1          ; stdout file descriptor
    mov rsi, message    ; pointer to message
    mov rdx, msglen     ; message length
    syscall
    
    ; sys_getpid (get process ID)
    mov rax, 39         ; sys_getpid
    syscall
    ; rax now contains the process ID
    
    ; sys_gettimeofday (get current time)
    sub rsp, 32         ; Allocate space for timespec structs
    mov rax, 96         ; sys_gettimeofday
    mov rdi, rsp        ; pointer to timeval struct
    mov rsi, rsp+16     ; pointer to timezone struct (can be NULL)
    syscall
    add rsp, 32         ; Deallocate space
    
    ; sys_exit (exit program)
    mov rax, 60         ; sys_exit
    xor rdi, rdi        ; exit code 0
    syscall
```

Remember to check the Linux syscall table for more available syscalls and their numbers. The syscall numbers may vary between different kernel versions, but the most common ones are stable.

## Testing and Running Examples

To assemble and run an example with NASembler:

```
nasembler example.asm -x
```

This will:
1. Assemble the `example.asm` file
2. Generate an ELF executable
3. Execute the resulting binary 