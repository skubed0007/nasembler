// ELF64 binary generator
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write, Seek, SeekFrom};
use std::mem;

use crate::parser::ast::{Program, Statement, Instruction, Directive, Operand};
use crate::encoder::MachineCodeEncoder;

// ELF Constants
const EI_NIDENT: usize = 16;
const ET_EXEC: u16 = 2;  // Executable file
const EM_X86_64: u16 = 62;  // AMD x86-64 architecture
const EV_CURRENT: u8 = 1;  // Current version

const PT_LOAD: u32 = 1;  // Loadable program segment
const PF_X: u32 = 1;  // Execute permission
const PF_W: u32 = 2;  // Write permission
const PF_R: u32 = 4;  // Read permission

const SHT_NULL: u32 = 0;  // Section header table entry unused
const SHT_PROGBITS: u32 = 1;  // Program data
const SHT_SYMTAB: u32 = 2;  // Symbol table
const SHT_STRTAB: u32 = 3;  // String table

const SHF_WRITE: u64 = 1;  // Writable
const SHF_ALLOC: u64 = 2;  // Occupies memory during execution
const SHF_EXECINSTR: u64 = 4;  // Executable

// ELF64 Header structure
#[repr(C, packed)]
struct Elf64Header {
    e_ident: [u8; EI_NIDENT],  // Magic number and other info
    e_type: u16,               // Object file type
    e_machine: u16,            // Architecture
    e_version: u32,            // Object file version
    e_entry: u64,              // Entry point virtual address
    e_phoff: u64,              // Program header table file offset
    e_shoff: u64,              // Section header table file offset
    e_flags: u32,              // Processor-specific flags
    e_ehsize: u16,             // ELF header size in bytes
    e_phentsize: u16,          // Program header table entry size
    e_phnum: u16,              // Program header table entry count
    e_shentsize: u16,          // Section header table entry size
    e_shnum: u16,              // Section header table entry count
    e_shstrndx: u16,           // Section header string table index
}

// ELF64 Program header structure
#[repr(C, packed)]
struct Elf64ProgramHeader {
    p_type: u32,               // Segment type
    p_flags: u32,              // Segment flags
    p_offset: u64,             // Segment file offset
    p_vaddr: u64,              // Segment virtual address
    p_paddr: u64,              // Segment physical address
    p_filesz: u64,             // Segment size in file
    p_memsz: u64,              // Segment size in memory
    p_align: u64,              // Segment alignment
}

// ELF64 Section header structure
#[repr(C, packed)]
struct Elf64SectionHeader {
    sh_name: u32,              // Section name (string table index)
    sh_type: u32,              // Section type
    sh_flags: u64,             // Section flags
    sh_addr: u64,              // Section virtual addr at execution
    sh_offset: u64,            // Section file offset
    sh_size: u64,              // Section size in bytes
    sh_link: u32,              // Link to another section
    sh_info: u32,              // Additional section information
    sh_addralign: u64,         // Section alignment
    sh_entsize: u64,           // Entry size if section holds table
}

// Section data structure
struct Section {
    name: String,
    name_idx: u32,           // Index into the section string table
    data: Vec<u8>,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

// Symbol structure
struct Symbol {
    name: String,
    name_idx: u32,           // Index into the string table
    value: u64,
    size: u64,
    info: u8,
    other: u8,
    shndx: u16,
}

// ELF Generator
pub struct ElfGenerator {
    sections: Vec<Section>,
    symbols: Vec<Symbol>,
    string_table: Vec<u8>,
    symbol_table: Vec<u8>,
    section_string_table: Vec<u8>,
    symbol_map: HashMap<String, u64>,
    current_section: String,
    text_address: u64,
    data_address: u64,
    bss_address: u64,
    entry_point: u64,
    program: Program,
    labels: HashMap<String, u64>,
    text_section: Vec<u8>,
    data_section: Vec<u8>,
    encoder: MachineCodeEncoder,
}

impl ElfGenerator {
    pub fn new(program: Program) -> Self {
        let mut generator = Self {
            sections: Vec::new(),
            symbols: Vec::new(),
            string_table: vec![0], // First byte is always 0 in string tables
            symbol_table: Vec::new(),
            section_string_table: vec![0], // First byte is always 0
            symbol_map: HashMap::new(),
            current_section: String::new(),
            text_address: 0x400000, // Default addresses for common sections
            data_address: 0x600000,
            bss_address: 0x800000,
            entry_point: 0,
            program,
            labels: HashMap::new(),
            text_section: Vec::new(),
            data_section: Vec::new(),
            encoder: MachineCodeEncoder::new(),
        };
        
        // Add standard sections
        generator.add_section(".text", SHT_PROGBITS, SHF_ALLOC | SHF_EXECINSTR, 16);
        generator.add_section(".data", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, 8);
        generator.add_section(".bss", SHT_PROGBITS, SHF_ALLOC | SHF_WRITE, 4);
        
        generator
    }
    
    fn add_section(&mut self, name: &str, sh_type: u32, sh_flags: u64, sh_addralign: u64) -> usize {
        let name_idx = self.add_to_section_string_table(name);
        
        let mut addr = 0;
        if name == ".text" {
            addr = self.text_address;
        } else if name == ".data" {
            addr = self.data_address;
        } else if name == ".bss" {
            addr = self.bss_address;
        }
        
        let section = Section {
            name: name.to_string(),
            name_idx,
            data: Vec::new(),
            sh_type,
            sh_flags,
            sh_addr: addr,
            sh_link: 0,
            sh_info: 0,
            sh_addralign,
            sh_entsize: 0,
        };
        
        self.sections.push(section);
        self.sections.len() - 1
    }
    
    fn add_to_section_string_table(&mut self, name: &str) -> u32 {
        let pos = self.section_string_table.len() as u32;
        self.section_string_table.extend_from_slice(name.as_bytes());
        self.section_string_table.push(0); // Null-terminate the string
        pos
    }
    
    fn add_to_string_table(&mut self, name: &str) -> u32 {
        let pos = self.string_table.len() as u32;
        self.string_table.extend_from_slice(name.as_bytes());
        self.string_table.push(0); // Null-terminate the string
        pos
    }
    
    fn get_section_index(&self, name: &str) -> Option<usize> {
        self.sections.iter().position(|s| s.name == name)
    }
    
    // Add a symbol to the symbol table
    fn add_symbol(&mut self, name: &str, value: u64, size: u64, info: u8, other: u8, shndx: u16) {
        let name_idx = self.add_to_string_table(name);
        
        let symbol = Symbol {
            name: name.to_string(),
            name_idx,
            value,
            size,
            info,
            other,
            shndx,
        };
        
        // Store mapping from symbol name to its value (address)
        self.symbol_map.insert(name.to_string(), value);
        
        self.symbols.push(symbol);
    }
    
    // Process the AST and generate machine code
    pub fn generate(&mut self, output_path: &str) -> Result<(), String> {
        // First pass: collect labels and generate machine code
        self.process_sections()?;
        
        // Create the ELF file
        let mut file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        // Calculate the layout
        let elf_header_size = mem::size_of::<Elf64Header>() as u64;
        let ph_size = mem::size_of::<Elf64ProgramHeader>() as u64 * 2;
        let headers_size = elf_header_size + ph_size;
        
        // Calculate actual offsets based on page alignment
        let text_offset = (headers_size + 0xFFF) & !0xFFF; // Align to 4KB
        let text_size_aligned = (self.text_section.len() as u64 + 0xFFF) & !0xFFF;
        let data_offset = text_offset + text_size_aligned;
        
        // Create headers with the calculated offsets
        let elf_header = self.create_elf_header();
        let text_header = Elf64ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: text_offset,
            p_vaddr: self.text_address,
            p_paddr: self.text_address,
            p_filesz: self.text_section.len() as u64,
            p_memsz: self.text_section.len() as u64,
            p_align: 0x1000,
        };
        
        let data_header = Elf64ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: data_offset,
            p_vaddr: self.data_address,
            p_paddr: self.data_address,
            p_filesz: self.data_section.len() as u64,
            p_memsz: self.data_section.len() as u64,
            p_align: 0x1000,
        };
        
        // Write the ELF header
        file.write_all(unsafe { 
            std::slice::from_raw_parts(
                &elf_header as *const Elf64Header as *const u8,
                std::mem::size_of::<Elf64Header>()
            )
        }).map_err(|e| format!("Failed to write ELF header: {}", e))?;
        
        // Write program headers
        file.write_all(unsafe {
            std::slice::from_raw_parts(
                &text_header as *const Elf64ProgramHeader as *const u8,
                std::mem::size_of::<Elf64ProgramHeader>()
            )
        }).map_err(|e| format!("Failed to write text segment header: {}", e))?;
        
        file.write_all(unsafe {
            std::slice::from_raw_parts(
                &data_header as *const Elf64ProgramHeader as *const u8,
                std::mem::size_of::<Elf64ProgramHeader>()
            )
        }).map_err(|e| format!("Failed to write data segment header: {}", e))?;
        
        // Add padding to reach text_offset
        let current_pos = file.seek(SeekFrom::Current(0))
            .map_err(|e| format!("Failed to get current file position: {}", e))?;
        let padding_size = text_offset - current_pos;
        let padding = vec![0u8; padding_size as usize];
        file.write_all(&padding)
            .map_err(|e| format!("Failed to write padding: {}", e))?;
        
        // Write text section
        file.write_all(&self.text_section)
            .map_err(|e| format!("Failed to write text section: {}", e))?;
        
        // Add padding to reach data_offset
        let current_pos = file.seek(SeekFrom::Current(0))
            .map_err(|e| format!("Failed to get current file position: {}", e))?;
        let padding_size = data_offset - current_pos;
        let padding = vec![0u8; padding_size as usize];
        file.write_all(&padding)
            .map_err(|e| format!("Failed to write padding: {}", e))?;
        
        // Write data section
        file.write_all(&self.data_section)
            .map_err(|e| format!("Failed to write data section: {}", e))?;
        
        // Ensure the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(output_path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?;
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o111); // Add executable bit
            fs::set_permissions(output_path, perms)
                .map_err(|e| format!("Failed to set file permissions: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Process sections and generate machine code
    fn process_sections(&mut self) -> Result<(), String> {
        let mut text_offset = 0;
        let mut data_offset = 0;
        let mut current_section = ".text".to_string(); // Default to text section
        
        // First, make a copy of the statements to avoid borrow issues
        let statements = self.program.statements.clone();
        
        // First, go through all statements and process
        for (idx, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Section(section) => {
                    // Update the current section
                    current_section = section.name.clone();
                },
                Statement::Directive(directive) => {
                    if directive.name == "global" || directive.name == "extern" {
                        // Handle global/extern symbols
                        if directive.operands.len() != 1 {
                            return Err(format!("{} directive requires exactly one operand", directive.name));
                        }
                        
                        match &directive.operands[0] {
                            Operand::Label(symbol_name) => {
                                if directive.name == "global" {
                                    self.add_symbol(symbol_name, 0, 0, 0x10, 0, 0); // GLOBAL FUNCTION SYMBOL
                                }
                            },
                            _ => return Err(format!("{} symbol name must be a label", directive.name)),
                        }
                    } else if directive.name == "equ" {
                        // Handle EQU directive - set a constant value
                        if directive.operands.len() != 1 {
                            return Err("equ directive requires exactly one operand".to_string());
                        }
                        
                        // For simplicity, we'll handle the basic case - more complex expressions would need an expr evaluator
                        match &directive.operands[0] {
                            Operand::Immediate(val) => {
                                // Parse the value and store in labels map
                                let value = parse_number(val)?;
                                // Find the index of the current statement
                                let index = statements.iter().position(|s| {
                                    if let Statement::Directive(d) = s {
                                        return d.line == directive.line;
                                    }
                                    false
                                });
                                
                                // Use the previous label as the target
                                if let Some(idx) = index {
                                    if idx > 0 {
                                        if let Statement::Label(label_name) = &statements[idx - 1] {
                                            self.labels.insert(label_name.clone(), value);
                                        }
                                    }
                                }
                            },
                            _ => return Err("equ directive operand must be a numeric value".to_string()),
                        }
                    } else if directive.name == "db" || directive.name == "dw" || directive.name == "dd" || directive.name == "dq" {
                        // Handle data directives
                        if current_section != ".data" {
                            return Err("Data directives should be in .data section".to_string());
                        }
                        
                        // Check if there's a label for this data
                        let label_name = if idx > 0 {
                            if let Statement::Label(name) = &statements[idx - 1] {
                                // Record the label's address in the data section
                                self.labels.insert(name.clone(), self.data_address + data_offset as u64);
                                Some(name.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        
                        // Process the data directive
                        self.process_data_directive(&directive.name, &directive.operands)?;
                        
                        // Update data offset
                        let size = self.calculate_directive_size(directive);
                        data_offset += size;
                        
                    }
                },
                Statement::Label(label) => {
                    // Record the label's address in the current section
                    if current_section == ".text" {
                        self.labels.insert(label.clone(), self.text_address + text_offset as u64);
                        
                        // Set the entry point if the label is "_start"
                        if label == "_start" {
                            self.entry_point = self.text_address + text_offset as u64;
                        }
                    } else if current_section == ".data" {
                        // Note: We record data labels when processing their associated directive
                        // This ensures the offset calculation takes into account the size of the data
                    }
                },
                Statement::Instruction(instruction) => {
                    if current_section != ".text" {
                        return Err("Instructions should be in .text section".to_string());
                    }
                    
                    // Use the encoder to generate machine code for the instruction
                    let machine_code = self.encoder.encode(instruction);
                    if !machine_code.is_empty() {
                        self.text_section.extend_from_slice(&machine_code);
                        text_offset += machine_code.len();
                    } else {
                        // For now, use NOPs as placeholders for unsupported instructions
                        // In a more complete implementation, you would handle all instruction types
                        self.text_section.extend_from_slice(&[0x90, 0x90, 0x90]); // NOP instructions
                        text_offset += 3;
                    }
                },
                _ => {}, // Ignore comments and empty statements
            }
        }
        
        // After processing all sections, patch any relocations (e.g., LEA instructions with label references)
        self.patch_relocations()?;
        
        Ok(())
    }
    
    /// Process a data directive
    fn process_data_directive(&mut self, directive_name: &str, operands: &[Operand]) -> Result<(), String> {
        // Add the data to the appropriate section
        for (idx, operand) in operands.iter().enumerate() {
            match operand {
                Operand::Immediate(imm) => {
                    let immediate = parse_number(imm)?;
                    let bytes = match directive_name {
                        "db" => vec![immediate as u8],
                        "dw" => (immediate as u16).to_le_bytes().to_vec(),
                        "dd" => (immediate as u32).to_le_bytes().to_vec(),
                        "dq" => immediate.to_le_bytes().to_vec(),
                        _ => return Err(format!("Unknown data directive: {}", directive_name)),
                    };
                    
                    self.data_section.extend(bytes);
                },
                Operand::String(s) => {
                    // For string operands, just store the raw bytes of the string
                    self.data_section.extend(s.as_bytes());
                    
                    // Add a null terminator for strings
                    self.data_section.push(0);
                },
                Operand::Label(label) => {
                    // For labels, we would need to add a relocation entry
                    // But for now, just error out as we don't support this yet
                    return Err(format!("Label operands not supported in data directives yet: {}", label));
                },
                _ => return Err(format!("Unsupported operand type in data directive: {:?}", operand)),
            }
        }
        
        Ok(())
    }
    
    /// Patch relocations for labels
    fn patch_relocations(&mut self) -> Result<(), String> {
        // Find all LEA instructions and their operands in our program
        let mut lea_instructions = Vec::new();
        
        for (idx, statement) in self.program.statements.iter().enumerate() {
            if let Statement::Instruction(instr) = statement {
                if instr.name.to_lowercase() == "lea" && instr.operands.len() == 2 {
                    // Check if the second operand is a label reference
                    if let Operand::Label(label) = &instr.operands[1] {
                        // We found a LEA instruction with a label reference
                        let offset = self.instruction_offset(idx);
                        lea_instructions.push((offset, label.clone()));
                    }
                }
            }
        }
        
        // Now patch the LEA instructions in the text section
        for (offset, label) in lea_instructions {
            // Find the LEA instruction in our text section
            let mut i = offset;
            while i < self.text_section.len() {
                // Look for the LEA opcode prefix (48 8D)
                if i + 6 < self.text_section.len() && self.text_section[i] == 0x48 && self.text_section[i+1] == 0x8D {
                    // This is a LEA instruction with a memory reference
                    // Check if it's a RIP-relative addressing
                    if (self.text_section[i+2] & 0xC7) == 0x05 { // mod 00, r/m 101 (RIP-relative)
                        
                        if let Some(target_address) = self.labels.get(&label) {
                            
                            // Calculate the RIP-relative offset
                            let rip_relative_offset = *target_address as i64 - (i as i64 + 7);
                            
                            // Update the displacement field with the calculated offset
                            let displacement_bytes = (rip_relative_offset as i32).to_le_bytes();
                            
                            self.text_section[i+3] = displacement_bytes[0];
                            self.text_section[i+4] = displacement_bytes[1];
                            self.text_section[i+5] = displacement_bytes[2];
                            self.text_section[i+6] = displacement_bytes[3];
                        } else {
                            return Err(format!("Label '{}' not found in labels map", label));
                        }
                        
                        break; // Found and patched the instruction, exit the inner loop
                    }
                }
                
                i += 1;
            }
        }
        
        Ok(())
    }
    
    /// Calculate the offset of an instruction in the text section
    fn instruction_offset(&self, statement_idx: usize) -> usize {
        let mut offset = 0;
        for i in 0..statement_idx {
            if let Statement::Instruction(instr) = &self.program.statements[i] {
                offset += instr.machine_code.len();
            }
        }
        
        offset
    }
    
    /// Calculate the size of a directive
    fn calculate_directive_size(&self, directive: &Directive) -> usize {
        let mut size = 0;
        
        for operand in &directive.operands {
            match operand {
                Operand::Immediate(_) => {
                    match directive.name.as_str() {
                        "db" => size += 1,
                        "dw" => size += 2,
                        "dd" => size += 4,
                        "dq" => size += 8,
                        _ => {} // Unsupported directive, handled elsewhere
                    }
                },
                Operand::String(string) => {
                    if directive.name == "db" {
                        // Add the size of the string plus null terminator if needed
                        size += string.len();
                        // Note: We always add a null terminator in process_data_directive
                        size += 1;
                    }
                },
                _ => {} // Unsupported operand type, handled elsewhere
            }
        }
        
        size
    }
    
    /// Create the ELF file header
    fn create_elf_header(&self) -> Elf64Header {
        let mut e_ident = [0u8; EI_NIDENT];
        
        // Set magic bytes and ELF identification
        e_ident[0..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        e_ident[4] = 2;    // 64-bit format
        e_ident[5] = 1;    // Little-endian
        e_ident[6] = 1;    // Current ELF version
        e_ident[7] = 0;    // System V ABI
        e_ident[8] = 0;    // No ABI version
        
        Elf64Header {
            e_ident,
            e_type: ET_EXEC,
            e_machine: EM_X86_64,
            e_version: EV_CURRENT as u32,
            e_entry: self.entry_point, // Use the entry point from _start label
            e_phoff: mem::size_of::<Elf64Header>() as u64, // Program header starts right after ELF header
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

// Helper function to convert a hex string to bytes
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let hex = hex.trim().replace(" ", "");
    
    let mut i = 0;
    while i < hex.len() {
        if i + 2 > hex.len() {
            return Err(format!("Invalid hex string length: {}", hex));
        }
        
        let byte_str = &hex[i..i+2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex byte '{}': {}", byte_str, e))?;
        
        bytes.push(byte);
        i += 2;
    }
    
    Ok(bytes)
}

// Helper function to parse a number string (decimal, hex, binary, octal)
fn parse_number(num: &str) -> Result<u64, String> {
    if num.starts_with("0x") || num.starts_with("0X") {
        // Hexadecimal
        u64::from_str_radix(&num[2..], 16)
            .map_err(|e| format!("Invalid hex number '{}': {}", num, e))
    } else if num.starts_with("0b") || num.starts_with("0B") {
        // Binary
        u64::from_str_radix(&num[2..], 2)
            .map_err(|e| format!("Invalid binary number '{}': {}", num, e))
    } else if num.starts_with('0') && num.len() > 1 {
        // Octal
        u64::from_str_radix(&num[1..], 8)
            .map_err(|e| format!("Invalid octal number '{}': {}", num, e))
    } else {
        // Decimal
        num.parse::<u64>()
            .map_err(|e| format!("Invalid decimal number '{}': {}", num, e))
    }
} 