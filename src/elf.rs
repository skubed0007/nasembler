use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::mem;
use colored::*;

use crate::parser::ast::{Program, Statement, Instruction, Directive, Operand};
use crate::encoder::MachineCodeEncoder;

const EI_NIDENT: usize = 16;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const EV_CURRENT: u8 = 1;
const PT_LOAD: u32 = 1;
const PF_R: u32 = 4;
const PF_W: u32 = 2;
const PF_X: u32 = 1;
const PAGE_SIZE: u64 = 0x1000;

#[repr(C, packed)]
struct Elf64Header {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C, packed)]
struct Elf64ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

fn round_up(value: u64, align: u64) -> u64 {
    if value % align == 0 { value } else { value + align - (value % align) }
}

pub struct ElfGenerator {
    text_address: u64,
    data_address: u64,
    entry_point: u64,
    program: Program,
    labels: HashMap<String, u64>,
    text_section: Vec<u8>,
    data_section: Vec<u8>,
    encoder: MachineCodeEncoder,
}

impl ElfGenerator {
    pub fn new(program: Program) -> Self {
        let gen = Self {
            text_address: 0x400000,
            data_address: 0x600000,
            entry_point: 0,
            program,
            labels: HashMap::new(),
            text_section: Vec::new(),
            data_section: Vec::new(),
            encoder: MachineCodeEncoder::new(),
        };
        println!("{}", "■ Initialized ELF generator".green());
        gen
    }

    pub fn generate(&mut self, output_path: &str) -> Result<(), String> {
        println!("{}", "■ Processing AST...".green());
        self.process_ast()?;
        println!("{}", "■ AST processed".green());
        let elf_header_size = mem::size_of::<Elf64Header>() as u64;
        let ph_size = mem::size_of::<Elf64ProgramHeader>() as u64 * 2;
        let headers_size = elf_header_size + ph_size;
        let text_offset = (headers_size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let text_filesz = self.text_section.len() as u64;
        let text_memsz = round_up(text_filesz, PAGE_SIZE);
        let data_offset = text_offset + text_memsz;
        let data_filesz = self.data_section.len() as u64;
        let data_memsz = round_up(data_filesz, PAGE_SIZE);
        println!("{}", format!("■ .text: offset=0x{:X} size={} bytes", text_offset, text_filesz).blue());
        println!("{}", format!("■ .data: offset=0x{:X} size={} bytes", data_offset, data_filesz).blue());
        let elf_header = self.create_elf_header();
        let text_header = Elf64ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: text_offset,
            p_vaddr: self.text_address,
            p_paddr: self.text_address,
            p_filesz: text_filesz,
            p_memsz: text_memsz,
            p_align: PAGE_SIZE,
        };
        let data_header = Elf64ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: data_offset,
            p_vaddr: self.data_address,
            p_paddr: self.data_address,
            p_filesz: data_filesz,
            p_memsz: data_memsz,
            p_align: PAGE_SIZE,
        };
        let mut file = File::create(output_path)
            .map_err(|e| format!("× Failed to create output file: {}", e))?;
        println!("{}", "■ Writing ELF header...".green());
        file.write_all(unsafe {
            std::slice::from_raw_parts(&elf_header as *const Elf64Header as *const u8, mem::size_of::<Elf64Header>())
        }).map_err(|e| format!("× Error writing ELF header: {}", e))?;
        file.write_all(unsafe {
            std::slice::from_raw_parts(&text_header as *const Elf64ProgramHeader as *const u8, mem::size_of::<Elf64ProgramHeader>())
        }).map_err(|e| format!("× Error writing .text header: {}", e))?;
        file.write_all(unsafe {
            std::slice::from_raw_parts(&data_header as *const Elf64ProgramHeader as *const u8, mem::size_of::<Elf64ProgramHeader>())
        }).map_err(|e| format!("× Error writing .data header: {}", e))?;
        let current_pos = file.seek(SeekFrom::Current(0)).map_err(|e| e.to_string())?;
        let pad_size = text_offset.checked_sub(current_pos).ok_or("× Negative padding for .text")?;
        file.write_all(&vec![0u8; pad_size as usize]).map_err(|e| e.to_string())?;
        file.write_all(&self.text_section).map_err(|e| e.to_string())?;
        let text_pad = text_memsz.checked_sub(text_filesz).ok_or("× Negative .text padding")?;
        if text_pad > 0 { file.write_all(&vec![0u8; text_pad as usize]).map_err(|e| e.to_string())?; }
        let current_pos = file.seek(SeekFrom::Current(0)).map_err(|e| e.to_string())?;
        let pad_size = data_offset.checked_sub(current_pos).ok_or("× Negative padding for .data")?;
        file.write_all(&vec![0u8; pad_size as usize]).map_err(|e| e.to_string())?;
        file.write_all(&self.data_section).map_err(|e| e.to_string())?;
        let data_pad = data_memsz.checked_sub(data_filesz).ok_or("× Negative .data padding")?;
        if data_pad > 0 { file.write_all(&vec![0u8; data_pad as usize]).map_err(|e| e.to_string())?; }
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(output_path).map_err(|e| e.to_string())?;
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(output_path, perms).map_err(|e| e.to_string())?;
        }
        println!("{}", format!("■ ELF file generated at '{}'", output_path).green());
        Ok(())
    }

    fn process_ast(&mut self) -> Result<(), String> {
        let statements = self.program.statements.clone();
        let mut current_section = ".text".to_string();
        for (idx, stmt) in statements.iter().enumerate() {
            match stmt {
                Statement::Section(sec) => {
                    current_section = sec.name.clone();
                    println!("{}", format!("■ Switched to section '{}'", current_section).cyan());
                }
                Statement::Label(label) => {
                    if current_section == ".text" {
                        let addr = self.text_address + self.text_section.len() as u64;
                        self.labels.insert(label.clone(), addr);
                        if label == "_start" { self.entry_point = addr; }
                    } else if current_section == ".data" {
                        let addr = self.data_address + self.data_section.len() as u64;
                        self.labels.insert(label.clone(), addr);
                    }
                }
                Statement::Directive(dir) => {
                    if dir.name == "global" || dir.name == "extern" {
                        if let Operand::Label(sym) = &dir.operands[0] {
                            if dir.name == "global" { self.labels.insert(sym.clone(), 0); }
                        } else { return Err("■ Directive operand must be a label".to_string()); }
                    } else if dir.name == "equ" {
                        if let Operand::Immediate(val) = &dir.operands[0] {
                            let value = parse_number(val)?;
                            if idx > 0 {
                                if let Statement::Label(prev) = &statements[idx - 1] {
                                    self.labels.insert(prev.clone(), value);
                                }
                            }
                        }
                    } else if dir.name == "db" || dir.name == "dw" || dir.name == "dd" || dir.name == "dq" {
                        if current_section != ".data" { return Err("■ Data directives must be in .data section".to_string()); }
                        if idx > 0 { if let Statement::Label(prev) = &statements[idx - 1] {
                            let addr = self.data_address + self.data_section.len() as u64;
                            self.labels.insert(prev.clone(), addr);
                        } }
                        self.process_data_directive(&dir.name, &dir.operands)?;
                    }
                }
                Statement::Instruction(instr) => {
                    if current_section != ".text" { return Err("■ Instructions must be in .text section".to_string()); }
                    let code = self.encoder.encode(instr);
                    self.text_section.extend_from_slice(&code);
                }
                Statement::Comment(_) | Statement::Empty => {}
            }
        }
        self.patch_relocations()?;
        Ok(())
    }

    fn process_data_directive(&mut self, dir_name: &str, operands: &[Operand]) -> Result<(), String> {
        for op in operands {
            match op {
                Operand::Immediate(val) => {
                    let num = parse_number(val)?;
                    let bytes = match dir_name {
                        "db" => vec![num as u8],
                        "dw" => (num as u16).to_le_bytes().to_vec(),
                        "dd" => (num as u32).to_le_bytes().to_vec(),
                        "dq" => num.to_le_bytes().to_vec(),
                        _ => return Err(format!("■ Unknown data directive '{}'", dir_name)),
                    };
                    self.data_section.extend(bytes);
                }
                Operand::String(s) => {
                    self.data_section.extend(s.as_bytes());
                    self.data_section.push(0);
                }
                _ => return Err("■ Unsupported operand in data directive".to_string()),
            }
        }
        Ok(())
    }

    fn patch_relocations(&mut self) -> Result<(), String> {
        let mut lea_list = Vec::new();
        for (idx, stmt) in self.program.statements.iter().enumerate() {
            if let Statement::Instruction(instr) = stmt {
                if instr.name.to_lowercase() == "lea" && instr.operands.len() == 2 {
                    if let Operand::Label(label) = &instr.operands[1] {
                        let offset = self.instruction_offset(idx);
                        lea_list.push((offset, label.clone()));
                    }
                }
            }
        }
        for (offset, label) in lea_list {
            if let Some(&target_addr) = self.labels.get(&label) {
                let rip = self.text_address + offset as u64 + 7;
                let disp = (target_addr as i64 - rip as i64) as i32;
                let disp_bytes = disp.to_le_bytes();
                if offset + 7 <= self.text_section.len() {
                    self.text_section[offset + 3 .. offset + 7].copy_from_slice(&disp_bytes);
                } else { return Err(format!("■ LEA patch offset out of bounds for label '{}'", label)); }
            } else { return Err(format!("■ Label '{}' not found for LEA patching", label)); }
        }
        Ok(())
    }

    fn instruction_offset(&self, idx: usize) -> usize {
        let mut offset = 0;
        for i in 0..idx {
            if let Some(Statement::Instruction(instr)) = self.program.statements.get(i) {
                offset += instr.machine_code.len();
            }
        }
        offset
    }

    fn create_elf_header(&self) -> Elf64Header {
        let mut e_ident = [0u8; EI_NIDENT];
        e_ident[0..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        e_ident[4] = 2;
        e_ident[5] = 1;
        e_ident[6] = 1;
        e_ident[7] = 0;
        e_ident[8] = 0;
        Elf64Header {
            e_ident,
            e_type: ET_EXEC,
            e_machine: EM_X86_64,
            e_version: EV_CURRENT as u32,
            e_entry: self.entry_point,
            e_phoff: mem::size_of::<Elf64Header>() as u64,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: mem::size_of::<Elf64Header>() as u16,
            e_phentsize: mem::size_of::<Elf64ProgramHeader>() as u16,
            e_phnum: 2,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        }
    }
}

fn parse_number(num: &str) -> Result<u64, String> {
    if num.starts_with("0x") || num.starts_with("0X") {
        u64::from_str_radix(&num[2..], 16).map_err(|e| format!("■ Invalid hex number '{}': {}", num, e))
    } else if num.starts_with("0b") || num.starts_with("0B") {
        u64::from_str_radix(&num[2..], 2).map_err(|e| format!("■ Invalid binary number '{}': {}", num, e))
    } else if num.starts_with('0') && num.len() > 1 {
        u64::from_str_radix(&num[1..], 8).map_err(|e| format!("■ Invalid octal number '{}': {}", num, e))
    } else {
        num.parse::<u64>().map_err(|e| format!("■ Invalid decimal number '{}': {}", num, e))
    }
}
