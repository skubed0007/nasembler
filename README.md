# nasembler - Modern x86-64 Assembler

nasembler is a modern assembler for x86-64 assembly language, designed with a focus on clear error messages, extensive documentation, and developer-friendly features.

## Features

- **Clean and Modern CLI**: Intuitive command-line interface with sensible defaults
- **Enhanced Error Messages**: Detailed and color-coded error messages with clear pointers to the exact issue
- **ELF Output**: Generates executable Linux ELF binaries
- **Debugging Tools**: Integrated tools for inspecting the assembly process
- **Support for x86-64 Instructions**: Comprehensive support for the x86-64 instruction set

## Installation

```bash
# Clone the repository
git clone https://github.com/skubed0007/nasembler.git
cd nasembler

# Build with Cargo
cargo build --release

# Optionally, install it system-wide
cargo install --path .
```

## Quick Start

```bash
# Assemble a file
nasembler hello.asm

# Assemble and run immediately
nasembler hello.asm -x

# Show verbose output
nasembler hello.asm -v
```

## Cross-Platform Building

nasembler is designed to be built for multiple platforms. The repository includes a build script that can generate binaries for various architectures and operating systems.

### Optimized Release Builds

The `Cargo.toml` file includes optimized release profile settings:

```toml
[profile.release]
opt-level = 3                # Maximum optimization
lto = "fat"                  # Link-time optimization
codegen-units = 1            # Better optimization
panic = "abort"              # Removes panic unwinding
strip = true                 # Strip symbols
```

These settings produce smaller, faster binaries at the cost of longer build times.

### Building for Multiple Platforms

The included `build.sh` script allows you to build nasembler for multiple target platforms:

```bash
# Make the script executable
chmod +x build.sh

# Run the build script
./build.sh
```

The script will:

1. Display a list of supported target platforms
2. Ask you to select platforms by number (space-separated)
3. Install any necessary Rust target toolchains
4. Build optimized binaries for each selected platform
5. Place the compiled binaries in the `cross-platform-builds` directory

Supported platforms include:
- Linux (x86_64, ARM, etc.)
- Windows (MSVC and MinGW)
- macOS (Intel and Apple Silicon)
- FreeBSD, NetBSD, OpenBSD
- And many more specialized targets

### Requirements for Cross-Compilation

Some platforms may require additional tools:
- Windows targets require either the MSVC toolchain or MinGW
- macOS targets require an OSX SDK if building from Linux
- Some targets may require specific linkers or system libraries

The script will attempt to install required Rust targets automatically.

## Documentation

Comprehensive documentation is available in the `Infos_MD` directory:

- [Syntax Reference](Infos_MD/SYNTAX_REFERENCE.md) - Detailed guide to nasembler syntax
- [Command-Line Reference](Infos_MD/CLI_REFERENCE.md) - All available command-line options
- [Example Programs](Infos_MD/EXAMPLES.md) - Example assembly programs and patterns
- [Architecture Overview](Infos_MD/ARCHITECTURE.md) - How nasembler works internally
- [Assembly Pipeline](Infos_MD/ASSEMBLER_PIPELINE.md) - Details of the assembly process
- [Debugging Guide](Infos_MD/DEBUGGING_GUIDE.md) - How to debug assembly programs
- [Instruction Reference](Infos_MD/INSTRUCTION_REFERENCE.md) - Guide to supported instructions
- [x86-64 Assembly Guide](Infos_MD/X86_64_ASSEMBLY_GUIDE.md) - General guide to x86-64 assembly

## Example

Here's a simple "Hello, World!" example:

```asm
section .data
    msg db "Hello, World!", 10, 0  ; String with newline and null terminator

section .text
    global _start

_start:
    ; Write the message to stdout
    mov rax, 1          ; syscall number for sys_write
    mov rdi, 1          ; file descriptor 1 (stdout)
    mov rsi, msg        ; pointer to message
    mov rdx, 14         ; message length (including newline)
    syscall

    ; Exit the program
    mov rax, 60         ; syscall number for sys_exit
    xor rdi, rdi        ; exit code 0
    syscall
```

Save this as `hello.asm` and assemble it with:

```bash
nasembler hello.asm -ex
```

## Error Messages

nasembler provides clear and helpful error messages:

```
■ FILE:test_errors.asm

×01 8:9 Bad Operand Invalid first operand for 'jmp' instruction
  │ jmp missing_label
  └→ ^^^~~~~~~~~~~~~

×02 19:0 Dup Label Duplicate label 'duplicate' found
  │ duplicate:
  └→ ^~~~~~~~~

×03 25:4 Unknown Instr Unknown x86-64 instruction 'invalidinstr'
  │ invalidinstr rax, rbx
  └→ ~~~~~~~~~~~~

×04 28:8 Bad Operand Instruction 'mov' requires 2 operands, but found 1
  │ mov rax
  └→ ~~~~~~~

×05 31:8 Bad Operand Invalid first operand for 'mov' instruction
  │ mov xyz, 42
  └→ ~~~~^^^

×06 34:8 Sect Err Invalid section name
  │ section 123invalid
  └→ ~~~~~~~^^^^^^^^^

×07 37:13 Bad Operand Invalid second operand for 'rbx' instruction
  │ mov rax, [rbx+*4]
  └→ ~~~~~~~~~~~~~^~

×08 37:14 Syntax Err Unexpected token '4'
  │ mov rax, [rbx+*4]
  └→ ~~~~~~~~~~~~~~^

×09 41:18 String Error Unclosed string literal
  │ message db "Hello, World 
  └→ ~~~~~~~~~~~~~~~~^~~~

═══════════════════════════════
×9 errs
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
