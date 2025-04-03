use crate::parser::ast::{Instruction, Operand, MemoryReference};

pub struct MachineCodeEncoder;

impl MachineCodeEncoder {
    pub fn new() -> Self {
        MachineCodeEncoder
    }
    
    /// Encode an instruction into its machine code representation
    pub fn encode(&self, instruction: &Instruction) -> Vec<u8> {
        match instruction.name.as_str() {
            "mov" => self.encode_mov(instruction),
            "lea" => self.encode_lea(instruction),
            "xor" => self.encode_xor(instruction),
            "syscall" => self.encode_syscall(),
            _ => Vec::new(), // Default to empty machine code for unsupported instructions
        }
    }
    
    /// Encode mov instruction
    fn encode_mov(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new(); // Invalid number of operands
        }
        
        match (&instruction.operands[0], &instruction.operands[1]) {
            // mov rax, 1 -> B8 01 00 00 00
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rax" => {
                let immediate = parse_immediate(src).unwrap_or(0);
                let mut machine_code = vec![0x48, 0xB8]; // REX.W + mov rax, imm64
                machine_code.extend_from_slice(&immediate.to_le_bytes());
                machine_code
            },
            // mov rdi, 1 -> BF 01 00 00 00
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rdi" => {
                let immediate = parse_immediate(src).unwrap_or(0);
                let mut machine_code = vec![0x48, 0xBF]; // REX.W + mov rdi, imm64
                machine_code.extend_from_slice(&immediate.to_le_bytes());
                machine_code
            },
            // mov rdx, immediate -> 48 BA XX XX XX XX XX XX XX XX
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rdx" => {
                let immediate = parse_immediate(src).unwrap_or(0);
                let mut machine_code = vec![0x48, 0xBA]; // REX.W + mov rdx, imm64
                machine_code.extend_from_slice(&immediate.to_le_bytes());
                machine_code
            },
            // mov rsi, immediate -> 48 BE XX XX XX XX XX XX XX XX
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rsi" => {
                let immediate = parse_immediate(src).unwrap_or(0);
                let mut machine_code = vec![0x48, 0xBE]; // REX.W + mov rsi, imm64
                machine_code.extend_from_slice(&immediate.to_le_bytes());
                machine_code
            },
            // mov rsi, [mem] -> 48 8B 35 XX XX XX XX
            (Operand::Register(dst), Operand::Memory(_)) if dst == "rsi" => {
                // This is a placeholder - actual address resolution would be needed
                vec![0x48, 0x8B, 0x35, 0x00, 0x00, 0x00, 0x00]
            },
            _ => Vec::new(), // Unsupported mov variant
        }
    }
    
    /// Encode lea instruction
    fn encode_lea(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new(); // Invalid number of operands
        }
        
        match (&instruction.operands[0], &instruction.operands[1]) {
            // lea rsi, [msg] -> 48 8D 35 XX XX XX XX
            (Operand::Register(dst), Operand::Memory(_)) if dst == "rsi" => {
                // This is a simplified encoding - actual address resolution would be needed
                vec![0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]
            },
            (Operand::Register(dst), Operand::Label(_)) if dst == "rsi" => {
                // LEA RIP-relative addressing for labels
                vec![0x48, 0x8D, 0x35, 0x00, 0x00, 0x00, 0x00]
            },
            _ => Vec::new(), // Unsupported lea variant
        }
    }
    
    /// Encode xor instruction
    fn encode_xor(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new(); // Invalid number of operands
        }
        
        match (&instruction.operands[0], &instruction.operands[1]) {
            // xor rax, rax -> 48 31 C0
            (Operand::Register(dst), Operand::Register(src)) if dst == "rax" && src == "rax" => {
                vec![0x48, 0x31, 0xC0]
            },
            // xor rdi, rdi -> 48 31 FF
            (Operand::Register(dst), Operand::Register(src)) if dst == "rdi" && src == "rdi" => {
                vec![0x48, 0x31, 0xFF]
            },
            // xor rsi, rsi -> 48 31 F6
            (Operand::Register(dst), Operand::Register(src)) if dst == "rsi" && src == "rsi" => {
                vec![0x48, 0x31, 0xF6]
            },
            // xor rdx, rdx -> 48 31 D2
            (Operand::Register(dst), Operand::Register(src)) if dst == "rdx" && src == "rdx" => {
                vec![0x48, 0x31, 0xD2]
            },
            _ => Vec::new(), // Unsupported xor variant
        }
    }
    
    /// Encode syscall instruction
    fn encode_syscall(&self) -> Vec<u8> {
        vec![0x0F, 0x05] // syscall is 0F 05
    }
}

/// Parse an immediate value from a string
fn parse_immediate(value: &str) -> Option<u64> {
    // Handle hexadecimal values
    if value.starts_with("0x") || value.starts_with("0X") {
        u64::from_str_radix(&value[2..], 16).ok()
    } 
    // Handle binary values
    else if value.starts_with("0b") || value.starts_with("0B") {
        u64::from_str_radix(&value[2..], 2).ok()
    } 
    // Handle octal values
    else if value.starts_with("0o") || value.starts_with("0O") {
        u64::from_str_radix(&value[2..], 8).ok()
    }
    // Handle decimal values
    else {
        value.parse::<u64>().ok()
    }
} 