# nasembler - Architecture Overview

This document provides a detailed explanation of the architecture and internal components of nasembler, a native x86-64 assembler written in Rust.

## High-Level Architecture

nasembler follows a traditional compiler architecture with the following pipeline:

1. **Lexical Analysis (Tokenization)**: Converts raw source code into tokens
2. **Parsing**: Transforms tokens into an Abstract Syntax Tree (AST)
3. **Semantic Analysis**: Validates the correctness of the AST
4. **Code Generation**: Transforms the AST into machine code
5. **Output Generation**: Creates an executable ELF file

## Component Breakdown

### Tokenizer (`src/tokenizer.rs`)

The tokenizer is responsible for converting a string of assembly code into a stream of tokens. Each token represents a meaningful element in the code, such as:

- Instructions
- Registers
- Directives
- Labels
- Numeric literals
- String literals
- Comments
- Punctuation (commas, brackets, etc.)

#### Implementation Details

- Uses a state machine design to process input character by character
- Handles special token types like string literals, numeric literals in multiple bases
- Maintains line and column information for error reporting
- Implements lookahead for context-sensitive token recognition

### Parser (`src/parser/`)

The parser converts the token stream into an Abstract Syntax Tree (AST), which represents the hierarchical structure of the assembly program.

#### Key Components

- `ast.rs`: Defines the AST data structures
- `mod.rs`: Main parser implementation
- `directive.rs`: Handles parsing of directives
- `instruction.rs`: Handles parsing of instructions
- `section.rs`: Handles parsing of section declarations
- `label.rs`: Handles parsing of label definitions

#### Parser Implementation

- Uses a recursive descent parsing approach
- Handles label resolution and symbol table management
- Tracks section information (.text, .data, .bss)
- Validates instruction syntax and operand types

### Encoder (`src/encoder/`)

The encoder transforms AST nodes representing instructions into machine code bytes.

#### Features

- Handles different addressing modes (register, immediate, memory)
- Selects appropriate instruction encodings based on operand types
- Implements REX prefixes for 64-bit operations
- Handles ModR/M and SIB byte construction
- Manages instruction displacement and immediate values

### ELF Generator (`src/elf.rs`)

The ELF generator creates a valid ELF executable file from the assembled code.

#### Implementation Details

- Creates ELF headers with proper section information
- Handles program headers for loadable segments
- Manages section layout and memory mapping
- Implements relocations for labels and symbols
- Sets appropriate permissions on sections (executable, writable)

### Error Handling (`src/error.rs`)

The error handling system provides detailed and helpful error messages for assembly errors.

#### Features

- Tracks source locations (file, line, column) for each error
- Supports error categories (syntax, semantic, internal)
- Provides contextual information about the error
- Handles error recovery for better user experience

## Data Flow

1. The input assembly file is read into memory
2. The tokenizer processes the input and generates a stream of tokens
3. The parser processes the tokens and builds an AST
4. The semantic analyzer validates the AST
5. The encoder transforms AST instructions into machine code
6. The ELF generator creates an executable file with the machine code

## Memory Management

- The assembler uses Rust's ownership system for memory safety
- Most data structures use owned values rather than references to avoid lifetime issues
- The AST uses `Clone` to allow multiple passes over the program

## Performance Considerations

- The tokenizer is optimized for single-pass processing
- The parser maintains a symbol table for efficient label resolution
- Machine code is generated directly in the correct format to avoid unnecessary conversions
- ELF file generation uses direct memory layout to minimize copying

## Future Improvements

- Support for more complex addressing modes
- Enhanced macro capabilities
- Improved error recovery and suggestions
- Optimization passes
- Support for other object file formats (COFF, Mach-O) 