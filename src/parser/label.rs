use crate::parser::ast::{Statement, Label};
use crate::tokenizer::{TokenType, Token};
use crate::parser::Parser;

/// Parse a label definition
pub fn parse_label(parser: &mut Parser) -> Result<Statement, String> {
    // Expect a label token
    let (token, line) = match parser.peek() {
        Some(t) => t,
        None => return Err("Unexpected end of tokens while parsing label".to_string()),
    };
    
    if token.token_type != TokenType::Label {
        return Err(format!("Expected label at line {}", line));
    }
    let label_name = token.value.clone();
    parser.advance(); // Consume the label
    
    // Return the label
    Ok(Statement::Label(label_name))
} 