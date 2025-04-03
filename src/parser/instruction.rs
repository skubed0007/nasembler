use crate::parser::ast::{Statement, Instruction, Operand, MemoryReference};
use crate::tokenizer::TokenType;
use crate::parser::Parser;
use crate::error::ErrorType;

/// Parse an instruction statement (e.g., "mov eax, ebx")
pub fn parse_instruction(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.current_token();
    
    let instruction_name = token.value.to_lowercase();
    let line = token.line;
    
    // Advance past the instruction token
    parser.next_token();
    
    // Parse operands
    let operands = match parse_operands(parser) {
        Ok(ops) => ops,
        Err(err) => {
            // Get current token information before borrowing
            let current_token = parser.current_token();
            let file_name = parser.file_name.clone();
            
            // Report error to collector and continue if possible
            if let Some(collector) = &mut parser.error_collector {
                collector.add_error_with_location(
                    ErrorType::InvalidOperand,
                    &format!("Invalid operand syntax in '{}' instruction: {}", instruction_name, err),
                    &file_name,
                    current_token.line,
                    current_token.column
                );
            }
            
            if parser.continue_on_errors {
                // Skip to next line
                while parser.check(TokenType::NewLine) == false && !parser.is_at_end() {
                    parser.advance();
                }
                Vec::new() // Return empty operands to continue
            } else {
                return Err(err);
            }
        }
    };
    
    Ok(Statement::Instruction(Instruction {
        name: instruction_name,
        operands,
        machine_code: Vec::new(), // Machine code will be filled in later
        line,
    }))
}

/// Parse operands for an instruction
fn parse_operands(parser: &mut Parser) -> Result<Vec<Operand>, String> {
    let mut operands = Vec::new();
    
    // Check if we have any operands at all
    if parser.check(TokenType::NewLine) || parser.check(TokenType::EOF) {
        // For instructions that require operands (like mov), this is an error
        let instruction = parser.tokens[parser.current - 1].0.value.to_lowercase();
        if requires_operands(&instruction) {
            let token = &parser.tokens[parser.current - 1].0;
            let file_name = parser.file_name.clone();
            
            if let Some(collector) = &mut parser.error_collector {
                let operand_examples = get_example_operands(&instruction);
                let msg = format!(
                    "Instruction '{}' requires {} but none were provided. {}",
                    instruction,
                    if required_operand_count(&instruction).unwrap_or(1) == 1 {
                        "an operand"
                    } else {
                        "operands"
                    },
                    operand_examples
                );
                
                collector.add_error_with_location(
                    ErrorType::InvalidOperand,
                    &msg,
                    &file_name,
                    token.line,
                    token.column
                );
            }
            
            if !parser.continue_on_errors {
                return Err(format!("Missing operand for instruction '{}'", instruction));
            }
        }
        return Ok(operands);
    }
    
    // Process first operand
    match parse_operand(parser) {
        Ok(op) => operands.push(op),
        Err(err) => {
            // Get current token information before borrowing
            let current_token = parser.current_token();
            let file_name = parser.file_name.clone();
            
            if let Some(collector) = &mut parser.error_collector {
                let instruction = parser.tokens[parser.current - 1].0.value.to_lowercase();
                let operand_examples = get_example_operands(&instruction);
                
                collector.add_error_with_location(
                    ErrorType::InvalidOperand,
                    &format!("Invalid first operand for '{}' instruction: {}. {}", 
                             instruction, err, operand_examples),
                    &file_name,
                    current_token.line,
                    current_token.column
                );
            }
            
            if parser.continue_on_errors {
                // Skip to next token
                parser.advance();
            } else {
                return Err(err);
            }
        }
    }
    
    // Process remaining operands (if any)
    while parser.check(TokenType::Comma) {
        // Skip the comma
        parser.next_token();
        
        match parse_operand(parser) {
            Ok(op) => operands.push(op),
            Err(err) => {
                // Get current token information before borrowing
                let current_token = parser.current_token();
                let file_name = parser.file_name.clone();
                
                if let Some(collector) = &mut parser.error_collector {
                    let instruction = parser.tokens[parser.current - operands.len() - 1].0.value.to_lowercase();
                    let position = operands.len() + 1; // 2nd, 3rd, etc.
                    let position_str = match position {
                        2 => "second",
                        3 => "third",
                        _ => "next",
                    };
                    
                    collector.add_error_with_location(
                        ErrorType::InvalidOperand,
                        &format!("Invalid {} operand for '{}' instruction: {}. Expected a register, immediate value, or memory reference.",
                                 position_str, instruction, err),
                        &file_name,
                        current_token.line,
                        current_token.column
                    );
                }
                
                if parser.continue_on_errors {
                    // Skip to next token
                    parser.advance();
                    break; // Stop processing operands
                } else {
                    return Err(err);
                }
            }
        }
    }
    
    // Check if instruction requires specific number of operands
    let instruction = parser.tokens[parser.current - operands.len() - 1].0.value.to_lowercase();
    if let Some(required) = required_operand_count(&instruction) {
        if operands.len() != required {
            let token = &parser.tokens[parser.current - operands.len() - 1].0;
            let file_name = parser.file_name.clone();
            
            if let Some(collector) = &mut parser.error_collector {
                let operand_examples = get_example_operands(&instruction);
                let message = if operands.len() < required {
                    format!("Instruction '{}' requires {} operands, but found {}. {}", 
                            instruction, required, operands.len(), operand_examples)
                } else {
                    format!("Instruction '{}' requires exactly {} operands, but found {}. Remove extra operands.", 
                            instruction, required, operands.len())
                };
                
                collector.add_error_with_location(
                    ErrorType::InvalidOperand,
                    &message,
                    &file_name,
                    token.line,
                    token.column
                );
            }
            
            if !parser.continue_on_errors {
                return Err(format!("Instruction '{}' requires {} operands, found {}",
                                  instruction, required, operands.len()));
            }
        }
    }
    
    Ok(operands)
}

/// Get example operands for an instruction
fn get_example_operands(instruction: &str) -> &'static str {
    match instruction {
        "mov" => "Example: mov rax, rbx or mov rax, [rbx] or mov rax, 42",
        "add" | "sub" | "and" | "or" | "xor" | "cmp" => 
            match instruction {
                "add" => "Example: add rax, rbx or add rax, 42",
                "sub" => "Example: sub rax, rbx or sub rax, 42", 
                "and" => "Example: and rax, rbx or and rax, 42",
                "or" => "Example: or rax, rbx or or rax, 42",
                "xor" => "Example: xor rax, rbx or xor rax, 42",
                "cmp" => "Example: cmp rax, rbx or cmp rax, 42",
                _ => "Example: op rax, rbx or op rax, 42", // Should never happen
            },
        "mul" | "div" => 
            if instruction == "mul" {
                "Example: mul rax"
            } else {
                "Example: div rax"
            },
        "push" => "Example: push rax or push 42",
        "pop" => "Example: pop rax",
        "jmp" | "je" | "jne" | "jg" | "jge" | "jl" | "jle" => 
            match instruction {
                "jmp" => "Example: jmp label",
                "je" => "Example: je label",
                "jne" => "Example: jne label",
                "jg" => "Example: jg label",
                "jge" => "Example: jge label",
                "jl" => "Example: jl label", 
                "jle" => "Example: jle label",
                _ => "Example: jXX label", // Should never happen
            },
        "call" => "Example: call function_name",
        "lea" => "Example: lea rax, [rbx + 8]",
        "shl" | "shr" => 
            if instruction == "shl" {
                "Example: shl rax, 2"
            } else {
                "Example: shr rax, 2"
            },
        "ret" => "This instruction doesn't need any operands",
        "syscall" => "This instruction doesn't need any operands",
        "nop" => "This instruction doesn't need any operands",
        _ => "Check the x86-64 assembly manual for correct syntax",
    }
}

/// Determine if an instruction requires operands
fn requires_operands(instruction: &str) -> bool {
    match instruction {
        "mov" | "add" | "sub" | "mul" | "div" | "and" | "or" | "xor" | "cmp" |
        "shl" | "shr" | "jmp" | "je" | "jne" | "jg" | "jge" | "jl" | "jle" |
        "call" | "lea" => true,
        _ => false,
    }
}

/// Determine the required number of operands for an instruction
fn required_operand_count(instruction: &str) -> Option<usize> {
    match instruction {
        "mov" | "add" | "sub" | "and" | "or" | "xor" | "cmp" |
        "shl" | "shr" | "lea" => Some(2),  // Two operands
        "mul" | "div" | "jmp" | "je" | "jne" | "jg" | "jge" | "jl" | "jle" |
        "call" | "push" | "pop" => Some(1),  // One operand
        "ret" | "syscall" | "nop" => Some(0),  // No operands
        _ => None,  // Unknown instruction
    }
}

/// Parse a single operand
fn parse_operand(parser: &mut Parser) -> Result<Operand, String> {
    let token = parser.current_token();
    
    match token.token_type {
        TokenType::Register | TokenType::Reg64Bit | TokenType::Reg32Bit | 
        TokenType::Reg16Bit | TokenType::Reg8Bit | TokenType::RegXMM | 
        TokenType::RegYMM | TokenType::RegZMM | TokenType::RegSpecial => {
            let register = token.value.to_lowercase();
            parser.next_token();
            Ok(Operand::Register(register))
        },
        TokenType::Immediate => {
            let immediate = token.value.clone();
            parser.next_token();
            Ok(Operand::Immediate(immediate))
        },
        TokenType::LabelRef => {
            let label = token.value.clone();
            parser.next_token();
            Ok(Operand::Label(label))
        },
        TokenType::OpenBracket => {
            // This is a memory reference
            parse_memory_reference(parser)
        },
        _ => {
            Err(format!("Unexpected token in operand: {:?}. Expected a register, immediate value, or memory reference", token.token_type))
        }
    }
}

/// Parse a memory reference (e.g., [rax], [rbx+4], [rcx+rdx*2+8])
fn parse_memory_reference(parser: &mut Parser) -> Result<Operand, String> {
    // Skip the opening bracket
    parser.next_token();
    
    let token = parser.current_token();
    
    // Check for register or label
    let base = if token.token_type == TokenType::Register || 
               token.token_type == TokenType::Reg64Bit || 
               token.token_type == TokenType::Reg32Bit || 
               token.token_type == TokenType::Reg16Bit || 
               token.token_type == TokenType::Reg8Bit {
        let register = token.value.to_lowercase();
        parser.next_token();
        Some(register)
    } else if token.token_type == TokenType::LabelRef || token.token_type == TokenType::Identifier {
        let label = token.value.clone();
        parser.next_token();
        
        // For simplicity, treat labels as special case and return early
        if parser.check(TokenType::CloseBracket) {
            parser.next_token(); // Skip closing bracket
            return Ok(Operand::Label(label));
        } else {
            // Check for displacement operations like [label-1]
            if parser.check(TokenType::Minus) || parser.check(TokenType::Plus) {
                // Skip the operator and parse the rest
                parser.next_token(); 
                // For now, we'll ignore the displacement and just return the label
                // In a real implementation, we'd handle the displacement properly
                
                // Skip any immediate values
                if parser.current_token().token_type == TokenType::Immediate {
                    parser.next_token();
                }
                
                // Skip to the closing bracket
                if parser.check(TokenType::CloseBracket) {
                    parser.next_token();
                    return Ok(Operand::Label(label));
                }
            }
            
            return Err(format!("Expected closing bracket ']' after label in memory reference. Memory references with labels should be in the form [label] or [label+offset]"))
        }
    } else {
        None
    };
    
    // Check for the rest of the components
    let index = None; // We're simplifying for now
    let scale = None;
    let displacement = None;
    
    // Handle operators and additional components
    if parser.check(TokenType::Plus) || parser.check(TokenType::Minus) {
        // Get operator type for better error messages
        let operator = parser.current_token().token_type.clone();
        parser.next_token(); // Skip the operator
        
        // Check for the next token
        let next_token = parser.current_token();
        
        // If it's an unexpected token type, provide a better error message
        if next_token.token_type != TokenType::Register && 
           next_token.token_type != TokenType::Reg64Bit && 
           next_token.token_type != TokenType::Reg32Bit && 
           next_token.token_type != TokenType::Reg16Bit && 
           next_token.token_type != TokenType::Reg8Bit && 
           next_token.token_type != TokenType::Immediate {
            
            return Err(format!("Invalid expression in memory reference after '{}'. Expected a register or immediate value, found {:?}. Valid forms: [reg], [reg+offset], [reg+reg*scale]", 
                              if operator == TokenType::Plus { "+" } else { "-" }, 
                              next_token.token_type));
        }
        
        // Skip to the closing bracket even if we have an error, to continue parsing
        while !parser.check(TokenType::CloseBracket) && !parser.is_at_end() {
            parser.next_token();
        }
    }
    
    // Skip to the closing bracket
    if !parser.check(TokenType::CloseBracket) {
        return Err(format!("Expected closing bracket ']' in memory reference. Memory references should be in the form [register], [register+offset], or [label]"))
    }
    
    // Skip the closing bracket
    parser.next_token();
    
    Ok(Operand::Memory(MemoryReference {
        base,
        index,
        scale,
        displacement,
    }))
}

