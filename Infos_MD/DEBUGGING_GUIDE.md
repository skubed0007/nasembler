# NASembler Debugging Guide

This guide provides strategies and tools for debugging assembly code using NASembler.

## Common Issues and Solutions

### Syntax Errors

#### Incorrect Token Syntax

**Symptoms:**
- Error messages about unexpected tokens
- Tokenization fails

**Common Causes:**
- Missing commas between operands
- Incorrect register names
- Missing brackets for memory references
- Unterminated string literals

**Solutions:**
- Use the `--dump-tokens` flag to see how your code is being tokenized
- Check for proper separators between operands
- Verify register names match x86-64 conventions (rax, rbx, etc.)
- Ensure brackets are properly matched for memory references

#### Example:

Incorrect:
```asm
mov rax rbx      ; Missing comma
mov [rax+8 rdx   ; Missing closing bracket
```

Correct:
```asm
mov rax, rbx     ; Added comma
mov [rax+8], rdx  ; Fixed bracket
```

### Parsing Errors

**Symptoms:**
- Error messages about invalid syntax
- Parser fails but tokenization succeeds

**Common Causes:**
- Incorrect instruction formats
- Using unsupported addressing modes
- Incorrect directive usage
- Invalid operand combinations

**Solutions:**
- Use the `--parse-only` flag to validate your assembly syntax
- Check instruction documentation for proper operand order
- Verify that directives are used correctly
- Use `--dump-ast` to see how your code is being interpreted

#### Example:

Incorrect:
```asm
mov 42, rax      ; Immediate can't be destination
add [rax], 10, 5  ; Too many operands
```

Correct:
```asm
mov rax, 42      ; Register as destination
add [rax], 10     ; Proper operand count
```

### Label Resolution Issues

**Symptoms:**
- "Undefined label" errors
- Incorrect jumps or memory references

**Common Causes:**
- Referencing a label that's defined later without a forward declaration
- Typos in label names
- Labels defined in different sections than expected

**Solutions:**
- Check for typos in label names
- Ensure labels are properly defined before use (or use forward declarations)
- Verify that code and data labels are in the correct sections

#### Example:

Incorrect:
```asm
jmp mylabel     ; Label doesn't exist yet or has a typo

my_label:       ; Different name than referenced
    nop
```

Correct:
```asm
jmp my_label    ; Correct label name

my_label:       ; Matches the label used in the jump
    nop
```

### Section Errors

**Symptoms:**
- Errors about instructions in data sections
- Data in text sections
- Missing or incorrect section directives

**Common Causes:**
- Missing section directives
- Putting code in data sections or vice versa
- Incorrect section names

**Solutions:**
- Ensure proper section directives are used
- Keep code in the `.text` section
- Keep data in the `.data` or `.bss` sections
- Double-check section names

#### Example:

Incorrect:
```asm
; No section directive
start:
    mov rax, 42
    
data1 db "Hello"  ; Data not in data section
```

Correct:
```asm
section .text
start:
    mov rax, 42
    
section .data
data1 db "Hello"
```

### Memory Addressing Errors

**Symptoms:**
- Errors about invalid addressing modes
- Incorrect memory references

**Common Causes:**
- Invalid base/index combinations
- Incorrect syntax for memory references
- Missing size specifiers

**Solutions:**
- Review x86-64 addressing mode syntax
- Ensure memory references use valid register combinations
- Check for proper bracket syntax

#### Example:

Incorrect:
```asm
mov rax, [+rbx]      ; Invalid syntax
mov rcx, [rax+rbx+rcx]  ; Too many registers without scale
```

Correct:
```asm
mov rax, [rbx]          ; Correct syntax
mov rcx, [rax+rbx*1+rcx]  ; Added scale factor
```

### ELF Generation Issues

**Symptoms:**
- Assembly completes but ELF file is invalid
- Executable crashes or behaves unexpectedly

**Common Causes:**
- Missing entry point (`_start` label)
- Incorrect section alignment
- Improper syscall usage
- Missing null terminators for strings

**Solutions:**
- Ensure you have a `_start` label and it's marked as `global`
- Check syscall numbers and arguments
- Verify strings have proper termination if needed
- Test with simple code first

#### Example:

Incorrect:
```asm
; Missing global directive
_start:
    mov rax, 60
    mov rdi, 0
    syscall
```

Correct:
```asm
global _start
_start:
    mov rax, 60
    mov rdi, 0
    syscall
```

## Debugging Tools

NASembler provides several debugging tools:

### Command Line Options

- `--tokenize-only`: Stop after tokenization, show token information
- `--parse-only`: Parse but don't generate code
- `--dump-tokens`: Display all tokens after tokenization
- `--dump-ast`: Display the Abstract Syntax Tree (AST)
- `--verbose`: Show detailed processing information

### Using Debugging Options

#### Tokenization Debugging

To see how your code is tokenized:

```bash
nasembler --file test.asm --dump-tokens
```

This will show all tokens with their types and values, helping identify syntax issues.

#### Parsing Debugging

To check if your code is parsed correctly:

```bash
nasembler --file test.asm --dump-ast
```

This shows the complete AST, useful for verifying that instructions and operands are interpreted correctly.

#### Full Verbosity

For maximum debugging information:

```bash
nasembler --file test.asm --elf --verbose
```

This shows detailed information about the entire assembly process.

## External Debugging Tools

### Using objdump

To examine the generated ELF file:

```bash
objdump -d test.bin
```

This disassembles the executable, letting you verify the machine code.

### Using strace

To trace system calls when running your program:

```bash
strace ./test.bin
```

This shows all system calls, helping debug runtime issues.

### Using GDB

For step-by-step debugging:

```bash
gdb ./test.bin
```

Common GDB commands:
- `break _start`: Set a breakpoint at the entry point
- `disas`: Disassemble current function
- `info registers`: Show register values
- `x/10i $rip`: Show next 10 instructions
- `stepi`: Execute next instruction

## Debugging Examples

### Example 1: Finding a Syntax Error

Suppose you have a file `buggy.asm` with the following content:

```asm
section .text
global _start

_start:
    mov rax 1         ; Missing comma
    mov rdi, 1
    lea rsi [message] ; Missing comma
    mov rdx, 13
    syscall

section .data
    message db 'Hello, World!', 0
```

Debugging process:

1. Run with token dump:
```bash
nasembler --file buggy.asm --dump-tokens
```

2. Notice that `mov rax 1` is tokenized as separate tokens without recognizing the syntax error
3. Fix the missing commas
4. Rerun to verify

### Example 2: Debugging a Label Resolution Issue

Suppose your program jumps to an undefined label:

```asm
section .text
global _start

_start:
    jmp display_message
    mov rax, 60
    xor rdi, rdi
    syscall

display:              ; Typo: should be display_message
    mov rax, 1
    mov rdi, 1
    lea rsi, [message]
    mov rdx, 13
    syscall
    jmp _start+20

section .data
    message db 'Hello, World!', 0
```

Debugging process:

1. Run with AST dump:
```bash
nasembler --file buggy.asm --dump-ast
```

2. Notice the error about an undefined label `display_message`
3. Check the file and find that the label is defined as `display`
4. Fix the label name to match
5. Rerun to verify

## Advanced Debugging Techniques

### Tracing Memory Values

To track memory values, add debug prints:

```asm
section .text
global _start

_start:
    ; Debug print of a value
    mov rax, 1           ; write syscall
    mov rdi, 1           ; stdout
    lea rsi, [debug_msg]
    mov rdx, debug_len
    syscall
    
    ; Display a register value (e.g., rax = 42)
    mov rax, 42
    call print_rax
    
    ; Continue with program
    
print_rax:
    ; Convert RAX to ASCII and print
    ; (implementation details omitted for brevity)
    ret

section .data
    debug_msg db 'Debug point reached', 10
    debug_len equ $ - debug_msg
```

### Incremental Development

Build your program in small, testable steps:

1. Start with a minimal working program
2. Add one feature at a time
3. Test after each addition
4. Use debug prints to verify values
5. Backup working stages

## Common Error Messages and Their Meanings

| Error Message | Likely Cause |
|---------------|--------------|
| "Expected comma" | Missing separator between operands |
| "Invalid operand combination" | Incompatible operands for instruction |
| "Undefined label" | Reference to non-existent label |
| "Invalid instruction" | Unrecognized mnemonic |
| "Expected operand" | Missing required operand |
| "Section not found" | Reference to undefined section |
| "Invalid addressing mode" | Incorrect memory reference syntax |

## Tips for Effective Debugging

1. **Start Simple**: Begin with minimal working code and build up
2. **Divide and Conquer**: Isolate issues by commenting out sections
3. **Check Syntax First**: Ensure syntax is correct before debugging logic
4. **Use Verbose Output**: Enable all debugging flags initially
5. **Compare with Known Working Code**: Use working examples as references
6. **Validate Syscalls**: Double-check syscall numbers and arguments
7. **Test on Small Inputs**: Use small, predictable test cases

## Help and Resources

If you're stuck, consider:

1. Checking the NASembler documentation
2. Reviewing x86-64 instruction references
3. Looking at example code in the repository
4. Consulting online forums for assembly programming
5. Using external debugging tools (GDB, objdump) 