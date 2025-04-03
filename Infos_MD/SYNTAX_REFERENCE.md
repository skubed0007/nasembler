# nasembler Syntax Reference

This document provides a comprehensive guide to the syntax supported by nasembler, a modern x86-64 assembler. Understanding this syntax is essential for writing assembly code that can be correctly processed by nasembler.

## Table of Contents

- [Basic Structure](#basic-structure)
- [Sections](#sections)
- [Labels](#labels)
- [Instructions](#instructions)
- [Directives](#directives)
- [Operands](#operands)
- [Comments](#comments)
- [Constants and Literals](#constants-and-literals)
- [Common Errors](#common-errors)

## Basic Structure

A nasembler program consists of a sequence of statements, each on its own line. Each statement can be:

- A section declaration
- A label definition
- An instruction
- A directive
- A comment
- A blank line

```asm
; Basic program structure example
section .text      ; Section declaration
global _start      ; Directive

_start:            ; Label definition
    mov rax, 60    ; Instruction with two operands
    xor rdi, rdi   ; Another instruction
    syscall        ; Instruction with no operands
```

## Sections

Assembly programs are organized into sections. Each section has a specific purpose:

```asm
section .text      ; Contains executable code
section .data      ; Contains initialized data
section .bss       ; Contains uninitialized data
section .rodata    ; Contains read-only data
```

Section names must begin with a dot (`.`) followed by a valid identifier. The section directive must appear at the beginning of a line.

## Labels

Labels are identifiers that represent memory addresses. They can be used to mark locations in code or data:

```asm
label_name:        ; Define a label
_start:            ; Entry point label
data_label:        ; Data label
    db 10          ; Data following the label
```

Label syntax rules:
- Must be followed by a colon (`:`)
- Can contain letters, numbers, underscores, and dots
- Cannot start with a number
- Case-sensitive
- Cannot be the same as reserved keywords or instructions
- Cannot be redefined (duplicate labels are errors)

Labels can be referenced in instructions:

```asm
    jmp label_name         ; Jump to the label
    mov rax, [data_label]  ; Access data at the label
```

## Instructions

Instructions are mnemonics for CPU operations. They follow this general syntax:

```asm
[label:] mnemonic [operand1[, operand2[, operand3]]] [; comment]
```

Examples of different instruction formats:

```asm
    mov rax, 60       ; Two operands: destination, source
    inc rax           ; One operand
    syscall           ; No operands
    imul rax, rbx, 4  ; Three operands: dest, source, immediate
```

## Directives

Directives are commands for the assembler, not for the CPU. They control how the program is assembled:

```asm
    global _start     ; Makes _start visible to the linker
    extern printf     ; Declares an external symbol
    
    db 65             ; Define byte (8-bit)
    dw 1234           ; Define word (16-bit)
    dd 0x12345678     ; Define double word (32-bit)
    dq 0x1234567890ABCDEF ; Define quad word (64-bit)
    
    times 10 db 0     ; Repeat directive, creates 10 bytes of zeros
```

## Operands

Instructions can take various types of operands:

### Registers

```asm
    mov rax, rbx      ; General purpose registers
    push r15          ; Extended registers
```

### Immediate Values

```asm
    mov rax, 42       ; Decimal
    mov rbx, 0xFF     ; Hexadecimal
    mov rcx, 0b1010   ; Binary
    mov rdx, 'A'      ; Character
```

### Memory References

```asm
    mov rax, [rbx]            ; Simple memory reference
    mov [array + 8], rcx      ; Memory with offset
    mov rdx, [rbx + 4*rcx]    ; Memory with scale and index
    mov [rbp - 8], rsi        ; Memory with negative offset
```

The general syntax for memory references is:

```
[base + scale*index + displacement]
```

Where:
- `base` is a register
- `scale` is 1, 2, 4, or 8
- `index` is a register
- `displacement` is an immediate value or label

## Comments

Comments begin with a semicolon (`;`) and continue to the end of the line:

```asm
    mov rax, 60   ; This is a comment
; This entire line is a comment
```

## Constants and Literals

### Numeric Literals

```asm
42        ; Decimal
0x2A      ; Hexadecimal
0b101010  ; Binary
0o52      ; Octal
```

### String Literals

```asm
message db "Hello, World!", 0   ; Null-terminated string
char db 'A'                     ; Single character
```

### Character Escape Sequences

```asm
newline db 0x0A, "Line1", 0x0A, "Line2", 0   ; Explicit newline
str db "Line1\nLine2\0"                      ; Escape sequence
```

## Common Errors

Here are common syntax errors to avoid:

1. **Undefined Label**: Referencing a label that hasn't been defined
   ```asm
   jmp nonexistent_label   ; Error if this label doesn't exist
   ```

2. **Duplicate Label**: Defining the same label more than once
   ```asm
   label:    ; First definition
   mov rax, 1
   label:    ; Error: duplicate label
   ```

3. **Invalid Instruction**: Using an instruction that doesn't exist
   ```asm
   invalidinstr rax, rbx   ; Error: not a valid x86-64 instruction
   ```

4. **Missing Operand**: Not providing all required operands
   ```asm
   mov rax   ; Error: mov requires two operands
   ```

5. **Invalid Register**: Using a register name that doesn't exist
   ```asm
   mov xyz, 42   ; Error: xyz is not a valid register
   ```

6. **Invalid Section Name**: Using an improper section name
   ```asm
   section 123invalid   ; Error: section name must be a valid identifier
   ```

7. **Invalid Memory Reference**: Incorrect memory addressing syntax
   ```asm
   mov rax, [rbx+*4]   ; Error: invalid syntax for memory reference
   ```

8. **Unclosed String**: Not terminating a string literal
   ```asm
   message db "Hello, World   ; Error: missing closing quote
   ```

When encountering errors, nasembler provides detailed error messages with file locations, line and column numbers, and suggestions for fixing the issues. 