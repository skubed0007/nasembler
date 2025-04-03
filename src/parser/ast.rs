use std::fmt;
use std::collections::HashMap;

/// Abstract Syntax Tree (AST) structures for the assembly parser
#[derive(Debug, Clone)]
pub enum Statement {
    Instruction(Instruction),
    Directive(Directive),
    Label(String),
    Comment(String),
    Empty,
    Section(Section),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: String,
    pub operands: Vec<Operand>,
    pub machine_code: Vec<u8>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub operands: Vec<Operand>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(String),
    Immediate(String),
    Memory(MemoryReference),
    Label(String),
    String(String),
}

#[derive(Debug, Clone)]
pub struct MemoryReference {
    pub base: Option<String>,
    pub index: Option<String>,
    pub scale: Option<u8>,
    pub displacement: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub sections: HashMap<String, SectionInfo>,
    pub labels: HashMap<String, LabelInfo>,
}

#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub size: usize,
    pub statements: Vec<usize>, // Indices into the statements vec
}

#[derive(Debug, Clone)]
pub struct LabelInfo {
    pub offset: u64,
    pub section: Option<String>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statements: Vec::new(),
            sections: HashMap::new(),
            labels: HashMap::new(),
        }
    }
    
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
    
    pub fn add_section(&mut self, name: String, size: usize) {
        self.sections.insert(name, SectionInfo {
            size,
            statements: Vec::new(),
        });
    }
    
    pub fn add_label(&mut self, name: String, offset: u64, section: Option<String>) {
        self.labels.insert(name, LabelInfo {
            offset,
            section,
        });
    }
}

/// Represents a section in the assembly
#[derive(Debug, Clone)]
pub struct Section {
    /// Name of the section (e.g., ".text", ".data")
    pub name: String,
    /// Line number where the section appears
    pub line: usize,
}

/// Represents a label in the assembly
#[derive(Debug, Clone)]
pub struct Label {
    /// Name of the label
    pub name: String,
    /// Line number where the label appears
    pub line: usize,
}

/// Represents a data value for db, dw, dd, etc. directives
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    /// A numeric value
    Number(String),
    /// A string literal
    String(String),
    /// A character literal
    Char(char),
    /// A label reference
    Label(String),
}

// Implement Display for better error messages and debugging
impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg),
            Operand::Immediate(imm) => write!(f, "{}", imm),
            Operand::Memory(mem) => {
                write!(f, "[")?;
                
                if let Some(base) = &mem.base {
                    write!(f, "{}", base)?;
                }
                
                if let Some(index) = &mem.index {
                    if mem.base.is_some() {
                        write!(f, "+")?;
                    }
                    
                    write!(f, "{}", index)?;
                    
                    if let Some(scale) = mem.scale {
                        write!(f, "*{}", scale)?;
                    }
                }
                
                if let Some(disp) = &mem.displacement {
                    if mem.base.is_some() || mem.index.is_some() {
                        if disp.starts_with('-') {
                            write!(f, "{}", disp)?;
                        } else {
                            write!(f, "+{}", disp)?;
                        }
                    } else {
                        write!(f, "{}", disp)?;
                    }
                }
                
                write!(f, "]")
            },
            Operand::String(str) => write!(f, "\"{}\"", str),
            Operand::Label(label) => write!(f, "{}", label),
        }
    }
} 