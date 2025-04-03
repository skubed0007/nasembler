# nasembler Source Code Walkthrough

This document provides a detailed explanation of each source file in the nasembler project, going through the codebase file by file to explain the purpose, structure, and implementation details.

## Table of Contents

1. [main.rs](#mainrs)
2. [tokenizer.rs](#tokenizerrs)
3. [parser/](#parser)
   - [mod.rs](#parsermodrs)
   - [ast.rs](#parserastrs)
   - [directive.rs](#parserdirectivers)
   - [instruction.rs](#parserinstructionrs)
4. [encoder/](#encoder)
   - [mod.rs](#encodermodrs)
5. [elf.rs](#elfrs)

## main.rs

`main.rs` serves as the entry point for the nasembler application, handling command-line arguments, file I/O, and orchestrating the overall assembly process.

### Key Components:

1. **Command-Line Argument Parsing**:
   - Uses libraries like `clap` or Rust's standard argument parsing
   - Processes flags for input file (`-f`), output format (`-m`), and output file (`-o`)

2. **File I/O**:
   - Reads assembly source code from input files
   - Writes generated machine code to output files

3. **Assembly Pipeline Coordination**:
   - Creates instances of the tokenizer, parser, encoder, and ELF generator
   - Manages data flow between these components
   - Handles errors and provides user feedback

### Implementation Flow:

```rust
fn main() {
    // 1. Parse command-line arguments
    // 2. Read input assembly file
    // 3. Initialize tokenizer with source code
    // 4. Tokenize the source code into tokens
    // 5. Initialize parser with tokens
    // 6. Parse tokens into an AST
    // 7. Initialize ELF generator
    // 8. Generate ELF file from the AST
    // 9. Write output file
    // 10. Handle errors and provide feedback
}
```

## tokenizer.rs

`tokenizer.rs` contains the lexical analyzer for the assembler, which breaks down the source code into meaningful tokens such as instructions, registers, labels, directives, etc.

### Key Components:

1. **TokenType Enum**:
   - Defines all possible token types (Instruction, Register, Immediate, Label, etc.)
   - Includes specialized categories like Reg64Bit, InstrArith, etc. for more precise handling

2. **Token Struct**:
   - Represents a single token with fields for:
     - Token type
     - Value (the actual text)
     - Line and column information (for error reporting)

3. **Tokenizer Struct**:
   - Maintains the current state of tokenization
   - Tracks position in the source code
   - Contains methods for token recognition and extraction

4. **Lookup Tables**:
   - `INSTRUCTIONS`: Maps instruction names to their types and opcode patterns
   - `REGISTERS`: Maps register names to their types
   - Uses `Lazy<HashMap>` for efficient lookup

5. **Tokenizing Methods**:
   - `tokenize()`: Main entry point that processes the entire source
   - `tokenize_identifier()`: Handles words (instructions, registers, labels)
   - `tokenize_number()`: Processes numeric literals
   - `tokenize_string()`: Handles string literals with escape sequences
   - `tokenize_comment()`: Processes comments

### Implementation Details:

```rust
// Example of the token recognition logic
impl Tokenizer {
    // Main tokenization loop, processes the entire source
    fn tokenize(&mut self) -> &Vec<Token> {
        while !self.is_eof() {
            match self.current_char() {
                // Handle different character types
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    // Handle identifiers (instructions, registers, labels)
                    self.tokens.push(self.tokenize_identifier());
                },
                Some(ch) if ch.is_digit(10) => {
                    // Handle numeric literals
                    self.tokens.push(self.tokenize_number());
                },
                Some('"') => {
                    // Handle string literals
                    self.tokens.push(self.tokenize_string());
                },
                // Handle other token types...
            }
        }
        
        // Add EOF token at the end
        self.tokens.push(Token::new(TokenType::EOF, "".to_string(), self.line, self.column));
        
        &self.tokens
    }
}
```

## parser/

The `parser` directory contains modules for analyzing the token stream and building an Abstract Syntax Tree (AST) representing the assembly program's structure.

### mod.rs

`parser/mod.rs` defines the core Parser struct and implements the multi-pass parsing process.

#### Key Components:

1. **Parser Struct**:
   - Maintains the token stream and current position
   - Stores a symbol table for label resolution
   - Provides methods for navigating and analyzing tokens

2. **Parsing Methods**:
   - `parse()`: Main entry point that orchestrates the multi-pass parsing
   - `collect_labels()`: First pass to build the label table
   - `parse_statement()`: Parses a single statement
   - `encode_instructions()`: Encodes instructions with machine code

3. **Helper Methods**:
   - `is_at_end()`, `peek()`, `advance()`: Token navigation
   - `check()`, `check_value()`: Token validation
   - `previous()`, `current_token()`, `next_token()`: Token access

#### Implementation Flow:

```rust
impl Parser {
    // Main parsing method
    pub fn parse(&mut self) -> Result<ast::Program, String> {
        let mut program = ast::Program::new();
        
        // First pass: collect labels
        self.collect_labels()?;
        
        // Reset for second pass
        self.current = 0;
        
        // Second pass: parse statements
        while !self.is_at_end() {
            // Handle EOF
            if let Some((token, _)) = self.peek() {
                if token.token_type == TokenType::EOF {
                    break;
                }
            }
            
            match self.parse_statement() {
                Ok(statement) => program.add_statement(statement),
                Err(error) => return Err(error),
            }
        }
        
        // Third pass: encode instructions
        self.encode_instructions(&mut program)?;
        
        Ok(program)
    }
    
    // Helper methods...
}
```

### ast.rs

`parser/ast.rs` defines the Abstract Syntax Tree structures that represent the assembly program.

#### Key Components:

1. **Program Struct**:
   - Container for all statements in the assembly program
   - Methods for adding statements and accessing the program structure

2. **Statement Enum**:
   - Represents different statement types:
     - Instruction: An assembly instruction
     - Directive: An assembler directive
     - Label: A label definition
     - Comment: A comment line
     - Empty: An empty line

3. **Instruction Struct**:
   - Represents a single assembly instruction
   - Fields for name, operands, machine code, and line information

4. **Operand Enum**:
   - Represents different operand types:
     - Register: CPU register
     - Immediate: Constant value
     - Memory: Memory reference
     - Label: Reference to a label

5. **MemoryReference Struct**:
   - Represents complex memory addressing modes
   - Fields for base register, index register, scale, and displacement

6. **Directive Struct**:
   - Represents assembler directives
   - Fields for directive name, operands, and line information

#### Implementation Example:

```rust
pub enum Statement {
    Instruction(Instruction),
    Directive(Directive),
    Label(String),
    Comment(String),
    Empty,
}

pub struct Instruction {
    pub name: String,
    pub operands: Vec<Operand>,
    pub machine_code: Vec<u8>,
    pub line: usize,
}

pub enum Operand {
    Register(String),
    Immediate(String),
    Memory(MemoryReference),
    Label(String),
}
```

### directive.rs

`parser/directive.rs` contains the logic for parsing assembler directives like section, global, db, etc.

#### Key Components:

1. **Directive Parsing Functions**:
   - `parse_directive()`: Main entry point for directive parsing
   - Specialized handlers for different directive types:
     - `parse_section_directive()`
     - `parse_global_directive()`
     - `parse_extern_directive()`
     - `parse_data_directive()` (for db, dw, dd, dq)
     - `parse_equ_directive()`

2. **Directive Validation**:
   - Checks for correct syntax and operands for each directive type
   - Handles special cases like $ - label constructs in equ directives

#### Implementation Example:

```rust
pub fn parse_directive(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if token.token_type != TokenType::Directive {
        return Err(format!("Expected directive token, got {:?}", token.token_type));
    }
    
    let directive_name = token.value.clone();
    let line = token.line;
    
    // Advance past the directive token
    parser.next_token();
    
    // Match on the directive name and parse accordingly
    match directive_name.as_str() {
        "section" => parse_section_directive(parser, line),
        "global" => parse_global_directive(parser, line),
        "db" => parse_data_directive(parser, line, "db"),
        // Other directives...
        _ => Err(format!("Unsupported directive: {}", directive_name))
    }
}
```

### instruction.rs

`parser/instruction.rs` handles the parsing of assembly instructions and their operands.

#### Key Components:

1. **Instruction Parsing Functions**:
   - `parse_instruction()`: Main entry point for instruction parsing
   - Extracts instruction name and calls specialized handlers

2. **Operand Parsing**:
   - Functions for parsing different operand types:
     - Registers
     - Immediate values
     - Memory references
     - Label references

3. **Instruction Validation**:
   - Verifies correct number and types of operands
   - Ensures valid combinations for each instruction type

#### Implementation Example:

```rust
pub fn parse_instruction(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if !token.token_type.is_instruction() {
        return Err(format!("Expected instruction token, got {:?}", token.token_type));
    }
    
    let instruction_name = token.value.clone();
    let line = token.line;
    
    // Advance past the instruction token
    parser.next_token();
    
    // Parse operands
    let operands = parse_operands(parser)?;
    
    // Create instruction statement
    Ok(Statement::Instruction(Instruction {
        name: instruction_name,
        operands,
        machine_code: Vec::new(), // Will be filled in later
        line,
    }))
}
```

## encoder/

The `encoder` directory contains modules for translating parsed instructions into machine code.

### mod.rs

`encoder/mod.rs` defines the `MachineCodeEncoder` struct and implements the logic for encoding instructions as binary machine code.

#### Key Components:

1. **MachineCodeEncoder Struct**:
   - Maintains encoding tables and state
   - Provides methods for encoding different instruction types

2. **Encoding Methods**:
   - `encode()`: Main entry point that dispatches to specialized encoders
   - Specialized encoders for different instruction categories:
     - Data movement instructions
     - Arithmetic instructions
     - Logical instructions
     - Control flow instructions
     - SIMD instructions

3. **Operand Encoding**:
   - Methods for encoding different operand types:
     - Register encodings
     - Immediate value encodings
     - Memory operand encodings
     - ModR/M and SIB byte generation

4. **Instruction Format Handling**:
   - Functions for generating REX prefixes
   - Functions for building complete instruction encodings

#### Implementation Example:

```rust
impl MachineCodeEncoder {
    pub fn encode(&self, instruction: &Instruction) -> Vec<u8> {
        let mut encoding = Vec::new();
        
        match instruction.name.as_str() {
            "mov" => self.encode_mov(instruction, &mut encoding),
            "add" => self.encode_add(instruction, &mut encoding),
            "jmp" => self.encode_jmp(instruction, &mut encoding),
            "syscall" => encoding.extend_from_slice(&[0x0F, 0x05]), // Simple case
            // Other instructions...
            _ => {} // Handle unknown instruction
        }
        
        encoding
    }
    
    // Specialized encoders for different instructions...
}
```

## elf.rs

`elf.rs` handles the generation of ELF (Executable and Linkable Format) files from the assembled machine code.

### Key Components:

1. **Constants and Structures**:
   - ELF header constants (magic numbers, file types, etc.)
   - Structure definitions for ELF components:
     - `Elf64Header`: ELF header structure
     - `Elf64ProgramHeader`: Program header structure
     - `Elf64SectionHeader`: Section header structure
     - `Section`: Internal representation of a section
     - `Symbol`: Symbol table entry

2. **ElfGenerator Struct**:
   - Main class responsible for ELF file generation
   - Maintains state such as sections, symbols, and memory mappings

3. **ELF Construction Methods**:
   - `generate_elf()`: Main entry point for ELF generation
   - `create_text_section()`: Creates code section
   - `create_data_section()`: Creates data section
   - `write_elf_file()`: Outputs the complete ELF file

4. **Program Layout Logic**:
   - Functions for determining memory layout
   - Address assignment for sections and symbols
   - Alignment and padding calculations

5. **Binary Writing Methods**:
   - Methods for writing binary data in the correct format
   - Endianness handling
   - Structure packing

### Implementation Example:

```rust
impl ElfGenerator {
    pub fn generate_elf(&mut self, program: &Program, output_file: &str) -> Result<(), String> {
        // Process program and create sections
        self.process_program(program)?;
        
        // Create ELF headers
        let elf_header = self.create_elf_header();
        let program_headers = self.create_program_headers();
        
        // Write ELF file
        self.write_elf_file(output_file, &elf_header, &program_headers)?;
        
        Ok(())
    }
    
    // Helper methods for ELF generation...
}
```

## Implementation Patterns and Techniques

Throughout the codebase, several consistent patterns and techniques are used:

1. **Error Handling**:
   - Extensive use of `Result<T, String>` for error propagation
   - Detailed error messages with line/column information

2. **Memory Efficiency**:
   - Reuse of data structures between passes
   - Use of references instead of cloning where possible

3. **Performance Optimizations**:
   - Fast lookup tables using `HashMap`
   - Specialized handling for common cases
   - Inline annotations for hot functions

4. **Code Organization**:
   - Clear separation of concerns between modules
   - Progressive refinement through multiple passes
   - Consistent API design across components 