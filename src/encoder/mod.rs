use crate::parser::ast::{Instruction, Operand, MemoryReference};
use colored::*;

pub struct MachineCodeEncoder;

impl MachineCodeEncoder {
    pub fn new() -> Self {
        MachineCodeEncoder
    }
    
    pub fn encode(&self, instruction: &Instruction) -> Vec<u8> {
        match instruction.name.as_str() {
            "mov" => self.encode_mov(instruction),
            "lea" => self.encode_lea(instruction),
            "xor" => self.encode_xor(instruction),
            "syscall" => self.encode_syscall(),
            _ => {
                Vec::new()
            }
        }
    }
    
    fn encode_mov(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new();
        }
        match (&instruction.operands[0], &instruction.operands[1]) {
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rax" => {
                let imm = parse_immediate(src).unwrap_or(0);
                let mut code = vec![0x48, 0xB8];
                code.extend_from_slice(&imm.to_le_bytes());
                code
            },
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rdi" => {
                let imm = parse_immediate(src).unwrap_or(0);
                let mut code = vec![0x48, 0xBF];
                code.extend_from_slice(&imm.to_le_bytes());
                code
            },
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rdx" => {
                let imm = parse_immediate(src).unwrap_or(0);
                let mut code = vec![0x48, 0xBA];
                code.extend_from_slice(&imm.to_le_bytes());
                code
            },
            (Operand::Register(dst), Operand::Immediate(src)) if dst == "rsi" => {
                let imm = parse_immediate(src).unwrap_or(0);
                let mut code = vec![0x48, 0xBE];
                code.extend_from_slice(&imm.to_le_bytes());
                code
            },
            (Operand::Register(dst), Operand::Memory(_)) if dst == "rsi" => {
                vec![0x48, 0x8B, 0x35, 0, 0, 0, 0]
            },
            _ => {
                Vec::new()
            }
        }
    }
    
    fn encode_lea(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new();
        }
        match (&instruction.operands[0], &instruction.operands[1]) {
            (Operand::Register(dst), Operand::Label(label)) if dst == "rsi" => {
                vec![0x48, 0x8D, 0x35, 0, 0, 0, 0]
            },
            _ => {
                Vec::new()
            }
        }
    }
    
    fn encode_xor(&self, instruction: &Instruction) -> Vec<u8> {
        if instruction.operands.len() != 2 {
            return Vec::new();
        }
        match (&instruction.operands[0], &instruction.operands[1]) {
            (Operand::Register(dst), Operand::Register(src)) if dst == "rax" && src == "rax" => {
                vec![0x48, 0x31, 0xC0]
            },
            (Operand::Register(dst), Operand::Register(src)) if dst == "rdi" && src == "rdi" => {
                vec![0x48, 0x31, 0xFF]
            },
            (Operand::Register(dst), Operand::Register(src)) if dst == "rsi" && src == "rsi" => {
                vec![0x48, 0x31, 0xF6]
            },
            (Operand::Register(dst), Operand::Register(src)) if dst == "rdx" && src == "rdx" => {
                vec![0x48, 0x31, 0xD2]
            },
            _ => {
                Vec::new()
            }
        }
    }
    
    fn encode_syscall(&self) -> Vec<u8> {
        vec![0x0F, 0x05]
    }
}

fn parse_immediate(value: &str) -> Option<u64> {
    if value.starts_with("0x") || value.starts_with("0X") {
        u64::from_str_radix(&value[2..], 16).ok()
    } else if value.starts_with("0b") || value.starts_with("0B") {
        u64::from_str_radix(&value[2..], 2).ok()
    } else if value.starts_with("0o") || value.starts_with("0O") {
        u64::from_str_radix(&value[2..], 8).ok()
    } else {
        value.parse::<u64>().ok()
    }
}
