use std::collections::HashMap;
use crate::tokenizer::{Token, TokenType};
use crate::encoder::MachineCodeEncoder;
use crate::error::{ErrorCollector, ErrorType};

pub mod ast;
pub mod directive;
pub mod instruction;
pub mod section;
pub mod label;

pub struct Parser {
    tokens: Vec<(Token, usize)>,
    current: usize,
    labels: HashMap<String, usize>,
    label_offsets: HashMap<String, u64>,
    current_section: String,
    text_offset: u64,
    data_offset: u64,
    bss_offset: u64,
    error_collector: Option<ErrorCollector>,
    file_name: String,
    continue_on_errors: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens_with_index: Vec<(Token, usize)> = tokens.into_iter()
            .enumerate()
            .map(|(i, token)| (token, i))
            .collect();
        
        Self {
            tokens: tokens_with_index,
            current: 0,
            labels: HashMap::new(),
            label_offsets: HashMap::new(),
            current_section: ".text".to_string(),
            text_offset: 0x400000,
            data_offset: 0x600000,
            bss_offset: 0x800000,
            error_collector: None,
            file_name: "unknown".to_string(),
            continue_on_errors: false,
        }
    }
    
    /// Set the error collector to use for parsing
    pub fn with_error_collector(mut self, collector: ErrorCollector) -> Self {
        self.error_collector = Some(collector);
        self
    }
    
    /// Set the file name for better error reporting
    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = file_name;
        self
    }
    
    /// Set whether to continue on errors
    pub fn with_continue_on_errors(mut self, continue_on_errors: bool) -> Self {
        self.continue_on_errors = continue_on_errors;
        self
    }
    
    /// Add an error to the collector
    fn add_error(&mut self, error_type: ErrorType, message: &str, token: &Token) {
        if let Some(collector) = &mut self.error_collector {
            collector.add_error_with_location(
                error_type,
                message,
                &self.file_name,
                token.line,
                token.column
            );
        }
    }
    
    /// Check if there are any errors
    fn has_errors(&self) -> bool {
        if let Some(collector) = &self.error_collector {
            collector.has_errors()
        } else {
            false
        }
    }
    
    pub fn parse(&mut self) -> Result<ast::Program, String> {
        let mut program = ast::Program::new();
        
        // First pass: collect labels and track sections
        match self.collect_labels_and_sections() {
            Ok(_) => {},
            Err(err) => {
                if !self.continue_on_errors {
                    return Err(err);
                }
                // Otherwise continue with what we've collected
            }
        }
        
        // Populate labels and sections in the Program
        for (label_name, offset) in &self.label_offsets {
            let section = if *offset >= self.text_offset && *offset < self.data_offset {
                Some(".text".to_string())
            } else if *offset >= self.data_offset && *offset < self.bss_offset {
                Some(".data".to_string())
            } else if *offset >= self.bss_offset {
                Some(".bss".to_string())
            } else {
                None
            };
            
            program.add_label(label_name.clone(), *offset, section);
        }
        
        // Add default sections with sizes
        program.add_section(".text".to_string(), 0x1000);
        program.add_section(".data".to_string(), 0x1000);
        program.add_section(".bss".to_string(), 0x1000);
        
        // Reset for second pass
        self.current = 0;
        
        // Second pass: parse statements and encode instructions
        while !self.is_at_end() {
            // Check for EOF token
            if let Some((token, _)) = self.peek() {
                if token.token_type == TokenType::EOF {
                    break; // Stop parsing on EOF
                }
            }
            
            match self.parse_statement() {
                Ok(statement) => {
                    program.add_statement(statement);
                },
                Err(error) => {
                    // If we have an error collector, add the error to it and continue
                    // Otherwise, return the error immediately
                    if self.error_collector.is_some() && self.continue_on_errors {
                        // Skip to the next line to continue parsing
                        while !self.is_at_end() && !self.check(TokenType::NewLine) {
                            self.advance();
                        }
                        
                        // Skip the newline if present
                        if self.check(TokenType::NewLine) {
                            self.advance();
                        }
                    } else {
                        return Err(error);
                    }
                }
            }
        }
        
        // Third pass: encode instructions with machine code
        match self.encode_instructions(&mut program) {
            Ok(_) => {},
            Err(err) => {
                if !self.continue_on_errors || self.error_collector.is_none() {
                    return Err(err);
                }
                // Otherwise continue with what we've encoded
            }
        }
        
        // If we have errors but we're not continuing on errors, return the error
        if self.has_errors() && !self.continue_on_errors {
            return Err("Errors occurred during parsing".to_string());
        }
        
        Ok(program)
    }
    
    // Enhanced label collection method that also tracks sections
    fn collect_labels_and_sections(&mut self) -> Result<(), String> {
        let mut statement_index = 0;
        let mut current_offset = self.text_offset; // Start at text base
        
        while !self.is_at_end() {
            let token_info = match self.peek() {
                Some((token, _)) => (token.clone(), token.token_type.clone(), token.value.clone(), token.line, token.column),
                None => break,
            };
            
            let (token, token_type, token_value, token_line, token_column) = token_info;
            
            match token_type {
                TokenType::Label => {
                    let label = token_value;
                    
                    // Check for duplicate labels
                    if self.labels.contains_key(&label) {
                        let error_msg = format!("Duplicate label '{}' found", label);
                        
                        if let Some(collector) = &mut self.error_collector {
                            collector.add_error_with_location(
                                ErrorType::DuplicateLabel,
                                &error_msg,
                                &self.file_name,
                                token_line,
                                token_column
                            );
                            
                            // Skip this label if we're continuing on errors
                            if self.continue_on_errors {
                                self.advance();
                                
                                // Skip colon if present
                                if self.check(TokenType::Colon) {
                                    self.advance();
                                }
                                
                                continue;
                            } else {
                                return Err(error_msg);
                            }
                        } else {
                            return Err(error_msg);
                        }
                    }
                    
                    self.labels.insert(label.clone(), statement_index);
                    
                    // Store actual memory offset for this label
                    self.label_offsets.insert(label, current_offset);
                    
                    self.advance();
                    
                    // Skip colon if present
                    if self.check(TokenType::Colon) {
                        self.advance();
                    }
                },
                TokenType::Directive => {
                    if token_value == "section" {
                        self.advance(); // Consume directive
                        
                        // Get section name
                        let section_info = match self.peek() {
                            Some((section_token, _)) => {
                                (section_token.clone(), section_token.token_type.clone(), 
                                 section_token.value.clone(), section_token.line, section_token.column)
                            },
                            None => {
                                let error_msg = "Missing section name after section directive".to_string();
                                
                                if let Some(collector) = &mut self.error_collector {
                                    collector.add_error_with_location(
                                        ErrorType::SectionError,
                                        &error_msg,
                                        &self.file_name,
                                        token_line,
                                        token_column
                                    );
                                    
                                    if self.continue_on_errors {
                                        // Skip to next line and continue
                                        while !self.is_at_end() && !self.check(TokenType::NewLine) {
                                            self.advance();
                                        }
                                        
                                        if self.check(TokenType::NewLine) {
                                            self.advance();
                                        }
                                        
                                        statement_index += 1;
                                        continue;
                                    } else {
                                        return Err(error_msg);
                                    }
                                } else {
                                    return Err(error_msg);
                                }
                            }
                        };
                        
                        let (_, section_token_type, section_name, section_line, section_column) = section_info;
                        
                        // Allow both LabelRef and Identifier for section names
                        if section_token_type == TokenType::LabelRef || section_token_type == TokenType::Identifier {
                            self.current_section = section_name.clone();
                            
                            // Update current offset based on section
                            match section_name.as_str() {
                                ".text" => current_offset = self.text_offset,
                                ".data" => current_offset = self.data_offset,
                                ".bss" => current_offset = self.bss_offset,
                                _ => {
                                    // Custom section, use text offset for now
                                    current_offset = self.text_offset;
                                }
                            }
                            
                            // Skip to next line
                            while !self.is_at_end() && !self.check(TokenType::NewLine) {
                                self.advance();
                            }
                            
                            // Skip the newline
                            if self.check(TokenType::NewLine) {
                                self.advance();
                            }
                            
                            statement_index += 1;
                            continue;
                        } else {
                            // Invalid section name
                            let error_msg = format!("Invalid section name, expected identifier or label reference");
                            
                            if let Some(collector) = &mut self.error_collector {
                                collector.add_error_with_location(
                                    ErrorType::SectionError,
                                    &error_msg,
                                    &self.file_name,
                                    section_line,
                                    section_column
                                );
                                
                                if self.continue_on_errors {
                                    // Skip to next line and continue
                                    while !self.is_at_end() && !self.check(TokenType::NewLine) {
                                        self.advance();
                                    }
                                    
                                    if self.check(TokenType::NewLine) {
                                        self.advance();
                                    }
                                    
                                    statement_index += 1;
                                    continue;
                                } else {
                                    return Err(error_msg);
                                }
                            } else {
                                return Err(error_msg);
                            }
                        }
                    } else {
                        // Skip other directives for now
                        while !self.is_at_end() && !self.check(TokenType::NewLine) {
                            self.advance();
                        }
                        
                        // Skip the newline
                        if self.check(TokenType::NewLine) {
                            self.advance();
                        }
                        
                        statement_index += 1;
                        // Estimate offset increase for directives (approx)
                        current_offset += 8;
                        continue;
                    }
                },
                TokenType::NewLine => {
                    self.advance();
                    // Skip empty lines when counting statements
                },
                TokenType::Comment => {
                    self.advance();
                    statement_index += 1;
                },
                TokenType::Instruction | TokenType::InstrData | TokenType::InstrArith 
                | TokenType::InstrLogic | TokenType::InstrJump | TokenType::InstrSIMD => {
                    // For instructions, estimate size (approx. 8 bytes per instruction)
                    current_offset += 8;
                    
                    // Count non-empty, non-label statements
                    statement_index += 1;
                    
                    // Skip to next line
                    while !self.is_at_end() && !self.check(TokenType::NewLine) {
                        self.advance();
                    }
                    
                    // Skip the newline
                    if self.check(TokenType::NewLine) {
                        self.advance();
                    }
                },
                _ => {
                    // Count non-empty, non-label statements
                    statement_index += 1;
                    
                    // Skip to next line
                    while !self.is_at_end() && !self.check(TokenType::NewLine) {
                        self.advance();
                    }
                    
                    // Skip the newline
                    if self.check(TokenType::NewLine) {
                        self.advance();
                    }
                }
            }
        }
        
        // Reset position for next pass
        self.current = 0;
        
        Ok(())
    }
    
    /// Get a string with examples of common x86-64 instructions
    fn get_common_instruction_examples() -> &'static str {
        "Common x86-64 instructions include: mov, add, sub, mul, div, push, pop, call, ret, jmp, je, jne, cmp, and, or, xor, shl, shr, lea"
    }
    
    // Parse a single statement (instruction, directive, label, comment)
    fn parse_statement(&mut self) -> Result<ast::Statement, String> {
        match self.peek() {
            Some((token, _)) => {
                match token.token_type {
                    TokenType::Instruction | TokenType::InstrData | TokenType::InstrArith 
                    | TokenType::InstrLogic | TokenType::InstrJump | TokenType::InstrSIMD => {
                        instruction::parse_instruction(self)
                    },
                    TokenType::Directive => {
                        // Special handling for section directives
                        if token.value == "section" {
                            // Get a copy of the directive token before advancing
                            let directive_token = token.clone();
                            self.advance(); // consume the directive
                            
                            // Check for the section name
                            if let Some((section_token, _)) = self.peek() {
                                if section_token.token_type == TokenType::Identifier || section_token.token_type == TokenType::LabelRef {
                                    let section_name = section_token.value.clone();
                                    let section_line = section_token.line;
                                    self.advance(); // consume the section name
                                    
                                    // Create a Section statement
                                    return Ok(ast::Statement::Section(ast::Section {
                                        name: section_name,
                                        line: section_line,
                                    }));
                                }
                            }
                            
                            // If we got here, the section name is not an identifier or LabelRef
                            // Fall back to normal directive parsing
                            return directive::parse_directive(self);
                        }
                        
                        directive::parse_directive(self)
                    },
                    TokenType::Label => {
                        let label = token.value.clone();
                        self.advance();
                        
                        // Check if there's a colon after the label and consume it
                        if let Some((next, _)) = self.peek() {
                            if next.token_type == TokenType::Colon {
                                self.advance(); // Consume the colon
                            }
                        }
                        
                        Ok(ast::Statement::Label(label))
                    },
                    TokenType::Comment => {
                        let comment = token.value.clone();
                        self.advance();
                        Ok(ast::Statement::Comment(comment))
                    },
                    TokenType::NewLine => {
                        self.advance();
                        Ok(ast::Statement::Empty)
                    },
                    TokenType::EOF => {
                        // Just return Empty statement for EOF
                        self.advance();
                        Ok(ast::Statement::Empty)
                    },
                    TokenType::Identifier => {
                        // Check if it's followed by a colon - then it's a label
                        let current_token = token.clone();
                        
                        if let Some((next_token, _)) = self.peek_ahead(1) {
                            let next_token_clone = next_token.clone();
                            
                            if next_token_clone.token_type == TokenType::Colon {
                                let label = current_token.value.clone();
                                self.advance(); // Consume the identifier
                                self.advance(); // Consume the colon
                                return Ok(ast::Statement::Label(label));
                            }
                            // Check if it's followed by a directive like 'db', 'dw', etc. - then it's a variable declaration
                            else if next_token_clone.token_type == TokenType::Directive {
                                // This is a variable declaration (e.g., hello db 'Hello, World!', 0)
                                let var_name = current_token.value.clone();
                                let directive_name = next_token_clone.value.clone();
                                
                                // Advance past the identifier
                                self.advance();
                                
                                // Create a label for the variable and return it
                                // The parse() method will be called again for the directive
                                return Ok(ast::Statement::Label(var_name));
                            }
                            // If followed by register, immediate, or other operand types, 
                            // this could be an unrecognized instruction
                            else if next_token_clone.token_type == TokenType::Register || 
                                   next_token_clone.token_type == TokenType::Reg64Bit ||
                                   next_token_clone.token_type == TokenType::Reg32Bit ||
                                   next_token_clone.token_type == TokenType::Reg16Bit ||
                                   next_token_clone.token_type == TokenType::Reg8Bit ||
                                   next_token_clone.token_type == TokenType::Immediate ||
                                   next_token_clone.token_type == TokenType::OpenBracket {
                                    
                                if let Some(collector) = &mut self.error_collector {
                                    let recognized_instructions = Self::get_common_instruction_examples();
                                    
                                    collector.add_error_with_location(
                                        ErrorType::UnknownInstruction,
                                        &format!("Unknown x86-64 instruction '{}'. {}",
                                                current_token.value, recognized_instructions),
                                        &self.file_name,
                                        current_token.line,
                                        current_token.column
                                    );
                                }
                                
                                if self.continue_on_errors {
                                    // Skip to the next line and return an empty statement
                                    while !self.is_at_end() && !self.check(TokenType::NewLine) {
                                        self.advance();
                                    }
                                    
                                    if self.check(TokenType::NewLine) {
                                        self.advance();
                                    }
                                    
                                    return Ok(ast::Statement::Empty);
                                } else {
                                    return Err(format!("Unknown instruction '{}' at line {}. Check for typos or use a valid x86-64 instruction.", 
                                                    current_token.value, current_token.line));
                                }
                            }
                        }
                        
                        // Otherwise, it's an unexpected token
                        if let Some(collector) = &mut self.error_collector {
                            collector.add_error_with_location(
                                ErrorType::SyntaxError,
                                &format!("Unexpected identifier '{}'. Identifiers must be followed by a colon for labels, a directive for variable declarations, or must be a valid instruction.",
                                        current_token.value),
                                &self.file_name,
                                current_token.line,
                                current_token.column
                            );
                        }
                        
                        Err(format!("Unexpected token type {:?} at line {}. In x86-64 assembly, lines typically start with a label, instruction, or directive.", 
                                    current_token.token_type, current_token.line))
                    },
                    _ => {
                        // Store the token information before borrowing
                        let token_value = token.value.clone();
                        let token_type = token.token_type.clone();
                        let token_line = token.line;
                        let token_column = token.column;
                        
                        if let Some(collector) = &mut self.error_collector {
                            collector.add_error_with_location(
                                ErrorType::SyntaxError,
                                &format!("Unexpected token '{}' of type {:?}. Assembly statements must start with a label, instruction, or directive.",
                                        token_value, token_type),
                                &self.file_name,
                                token_line,
                                token_column
                            );
                        }
                        
                        Err(format!("Unexpected token type {:?} at line {}. Each line should begin with a label, instruction, or directive.", token_type, token_line))
                    }
                }
            },
            None => Ok(ast::Statement::Empty),
        }
    }
    
    // Enhanced encoding method that resolves label references
    fn encode_instructions(&self, program: &mut ast::Program) -> Result<(), String> {
        let encoder = MachineCodeEncoder::new();
        
        for statement in &mut program.statements {
            if let ast::Statement::Instruction(ref mut instruction) = statement {
                // For LEA instructions, don't try to resolve the label
                if instruction.name.to_lowercase() == "lea" {
                    instruction.machine_code = encoder.encode(instruction);
                    continue;
                }
                
                // Check for and resolve label references in operands
                for operand in &mut instruction.operands {
                    if let ast::Operand::Label(label) = operand {
                        if let Some(offset) = self.label_offsets.get(label) {
                            // Replace label with resolved address
                            *operand = ast::Operand::Immediate(format!("0x{:x}", offset));
                        } else {
                            return Err(format!("Undefined label reference: {}", label));
                        }
                    }
                }
                
                // Now encode with resolved operands
                instruction.machine_code = encoder.encode(instruction);
            }
        }
        
        Ok(())
    }
    
    // Helper method to check if we are at the end of the tokens
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    // Helper method to peek at the current token without consuming it
    pub fn peek(&self) -> Option<&(Token, usize)> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current])
        }
    }
    
    // Helper method to advance to the next token
    pub fn advance(&mut self) -> &(Token, usize) {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    // Helper method to get the previous token
    fn previous(&self) -> &(Token, usize) {
        &self.tokens[self.current - 1]
    }
    
    // Helper method to check if the current token has the expected type
    pub fn check(&self, token_type: TokenType) -> bool {
        if let Some((token, _)) = self.peek() {
            token.token_type == token_type
        } else {
            false
        }
    }
    
    // Helper method to check if the current token has the expected type and value
    pub fn check_value(&self, token_type: TokenType, value: &str) -> bool {
        if let Some((token, _)) = self.peek() {
            token.token_type == token_type && token.value == value
        } else {
            false
        }
    }
    
    // Helper method to get the current token
    pub fn current_token(&self) -> Token {
        if let Some((token, _)) = self.peek() {
            token.clone()
        } else {
            // Return an EOF token if we're at the end
            Token {
                token_type: TokenType::EOF,
                value: "".to_string(),
                line: 0,
                column: 0,
            }
        }
    }
    
    // Helper method to advance to the next token and return the current token
    pub fn next_token(&mut self) -> Token {
        let current = self.current_token();
        self.advance();
        current
    }
    
    // Added: Get label offset for a given label name
    pub fn get_label_offset(&self, label: &str) -> Option<u64> {
        self.label_offsets.get(label).cloned()
    }
    
    // Added: Get current section name
    pub fn get_current_section(&self) -> &str {
        &self.current_section
    }
    
    // Helper method to peek at a token n positions ahead without consuming it
    pub fn peek_ahead(&self, n: usize) -> Option<&(Token, usize)> {
        if self.current + n >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.current + n])
        }
    }
    
    /// Get the current error collector
    pub fn get_error_collector(&self) -> Option<ErrorCollector> {
        self.error_collector.clone()
    }
} 