use std::fmt;
use std::collections::HashMap;
use std::path::Path;
use colored::*;

/// Error type for the assembler
#[derive(Debug, Clone)]
pub enum ErrorType {
    // Tokenization errors
    UnexpectedCharacter,
    InvalidToken,
    UnclosedString,
    
    // Parsing errors
    UnexpectedToken,
    ExpectedToken,
    UnknownDirective,
    UnknownInstruction,
    InvalidOperand,
    InvalidMemoryReference,
    
    // Label errors
    UndefinedLabel,
    DuplicateLabel,
    MalformedLabel,
    
    // Code generation errors
    EncodingError,
    InvalidAddressing,
    InvalidCombination,
    
    // ELF generation errors
    SectionError,
    ElfWriteError,
    
    // I/O errors
    FileError,
    
    // Other errors
    SyntaxError,
    SemanticError,
    InternalError,
    Other
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            // Tokenization errors - bright magenta for tokenization issues
            ErrorType::UnexpectedCharacter => "Char Error".magenta().bold(),
            ErrorType::InvalidToken => "Token Error".magenta().bold(),
            ErrorType::UnclosedString => "String Error".magenta().bold(),
            
            // Parsing errors - bright red for parsing issues
            ErrorType::UnexpectedToken => "Unexpected Tok".bright_red().bold(),
            ErrorType::ExpectedToken => "Missing Tok".bright_red().bold(),
            ErrorType::UnknownDirective => "Unknown Dir".bright_red().bold(),
            ErrorType::UnknownInstruction => "Unknown Instr".bright_red().bold(),
            ErrorType::InvalidOperand => "Bad Operand".bright_red().bold(),
            ErrorType::InvalidMemoryReference => "Bad MemRef".bright_red().bold(),
            
            // Label errors - bright yellow for label issues
            ErrorType::UndefinedLabel => "Undef Label".bright_yellow().bold(),
            ErrorType::DuplicateLabel => "Dup Label".bright_yellow().bold(),
            ErrorType::MalformedLabel => "Bad Label".bright_yellow().bold(),
            
            // Code generation errors - bright cyan for encoding issues
            ErrorType::EncodingError => "Encode Err".bright_cyan().bold(),
            ErrorType::InvalidAddressing => "Bad Addr".bright_cyan().bold(),
            ErrorType::InvalidCombination => "Bad Combo".bright_cyan().bold(),
            
            // ELF generation errors - bright blue for ELF issues
            ErrorType::SectionError => "Sect Err".bright_blue().bold(),
            ErrorType::ElfWriteError => "ELF Err".bright_blue().bold(),
            
            // I/O errors - bright green for file issues
            ErrorType::FileError => "File Err".bright_green().bold(),
            
            // Other errors - default styles
            ErrorType::SyntaxError => "Syntax Err".red().bold(),
            ErrorType::SemanticError => "Semantic Err".yellow().bold(),
            ErrorType::InternalError => "Internal Err".bright_red().bold().on_white(),
            ErrorType::Other => "Other Err".white().bold(),
        };
        
        write!(f, "{}", s)
    }
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub line_content: Option<String>,
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            line_content: None,
        }
    }
    
    pub fn with_line_content(mut self, content: String) -> Self {
        self.line_content = Some(content);
        self
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Error details
#[derive(Debug, Clone)]
pub struct ErrorDetail {
    pub message: String,
    pub help: Option<String>,
    pub note: Option<String>,
}

impl ErrorDetail {
    pub fn new(message: String) -> Self {
        Self {
            message,
            help: None,
            note: None,
        }
    }
    
    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }
    
    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }
}

/// Assembler error
#[derive(Debug, Clone)]
pub struct Error {
    pub error_type: ErrorType,
    pub location: Option<SourceLocation>,
    pub detail: ErrorDetail,
    pub sub_errors: Vec<Error>,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Fatal,
    Error,
    Warning,
    Info,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorSeverity::Fatal => "fatal error".bright_red().bold().on_black(),
            ErrorSeverity::Error => "error".bright_red().bold(),
            ErrorSeverity::Warning => "warning".bright_yellow().bold(),
            ErrorSeverity::Info => "info".bright_blue().bold(),
        };
        write!(f, "{}", s)
    }
}

impl Error {
    pub fn new(error_type: ErrorType, detail: ErrorDetail) -> Self {
        Self {
            error_type,
            location: None,
            detail,
            sub_errors: Vec::new(),
            severity: ErrorSeverity::Error,
        }
    }
    
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
    
    pub fn with_sub_error(mut self, error: Error) -> Self {
        self.sub_errors.push(error);
        self
    }
    
    pub fn add_sub_error(&mut self, error: Error) {
        self.sub_errors.push(error);
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    /// Generate a colorized tree-like display of the error
    pub fn display(&self) -> String {
        let mut output = String::new();
        
        // Error header with error type
        let error_type_str = format!("{}", self.error_type);
        let header = format!("{} [{}]: {}", 
            self.severity, 
            error_type_str,
            self.detail.message.white().bold()
        );
        output.push_str(&header);
        output.push('\n');
        
        // Location with prettier formatting
        if let Some(ref location) = self.location {
            let location_str = format!("  {} {}", 
                "→".bright_cyan().bold(), 
                location.to_string().bright_blue().underline()
            );
            output.push_str(&location_str);
            output.push('\n');
            
            // Line content if available - with syntax highlighting for assembly
            if let Some(ref line_content) = location.line_content {
                // Basic syntax highlighting
                let highlighted_line = highlight_assembly_line(line_content);
                output.push_str(&format!("    {}\n", highlighted_line));
                
                // Enhanced pointer to the column
                let mut pointer = String::new();
                for _ in 0..location.column {
                    pointer.push(' ');
                }
                
                // Use a caret with color for better visibility
                pointer.push_str(&"^".bright_red().bold().to_string());
                
                // Add a wavy underline for affected text if we can determine it
                if let Some(affected_length) = get_affected_token_length(line_content, location.column) {
                    for _ in 0..affected_length.saturating_sub(1) {
                        pointer.push_str(&"~".bright_red().bold().to_string());
                    }
                }
                
                output.push_str(&format!("    {}\n", pointer));
            }
        }
        
        // Help message with nicer formatting
        if let Some(ref help) = self.detail.help {
            output.push_str(&format!("  {} {}\n", 
                "💡".to_string() + &" help:".green().bold().to_string(), 
                help.bright_green()
            ));
        }
        
        // Note with nicer formatting
        if let Some(ref note) = self.detail.note {
            output.push_str(&format!("  {} {}\n", 
                "ℹ️".to_string() + &" note:".bright_cyan().bold().to_string(), 
                note.cyan()
            ));
        }
        
        // Sub-errors with improved tree formatting
        if !self.sub_errors.is_empty() {
            output.push_str(&format!("  {} {}\n", 
                "⤷".bright_blue().bold(), 
                "caused by:".bright_blue().underline()
            ));
            
            for (i, error) in self.sub_errors.iter().enumerate() {
                let is_last = i == self.sub_errors.len() - 1;
                let prefix = if is_last { "  └─ " } else { "  ├─ " };
                
                // First line of sub-error
                let sub_error_first_line = format!("{}{}: {}", 
                    prefix.bright_blue(), 
                    error.severity, 
                    error.detail.message.white().bold()
                );
                output.push_str(&sub_error_first_line);
                output.push('\n');
                
                // Location for sub-error
                if let Some(ref location) = error.location {
                    let location_prefix = if is_last { "     " } else { "  │  " };
                    let location_str = format!("{}{} {}", 
                        location_prefix.bright_blue(), 
                        "→".bright_cyan().bold(), 
                        location.to_string().bright_blue().underline()
                    );
                    output.push_str(&location_str);
                    output.push('\n');
                    
                    // Line content if available - with highlighting
                    if let Some(ref line_content) = location.line_content {
                        let content_prefix = if is_last { "       " } else { "  │    " };
                        let highlighted_line = highlight_assembly_line(line_content);
                        output.push_str(&format!("{}{}\n", content_prefix.bright_blue(), highlighted_line));
                        
                        // Enhanced pointer with wavy underline
                        let mut pointer = String::new();
                        for _ in 0..location.column {
                            pointer.push(' ');
                        }
                        
                        pointer.push_str(&"^".bright_red().bold().to_string());
                        
                        // Add wavy underline for affected text
                        if let Some(affected_length) = get_affected_token_length(line_content, location.column) {
                            for _ in 0..affected_length.saturating_sub(1) {
                                pointer.push_str(&"~".bright_red().bold().to_string());
                            }
                        }
                        
                        let pointer_prefix = if is_last { "       " } else { "  │    " };
                        output.push_str(&format!("{}{}\n", pointer_prefix.bright_blue(), pointer));
                    }
                }
                
                // Help message for sub-error
                if let Some(ref help) = error.detail.help {
                    let help_prefix = if is_last { "     " } else { "  │  " };
                    output.push_str(&format!("{}{} {}\n", 
                        help_prefix.bright_blue(), 
                        "💡 help:".green().bold(), 
                        help.bright_green()
                    ));
                }
                
                // Note for sub-error
                if let Some(ref note) = error.detail.note {
                    let note_prefix = if is_last { "     " } else { "  │  " };
                    output.push_str(&format!("{}{} {}\n", 
                        note_prefix.bright_blue(), 
                        "ℹ️ note:".bright_cyan().bold(), 
                        note.cyan()
                    ));
                }
                
                // Nested sub-errors are not shown directly
                if !error.sub_errors.is_empty() {
                    let nested_prefix = if is_last { "     " } else { "  │  " };
                    output.push_str(&format!("{}{} {} more nested errors not shown\n", 
                        nested_prefix.bright_blue(), 
                        "⚠️ note:".bright_cyan().bold(), 
                        error.sub_errors.len().to_string().yellow().bold()
                    ));
                }
            }
        }
        
        output
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Error collector for accumulating multiple errors
#[derive(Debug, Default, Clone)]
pub struct ErrorCollector {
    errors: Vec<Error>,
    file_contents: HashMap<String, Vec<String>>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            file_contents: HashMap::new(),
        }
    }
    
    /// Add an error to the collector
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// Add a simple error with just a message
    pub fn add_simple_error(&mut self, error_type: ErrorType, message: &str) {
        let error = Error::new(
            error_type,
            ErrorDetail::new(message.to_string())
        );
        self.add_error(error);
    }

    /// Add an error with location
    pub fn add_error_with_location(&mut self, 
        error_type: ErrorType, 
        message: &str,
        file: &str,
        line: usize,
        column: usize
    ) {
        // Load file content if needed
        if !self.file_contents.contains_key(file) {
            if let Ok(content) = std::fs::read_to_string(file) {
                let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                self.file_contents.insert(file.to_string(), lines);
            }
        }

        // Get line content if available
        let line_content = if let Some(lines) = self.file_contents.get(file) {
            if line > 0 && line <= lines.len() {
                Some(lines[line - 1].clone())
            } else {
                None
            }
        } else {
            None
        };

        // Create location
        let mut location = SourceLocation::new(file.to_string(), line, column);
        if let Some(content) = line_content {
            location = location.with_line_content(content);
        }

        // Create and add error
        let error = Error::new(
            error_type,
            ErrorDetail::new(message.to_string())
        ).with_location(location);

        self.add_error(error);
    }
    
    /// Check if there are any errors (not including warnings)
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == ErrorSeverity::Error || e.severity == ErrorSeverity::Fatal)
    }
    
    /// Check if there are any errors of fatal severity
    pub fn has_fatal_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == ErrorSeverity::Fatal)
    }
    
    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.iter().filter(|e| e.severity == ErrorSeverity::Error || e.severity == ErrorSeverity::Fatal).count()
    }
    
    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.errors.iter().filter(|e| e.severity == ErrorSeverity::Warning).count()
    }
    
    /// Display all errors in a beautifully formatted output
    pub fn display_errors(&self) -> String {
        if self.errors.is_empty() {
            return "✓ ".green().bold().to_string() + &"No errors or warnings.".green().to_string();
        }
        
        let mut output = String::new();
        
        // Sort errors by severity, then by file, then by line
        let mut sorted_errors = self.errors.clone();
        sorted_errors.sort_by(|a, b| {
            let a_severity = a.severity;
            let b_severity = b.severity;
            
            if a_severity != b_severity {
                return a_severity.cmp(&b_severity);
            }
            
            // Compare file paths, defaulting to empty string for missing locations
            let a_file = match &a.location {
                Some(loc) => &loc.file,
                None => "",
            };
            
            let b_file = match &b.location {
                Some(loc) => &loc.file,
                None => "",
            };
            
            if a_file != b_file {
                return a_file.cmp(b_file);
            }
            
            let a_line = a.location.as_ref().map(|l| l.line).unwrap_or(0);
            let b_line = b.location.as_ref().map(|l| l.line).unwrap_or(0);
            
            a_line.cmp(&b_line)
        });
        
        // Group errors by file
        let mut current_file = String::new();
        let mut error_counter = 1;
        
        for error in sorted_errors {
            let file = if let Some(ref location) = error.location {
                let path = Path::new(&location.file);
                path.file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| location.file.clone())
            } else {
                "unknown".to_string()
            };
            
            // Add file header if we're switching to a new file
            if file != current_file {
                if !current_file.is_empty() {
                    output.push('\n');
                }
                
                output.push_str(&format!("{}{}:{}\n", 
                    "■ ".bright_blue(),
                    "FILE".bright_white(),
                    file.bright_white().bold()
                ));
                current_file = file;
            }
            
            // Add error number prefix with colored symbol
            let (severity_symbol, severity_color) = match error.severity {
                ErrorSeverity::Fatal => ("✕", "red"),
                ErrorSeverity::Error => ("×", "red"),
                ErrorSeverity::Warning => ("!", "yellow"),
                ErrorSeverity::Info => ("i", "blue"),
            };
            
            let error_number = format!("{}{}", 
                severity_symbol.color(severity_color).bold(),
                format!("{:02}", error_counter).bright_white()
            );
            error_counter += 1;
            
            // Compact error header with error type and message
            let location_info = if let Some(ref location) = error.location {
                format!("{}:{}", location.line, location.column).bright_blue().bold().to_string()
            } else {
                "".to_string()
            };
            
            // Make the error header more compact - inline all the error info
            let mut error_header = format!("{} {} {} ", 
                error_number,
                location_info,
                error.error_type
            );
            
            // Truncate message if it's too long for better display
            let message = error.detail.message.clone();
            
            error_header.push_str(&message.white().to_string());
            
            output.push_str(&format!("{}\n", error_header));
            
            // Add code snippet in a more compact way if available
            if let Some(ref location) = error.location {
                if let Some(ref line_content) = location.line_content {
                    // Highlighted code with pointer on the same line
                    let highlighted_line = highlight_assembly_line(line_content);
                    
                    // Create pointer
                    let mut pointer = String::new();
                    for _ in 0..location.column {
                        pointer.push(' ');
                    }
                    
                    pointer.push_str(&"^".bright_red().bold().to_string());
                    
                    if let Some(affected_length) = get_affected_token_length(line_content, location.column) {
                        for _ in 0..affected_length.saturating_sub(1) {
                            pointer.push_str(&"~".bright_red().bold().to_string());
                        }
                    }
                    
                    // More compact code snippet display
                    output.push_str(&format!("  {}│ {}\n", " ".white(), highlighted_line));
                    output.push_str(&format!("  {}└→ {}\n", " ".white(), pointer));
                }
            }
            
            // Add help and note in a compact inline format
            let mut hints = String::new();
            
            if let Some(ref help) = error.detail.help {
                hints.push_str(&format!("→ {}", help.bright_green()));
            }
            
            if let Some(ref note) = error.detail.note {
                if !hints.is_empty() {
                    hints.push_str(" ");
                }
                hints.push_str(&format!("ⓘ {}", note.cyan()));
            }
            
            if !hints.is_empty() {
                output.push_str(&format!("  {}\n", hints));
            }
            
            // Add a minimal separator between errors
            output.push_str(&format!("  {}\n", "―".repeat(25).bright_blue()));
        }
        
        // Add summary with enhanced styling
        let error_count = self.error_count();
        let warning_count = self.warning_count();
        
        let mut summary = String::new();
        
        if error_count > 0 {
            summary.push_str(&format!("{} {} {}", 
                "×".bright_red().bold(), 
                error_count.to_string().bright_red().bold(), 
                if error_count == 1 { "err" } else { "errs" }
            ));
        }
        
        if warning_count > 0 {
            if !summary.is_empty() {
                summary.push_str(" + ");
            }
            
            summary.push_str(&format!("{} {} {}", 
                "!".bright_yellow().bold(), 
                warning_count.to_string().bright_yellow().bold(), 
                if warning_count == 1 { "warn" } else { "warns" }
            ));
        }
        
        if summary.is_empty() {
            summary.push_str(&format!("{} {}", "✓".green().bold(), "No issues"));
        }
        
        output.push_str(&format!("{}\n{}\n", "═".repeat(30).bright_blue(), summary));
        
        output
    }
    
    /// Return a new collector with the same settings but no errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

// Helper functions to create common errors

// Token error
pub fn token_error(message: String, file: String, line: usize, column: usize, token: &str) -> Error {
    let error_detail = ErrorDetail::new(message)
        .with_help(format!("Check the syntax near '{}'", token));
    
    let location = SourceLocation::new(file, line, column);
    
    Error::new(ErrorType::InvalidToken, error_detail)
        .with_location(location)
}

// Parse error
pub fn parse_error(message: String, file: String, line: usize, column: usize, line_content: Option<String>) -> Error {
    let error_detail = ErrorDetail::new(message);
    
    let mut location = SourceLocation::new(file, line, column);
    
    if let Some(content) = line_content {
        location = location.with_line_content(content);
    }
    
    Error::new(ErrorType::SyntaxError, error_detail)
        .with_location(location)
}

// Label error
pub fn label_error(message: String, label: &str) -> Error {
    let error_detail = ErrorDetail::new(message)
        .with_help(format!("Check the declaration and usage of label '{}'", label));
    
    Error::new(ErrorType::UndefinedLabel, error_detail)
}

// Encoding error
pub fn encoding_error(message: String, instruction: &str) -> Error {
    let error_detail = ErrorDetail::new(message)
        .with_help(format!("Check the instruction '{}' and its operands", instruction));
    
    Error::new(ErrorType::EncodingError, error_detail)
}

// File error
pub fn file_error(message: String, path: &str) -> Error {
    let error_detail = ErrorDetail::new(message)
        .with_help(format!("Check if the file '{}' exists and is accessible", path));
    
    Error::new(ErrorType::FileError, error_detail)
}

// Internal error
pub fn internal_error(message: String) -> Error {
    let error_detail = ErrorDetail::new(message)
        .with_note("This is an internal error and should be reported".to_string());
    
    Error::new(ErrorType::InternalError, error_detail)
        .with_severity(ErrorSeverity::Fatal)
}

/// Custom Result type that uses our Error type
pub type Result<T> = std::result::Result<T, Error>;

// Helper function to highlight assembly syntax
fn highlight_assembly_line(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return line.to_string();
    }
    
    let mut result = String::new();
    let trimmed = line.trim_start();
    
    // Add original indentation
    let indent_len = line.len() - trimmed.len();
    if indent_len > 0 {
        result.push_str(&line[0..indent_len]);
    }
    
    // Check for label (ends with :)
    if parts[0].ends_with(':') {
        // Label
        result.push_str(&parts[0].bright_green().bold().to_string());
        if parts.len() > 1 {
            result.push(' ');
            let remainder = trimmed[parts[0].len()..].trim_start();
            result.push_str(&highlight_assembly_remainder(remainder));
        }
        return result;
    }
    
    // Check for instruction or directive
    if parts[0].starts_with('.') {
        // Directive
        result.push_str(&parts[0].bright_cyan().bold().to_string());
        if parts.len() > 1 {
            result.push(' ');
            let remainder = trimmed[parts[0].len()..].trim_start();
            result.push_str(&highlight_assembly_remainder(remainder));
        }
    } else {
        // Instruction
        result.push_str(&parts[0].bright_yellow().bold().to_string());
        if parts.len() > 1 {
            result.push(' ');
            let remainder = trimmed[parts[0].len()..].trim_start();
            result.push_str(&highlight_assembly_remainder(remainder));
        }
    }
    
    result
}

// Helper function to highlight the remainder of an assembly line
fn highlight_assembly_remainder(remainder: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_comment = false;
    let mut current_token = String::new();
    
    for c in remainder.chars() {
        if in_comment {
            // Everything after ; is a comment
            result.push_str(&c.to_string().bright_black().to_string());
            continue;
        }
        
        if c == '"' {
            if in_string {
                // End of string
                current_token.push(c);
                result.push_str(&current_token.green().to_string());
                current_token.clear();
                in_string = false;
            } else {
                // Start of string
                if !current_token.is_empty() {
                    result.push_str(&highlight_assembly_token(&current_token));
                    current_token.clear();
                }
                current_token.push(c);
                in_string = true;
            }
        } else if in_string {
            // Inside string
            current_token.push(c);
        } else if c == ';' {
            // Start of comment
            if !current_token.is_empty() {
                result.push_str(&highlight_assembly_token(&current_token));
                current_token.clear();
            }
            result.push_str(&c.to_string().bright_black().to_string());
            in_comment = true;
        } else if c.is_whitespace() {
            // Whitespace
            if !current_token.is_empty() {
                result.push_str(&highlight_assembly_token(&current_token));
                current_token.clear();
            }
            result.push(c);
        } else if c == ',' || c == '[' || c == ']' || c == '+' || c == '-' || c == '*' {
            // Special chars
            if !current_token.is_empty() {
                result.push_str(&highlight_assembly_token(&current_token));
                current_token.clear();
            }
            result.push_str(&c.to_string().bright_magenta().to_string());
        } else {
            // Part of a token
            current_token.push(c);
        }
    }
    
    // Don't forget any remaining token
    if !current_token.is_empty() {
        result.push_str(&highlight_assembly_token(&current_token));
    }
    
    result
}

// Helper function to highlight a token based on its content
fn highlight_assembly_token(token: &str) -> String {
    if token.starts_with('r') || token == "rax" || token == "rbx" || token == "rcx" || token == "rdx" || 
       token == "rsi" || token == "rdi" || token == "rbp" || token == "rsp" || 
       token.starts_with("xmm") || token.starts_with("ymm") || token.starts_with("zmm") {
        // Register
        token.bright_blue().to_string()
    } else if token.starts_with("0x") || token.chars().all(|c| c.is_digit(10)) {
        // Numeric literal
        token.bright_cyan().to_string()
    } else {
        // Default - likely a label reference or other identifier
        token.white().to_string()
    }
}

// Helper function to guess the length of the token at the given column
fn get_affected_token_length(line: &str, column: usize) -> Option<usize> {
    if column >= line.len() {
        return None;
    }
    
    let chars: Vec<char> = line.chars().collect();
    let mut end = column;
    
    // Find end of token
    while end < chars.len() && !chars[end].is_whitespace() && chars[end] != ',' && chars[end] != ';' {
        end += 1;
    }
    
    Some(end - column)
}
