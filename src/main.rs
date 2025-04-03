use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::Instant;
use clap::{Parser as ClapParser, ArgGroup, ValueEnum};
use colored::*;

mod tokenizer;
mod parser;
mod elf;
mod encoder;
mod error;

use tokenizer::Tokenizer;
use parser::Parser;
use parser::ast::Program;
use elf::ElfGenerator;
use error::{ErrorCollector, Error, ErrorType, ErrorDetail, ErrorSeverity};

/// NASimulator - A modern x86-64 assembler
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(group(ArgGroup::new("output_mode").args(&["parse_only", "tokenize_only", "dump_tokens", "dump_ast"])))]
struct Args {
    /// Input file to assemble (required)
    #[arg(index = 1, required = true)]
    file: String,
    
    /// Output file for the assembled code
    #[arg(short, long)]
    output: Option<String>,
    
    /// Only parse the file, don't generate output
    #[arg(long, group = "mode")]
    parse_only: bool,
    
    /// Only tokenize the file, don't parse or generate output
    #[arg(long, group = "mode")]
    tokenize_only: bool,
    
    /// Dump tokens after tokenization
    #[arg(long, group = "mode")]
    dump_tokens: bool,
    
    /// Dump the Abstract Syntax Tree (AST) after parsing
    #[arg(long, group = "mode")]
    dump_ast: bool,
    
    /// Print verbose information during compilation
    #[arg(short, long)]
    verbose: bool,
    
    /// Path to the opcodes definition file
    #[arg(short = 'p', long)]
    opcodes: Option<PathBuf>,
    
    /// Output format for the compiled binary [default: elf]
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Elf)]
    format: OutputFormat,
    
    /// Stop on first error instead of collecting all errors
    #[arg(short = 's', long)]
    stop_on_first_error: bool,
    
    /// Silent mode - only show errors, not warnings
    #[arg(long)]
    silent: bool,
    
    /// Execute the compiled binary after successful assembly
    #[arg(short = 'x', long)]
    execute: bool,
    
    /// Make the output file executable (chmod +x)
    #[arg(short = 'e', long)]
    make_executable: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Raw binary output
    Bin,
    /// Intel hex format
    Hex,
    /// ELF executable (default)
    Elf,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    
    // Create error collector
    let mut error_collector = ErrorCollector::new();
    
    // Header message
    if !args.silent {
        println!("{}", "─".repeat(60).bright_blue());
        println!("{} {}", "NASimulator".bright_white().bold(), "- x86-64 Assembler".bright_blue());
        println!("{}", "─".repeat(60).bright_blue());
    }
    
    // Load file content
    let start_time = Instant::now();
    let file_content = match fs::read_to_string(&args.file) {
        Ok(content) => content,
        Err(e) => {
            let file_error = error::file_error(
                format!("Failed to read input file: {}", e),
                &args.file
            );
            error_collector.add_error(file_error);
            
            // If we can't even read the file, we can't proceed
            println!("{}", error_collector.display_errors());
            process::exit(1);
        }
    };
    
    if args.verbose {
        println!("{} {} in {:.2?}",
            "→".bright_blue(),
            "File loaded".bright_white().bold(),
            start_time.elapsed());
    }
    
    // Tokenize the file
    let start = Instant::now();
    let mut tokenizer = Tokenizer::new(&file_content);
    let tokens = tokenizer.tokenize();
    let tokenize_time = start.elapsed();
    
    if args.verbose {
        println!("{} {} in {:.2?} ({} tokens)",
            "→".bright_blue(),
            "Tokenization completed".bright_white().bold(),
            tokenize_time,
            tokens.len());
    }
    
    // If tokenize_only or dump_tokens flag is set, show tokens and stop
    if args.tokenize_only || args.dump_tokens {
        println!("\n{}", "Tokens:".bright_white().bold().underline());
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}. {:?}", i+1, token);
        }
        println!("\n{} {} tokens", "✓".green().bold(), tokens.len());
        return Ok(());
    }
    
    // Parse tokens
    let start = Instant::now();
    let mut parser = Parser::new(tokens.clone())
        .with_error_collector(error_collector.clone())
        .with_file_name(args.file.clone())
        .with_continue_on_errors(!args.stop_on_first_error);
    
    // Parse the program
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(err_msg) => {
            // If we're continuing on errors, use an empty program, otherwise exit
            if !args.stop_on_first_error {
                Program::new()
            } else {
                // Get the error collector from the parser before exiting
                error_collector = parser.get_error_collector().unwrap_or(error_collector);
                println!("{}", error_collector.display_errors());
                eprintln!("{} {}", "✗".bright_red().bold(), err_msg.bright_red());
                process::exit(1);
            }
        }
    };
    
    // Update the error collector with any errors collected during parsing
    error_collector = parser.get_error_collector().unwrap_or(error_collector);
    
    let parse_time = start.elapsed();
    
    if args.verbose {
        println!("{} {} in {:.2?} ({} statements)",
            "→".bright_blue(),
            "Parsing completed".bright_white().bold(),
            parse_time,
            program.statements.len());
    }
    
    // Dump AST if requested
    if args.dump_ast {
        println!("\n{}", "Abstract Syntax Tree:".bright_white().bold().underline());
        dump_ast(&program);
        return Ok(());
    }
    
    // If parse_only flag is set, stop here
    if args.parse_only {
        // If we have errors, display them
        if error_collector.has_errors() || (error_collector.warning_count() > 0 && !args.silent) {
            println!("{}", error_collector.display_errors());
            if error_collector.has_fatal_errors() || error_collector.error_count() > 0 {
                process::exit(1);
            }
        } else {
            println!("\n{} {}", "✓".green().bold(), "Parsing completed successfully with no errors".green());
        }
        return Ok(());
    }
    
    // Define output path
    let output_path = match args.output {
        Some(path) => path,
        None => {
            let path = PathBuf::from(&args.file);
            let stem = path.file_stem().unwrap_or_default();
            let extension = match args.format {
                OutputFormat::Bin => "bin",
                OutputFormat::Hex => "hex",
                OutputFormat::Elf => "",  // No extension for ELF executables by default
            };
            if extension.is_empty() {
                format!("{}", stem.to_string_lossy())
            } else {
                format!("{}.{}", stem.to_string_lossy(), extension)
            }
        },
    };
    
    // Generate output based on format
    let generation_start = Instant::now();
    let mut output_successful = false;
    
    if args.format == OutputFormat::Elf {
        let mut elf_generator = ElfGenerator::new(program);
        
        match elf_generator.generate(&output_path) {
            Ok(_) => {
                output_successful = true;
                
                // Make executable if requested
                if args.make_executable {
                    if let Err(err) = std::process::Command::new("chmod")
                        .args(&["+x", &output_path])
                        .output() {
                        eprintln!("{} Failed to make output file executable: {}", 
                            "⚠".yellow().bold(), 
                            err);
                    }
                }
                
                if args.verbose {
                    println!("{} {} in {:.2?}",
                        "→".bright_blue(),
                        "ELF generation completed".bright_white().bold(),
                        generation_start.elapsed());
                }
            },
            Err(err_msg) => {
                // Convert the string error to our Error type
                let elf_error = Error::new(
                    ErrorType::ElfWriteError,
                    ErrorDetail::new(err_msg.clone())
                ).with_severity(ErrorSeverity::Error);
                
                error_collector.add_error(elf_error);
            }
        }
    } else if args.format == OutputFormat::Bin || args.format == OutputFormat::Hex {
        // Placeholder for binary and hex output formats
        let error = Error::new(
            ErrorType::Other,
            ErrorDetail::new("Binary and hex output formats not implemented yet".to_string())
        ).with_severity(ErrorSeverity::Error);
        
        error_collector.add_error(error);
    }
    
    // Display any errors collected during processing
    if error_collector.has_errors() || (error_collector.warning_count() > 0 && !args.silent) {
        println!("{}", error_collector.display_errors());
        
        if error_collector.has_fatal_errors() || error_collector.has_errors() {
            process::exit(1);
        }
    }
    
    // Show summary if compilation was successful
    if output_successful {
        let canonical_path = std::fs::canonicalize(&output_path).unwrap_or_else(|_| PathBuf::from(&output_path));
        
        println!("\n{} {}", "✓".green().bold(), "Assembly completed successfully".green().bold());
        println!("{} Output binary at: {}", 
            "→".bright_blue().bold(), 
            canonical_path.display().to_string().bright_white().bold().underline());
        
        if args.verbose {
            println!("{} Total time: {:.2?}", 
                "→".bright_blue().bold(), 
                start_time.elapsed());
        }
        
        // Execute the binary if requested
        if args.execute {
            println!("\n{} {}", "►".bright_green().bold(), "Executing output binary:".bright_green());
            println!("{}", "─".repeat(60).bright_blue());
            
            let status = std::process::Command::new(canonical_path)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("{} Failed to execute binary: {}", "✗".bright_red().bold(), e);
                    process::exit(1);
                });
            
            println!("{}", "─".repeat(60).bright_blue());
            println!("{} Exit code: {}", 
                "→".bright_blue().bold(), 
                status.code().unwrap_or(-1));
        }
    } else if !error_collector.has_errors() {
        // This should not happen, but just in case
        eprintln!("{} {}", "✗".bright_red().bold(), "Failed to generate output for unknown reason".bright_red());
        process::exit(1);
    }
    
    Ok(())
}

/// Print a summary of the AST
fn print_ast_summary(program: &Program) {
    // Count of different types of statements
    let mut instruction_count = 0;
    let mut label_count = 0;
    let mut directive_count = 0;
    let mut section_count = 0;
    let mut comment_count = 0;
    let mut empty_count = 0;
    
    for statement in &program.statements {
        match statement {
            parser::ast::Statement::Instruction(_) => instruction_count += 1,
            parser::ast::Statement::Label(_) => label_count += 1,
            parser::ast::Statement::Directive(_) => directive_count += 1,
            parser::ast::Statement::Section(_) => section_count += 1,
            parser::ast::Statement::Comment(_) => comment_count += 1,
            parser::ast::Statement::Empty => empty_count += 1,
        }
    }
    
    println!("  AST Summary:");
    println!("    Instructions: {}", instruction_count);
    println!("    Labels: {}", label_count);
    println!("    Directives: {}", directive_count);
    println!("    Sections: {}", section_count);
    println!("    Comments: {}", comment_count);
    println!("    Empty statements: {}", empty_count);
}

/// Dump the AST in a slightly pretty format
fn dump_ast(program: &Program) {
    // List sections
    println!("Sections:");
    for (section_name, section) in &program.sections {
        println!("  {}: {} bytes", section_name, section.size);
    }
    
    // List labels
    println!("\nLabels:");
    for (label_name, label_info) in &program.labels {
        println!("  {}: offset {} in section '{}'", 
                 label_name, 
                 label_info.offset, 
                 label_info.section.as_deref().unwrap_or("unknown"));
    }
    
    // List statements
    println!("\nStatements:");
    for (i, statement) in program.statements.iter().enumerate() {
        println!("  {}: {:?}", i, statement);
    }
}
