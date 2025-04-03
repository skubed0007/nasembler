# nasembler Command-Line Reference

This document details the command-line interface (CLI) options available in nasembler.

## Basic Usage

```bash
nasembler [OPTIONS] <INPUT_FILE>
```

The nasembler command-line interface provides a range of options to control the assembly process. The only required argument is the input file path.

## Core Options

| Option | Long Option | Description |
|--------|-------------|-------------|
| `-o <FILE>` | `--output <FILE>` | Specify the output file name. If omitted, nasembler will use the input file name without the extension for ELF format, or with appropriate extension for other formats. |
| `-f <FORMAT>` | `--format <FORMAT>` | Specify the output format. Available options: `elf` (default), `bin`, `hex`. |
| `-v` | `--verbose` | Enable verbose output with detailed information about the assembly process. |
| `-x` | `--execute` | Execute the compiled binary after successful assembly. |
| `-e` | `--make-executable` | Make the output file executable (chmod +x). |
| `-s` | `--stop-on-first-error` | Stop assembly on the first error instead of collecting all errors. |
| | `--silent` | Silent mode - only show errors, not warnings. |

## Debugging Options

These options are useful for debugging and understanding the assembly process:

| Option | Long Option | Description |
|--------|-------------|-------------|
| | `--parse-only` | Only parse the file, don't generate output. |
| | `--tokenize-only` | Only tokenize the file, don't parse or generate output. |
| | `--dump-tokens` | Dump tokens after tokenization. |
| | `--dump-ast` | Dump the Abstract Syntax Tree (AST) after parsing. |

## Configuration Options

| Option | Long Option | Description |
|--------|-------------|-------------|
| `-p <FILE>` | `--opcodes <FILE>` | Path to the opcodes definition file. |

## Examples

### Basic Assembly

```bash
# Assemble a file and create an executable
nasembler program.asm

# Specify a custom output file name
nasembler program.asm -o myprogram

# Assemble and run immediately
nasembler program.asm -x
```

### Output Formats

```bash
# Generate raw binary output
nasembler program.asm -f bin

# Generate Intel HEX format (not fully implemented)
nasembler program.asm -f hex
```

### Debugging and Analysis

```bash
# Run with verbose output to see detailed information
nasembler program.asm -v

# Only tokenize the file and print the tokens
nasembler program.asm --tokenize-only

# Only parse the file without generating code
nasembler program.asm --parse-only

# Dump tokens for analysis
nasembler program.asm --dump-tokens

# Dump the Abstract Syntax Tree
nasembler program.asm --dump-ast
```

## Error Handling

By default, nasembler will collect all errors in a file and report them together. You can change this behavior:

```bash
# Stop on the first error
nasembler program.asm -s

# Silent mode (only show errors, not warnings)
nasembler program.asm --silent
```

## Format-Specific Options

### ELF Format (default)

The ELF format produces a standard Linux executable file.

```bash
# Make the output file executable (chmod +x)
nasembler program.asm -e

# Create executable and run it
nasembler program.asm -ex
```

## Exit Codes

nasembler returns the following exit codes:

| Exit Code | Description |
|-----------|-------------|
| 0 | Success |
| 1 | Assembly failed due to errors |
| 2 | Invalid command-line arguments |
| 3 | Failed to read input file |
| 4 | Failed to write output file |

## Environment Variables

Currently, nasembler does not use any environment variables for configuration.

## Tips and Tricks

1. **Faster Development Cycle**: Use the `-x` flag to assemble and run in one step.

   ```bash
   nasembler program.asm -x
   ```

2. **Debugging Assembly**: Use `--dump-ast` to understand how your code is being parsed.

   ```bash
   nasembler program.asm --dump-ast
   ```

3. **Understanding Errors**: Use verbose mode with `-v` to get more context about errors.

   ```bash
   nasembler program.asm -v
   ```

4. **Clean Output**: Use `--silent` to only see errors, not warnings.

   ```bash
   nasembler program.asm --silent
   ``` 