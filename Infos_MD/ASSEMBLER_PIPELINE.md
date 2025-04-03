# NASembler Pipeline

This document explains the complete assembly pipeline in NASembler, from source code to executable binary.

## Overview

NASembler follows a traditional compiler pipeline with these main phases:

1. **Source Code Reading**: Reading the assembly source file
2. **Lexical Analysis (Tokenization)**: Converting source text into tokens
3. **Parsing**: Building an Abstract Syntax Tree (AST)
4. **Code Generation**: Encoding instructions as machine code
5. **ELF File Generation**: Creating the final executable

## Pipeline Stages in Detail

### 1. Source Code Reading

The assembly source file is read into memory as a string. This operation is performed in `main.rs`:

```rust
let content = std::fs::read_to_string(&args.file)
    .map_err(|e| format!("Failed to read input file: {}", e))?;
```

### 2. Lexical Analysis (Tokenization)

The tokenizer (`src/tokenizer.rs`) converts the source text into a series of tokens, representing the smallest meaningful units in the assembly language.

#### Token Types

- **Instructions**: `mov`, `add`, `jmp`, etc.
- **Registers**: `rax`, `rbx`, `r15`, etc.
- **Identifiers**: Used for labels and variables
- **Immediate Values**: Numeric constants
- **String Literals**: Text enclosed in quotes
- **Directives**: `.section`, `.global`, etc.
- **Punctuation**: Commas, colons, brackets, etc.
- **Comments**: Text following a semicolon

#### Tokenization Process

1. The tokenizer scans through the input character by character
2. It identifies token boundaries based on syntax rules
3. It classifies each token into a specific token type
4. It stores line/column information for error reporting
5. It returns a vector of tokens

Example code flow:

```rust
let mut tokenizer = Tokenizer::new(&content);
let tokens = tokenizer.tokenize();
```

### 3. Parsing

The parser (`src/parser/mod.rs`) converts the token stream into an Abstract Syntax Tree (AST), which represents the hierarchical structure of the program.

#### Multi-Pass Parsing

NASembler uses a multi-pass approach to parsing:

1. **Label Collection Pass**: Scans for label definitions and builds a symbol table
2. **Statement Parsing Pass**: Parses tokens into structured AST statements
3. **Code Generation Pass**: Traverses the AST to generate machine code

#### AST Structure

The AST (`src/parser/ast.rs`) defines several key data structures:

- **Program**: Container for all statements
- **Statement**: An assembly statement (instruction, directive, label, etc.)
- **Instruction**: Machine instruction with operands
- **Directive**: Assembler directive with operands
- **Operand**: Instruction operand (register, immediate, memory, label)

Sample parsing code:

```rust
let mut parser = Parser::new(tokens);
let program = parser.parse()?;
```

#### Parsing Components

1. **Instruction Parsing** (`src/parser/instruction.rs`):
   - Identifies instruction mnemonic
   - Parses operands with type checking
   - Validates instruction/operand combinations

2. **Directive Parsing** (`src/parser/directive.rs`):
   - Parses data definition directives (db, dw, dd, dq)
   - Handles section declarations
   - Processes symbol directives (global, extern)

3. **Section Management**:
   - Tracks current section (.text, .data, .bss)
   - Ensures instructions/data go in appropriate sections

4. **Label Handling**:
   - Resolves label references
   - Calculates memory offsets

### 4. Code Generation

The encoder (`src/encoder/mod.rs`) translates AST instructions into x86-64 machine code bytes.

#### Encoding Process

1. **Instruction Selection**: Determine appropriate encoding based on operands
2. **Prefix Generation**: Add REX prefixes for 64-bit operations
3. **Opcode Generation**: Output primary opcode bytes
4. **Operand Encoding**: Generate ModR/M, SIB, displacement, and immediate bytes

#### Encoding Challenges

- **Varied Instruction Formats**: x86-64 has complex instruction formats
- **Addressing Modes**: Many combinations of register/memory operands
- **Operand Sizes**: Different operand sizes require different encodings
- **Special Cases**: Many instructions have special encoding rules

Example encoding flow:

```rust
// In the encode_instructions method
for statement in &mut program.statements {
    if let ast::Statement::Instruction(ref mut instruction) = statement {
        instruction.machine_code = encoder.encode(instruction);
    }
}
```

### 5. ELF File Generation

The ELF generator (`src/elf.rs`) creates a valid ELF executable from the assembled code.

#### ELF Structure

An ELF file consists of:

1. **ELF Header**: File type, architecture, entry point
2. **Program Headers**: Define loadable segments
3. **Sections**: .text, .data, etc.
4. **String Tables**: For section and symbol names
5. **Symbol Table**: For exported/imported symbols

#### Generation Process

1. **Calculate Layout**: Determine file offsets and virtual addresses
2. **Generate Headers**: Create ELF and program headers
3. **Arrange Sections**: Prepare text and data sections
4. **Apply Relocations**: Patch label references
5. **Write File**: Output the complete ELF structure

Key steps in code:

```rust
// Create ELF generator
let mut elf_generator = ElfGenerator::new(program);

// Generate ELF file
elf_generator.generate(&output_path)?;
```

#### Memory Layout

NASembler uses a fixed memory layout for simplicity:

- **Text Section**: 0x400000 (4 MB)
- **Data Section**: 0x600000 (6 MB)
- **BSS Section**: 0x800000 (8 MB)

### 6. Special Processing

#### Label Resolution

Labels are resolved through a multi-step process:

1. **Collection**: All labels are collected in the first parsing pass
2. **Address Assignment**: Each label is assigned a virtual address
3. **Reference Resolution**: References to labels are replaced with addresses
4. **Relocation**: LEA instructions using labels are patched with correct offsets

#### String Handling

Strings are processed specially:

1. **Tokenization**: String literals are identified and escaped
2. **AST Representation**: Strings become operands to data directives
3. **Data Generation**: Strings are converted to byte sequences
4. **Null Termination**: A null byte is added to strings by default

## Example Pipeline Flow

Let's trace the assembly of a simple program:

```asm
section .data
    hello db 'Hello, World!', 0

section .text
    global _start

_start:
    ; Write message
    mov rax, 1
    mov rdi, 1
    lea rsi, [hello]
    mov rdx, 13
    syscall

    ; Exit
    mov rax, 60
    xor rdi, rdi
    syscall
```

### 1. Tokenization

The tokenizer breaks this into tokens like:
- `Directive("section")`
- `LabelRef(".data")`
- `Identifier("hello")`
- `Directive("db")`
- `StringLiteral("Hello, World!")`
- `Immediate("0")`
- etc.

### 2. Parsing

The parser creates an AST with:
- Section statements for `.data` and `.text`
- A data directive for the string
- Label definitions for `hello` and `_start`
- Instruction statements for each assembly instruction

### 3. Code Generation

The encoder generates machine code for each instruction:
- `mov rax, 1` → `48 C7 C0 01 00 00 00`
- `lea rsi, [hello]` → `48 8D 35 XX XX XX XX` (placeholder for address)
- etc.

### 4. ELF Generation

The ELF generator:
1. Creates ELF and program headers
2. Arranges text and data sections
3. Calculates the address of the `hello` label
4. Patches the `lea` instruction with the correct offset
5. Writes the complete ELF file

## Debugging Tools

NASembler provides several debugging options:

- **--tokenize-only**: Stop after tokenization
- **--parse-only**: Stop after parsing
- **--dump-tokens**: Display all tokens
- **--dump-ast**: Display the complete AST
- **--verbose**: Show detailed processing information

These help diagnose issues in different pipeline stages.

## Performance Considerations

NASembler optimizes performance in several ways:

1. **Single-Pass Tokenization**: Minimizes string handling
2. **Early Label Resolution**: Avoids multiple passes over code
3. **Direct Machine Code Generation**: No intermediate representation
4. **Efficient Memory Management**: Minimizes copies and allocations

## Error Handling

Errors can occur at different pipeline stages:

1. **Tokenization Errors**: Invalid characters, unclosed strings
2. **Parsing Errors**: Invalid syntax, unknown instructions
3. **Semantic Errors**: Invalid operand combinations, undefined labels
4. **Encoding Errors**: Unsupported instructions or addressing modes
5. **ELF Generation Errors**: File writing issues

Each stage provides detailed error information to help diagnose and fix problems. 