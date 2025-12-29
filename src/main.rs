use clap::Parser;
use clap::CommandFactory;

use std::path::Path;
use ys1r::io::file_read;

/// Accepts:
/// 1) an existing .rs file
/// 2) a non-negative integer
#[derive(Parser)]
#[command(name = "modrust")]
#[command(about = "Display Modified Rust Code by Ollama")]
struct Args {
    /// Input file (must be an existing .rs file)
    #[arg(value_parser = validate_rs_file)]
    filename: String,

    #[arg(help = "Numeric argument\n1. refactoring\n2. add test code\n3. add or update comment")]
    #[arg(value_parser = clap::value_parser!(u8))]
    number: u8,
}

fn validate_rs_file(s: &str) -> Result<String, String> {
    let path = Path::new(s);

    if !path.is_file() {
        return Err("The file does not exist".into());
    }

    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        return Err("Only .rs files are allowed".into());
    }

    Ok(s.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Args::try_parse().unwrap_or_else(|err| {
        let mut cmd = Args::command();
        cmd.print_help().unwrap();
        println!();
        std::process::exit(1);
    });

    let args = Args::parse();

    let rust_program = file_read(&args.filename);

    let prompt = build_prompt(args.number, &rust_program?);
    let raw_response = ys1r::ollama::request_ollama(&prompt?, None, Some(false)).await;
    println!("{}", &raw_response);
    Ok(())
}

use std::{error::Error, fmt};

#[derive(Debug)]
enum InstructionError {
    InvalidNumber(u8),
}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionError::InvalidNumber(n) => {
                write!(f, "invalid instruction number: {n}")
            }
        }
    }
}

impl Error for InstructionError {}

fn instruction(number: u8) -> Result<String, InstructionError> {
    match number {
        1 => Ok("Please refactor the following rust code.".to_string()),
        2 => Ok("Please add appropriate tests to the following rust code.".to_string()),
        3 => Ok("Please add or update rustdoc comments for the following rust code.".to_string()),
        n => Err(InstructionError::InvalidNumber(n)),
    }
}

fn build_prompt(number: u8, rust_code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let inst = instruction(number)?;
    Ok(format!(
        r#"
- {}
- Target is Rust.
---
{}
"#,
        inst, rust_code
    ))
}
