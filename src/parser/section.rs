use crate::parser::ast::{Statement, Section};
use crate::tokenizer::{TokenType, Token};
use crate::parser::Parser;

/// Parse a section directive
pub fn parse_section(parser: &mut Parser) -> Result<Statement, String> {
    // Get the current token instead of peeking
    let token = parser.current_token();
    let line = token.line;
    
    // Check if it's a directive token (section names start with '.')
    if token.token_type != TokenType::Directive {
        return Err(format!("Expected section directive at line {}", line));
    }
    
    let section_name = token.value.clone();
    parser.next_token(); // Move to the next token
    
    // Create the section
    let section = Section {
        name: section_name,
        line,
    };
    
    Ok(Statement::Section(section))
} 