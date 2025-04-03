use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

/// Information about an opcode
#[derive(Debug, Clone)]
pub struct OpcodeInfo {
    pub name: String,
    pub category: String,
    pub operands: Vec<String>,
    pub machine_code: Option<String>,
    pub encoding: Option<String>,
}

/// Table of opcodes loaded from OPCODES.txt
#[derive(Debug, Clone)]
pub struct OpcodeTable {
    opcodes: Vec<OpcodeInfo>,
    opcode_map: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionCategory {
    General,
    SystemCall,
    JumpCall,
    DataMovement,
    Arithmetic,
    Logic,
    Shift,
    String,
    IO,
    Other,
}

impl OpcodeTable {
    /// Create a new, empty opcode table
    pub fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            opcode_map: HashMap::new(),
        }
    }
    
    /// Load opcodes from a file
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|e| format!("Failed to open opcode file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut content = String::new();
        reader.buffer().read_to_string(&mut content)
            .map_err(|e| format!("Failed to read opcode file: {}", e))?;
        
        Self::from_string(&content)
    }
    
    /// Parse and load opcodes from a string
    pub fn from_string(content: &str) -> Result<Self, String> {
        let mut table = Self::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 3 {
                continue;
            }
            
            let name = parts[0].trim().to_lowercase();
            let category = parts[1].trim().to_string();
            let operands = parts[2].trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
                
            let machine_code = if parts.len() > 3 && !parts[3].trim().is_empty() {
                Some(parts[3].trim().to_string())
            } else {
                None
            };
            
            let encoding = if parts.len() > 4 && !parts[4].trim().is_empty() {
                Some(parts[4].trim().to_string())
            } else {
                None
            };
            
            let opcode_info = OpcodeInfo {
                name: name.clone(),
                category,
                operands,
                machine_code,
                encoding,
            };
            
            let index = table.opcodes.len();
            table.opcodes.push(opcode_info);
            
            table.opcode_map.entry(name)
                .or_insert_with(Vec::new)
                .push(index);
        }
        
        Ok(table)
    }
    
    /// Lookup opcodes for an instruction
    pub fn lookup(&self, name: &str) -> Option<&[usize]> {
        self.opcode_map.get(name).map(|v| v.as_slice())
    }
    
    /// Get information about an instruction
    pub fn get_info(&self, name: &str) -> Option<&OpcodeInfo> {
        self.lookup(name)
            .and_then(|indices| indices.first())
            .map(|&index| &self.opcodes[index])
    }
    
    /// Get all instructions in a category
    pub fn get_category(&self, category: &InstructionCategory) -> Option<Vec<String>> {
        let category_str = match category {
            InstructionCategory::General => "general",
            InstructionCategory::SystemCall => "syscall",
            InstructionCategory::JumpCall => "jumpcall",
            InstructionCategory::DataMovement => "datamov",
            InstructionCategory::Arithmetic => "arith",
            InstructionCategory::Logic => "logic",
            InstructionCategory::Shift => "shift",
            InstructionCategory::String => "string",
            InstructionCategory::IO => "io",
            InstructionCategory::Other => "other",
        };
        
        let opcodes = self.opcodes.iter()
            .filter(|o| o.category.eq_ignore_ascii_case(category_str))
            .map(|o| o.name.clone())
            .collect::<Vec<_>>();
            
        if opcodes.is_empty() {
            None
        } else {
            Some(opcodes)
        }
    }
} 