use std::collections::HashMap;
use std::fmt;
use once_cell::sync::Lazy;

#[allow(dead_code)]
/// Different types of tokens that can be recognized in assembly code
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Main categories
    Instruction,    // Assembly instructions (mov, push, add, etc.)
    Register,       // CPU registers (rax, rbx, etc.)
    Immediate,      // Immediate values (numbers like 42, 0x1F)
    Label,          // Code labels (function:, loop_start:)
    LabelRef,       // References to labels (call function, jmp loop_start)
    Directive,      // Assembler directives (section, global, etc.)
    StringLiteral,  // String literals ("hello world")
    Comment,        // Comments (; this is a comment)
    Identifier,     // Unrecognized identifiers (let parser decide)
    // Specific register types for faster lookup
    Reg64Bit,       // 64-bit registers (rax, rbx, etc.)
    Reg32Bit,       // 32-bit registers (eax, ebx, etc.)
    Reg16Bit,       // 16-bit registers (ax, bx, etc.)
    Reg8Bit,        // 8-bit registers (al, ah, etc.)
    RegXMM,         // XMM registers (xmm0, xmm1, etc.) for SIMD
    RegYMM,         // YMM registers (ymm0, ymm1, etc.) for SIMD
    RegZMM,         // ZMM registers (zmm0, zmm1, etc.) for SIMD
    RegSpecial,     // Special registers (cr0, dr0, etc.)
    // Instruction categories for optimization
    InstrData,      // Data movement instructions (mov, push, etc.)
    InstrArith,     // Arithmetic instructions (add, sub, etc.)
    InstrLogic,     // Logical instructions (and, or, etc.)
    InstrJump,      // Jump instructions (jmp, je, etc.)
    InstrSIMD,      // SIMD instructions (movdqa, paddb, etc.)
    // Syntax elements
    Memory,         // Memory references ([rax], [rbx+rcx*4])
    Comma,          // Commas separating operands
    Colon,          // Colons for label definitions
    Plus,           // Plus sign for address calculations
    Minus,          // Minus sign for address calculations
    Asterisk,       // Multiplication in address calculations
    OpenBracket,    // Opening brackets for memory references
    CloseBracket,   // Closing brackets for memory references
    Whitespace,     // Spaces, tabs, etc.
    NewLine,        // Line breaks
    Unknown,        // Unrecognized tokens
    EOF,            // End of file
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Token struct representing a single token in the assembly code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    #[inline(always)]
    pub fn new(token_type: TokenType, value: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            value,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.token_type, self.value)
    }
}

// Static lookup tables for fast token recognition
static INSTRUCTIONS: Lazy<HashMap<&'static str, (&'static str, TokenType)>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(200); // Pre-allocate capacity for better performance
    
    // Data Movement Instructions
    map.insert("mov", ("48 B8", TokenType::InstrData));
    map.insert("movq", ("48 B8", TokenType::InstrData));
    map.insert("movb", ("88", TokenType::InstrData));
    map.insert("movw", ("66 89", TokenType::InstrData));
    map.insert("movl", ("89", TokenType::InstrData));
    map.insert("movabs", ("48 B8", TokenType::InstrData));
    map.insert("lea", ("48 8D", TokenType::InstrData));
    map.insert("push", ("50", TokenType::InstrData));
    map.insert("pushq", ("50", TokenType::InstrData));
    map.insert("pop", ("58", TokenType::InstrData));
    map.insert("popq", ("58", TokenType::InstrData));
    map.insert("xchg", ("87", TokenType::InstrData));
    map.insert("cmovz", ("48 0F 44", TokenType::InstrData));
    map.insert("cmove", ("48 0F 44", TokenType::InstrData));
    map.insert("cmovne", ("48 0F 45", TokenType::InstrData));
    
    // Arithmetic Instructions
    map.insert("add", ("48 83 C0", TokenType::InstrArith));
    map.insert("addq", ("48 83 C0", TokenType::InstrArith));
    map.insert("sub", ("48 83 E8", TokenType::InstrArith));
    map.insert("subq", ("48 83 E8", TokenType::InstrArith));
    map.insert("mul", ("48 F7 E0", TokenType::InstrArith));
    map.insert("imul", ("48 F7 E8", TokenType::InstrArith));
    map.insert("div", ("48 F7 F0", TokenType::InstrArith));
    map.insert("idiv", ("48 F7 F8", TokenType::InstrArith));
    map.insert("inc", ("48 FF C0", TokenType::InstrArith));
    map.insert("dec", ("48 FF C8", TokenType::InstrArith));
    map.insert("neg", ("48 F7 D8", TokenType::InstrArith));
    
    // Logical Instructions
    map.insert("and", ("48 83 E0", TokenType::InstrLogic));
    map.insert("or", ("48 83 C8", TokenType::InstrLogic));
    map.insert("xor", ("48 83 F0", TokenType::InstrLogic));
    map.insert("not", ("48 F7 D0", TokenType::InstrLogic));
    map.insert("shl", ("48 C1 E0", TokenType::InstrLogic));
    map.insert("shr", ("48 C1 E8", TokenType::InstrLogic));
    map.insert("sal", ("48 C1 E0", TokenType::InstrLogic));
    map.insert("sar", ("48 C1 F8", TokenType::InstrLogic));
    map.insert("rol", ("48 C1 C0", TokenType::InstrLogic));
    map.insert("ror", ("48 C1 C8", TokenType::InstrLogic));
    map.insert("test", ("48 85", TokenType::InstrLogic));
    map.insert("cmp", ("48 39", TokenType::InstrLogic));
    
    // Control Flow Instructions
    map.insert("jmp", ("E9", TokenType::InstrJump));
    map.insert("je", ("74", TokenType::InstrJump));
    map.insert("jz", ("74", TokenType::InstrJump));
    map.insert("jne", ("75", TokenType::InstrJump));
    map.insert("jnz", ("75", TokenType::InstrJump));
    map.insert("jg", ("7F", TokenType::InstrJump));
    map.insert("jge", ("7D", TokenType::InstrJump));
    map.insert("jl", ("7C", TokenType::InstrJump));
    map.insert("jle", ("7E", TokenType::InstrJump));
    map.insert("ja", ("77", TokenType::InstrJump));
    map.insert("jae", ("73", TokenType::InstrJump));
    map.insert("jb", ("72", TokenType::InstrJump));
    map.insert("jbe", ("76", TokenType::InstrJump));
    map.insert("call", ("E8", TokenType::InstrJump));
    map.insert("ret", ("C3", TokenType::InstrJump));
    map.insert("syscall", ("0F 05", TokenType::InstrJump));
    
    // SIMD Instructions
    map.insert("movdqa", ("66 0F 6F", TokenType::InstrSIMD));
    map.insert("movdqu", ("F3 0F 6F", TokenType::InstrSIMD));
    map.insert("movaps", ("0F 28", TokenType::InstrSIMD));
    map.insert("movups", ("0F 10", TokenType::InstrSIMD));
    map.insert("movss", ("F3 0F 10", TokenType::InstrSIMD));
    map.insert("movsd", ("F2 0F 10", TokenType::InstrSIMD));
    map.insert("paddb", ("66 0F FC", TokenType::InstrSIMD));
    map.insert("paddw", ("66 0F FD", TokenType::InstrSIMD));
    map.insert("paddd", ("66 0F FE", TokenType::InstrSIMD));
    map.insert("paddq", ("66 0F D4", TokenType::InstrSIMD));
    map.insert("psubb", ("66 0F F8", TokenType::InstrSIMD));
    map.insert("psubw", ("66 0F F9", TokenType::InstrSIMD));
    map.insert("psubd", ("66 0F FA", TokenType::InstrSIMD));
    map.insert("psubq", ("66 0F FB", TokenType::InstrSIMD));
    map.insert("pand", ("66 0F DB", TokenType::InstrSIMD));
    map.insert("por", ("66 0F EB", TokenType::InstrSIMD));
    map.insert("pxor", ("66 0F EF", TokenType::InstrSIMD));
    
    // AVX Instructions
    map.insert("vmovdqa", ("C5 F9 6F", TokenType::InstrSIMD));
    map.insert("vmovdqu", ("C5 FA 6F", TokenType::InstrSIMD));
    map.insert("vmovaps", ("C5 F8 28", TokenType::InstrSIMD));
    map.insert("vmovups", ("C5 F8 10", TokenType::InstrSIMD));
    map.insert("vpaddb", ("C5 F9 FC", TokenType::InstrSIMD));
    map.insert("vpaddw", ("C5 F9 FD", TokenType::InstrSIMD));
    map.insert("vpaddd", ("C5 F9 FE", TokenType::InstrSIMD));
    map.insert("vpaddq", ("C5 F9 D4", TokenType::InstrSIMD));
    
    map
});

static REGISTERS: Lazy<HashMap<String, TokenType>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(100);
    
    // 64-bit registers
    map.insert("rax".to_string(), TokenType::Reg64Bit);
    map.insert("rbx".to_string(), TokenType::Reg64Bit);
    map.insert("rcx".to_string(), TokenType::Reg64Bit);
    map.insert("rdx".to_string(), TokenType::Reg64Bit);
    map.insert("rsi".to_string(), TokenType::Reg64Bit);
    map.insert("rdi".to_string(), TokenType::Reg64Bit);
    map.insert("rbp".to_string(), TokenType::Reg64Bit);
    map.insert("rsp".to_string(), TokenType::Reg64Bit);
    map.insert("r8".to_string(), TokenType::Reg64Bit);
    map.insert("r9".to_string(), TokenType::Reg64Bit);
    map.insert("r10".to_string(), TokenType::Reg64Bit);
    map.insert("r11".to_string(), TokenType::Reg64Bit);
    map.insert("r12".to_string(), TokenType::Reg64Bit);
    map.insert("r13".to_string(), TokenType::Reg64Bit);
    map.insert("r14".to_string(), TokenType::Reg64Bit);
    map.insert("r15".to_string(), TokenType::Reg64Bit);
    
    // 32-bit registers
    map.insert("eax".to_string(), TokenType::Reg32Bit);
    map.insert("ebx".to_string(), TokenType::Reg32Bit);
    map.insert("ecx".to_string(), TokenType::Reg32Bit);
    map.insert("edx".to_string(), TokenType::Reg32Bit);
    map.insert("esi".to_string(), TokenType::Reg32Bit);
    map.insert("edi".to_string(), TokenType::Reg32Bit);
    map.insert("ebp".to_string(), TokenType::Reg32Bit);
    map.insert("esp".to_string(), TokenType::Reg32Bit);
    map.insert("r8d".to_string(), TokenType::Reg32Bit);
    map.insert("r9d".to_string(), TokenType::Reg32Bit);
    map.insert("r10d".to_string(), TokenType::Reg32Bit);
    map.insert("r11d".to_string(), TokenType::Reg32Bit);
    map.insert("r12d".to_string(), TokenType::Reg32Bit);
    map.insert("r13d".to_string(), TokenType::Reg32Bit);
    map.insert("r14d".to_string(), TokenType::Reg32Bit);
    map.insert("r15d".to_string(), TokenType::Reg32Bit);
    
    // 16-bit registers
    map.insert("ax".to_string(), TokenType::Reg16Bit);
    map.insert("bx".to_string(), TokenType::Reg16Bit);
    map.insert("cx".to_string(), TokenType::Reg16Bit);
    map.insert("dx".to_string(), TokenType::Reg16Bit);
    map.insert("si".to_string(), TokenType::Reg16Bit);
    map.insert("di".to_string(), TokenType::Reg16Bit);
    map.insert("bp".to_string(), TokenType::Reg16Bit);
    map.insert("sp".to_string(), TokenType::Reg16Bit);
    map.insert("r8w".to_string(), TokenType::Reg16Bit);
    map.insert("r9w".to_string(), TokenType::Reg16Bit);
    map.insert("r10w".to_string(), TokenType::Reg16Bit);
    map.insert("r11w".to_string(), TokenType::Reg16Bit);
    map.insert("r12w".to_string(), TokenType::Reg16Bit);
    map.insert("r13w".to_string(), TokenType::Reg16Bit);
    map.insert("r14w".to_string(), TokenType::Reg16Bit);
    map.insert("r15w".to_string(), TokenType::Reg16Bit);
    
    // 8-bit registers
    map.insert("al".to_string(), TokenType::Reg8Bit);
    map.insert("bl".to_string(), TokenType::Reg8Bit);
    map.insert("cl".to_string(), TokenType::Reg8Bit);
    map.insert("dl".to_string(), TokenType::Reg8Bit);
    map.insert("ah".to_string(), TokenType::Reg8Bit);
    map.insert("bh".to_string(), TokenType::Reg8Bit);
    map.insert("ch".to_string(), TokenType::Reg8Bit);
    map.insert("dh".to_string(), TokenType::Reg8Bit);
    map.insert("sil".to_string(), TokenType::Reg8Bit);
    map.insert("dil".to_string(), TokenType::Reg8Bit);
    map.insert("bpl".to_string(), TokenType::Reg8Bit);
    map.insert("spl".to_string(), TokenType::Reg8Bit);
    map.insert("r8b".to_string(), TokenType::Reg8Bit);
    map.insert("r9b".to_string(), TokenType::Reg8Bit);
    map.insert("r10b".to_string(), TokenType::Reg8Bit);
    map.insert("r11b".to_string(), TokenType::Reg8Bit);
    map.insert("r12b".to_string(), TokenType::Reg8Bit);
    map.insert("r13b".to_string(), TokenType::Reg8Bit);
    map.insert("r14b".to_string(), TokenType::Reg8Bit);
    map.insert("r15b".to_string(), TokenType::Reg8Bit);
    
    // SIMD registers
    for i in 0..32 {
        map.insert(format!("xmm{}", i), TokenType::RegXMM);
        map.insert(format!("ymm{}", i), TokenType::RegYMM);
        map.insert(format!("zmm{}", i), TokenType::RegZMM);
    }
    
    // Special registers
    map.insert("rip".to_string(), TokenType::RegSpecial);
    map.insert("rflags".to_string(), TokenType::RegSpecial);
    map.insert("eflags".to_string(), TokenType::RegSpecial);
    map.insert("flags".to_string(), TokenType::RegSpecial);
    
    map
});

static DIRECTIVES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("section", "section");
    map.insert("segment", "segment");
    map.insert("global", "global");
    map.insert("extern", "extern");
    map.insert("db", "db");
    map.insert("dw", "dw");
    map.insert("dd", "dd");
    map.insert("dq", "dq");
    map.insert("dt", "dt");
    map.insert("equ", "equ");
    map.insert("times", "times");
    map.insert("align", "align");
    map.insert("default", "default");
    map.insert("rel", "rel");
    map.insert("abs", "abs");
    map.insert("org", "org");
    map.insert("bits", "bits");
    map.insert("use16", "use16");
    map.insert("use32", "use32");
    map.insert("use64", "use64");
    map
});

/// Fast tokenizer for x86_64 assembly code
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    // Adding a cache to improve performance for repeated lookups
    instruction_cache: HashMap<String, Option<TokenType>>,
    register_cache: HashMap<String, Option<TokenType>>,
}

impl Tokenizer {
    /// Create a new tokenizer for the given input string
    #[inline(always)]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::with_capacity(input.len() / 4), // Estimate token count
            instruction_cache: HashMap::new(),
            register_cache: HashMap::new(),
        }
    }

    /// Check if we've reached the end of the input
    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Get the current character under the cursor
    #[inline(always)]
    fn current_char(&self) -> Option<char> {
        if self.is_eof() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    /// Peek at the next character without advancing
    #[inline(always)]
    fn peek_char(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    /// Advance to the next character
    #[inline(always)]
    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    /// Skip whitespace characters
    #[inline(always)]
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Tokenize alphanumeric identifiers (instructions, registers, labels, etc.)
    #[inline]
    fn tokenize_identifier(&mut self, is_equ: bool) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        
        // Collect all alphanumeric chars and underscores
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' || (is_equ && (ch == '$' || ch == '-')) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Fast path: Check caches first
        if let Some(cached_type) = self.instruction_cache.get(&value) {
            if let Some(token_type) = cached_type {
                return Token::new(token_type.clone(), value, self.line, start_column);
            }
        }
        
        if let Some(cached_reg_type) = self.register_cache.get(&value) {
            if let Some(reg_type) = cached_reg_type {
                return Token::new(reg_type.clone(), value, self.line, start_column);
            }
        }
        
        // Determine token type based on the value
        let token_type = if let Some(&(_, ref instr_type)) = INSTRUCTIONS.get(value.as_str()) {
            // Cache this lookup for future use
            self.instruction_cache.insert(value.clone(), Some(instr_type.clone()));
            instr_type.clone()
        } else if let Some(reg_type) = REGISTERS.get(&value) {
            // Cache this lookup for future use
            self.register_cache.insert(value.clone(), Some(reg_type.clone()));
            reg_type.clone()
        } else if let Some(_) = DIRECTIVES.get(value.as_str()) {
            TokenType::Directive
        } else if self.current_char() == Some(':') {
            // This is a label definition (will consume the colon later)
            TokenType::Label
        } else if value.starts_with('.') {
            // Section names and other dotted identifiers are treated as label references
            TokenType::LabelRef
        } else {
            // Cache negative lookups too
            self.instruction_cache.insert(value.clone(), None);
            self.register_cache.insert(value.clone(), None);
            // This could be a label ref, var name, etc. Let parser decide.
            TokenType::Identifier
        };

        Token::new(token_type, value, self.line, start_column)
    }

    /// Tokenize numeric literals (immediate values)
    #[inline]
    fn tokenize_number(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        let mut is_hex = false;
        let mut is_binary = false;
        
        // Check for hex or binary prefix
        if self.current_char() == Some('0') {
            value.push('0');
            self.advance();
            
            if self.current_char() == Some('x') || self.current_char() == Some('X') {
                value.push(self.current_char().unwrap());
                self.advance();
                is_hex = true;
            } else if self.current_char() == Some('b') || self.current_char() == Some('B') {
                value.push(self.current_char().unwrap());
                self.advance();
                is_binary = true;
            }
        }
        
        // Collect all digits and hex/binary chars
        while let Some(ch) = self.current_char() {
            if ch.is_digit(10) || 
               (is_hex && (ch.is_digit(16) || ('a'..='f').contains(&ch) || ('A'..='F').contains(&ch))) ||
               (is_binary && (ch == '0' || ch == '1')) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Token::new(TokenType::Immediate, value, self.line, start_column)
    }

    /// Tokenize string literals (enclosed in quotes)
    #[inline]
    fn tokenize_string(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        let start_line = self.line;
        
        // Skip the opening quote
        self.advance();
        
        // Collect everything until the closing quote, handling escapes
        let mut is_escaped = false;
        let mut found_closing_quote = false;
        
        while let Some(ch) = self.current_char() {
            if is_escaped {
                // Handle escaped character
                match ch {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    '0' => value.push('\0'),
                    _ => value.push(ch),
                }
                is_escaped = false;
                self.advance();
            } else if ch == '\\' {
                is_escaped = true;
                self.advance();
            } else if ch == '"' {
                self.advance(); // Skip the closing quote
                found_closing_quote = true;
                break;
            } else if ch == '\n' {
                // We've hit a newline without closing the string
                break;
            } else {
                value.push(ch);
                self.advance();
            }
        }

        // Check if we found the closing quote
        if !found_closing_quote {
            // Create a token, but also indicate the error
            let token = Token::new(TokenType::StringLiteral, value, start_line, start_column);
            // Note: Since the tokenizer doesn't have a reference to the error collector,
            // we'll need to detect this issue in the parser
            return token;
        }

        Token::new(TokenType::StringLiteral, value, start_line, start_column)
    }

    /// Tokenize comments (starting with ; or #)
    #[inline]
    fn tokenize_comment(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        
        // Skip the comment marker (;)
        self.advance();
        
        // Collect everything until the end of the line
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Token::new(TokenType::Comment, value.trim().to_string(), self.line, start_column)
    }

    /// Tokenize string literals enclosed in single quotes
    #[inline]
    fn tokenize_single_quoted_string(&mut self) -> Token {
        let start_column = self.column;
        let mut value = String::new();
        let start_line = self.line;
        
        // Skip the opening quote
        self.advance();
        
        // Collect everything until the closing quote, handling escapes
        let mut is_escaped = false;
        let mut found_closing_quote = false;
        
        while let Some(ch) = self.current_char() {
            if is_escaped {
                // Handle escaped character
                match ch {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '\'' => value.push('\''),
                    '"' => value.push('"'),
                    '0' => value.push('\0'),
                    _ => value.push(ch),
                }
                is_escaped = false;
                self.advance();
            } else if ch == '\\' {
                is_escaped = true;
                self.advance();
            } else if ch == '\'' {
                self.advance(); // Skip the closing quote
                found_closing_quote = true;
                break;
            } else if ch == '\n' {
                // We've hit a newline without closing the string
                break;
            } else {
                value.push(ch);
                self.advance();
            }
        }

        // Check if we found the closing quote
        if !found_closing_quote {
            // Create a token, but also indicate the error
            let token = Token::new(TokenType::StringLiteral, value, start_line, start_column);
            // Note: Since the tokenizer doesn't have a reference to the error collector,
            // we'll need to detect this issue in the parser
            return token;
        }

        Token::new(TokenType::StringLiteral, value, start_line, start_column)
    }

    /// Tokenize the entire input
    #[inline]
    pub fn tokenize(&mut self) -> &Vec<Token> {
        while !self.is_eof() {
            match self.current_char() {
                Some(ch) if ch.is_whitespace() && ch != '\n' => {
                    self.skip_whitespace();
                },
                Some('\n') => {
                    self.tokens.push(Token::new(
                        TokenType::NewLine, 
                        "\n".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some(ch) if ch.is_alphabetic() || ch == '_' || ch == '.' => {
                    let token = self.tokenize_identifier(false);
                    self.tokens.push(token);
                },
                Some(ch) if ch.is_digit(10) => {
                    let token = self.tokenize_number();
                    self.tokens.push(token);
                },
                Some(';') => {
                    let token = self.tokenize_comment();
                    self.tokens.push(token);
                },
                Some('"') => {
                    let token = self.tokenize_string();
                    self.tokens.push(token);
                },
                Some('\'') => {
                    let token = self.tokenize_single_quoted_string();
                    self.tokens.push(token);
                },
                Some(',') => {
                    self.tokens.push(Token::new(
                        TokenType::Comma, 
                        ",".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some(':') => {
                    self.tokens.push(Token::new(
                        TokenType::Colon, 
                        ":".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some('+') => {
                    self.tokens.push(Token::new(
                        TokenType::Plus, 
                        "+".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some('-') => {
                    self.tokens.push(Token::new(
                        TokenType::Minus, 
                        "-".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some('*') => {
                    self.tokens.push(Token::new(
                        TokenType::Asterisk, 
                        "*".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some('[') => {
                    self.tokens.push(Token::new(
                        TokenType::OpenBracket, 
                        "[".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some(']') => {
                    self.tokens.push(Token::new(
                        TokenType::CloseBracket, 
                        "]".to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                Some(ch) => {
                    // Unknown token
                    self.tokens.push(Token::new(
                        TokenType::Unknown, 
                        ch.to_string(), 
                        self.line, 
                        self.column
                    ));
                    self.advance();
                },
                None => break,
            }
        }
        
        // Add EOF token
        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            self.line,
            self.column
        ));
        
        &self.tokens
    }

    /// Tokenize an expression for the equ directive
    pub fn tokenize_equ_expression(&mut self, input: &str) -> Vec<Token> {
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = Vec::new();
        
        while !tokenizer.is_eof() {
            match tokenizer.current_char() {
                Some(ch) if ch.is_whitespace() && ch != '\n' => {
                    tokenizer.skip_whitespace();
                },
                Some(ch) if ch.is_alphabetic() || ch == '_' || ch == '.' => {
                    let token = tokenizer.tokenize_identifier(true);
                    tokens.push(token);
                },
                Some(ch) if ch.is_digit(10) => {
                    let token = tokenizer.tokenize_number();
                    tokens.push(token);
                },
                Some('+') => {
                    tokens.push(Token::new(
                        TokenType::Plus, 
                        "+".to_string(), 
                        tokenizer.line, 
                        tokenizer.column
                    ));
                    tokenizer.advance();
                },
                Some('-') => {
                    tokens.push(Token::new(
                        TokenType::Minus, 
                        "-".to_string(), 
                        tokenizer.line, 
                        tokenizer.column
                    ));
                    tokenizer.advance();
                },
                Some('*') => {
                    tokens.push(Token::new(
                        TokenType::Asterisk, 
                        "*".to_string(), 
                        tokenizer.line, 
                        tokenizer.column
                    ));
                    tokenizer.advance();
                },
                Some(ch) => {
                    // Unknown token
                    tokens.push(Token::new(
                        TokenType::Unknown, 
                        ch.to_string(), 
                        tokenizer.line, 
                        tokenizer.column
                    ));
                    tokenizer.advance();
                },
                None => break,
            }
        }
        
        tokens
    }
}

// Function to format tokens for pretty printing
pub fn format_tokens(tokens: &[Token]) -> String {
    let mut result = String::new();
    
    let mut line_num = 1;
    result.push_str(&format!("Line {:4} | ", line_num));
    
    for token in tokens {
        if token.token_type == TokenType::NewLine {
            line_num += 1;
            result.push_str("\n");
            result.push_str(&format!("Line {:4} | ", line_num));
        } else if token.token_type != TokenType::EOF {
            result.push_str(&format!("{} ", token));
        }
    }
    
    result
}
