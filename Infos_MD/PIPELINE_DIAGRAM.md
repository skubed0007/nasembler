# Nasembler Assembly Pipeline Diagrams

This document provides visual diagrams of the complete assembly pipeline in Nasembler, showing how data flows from source code to executable binary.

## Complete Pipeline Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │    │                 │    │                 │
│  Source Code    │───▶│    Tokenizer    │───▶│     Parser      │───▶│    Encoder      │───▶│  ELF Generator  │
│   (ASM File)    │    │  (tokenizer.rs) │    │   (parser/*.rs) │    │ (encoder/mod.rs)│    │    (elf.rs)     │
│                 │    │                 │    │                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                              │                       │                       │                      │
                              ▼                       ▼                       ▼                      ▼
                        ┌─────────────┐        ┌─────────────┐         ┌─────────────┐        ┌─────────────┐
                        │  Tokens     │        │  AST        │         │ Machine Code│        │  ELF File   │
                        │ (Vec<Token>)│        │ (Program)   │         │ (Vec<u8>)   │        │ (Binary)    │
                        └─────────────┘        └─────────────┘         └─────────────┘        └─────────────┘
```

## Detailed Tokenization Process

```
┌──────────────────────┐
│   Assembly Source    │
│                      │
│ section .text        │
│ global _start        │
│ _start:              │
│   mov rax, 60        │
│   mov rdi, 0         │
│   syscall            │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐     ┌──────────────────────┐
│  Character Scanner   │────▶│  Token Classification │
└──────────┬───────────┘     └──────────┬───────────┘
           │                            │
           ▼                            ▼
┌──────────────────────────────────────────────────────┐
│                      Tokens                          │
│                                                      │
│ [Directive("section"), LabelRef(".text"), NewLine,   │
│  Directive("global"), LabelRef("_start"), NewLine,   │
│  Label("_start"), Colon, NewLine,                    │
│  Instruction("mov"), Register("rax"), Comma,         │
│  Immediate("60"), NewLine,                           │
│  Instruction("mov"), Register("rdi"), Comma,         │
│  Immediate("0"), NewLine,                            │
│  Instruction("syscall"), NewLine,                    │
│  EOF]                                                │
└──────────────────────────────────────────────────────┘
```

## Parser Multi-Pass System

```
┌───────────────┐
│  Token Stream │
└───────┬───────┘
        │
        ▼
┌───────────────────────┐
│ Pass 1: Label Collection │
│                      │
│ Scans tokens to build│
│ label symbol table   │
└──────────┬───────────┘
           │
           ▼
┌───────────────────────┐
│ Symbol Table          │
│                       │
│ "_start" -> 0         │
└──────────┬────────────┘
           │
           ▼
┌───────────────────────┐
│ Pass 2: Statement Parsing │
│                       │
│ Parses token stream   │
│ into AST nodes        │
└──────────┬────────────┘
           │
           ▼
┌───────────────────────┐
│ Abstract Syntax Tree  │
│                       │
│ Program:              │
│  - Directive("section")│
│  - Directive("global")│
│  - Label("_start")    │
│  - Instruction("mov") │
│  - Instruction("mov") │
│  - Instruction("syscall")│
└──────────┬────────────┘
           │
           ▼
┌───────────────────────┐
│ Pass 3: Encoding      │
│                       │
│ Generates machine code│
│ for each instruction  │
└──────────┬────────────┘
           │
           ▼
┌───────────────────────┐
│ Encoded Program       │
│                       │
│ Machine code attached │
│ to each instruction   │
└───────────────────────┘
```

## Instruction Encoding Process

```
┌───────────────────────┐
│ Instruction:          │
│ mov rax, 60           │
└──────────┬────────────┘
           │
           ▼
┌──────────────────────────────────────────┐
│ Instruction Analysis                     │
│                                          │
│ - Identify instruction type (mov)        │
│ - Classify operands:                     │
│   - Destination: Register("rax")         │
│   - Source: Immediate("60")              │
└──────────────────────┬───────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────┐
│ Encoding Selection                       │
│                                          │
│ - Look up encoding for mov reg, imm      │
│ - For mov rax, imm64 -> Opcode: 48 B8    │
└──────────────────────┬───────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────┐
│ Operand Encoding                         │
│                                          │
│ - Encode immediate value 60 (0x3C)       │
│ - For 64-bit immediate: 3C 00 00 00 00 00 00 00 │
└──────────────────────┬───────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────┐
│ Final Machine Code                       │
│                                          │
│ 48 B8 3C 00 00 00 00 00 00 00           │
│ REX.W + Opcode + Immediate Value         │
└──────────────────────────────────────────┘
```

## ELF File Generation

```
┌───────────────────────┐
│ Encoded AST Program   │
└──────────┬────────────┘
           │
           ▼
┌───────────────────────────────────────────────────┐
│ Section Organization                              │
│                                                   │
│ - .text section with executable code              │
│   (at virtual address 0x400000)                   │
│                                                   │
│ - .data section with data                         │
│   (at virtual address 0x600000)                   │
└──────────────────────────┬────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────┐
│ ELF Header Generation                             │
│                                                   │
│ - Magic number: 7F 45 4C 46 02 01 01 00 ...       │
│ - Type: ET_EXEC (executable)                      │
│ - Machine: EM_X86_64                              │
│ - Entry point: 0x400000                           │
└──────────────────────────┬────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────┐
│ Program Header Generation                         │
│                                                   │
│ - Text segment: RX (read+execute)                 │
│   Address: 0x400000                               │
│                                                   │
│ - Data segment: RW (read+write)                   │
│   Address: 0x600000                               │
└──────────────────────────┬────────────────────────┘
                           │
                           ▼
┌───────────────────────────────────────────────────┐
│ ELF Binary File                                   │
│                                                   │
│ Complete ELF file with:                           │
│ - ELF header                                      │
│ - Program headers                                 │
│ - .text section (code)                            │
│ - .data section (data)                            │
└───────────────────────────────────────────────────┘
```

## Memory Model in Generated Executables

```
┌───────────────────────────────────────────────────┐ 0xFFFFFFFFFFFFFFFF
│                  Kernel Space                     │
├───────────────────────────────────────────────────┤
│                      ...                          │
├───────────────────────────────────────────────────┤
│                     Stack                         │
├───────────────────────────────────────────────────┤
│                      ...                          │
├───────────────────────────────────────────────────┤
│                      ...                          │
├───────────────────────────────────────────────────┤ 0x0000000000600000
│                 .data Section                     │
│                                                   │
│           Data defined with db, dw, etc           │
├───────────────────────────────────────────────────┤
│                      ...                          │
├───────────────────────────────────────────────────┤ 0x0000000000400000
│                 .text Section                     │
│                                                   │
│         Machine code for instructions             │
│          Entry point (_start label)               │
├───────────────────────────────────────────────────┤
│                      ...                          │
└───────────────────────────────────────────────────┘ 0x0000000000000000
```

## Parser State Tracking

```
┌───────────────────────────────────────────────────┐
│                  Parser State                     │
├───────────────────────────────────────────────────┤
│ Current Position                                  │
│ ▼                                                 │
│ [Token₁, Token₂, Token₃, Token₄, ..., TokenN]     │
├───────────────────────────────────────────────────┤
│                                                   │
│ Labels Table:                                     │
│ {                                                 │
│   "label1" -> position1,                          │
│   "label2" -> position2,                          │
│   "_start" -> position3                           │
│ }                                                 │
└───────────────────────────────────────────────────┘
```

## Assembler Toolchain Comparison

```
┌───────────────────────────────────────────────────┐
│            Traditional Assembler Pipeline         │
├───────────────────────────────────────────────────┤
│  Assembly  │ Assembler │  Object  │  Linker  │ Executable │
│   (.asm)   │    (as)   │   (.o)   │   (ld)   │   (ELF)    │
└───────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────┐
│               Nasembler Pipeline                  │
├───────────────────────────────────────────────────┤
│  Assembly  │    Nasembler    │    Executable      │
│   (.asm)   │                 │      (ELF)         │
└───────────────────────────────────────────────────┘
``` 