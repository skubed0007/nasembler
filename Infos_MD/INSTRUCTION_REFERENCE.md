# x86-64 Instruction Reference

This document provides a reference for the x86-64 instructions supported by nasembler.

## Instruction Encoding Overview

x86-64 instructions can vary in length from 1 to 15 bytes, composed of:

1. Optional prefixes (0-4 bytes)
2. REX prefix for 64-bit operations (0-1 byte)
3. Opcode (1-3 bytes)
4. ModR/M byte (0-1 byte)
5. SIB (Scale-Index-Base) byte (0-1 byte)
6. Displacement (0, 1, 2, or 4 bytes)
7. Immediate data (0, 1, 2, 4, or 8 bytes)

## REX Prefix

The REX prefix extends registers and instruction capabilities in 64-bit mode:

```
0100WRXB
```

- **W**: 1 = 64-bit operand size, 0 = default operand size
- **R**: Extension to ModR/M.reg field
- **X**: Extension to SIB.index field
- **B**: Extension to ModR/M.rm or SIB.base field

## ModR/M Byte

The ModR/M byte specifies addressing modes and registers:

```
MMRRRBBB
```

- **MM**: Addressing mode (00, 01, 10, 11)
- **RRR**: Register operand or opcode extension
- **BBB**: Register/memory operand

## Common Instructions

### Data Movement Instructions

| Instruction | Description | Typical Encoding |
|-------------|-------------|-----------------|
| `mov r64, imm64` | Move immediate to register | `48 B8+r imm64` |
| `mov r64, r64` | Move register to register | `48 89 C0+r` |
| `mov r64, [r64]` | Move memory to register | `48 8B 00+r` |
| `mov [r64], r64` | Move register to memory | `48 89 00+r` |
| `lea r64, [mem]` | Load effective address | `48 8D /r` |
| `push r64` | Push register onto stack | `50+r` |
| `pop r64` | Pop register from stack | `58+r` |
| `xchg r64, r64` | Exchange registers | `48 87 /r` |

### Arithmetic Instructions

| Instruction | Description | Typical Encoding |
|-------------|-------------|-----------------|
| `add r64, r64` | Add register to register | `48 01 /r` |
| `add r64, imm32` | Add immediate to register | `48 81 /0 imm32` |
| `sub r64, r64` | Subtract register from register | `48 29 /r` |
| `sub r64, imm32` | Subtract immediate from register | `48 81 /5 imm32` |
| `inc r64` | Increment register | `48 FF /0` |
| `dec r64` | Decrement register | `48 FF /1` |
| `mul r64` | Unsigned multiply (RDX:RAX = RAX * r64) | `48 F7 /4` |
| `imul r64` | Signed multiply | `48 F7 /5` |
| `div r64` | Unsigned divide RDX:RAX by r64 | `48 F7 /6` |
| `idiv r64` | Signed divide RDX:RAX by r64 | `48 F7 /7` |
| `neg r64` | Two's complement negation | `48 F7 /3` |

### Logical Instructions

| Instruction | Description | Typical Encoding |
|-------------|-------------|-----------------|
| `and r64, r64` | Logical AND | `48 21 /r` |
| `and r64, imm32` | Logical AND with immediate | `48 81 /4 imm32` |
| `or r64, r64` | Logical OR | `48 09 /r` |
| `or r64, imm32` | Logical OR with immediate | `48 81 /1 imm32` |
| `xor r64, r64` | Logical XOR | `48 31 /r` |
| `xor r64, imm32` | Logical XOR with immediate | `48 81 /6 imm32` |
| `not r64` | Bitwise NOT | `48 F7 /2` |
| `shl r64, imm8` | Shift left | `48 C1 /4 imm8` |
| `shr r64, imm8` | Logical shift right | `48 C1 /5 imm8` |
| `sar r64, imm8` | Arithmetic shift right | `48 C1 /7 imm8` |

### Control Flow Instructions

| Instruction | Description | Typical Encoding |
|-------------|-------------|-----------------|
| `jmp rel32` | Unconditional jump (relative) | `E9 rel32` |
| `jmp r64` | Unconditional jump (register) | `FF /4` |
| `je/jz rel32` | Jump if equal/zero | `0F 84 rel32` |
| `jne/jnz rel32` | Jump if not equal/not zero | `0F 85 rel32` |
| `jg/jnle rel32` | Jump if greater | `0F 8F rel32` |
| `jge/jnl rel32` | Jump if greater or equal | `0F 8D rel32` |
| `jl/jnge rel32` | Jump if less | `0F 8C rel32` |
| `jle/jng rel32` | Jump if less or equal | `0F 8E rel32` |
| `call rel32` | Call procedure (relative) | `E8 rel32` |
| `call r64` | Call procedure (register) | `FF /2` |
| `ret` | Return from procedure | `C3` |
| `syscall` | System call | `0F 05` |

### Comparison Instructions

| Instruction | Description | Typical Encoding |
|-------------|-------------|-----------------|
| `cmp r64, r64` | Compare registers | `48 39 /r` |
| `cmp r64, imm32` | Compare register with immediate | `48 81 /7 imm32` |
| `test r64, r64` | Logical compare (AND) | `48 85 /r` |
| `test r64, imm32` | Logical compare with immediate | `48 F7 /0 imm32` |

## Register Encodings

In the encoding tables, `/r` refers to the ModR/M byte, and `+r` refers to adding the register code to the opcode.

### Register Codes (Lower 3 Bits)

| Register | Binary | Hex |
|----------|--------|-----|
| RAX/EAX/AX/AL | 000 | 0 |
| RCX/ECX/CX/CL | 001 | 1 |
| RDX/EDX/DX/DL | 010 | 2 |
| RBX/EBX/BX/BL | 011 | 3 |
| RSP/ESP/SP/SPL | 100 | 4 |
| RBP/EBP/BP/BPL | 101 | 5 |
| RSI/ESI/SI/SIL | 110 | 6 |
| RDI/EDI/DI/DIL | 111 | 7 |

### Extended Registers (R8-R15)

These require the REX prefix with the appropriate R or B bit set.

| Register | REX.R/REX.B Bit | Reg/RM Bits |
|----------|----------------|-------------|
| R8 | 1 | 000 |
| R9 | 1 | 001 |
| R10 | 1 | 010 |
| R11 | 1 | 011 |
| R12 | 1 | 100 |
| R13 | 1 | 101 |
| R14 | 1 | 110 |
| R15 | 1 | 111 |

## Addressing Modes

The addressing mode is determined by the ModR/M and SIB bytes.

### ModR/M Modes

| Mode (MM) | Description |
|-----------|-------------|
| 00 | [reg], no displacement (except when R/M=101, then RIP+disp32) |
| 01 | [reg]+disp8, 8-bit displacement |
| 10 | [reg]+disp32, 32-bit displacement |
| 11 | Direct register addressing |

### SIB Byte (Scale-Index-Base)

Used when ModR/M.R/M = 100 (binary).

```
SSIIIBBB
```

- **SS**: Scale factor (00=1, 01=2, 10=4, 11=8)
- **III**: Index register
- **BBB**: Base register

## Instruction Examples with Encodings

### Example 1: Basic Register Movement

```asm
mov rax, 42         ; 48 C7 C0 2A 00 00 00
mov rbx, rax        ; 48 89 C3
```

### Example 2: Memory Access

```asm
mov rax, [rbx]      ; 48 8B 03
mov [rax], rbx      ; 48 89 18
mov rcx, [rbx+16]   ; 48 8B 4B 10
```

### Example 3: LEA Instruction

```asm
lea rsi, [rdi+8]    ; 48 8D 77 08
lea rax, [rbx+rcx*4] ; 48 8D 04 8B
```

### Example 4: Arithmetic

```asm
add rax, rbx        ; 48 01 D8
sub rax, 10         ; 48 83 E8 0A
inc rcx             ; 48 FF C1
dec rdx             ; 48 FF CA
```

### Example 5: Control Flow

```asm
jmp label           ; E9 rel32
call function       ; E8 rel32
ret                 ; C3
syscall             ; 0F 05
```

## System Calls (Linux x86-64)

System calls in Linux x86-64 use the `syscall` instruction (opcode `0F 05`).

1. System call number goes in `rax`
2. Arguments go in `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9`
3. Return value is placed in `rax`

Common system calls:

| Number | Name | Arguments |
|--------|------|-----------|
| 0 | read | fd, buf, count |
| 1 | write | fd, buf, count |
| 2 | open | filename, flags, mode |
| 3 | close | fd |
| 60 | exit | error_code |
| 62 | kill | pid, sig |

## Encoding Tips

1. For 64-bit operations, use the REX.W prefix (0x48)
2. For operations involving extended registers (R8-R15), use the appropriate REX.R or REX.B bit
3. Displacements and immediates are stored in little-endian format
4. RIP-relative addressing uses ModR/M mode 00, R/M=101, and a 32-bit displacement

## Instruction Encoding Process

1. Determine if REX prefix is needed
2. Select the appropriate opcode
3. Construct ModR/M byte if needed
4. Add SIB byte if complex addressing mode
5. Add displacement if memory operand
6. Add immediate value if immediate operand
