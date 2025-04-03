# x86-64 Assembly Language Guide

This guide serves as a reference for x86-64 assembly language, focusing on the syntax and features supported by nasembler.

## Assembly Basics

Assembly language is a low-level programming language that has a strong correspondence between its statements and machine code instructions. x86-64 assembly specifically targets the 64-bit extension of the x86 instruction set used in modern processors from Intel and AMD.

### Syntax Elements

A typical assembly program consists of:

- **Instructions**: Machine operations (mov, add, jmp, etc.)
- **Directives**: Commands for the assembler, not the CPU (section, global, db, etc.)
- **Labels**: Named locations in code or data
- **Comments**: Text following a semicolon (;)

## Program Structure

A basic x86-64 assembly program has the following structure:

```asm
; This is a comment
section .data     ; Data section
    ; Data definitions go here
    
section .text     ; Code section
    global _start ; Entry point declaration
    
_start:           ; Entry point label
    ; Instructions go here
```

## Sections

Assembly programs are organized into sections:

- **`.text`**: Contains executable code
- **`.data`**: Contains initialized data
- **`.bss`**: Contains uninitialized data (zero-initialized)
- **`.rodata`**: Contains read-only data

## Registers

x86-64 provides 16 general-purpose 64-bit registers:

| 64-bit | 32-bit | 16-bit | 8-bit | Description |
|--------|--------|--------|-------|-------------|
| rax    | eax    | ax     | al    | Accumulator, often used for return values |
| rbx    | ebx    | bx     | bl    | Base register, general purpose |
| rcx    | ecx    | cx     | cl    | Counter register, for loop operations |
| rdx    | edx    | dx     | dl    | Data register, often used with rax for operations |
| rsi    | esi    | si     | sil   | Source index, for string operations |
| rdi    | edi    | di     | dil   | Destination index, for string operations |
| rbp    | ebp    | bp     | bpl   | Base pointer, for stack frames |
| rsp    | esp    | sp     | spl   | Stack pointer |
| r8     | r8d    | r8w    | r8b   | General purpose (introduced in x86-64) |
| r9     | r9d    | r9w    | r9b   | General purpose (introduced in x86-64) |
| r10    | r10d   | r10w   | r10b  | General purpose (introduced in x86-64) |
| r11    | r11d   | r11w   | r11b  | General purpose (introduced in x86-64) |
| r12    | r12d   | r12w   | r12b  | General purpose (introduced in x86-64) |
| r13    | r13d   | r13w   | r13b  | General purpose (introduced in x86-64) |
| r14    | r14d   | r14w   | r14b  | General purpose (introduced in x86-64) |
| r15    | r15d   | r15w   | r15b  | General purpose (introduced in x86-64) |

### Special Registers

- **rip**: Instruction pointer
- **rflags**: Flags register

## Data Directives

Data directives define data in the program:

- **`db`**: Define byte (8-bit)
- **`dw`**: Define word (16-bit)
- **`dd`**: Define doubleword (32-bit)
- **`dq`**: Define quadword (64-bit)

Examples:

```asm
byte_var db 42              ; Defines a byte with value 42
word_var dw 12345           ; Defines a word with value 12345
dword_var dd 0x12345678     ; Defines a doubleword with hexadecimal value
qword_var dq 1234567890123  ; Defines a quadword with large value
string_var db 'Hello', 0    ; Defines a null-terminated string
```

## Common Instructions

### Data Movement

- **mov**: Move data between registers, memory, or immediates
- **lea**: Load effective address (calculate address)
- **push**: Push value onto stack
- **pop**: Pop value from stack
- **xchg**: Exchange values

Examples:

```asm
mov rax, 42          ; Load immediate value 42 into rax
mov rbx, rax         ; Copy value from rax to rbx
mov [rbx], rcx       ; Store rcx into memory at address in rbx
lea rdx, [rbx+8*rcx] ; Calculate address and store in rdx
push rax             ; Push rax onto stack
pop rbx              ; Pop top of stack into rbx
```

### Arithmetic

- **add**: Addition
- **sub**: Subtraction
- **mul**: Unsigned multiplication
- **imul**: Signed multiplication
- **div**: Unsigned division
- **idiv**: Signed division
- **inc**: Increment
- **dec**: Decrement
- **neg**: Negate (two's complement)

Examples:

```asm
add rax, 5       ; rax = rax + 5
sub rbx, rcx     ; rbx = rbx - rcx
imul rax, rdx, 4 ; rax = rdx * 4
inc rcx          ; rcx = rcx + 1
dec rdx          ; rdx = rdx - 1
neg rax          ; rax = -rax
```

### Logical Operations

- **and**: Bitwise AND
- **or**: Bitwise OR
- **xor**: Bitwise XOR
- **not**: Bitwise NOT
- **shl/sal**: Shift left
- **shr**: Logical shift right
- **sar**: Arithmetic shift right

Examples:

```asm
and rax, 0xF        ; Clear all but lowest 4 bits
or rbx, 0x100       ; Set bit 8
xor rcx, rcx        ; Clear rcx (set to 0)
not rdx             ; Flip all bits in rdx
shl rax, 2          ; Shift left 2 bits (multiply by 4)
shr rbx, 1          ; Shift right 1 bit (divide by 2)
```

### Control Flow

- **jmp**: Unconditional jump
- **je/jz**: Jump if equal/zero
- **jne/jnz**: Jump if not equal/not zero
- **jg/jnle**: Jump if greater (signed)
- **jge/jnl**: Jump if greater or equal (signed)
- **jl/jnge**: Jump if less (signed)
- **jle/jng**: Jump if less or equal (signed)
- **ja/jnbe**: Jump if above (unsigned)
- **jae/jnb**: Jump if above or equal (unsigned)
- **jb/jnae**: Jump if below (unsigned)
- **jbe/jna**: Jump if below or equal (unsigned)
- **call**: Call function
- **ret**: Return from function

Examples:

```asm
jmp label       ; Jump to label
cmp rax, 0      ; Compare rax with 0
je zero_label   ; Jump to zero_label if rax == 0
call function   ; Call function
ret             ; Return from function
```

### Comparison

- **cmp**: Compare values and set flags
- **test**: Bitwise AND and set flags based on result

Examples:

```asm
cmp rax, rbx    ; Compare rax and rbx, set flags
test rcx, rcx   ; Test if rcx is zero
```

### System Calls (Linux x86-64)

System calls in x86-64 Linux are made using the `syscall` instruction:

1. System call number goes in `rax`
2. Arguments go in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`
3. Return value is placed in `rax`

Common system calls:

- `1`: write (file descriptor, buffer, size)
- `60`: exit (exit code)

Example:

```asm
; Write "Hello" to stdout
mov rax, 1          ; syscall number for write
mov rdi, 1          ; file descriptor (1 = stdout)
mov rsi, msg        ; pointer to message
mov rdx, 5          ; message length
syscall

; Exit with code 0
mov rax, 60         ; syscall number for exit
xor rdi, rdi        ; exit code 0
syscall
```

## Addressing Modes

x86-64 supports several addressing modes for memory operands:

- **Register**: `rax`
- **Immediate**: `42`
- **Direct**: `[address]`
- **Register Indirect**: `[rax]`
- **Base + Displacement**: `[rax+8]`
- **Base + Index × Scale + Displacement**: `[rax+rbx*4+16]`
- **RIP-Relative**: `[rip+offset]`

Examples:

```asm
mov rax, [rbx]            ; Load from address in rbx
mov [rcx], rdx            ; Store rdx at address in rcx
mov rsi, [rdi+8]          ; Load from address (rdi+8)
mov [rbp+rax*4], rbx      ; Store rbx at address (rbp+rax*4)
lea r8, [rip+label]       ; Load address of label relative to rip
```

## Complete Example

Here's a complete example that displays "Hello, World!" using Linux system calls:

```asm
section .data
    msg db 'Hello, World!', 10, 0  ; Message with newline and null terminator
    msg_len equ $ - msg - 1        ; Length of message (excluding null terminator)

section .text
    global _start

_start:
    ; Write message to stdout
    mov rax, 1          ; syscall: write
    mov rdi, 1          ; file descriptor: stdout
    lea rsi, [msg]      ; message address
    mov rdx, msg_len    ; message length
    syscall

    ; Exit program
    mov rax, 60         ; syscall: exit
    xor rdi, rdi        ; exit code: 0
    syscall
```

## Assembly Best Practices

1. **Use meaningful labels**: Choose descriptive names for labels
2. **Comment your code**: Explain complex operations and algorithms
3. **Use consistent formatting**: Align instructions and operands for readability
4. **Initialize registers**: Clear registers before using them
5. **Minimize memory access**: Use registers when possible
6. **Choose appropriate data sizes**: Use the smallest data size that fits your needs
7. **Preserve registers in functions**: Save and restore registers according to calling conventions

## Advanced Topics

- **SIMD Instructions**: Use SSE/AVX instructions for parallel processing
- **Inline Assembly**: Embed assembly in C/C++ code
- **Memory Management**: Understand stack and heap allocation
- **Calling Conventions**: Follow proper register usage for function calls
- **Optimizations**: Learn techniques for optimizing assembly code 