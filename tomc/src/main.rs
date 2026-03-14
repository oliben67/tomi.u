//! tomc - The tomi.u Compiler CLI
//!
//! Usage: tomc [OPTIONS] <INPUT>

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

use tomc::codegen::{Backend, CodeGenerator};
use tomc::error::ErrorReporter;
use tomc::lexer::Lexer;
use tomc::parser::TomiParser;

/// tomc - The tomi.u Compiler
#[derive(Parser, Debug)]
#[command(name = "tomc")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input tomi.u source file
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file (default: input stem with target extension)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Target backend for code generation
    #[arg(short, long, value_enum, default_value_t = Target::Rust)]
    target: Target,

    /// Print tokens (lexer output)
    #[arg(long)]
    print_tokens: bool,

    /// Print AST (parser output)
    #[arg(long)]
    print_ast: bool,

    /// Don't generate output file, just check syntax
    #[arg(long)]
    check: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
enum Target {
    /// Generate Rust code (.rs)
    Rust,
    // Future targets:
    // C,
    // Cpp,
}

impl From<Target> for Backend {
    fn from(target: Target) -> Self {
        match target {
            Target::Rust => Backend::Rust,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Read source file
    let source = match std::fs::read_to_string(&cli.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}: cannot read '{}': {}",
                "error".red().bold(),
                cli.input.display(),
                e
            );
            return ExitCode::FAILURE;
        }
    };

    let filename = cli.input.to_string_lossy().to_string();
    let reporter = ErrorReporter::new(&filename, &source);

    // Lexing
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(errors) => {
            for err in errors {
                reporter.report(&err);
            }
            return ExitCode::FAILURE;
        }
    };

    if cli.print_tokens {
        println!("{}", "=== Tokens ===".cyan().bold());
        for token in &tokens {
            println!("  {:?}", token);
        }
        println!();
    }

    // Parsing
    let mut parser = TomiParser::new(tokens).with_source(source.clone());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(errors) => {
            for err in errors {
                reporter.report(&err);
            }
            return ExitCode::FAILURE;
        }
    };

    if cli.print_ast {
        println!("{}", "=== AST ===".cyan().bold());
        println!("{:#?}", ast);
        println!();
    }

    if cli.check {
        println!("{}", "✓ Syntax check passed".green().bold());
        return ExitCode::SUCCESS;
    }

    // Code generation
    let backend: Backend = cli.target.into();
    let generator = CodeGenerator::new(backend);

    let output_code = match generator.generate(&ast) {
        Ok(code) => code,
        Err(err) => {
            reporter.report(&err);
            return ExitCode::FAILURE;
        }
    };

    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let stem = cli.input.file_stem().unwrap_or_default();
        let ext = match cli.target {
            Target::Rust => "rs",
        };
        PathBuf::from(format!("{}.{}", stem.to_string_lossy(), ext))
    });

    // Write output
    if let Err(e) = std::fs::write(&output_path, output_code) {
        eprintln!(
            "{}: cannot write '{}': {}",
            "error".red().bold(),
            output_path.display(),
            e
        );
        return ExitCode::FAILURE;
    }

    println!(
        "{} {} → {}",
        "✓".green().bold(),
        cli.input.display(),
        output_path.display()
    );

    ExitCode::SUCCESS
}
