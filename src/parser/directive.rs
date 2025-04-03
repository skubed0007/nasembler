use crate::parser::ast::{Statement, Directive, Operand};
use crate::tokenizer::TokenType;
use crate::parser::Parser;
use crate::error::ErrorType;

/// Parse a directive statement
pub fn parse_directive(parser: &mut Parser) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if token.token_type != TokenType::Directive {
        return Err(format!("Expected assembly directive (starts with a period), got {:?}", token.token_type));
    }
    
    let directive_name = token.value.to_lowercase();
    let line = token.line;
    
    // Advance past the directive token
    parser.next_token();
    
    // Parse operands based on directive type
    let operands = match directive_name.as_str() {
        "db" | "dw" | "dd" | "dq" => {
            // Get static operands
            let mut values = Vec::new();
            
            while !parser.check(TokenType::NewLine) && !parser.check(TokenType::EOF) {
                let token = parser.current_token();
                
                match token.token_type {
                    TokenType::StringLiteral => {
                        // Store token info before borrowing
                        let token_clone = token.clone();
                        let file_name = parser.file_name.clone();
                        
                        // Better check for unclosed string - if it's the last token or followed by EOF/newline
                        let is_last_token = parser.current == parser.tokens.len() - 1;
                        let next_is_newline_or_eof = parser.current + 1 < parser.tokens.len() && 
                            (parser.tokens[parser.current + 1].0.token_type == TokenType::NewLine ||
                             parser.tokens[parser.current + 1].0.token_type == TokenType::EOF);
                        
                        // For unclosed strings, look for trailing newline within the value
                        let str_value = token.value.clone();
                        let appears_unclosed = str_value.ends_with('\n') || str_value.ends_with('\r');
                        
                        if is_last_token || next_is_newline_or_eof || appears_unclosed {
                            if let Some(collector) = &mut parser.error_collector {
                                collector.add_error_with_location(
                                    ErrorType::UnclosedString,
                                    "Unclosed string literal. String literals must be properly terminated with matching quotes.",
                                    &file_name,
                                    token_clone.line,
                                    token_clone.column
                                );
                            }
                        }
                        
                        values.push(Operand::String(token.value.clone()));
                        parser.next_token();
                    },
                    TokenType::Immediate => {
                        let value = token.value.clone();
                        values.push(Operand::Immediate(value));
                        parser.next_token();
                    },
                    TokenType::LabelRef => {
                        let value = token.value.clone();
                        values.push(Operand::Label(value));
                        parser.next_token();
                    },
                    TokenType::NewLine | TokenType::EOF => {
                        break;
                    },
                    TokenType::Comma => {
                        // Skip over commas between values
                        parser.next_token();
                        continue;
                    },
                    TokenType::Comment => {
                        // Skip comments
                        break;
                    },
                    _ => {
                        let token_type = token.token_type.clone();
                        let token_value = token.value.clone();
                        
                        if let Some(collector) = &mut parser.error_collector {
                            let file_name = parser.file_name.clone();
                            let directive_type = match directive_name.as_str() {
                                "db" => "byte",
                                "dw" => "word (2 bytes)",
                                "dd" => "double word (4 bytes)",
                                "dq" => "quad word (8 bytes)",
                                _ => "data"
                            };
                            
                            collector.add_error_with_location(
                                ErrorType::InvalidOperand,
                                &format!("Invalid value for {} directive: {:?}. Expected a string literal, numeric value, or label reference. Example: {} val1, val2, \"string\"", 
                                          directive_type, token_value, directive_name),
                                &file_name,
                                token.line,
                                token.column
                            );
                        }
                        
                        return Err(format!("Unexpected token in data directive: {:?} at line {}", token_type, token.line));
                    }
                }
            }
            
            values
        },
        "section" => {
            if let Ok(Statement::Directive(directive)) = parse_section_directive(parser, line) {
                directive.operands
            } else {
                return Err(format!("Failed to parse section directive at line {}. Section directives should be in the format: section .text or section .data", line));
            }
        },
        "global" => {
            if let Ok(Statement::Directive(directive)) = parse_global_directive(parser, line) {
                directive.operands
            } else {
                return Err(format!("Failed to parse global directive at line {}. Global directives should be in the format: global symbol_name", line));
            }
        },
        "extern" => {
            if let Ok(Statement::Directive(directive)) = parse_extern_directive(parser, line) {
                directive.operands
            } else {
                return Err(format!("Failed to parse extern directive at line {}. Extern directives should be in the format: extern symbol_name", line));
            }
        },
        "equ" => {
            if let Ok(Statement::Directive(directive)) = parse_equ_directive(parser, line) {
                directive.operands
            } else {
                return Err(format!("Failed to parse equ directive at line {}. Equ directives should be in the format: symbol equ value", line));
            }
        },
        _ => {
            if let Some(collector) = &mut parser.error_collector {
                let file_name = parser.file_name.clone();
                
                collector.add_error_with_location(
                    ErrorType::UnknownDirective,
                    &format!("Unknown directive: '{}'. Common directives include: section, db, dw, dd, dq, global, extern, equ", directive_name),
                    &file_name,
                    line,
                    token.column
                );
            }
            
            return Err(format!("Unsupported directive: {} at line {}", directive_name, line))
        }
    };
    
    Ok(Statement::Directive(Directive {
        name: directive_name,
        operands,
        line,
    }))
}

/// Parse a section directive
fn parse_section_directive(parser: &mut Parser, line: usize) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if token.token_type != TokenType::LabelRef {
        if let Some(collector) = &mut parser.error_collector {
            let file_name = parser.file_name.clone();
            
            collector.add_error_with_location(
                ErrorType::SectionError,
                &format!("Expected section name after 'section' directive, got {:?}. Section names typically start with a period, like '.text', '.data', or '.bss'", token.token_type),
                &file_name,
                token.line,
                token.column
            );
        }
        
        return Err(format!("Expected section name after section directive, got {:?} at line {}", token.token_type, token.line));
    }
    
    let section_name = token.value.clone();
    
    // Advance past the section name
    parser.next_token();
    
    Ok(Statement::Directive(Directive {
        name: "section".to_string(),
        operands: vec![Operand::Label(section_name)],
        line,
    }))
}

/// Parse a global directive
fn parse_global_directive(parser: &mut Parser, line: usize) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if token.token_type != TokenType::LabelRef && token.token_type != TokenType::Identifier {
        if let Some(collector) = &mut parser.error_collector {
            let file_name = parser.file_name.clone();
            
            collector.add_error_with_location(
                ErrorType::InvalidOperand,
                &format!("Expected symbol name after 'global' directive, got {:?}. The global directive makes a symbol visible to the linker. Example: global _start", token.token_type),
                &file_name,
                token.line,
                token.column
            );
        }
        
        return Err(format!("Expected symbol name after global directive, got {:?} at line {}", token.token_type, token.line));
    }
    
    let symbol_name = token.value.clone();
    
    // Advance past the symbol name
    parser.next_token();
    
    Ok(Statement::Directive(Directive {
        name: "global".to_string(),
        operands: vec![Operand::Label(symbol_name)],
        line,
    }))
}

/// Parse an extern directive
fn parse_extern_directive(parser: &mut Parser, line: usize) -> Result<Statement, String> {
    let token = parser.current_token();
    
    if token.token_type != TokenType::LabelRef {
        if let Some(collector) = &mut parser.error_collector {
            let file_name = parser.file_name.clone();
            
            collector.add_error_with_location(
                ErrorType::InvalidOperand,
                &format!("Expected symbol name after 'extern' directive, got {:?}. The extern directive declares a symbol that is defined in another file. Example: extern printf", token.token_type),
                &file_name,
                token.line,
                token.column
            );
        }
        
        return Err(format!("Expected symbol name after extern directive, got {:?} at line {}", token.token_type, token.line));
    }
    
    let symbol_name = token.value.clone();
    
    // Advance past the symbol name
    parser.next_token();
    
    Ok(Statement::Directive(Directive {
        name: "extern".to_string(),
        operands: vec![Operand::Label(symbol_name)],
        line,
    }))
}

/// Parse an equ directive, which can use $ syntax
fn parse_equ_directive(parser: &mut Parser, line: usize) -> Result<Statement, String> {
    let mut operands = Vec::new();
    
    // For equ, we need to handle the special case of $ - label
    // This is commonly used to calculate the size of data
    let token = parser.current_token();
    
    if token.token_type == TokenType::Immediate && token.value == "$" {
        // This is a current location counter reference
        parser.next_token(); // Consume $
        
        // Check for minus operation
        if parser.check(TokenType::Minus) {
            parser.next_token(); // Consume minus
            
            // Check for label or another $ reference
            let next_token = parser.current_token();
            if next_token.token_type == TokenType::LabelRef {
                let label = next_token.value.clone();
                parser.next_token(); // Consume label
                
                // For now, we'll just add a placeholder value
                // In a real implementation, this would be resolved during assembly
                operands.push(Operand::Immediate("0".to_string()));
                
                // Handle further operations if needed (like -1)
                if parser.check(TokenType::Minus) {
                    parser.next_token(); // Consume minus
                    
                    let value_token = parser.current_token();
                    if value_token.token_type == TokenType::Immediate {
                        // Add another placeholder
                        operands.push(Operand::Immediate(value_token.value.clone()));
                        parser.next_token(); // Consume immediate
                    }
                }
            } else {
                if let Some(collector) = &mut parser.error_collector {
                    let file_name = parser.file_name.clone();
                    
                    collector.add_error_with_location(
                        ErrorType::InvalidOperand,
                        &format!("Expected label after '$ -' in equ directive. The '$ - label' format is used to calculate the size of a data block. Example: size equ $ - data_start"),
                        &file_name,
                        next_token.line,
                        next_token.column
                    );
                }
                
                return Err(format!("Expected label after $ - at line {}", line));
            }
        } else {
            // Just the $ by itself
            operands.push(Operand::Immediate("0".to_string()));
        }
    } else {
        // Regular immediate value or other operand
        operands.push(Operand::Immediate(token.value.clone()));
        parser.next_token();
    }
    
    Ok(Statement::Directive(Directive {
        name: "equ".to_string(),
        operands,
        line,
    }))
}

// Helper function to check if we're at the end of file
fn is_at_end_of_file(parser: &Parser) -> bool {
    parser.current >= parser.tokens.len() || 
    (parser.current < parser.tokens.len() && 
     parser.tokens[parser.current].0.token_type == TokenType::EOF)
} 