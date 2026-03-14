//! tomc - The tomi.u Compiler CLI
//!
//! Usage: tomc [OPTIONS] [INPUT]

use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use tomc::codegen::{Backend, CodeGenerator};
use tomc::error::ErrorReporter;
use tomc::lexer::Lexer;
use tomc::parser::TomiParser;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

/// tomc — The tomi.u Compiler
#[derive(Parser, Debug)]
#[command(name = "tomc")]
#[command(version, about, long_about = None)]
#[command(after_help = "Use `tomc --explain <ERROR_CODE>` (e.g. E0001) for a detailed error explanation.")]
struct Cli {
    /// Input tomi.u source file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Write output to <FILE>
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Target backend for code generation
    #[arg(short = 't', long, value_enum, default_value_t = Target::Rust)]
    target: Target,

    /// Comma-separated list of output types to emit: tokens,ast,code,metadata
    #[arg(long, value_name = "KIND,...")]
    emit: Option<String>,

    /// tomi.u edition to compile against
    #[arg(long, value_enum, default_value_t = Edition::E2024)]
    edition: Edition,

    /// Type of artifact to produce
    #[arg(long, value_enum, default_value_t = AlbumType::Bin)]
    album_type: AlbumType,

    /// Set a codegen option: KEY or KEY=VALUE
    ///
    /// Available options:
    ///   opt-level=<0|1|2|3>       Optimization level (default: 0)
    ///   overflow-checks=<yes|no>  Integer overflow checking (default: yes)
    ///   debug-info=<yes|no>       Embed debug information (default: no)
    ///   lto=<yes|no>              Link-time optimization (default: no)
    #[arg(short = 'C', value_name = "OPT[=VALUE]")]
    codegen_opts: Vec<String>,

    /// Set lint to warn
    #[arg(short = 'W', value_name = "LINT")]
    warn: Vec<String>,

    /// Set lint to deny (error)
    #[arg(short = 'D', value_name = "LINT")]
    deny: Vec<String>,

    /// Set lint to allow
    #[arg(short = 'A', value_name = "LINT")]
    allow: Vec<String>,

    /// Use verbose output (shows timing and pipeline stages)
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Only check syntax, do not generate output
    #[arg(long)]
    check: bool,

    /// Configure coloring of output
    #[arg(long, value_enum, default_value_t = Color::Auto)]
    color: Color,

    /// Print a detailed explanation for a compiler error code
    #[arg(long, value_name = "ERROR_CODE")]
    explain: Option<String>,

    // Backward-compatible hidden aliases
    /// [deprecated] Use --emit tokens instead
    #[arg(long, hide = true)]
    print_tokens: bool,

    /// [deprecated] Use --emit ast instead
    #[arg(long, hide = true)]
    print_ast: bool,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
enum Target {
    /// Generate Rust source code (.rs)
    Rust,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Rust => write!(f, "rust"),
        }
    }
}

impl From<Target> for Backend {
    fn from(t: Target) -> Self {
        match t {
            Target::Rust => Backend::Rust,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug, Default)]
enum Edition {
    /// tomi.u edition 2024 (current)
    #[default]
    #[value(name = "2024")]
    E2024,
}

impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edition::E2024 => write!(f, "2024"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug, Default)]
enum AlbumType {
    /// Compile a binary executable album
    #[default]
    Bin,
    /// Compile a library album
    Lib,
}

impl fmt::Display for AlbumType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlbumType::Bin => write!(f, "bin"),
            AlbumType::Lib => write!(f, "lib"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug, Default)]
enum Color {
    /// Auto-detect terminal color support
    #[default]
    Auto,
    /// Always emit ANSI color codes
    Always,
    /// Never emit color codes
    Never,
}

// ---------------------------------------------------------------------------
// Emit kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct EmitKinds {
    tokens: bool,
    ast: bool,
    code: bool,
    metadata: bool,
}

impl EmitKinds {
    fn parse(s: &str) -> Result<Self, String> {
        let mut kinds = EmitKinds::default();
        for part in s.split(',') {
            match part.trim() {
                "tokens" => kinds.tokens = true,
                "ast" => kinds.ast = true,
                "code" => kinds.code = true,
                "metadata" => kinds.metadata = true,
                other => return Err(format!("unknown emit kind `{other}` (valid: tokens, ast, code, metadata)")),
            }
        }
        Ok(kinds)
    }
}

// ---------------------------------------------------------------------------
// Codegen options (mirrors -C flags in rustc)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct CodegenConfig {
    opt_level: u8,
    overflow_checks: bool,
    debug_info: bool,
    lto: bool,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self { opt_level: 0, overflow_checks: true, debug_info: false, lto: false }
    }
}

fn parse_codegen_opts(opts: &[String]) -> Result<CodegenConfig, String> {
    let mut cfg = CodegenConfig::default();
    for opt in opts {
        let (key, val) = if let Some((k, v)) = opt.split_once('=') {
            (k, Some(v))
        } else {
            (opt.as_str(), None)
        };
        match key {
            "opt-level" => {
                let n = val.unwrap_or("2").parse::<u8>()
                    .map_err(|_| format!("-C opt-level: expected 0–3, got `{}`", val.unwrap_or("")))?;
                if n > 3 { return Err(format!("-C opt-level: value must be 0–3, got {n}")); }
                cfg.opt_level = n;
            }
            "overflow-checks" => cfg.overflow_checks = parse_bool_opt(val, "overflow-checks")?,
            "debug-info" => cfg.debug_info = parse_bool_opt(val, "debug-info")?,
            "lto" => cfg.lto = parse_bool_opt(val, "lto")?,
            other => return Err(format!("unknown codegen option: `{other}` (valid: opt-level, overflow-checks, debug-info, lto)")),
        }
    }
    Ok(cfg)
}

fn parse_bool_opt(val: Option<&str>, name: &str) -> Result<bool, String> {
    match val.unwrap_or("yes") {
        "yes" | "true" | "on" | "1" => Ok(true),
        "no" | "false" | "off" | "0" => Ok(false),
        other => Err(format!("-C {name}: expected yes/no, got `{other}`")),
    }
}

// ---------------------------------------------------------------------------
// Lint validation
// ---------------------------------------------------------------------------

const KNOWN_LINTS: &[&str] = &[
    "unused-variables",
    "unused-parameters",
    "unused-imports",
    "dead-code",
    "unreachable-code",
    "unused-mut",
    "warnings",
];

fn validate_lints(cli: &Cli) {
    let all: Vec<&str> = cli.warn.iter()
        .chain(cli.deny.iter())
        .chain(cli.allow.iter())
        .map(String::as_str)
        .collect();
    for lint in all {
        if !KNOWN_LINTS.contains(&lint) {
            eprintln!("{}: unknown lint: `{lint}`", "warning".yellow().bold());
            eprintln!("  known lints: {}", KNOWN_LINTS.join(", "));
        }
    }
}

// ---------------------------------------------------------------------------
// --explain error codes
// ---------------------------------------------------------------------------

fn handle_explain(code: &str) -> ExitCode {
    let upper = code.to_uppercase();
    match upper.as_str() {
        "E0001" => print_explanation("E0001", "Unexpected token",
            "The compiler encountered a token it did not expect at this position.\n\
             \n\
             Example:\n\
             \n\
             ```tomi.u\n\
             def foo() {\n\
                 ^^^ expected `:` but found `{`\n\
             ```\n\
             \n\
             Check that keywords and punctuation are correct for the construct you\n\
             are writing."),
        "E0002" => print_explanation("E0002", "Unterminated string literal",
            "A string literal was opened with `\"` but never closed.\n\
             \n\
             Example:\n\
             \n\
             ```tomi.u\n\
             let s = \"hello\n\
                      ^^^^^^ string literal not terminated\n\
             ```\n\
             \n\
             Add the closing `\"` at the end of the string."),
        "E0003" => print_explanation("E0003", "Unexpected end of file",
            "The compiler reached the end of the source file while still inside\n\
             a construct (block, expression, parameter list, etc.).\n\
             \n\
             Check for unclosed parentheses, missing colons on `def`/`if`/`for`,\n\
             or an incomplete last statement."),
        "E0004" => print_explanation("E0004", "Expected identifier",
            "An identifier (variable name, function name, type name) was required\n\
             here but a different token was found.\n\
             \n\
             Identifiers must start with a letter or `_` and may contain letters,\n\
             digits, and `_`."),
        "E0005" => print_explanation("E0005", "Invalid numeric literal",
            "A numeric literal contained characters or a format not recognised\n\
             by the compiler.\n\
             \n\
             Valid forms:\n\
             - Integer:  `42`, `0`, `1_000_000`\n\
             - Float:    `3.14`, `1.0e10`\n\
             - Hex:      `0xFF`\n\
             - Binary:   `0b1010`"),
        "E0006" => print_explanation("E0006", "Expected indented block",
            "A construct that requires a body (def, if, for, while, try, …) was\n\
             not followed by an indented block.\n\
             \n\
             Example:\n\
             \n\
             ```tomi.u\n\
             def greet():  # <-- the next line must be indented\n\
                 print(\"hi\")\n\
             ```"),
        "E0007" => print_explanation("E0007", "Inconsistent indentation",
            "The indentation level inside a block was inconsistent — some lines\n\
             used a different number of spaces or mixed tabs and spaces.\n\
             \n\
             tomi.u enforces a single, consistent indentation unit per block.\n\
             Prefer 4 spaces throughout."),
        "E0008" => print_explanation("E0008", "Invalid escape sequence",
            "A string literal contained a backslash escape sequence that is not\n\
             recognised.\n\
             \n\
             Valid escapes: `\\n`, `\\t`, `\\r`, `\\\\`, `\\\"`, `\\'`, `\\0`,\n\
             `\\xNN`, `\\u{NNNN}`."),
        "E0009" => print_explanation("E0009", "Expected `:` after function signature",
            "A `def` declaration was not followed by `:` before the function body.\n\
             \n\
             ```tomi.u\n\
             def foo()  # missing colon\n\
                 ...\n\
             \n\
             def foo():  # correct\n\
                 ...\n\
             ```"),
        "E0010" => print_explanation("E0010", "Mismatched parentheses or brackets",
            "An opening delimiter (`(`, `[`, `{`) was not matched by the\n\
             corresponding closing delimiter.\n\
             \n\
             Check all enclosed expressions and make sure every opener has a\n\
             matching closer."),
        _ => {
            eprintln!("{}: unknown error code: `{code}`", "error".red().bold());
            eprintln!("  example codes: E0001 … E0010");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn print_explanation(code: &str, title: &str, body: &str) {
    println!("{} — {}", code.cyan().bold(), title.bold());
    println!();
    println!("{body}");
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let cli = Cli::parse();

    // --explain is standalone: no input needed
    if let Some(ref code) = cli.explain {
        return handle_explain(code);
    }

    // Configure color
    match cli.color {
        Color::Always => colored::control::set_override(true),
        Color::Never => colored::control::set_override(false),
        Color::Auto => {}
    }

    // Input is required for compilation
    let input = match cli.input.clone() {
        Some(p) => p,
        None => {
            eprintln!("{}: no input file", "error".red().bold());
            eprintln!("  Usage: tomc [OPTIONS] <INPUT>");
            eprintln!("  Help:  tomc --help");
            return ExitCode::FAILURE;
        }
    };

    // Compute emit set (merge legacy flags)
    let mut emit = match cli.emit.as_deref() {
        Some(s) => match EmitKinds::parse(s) {
            Ok(e) => e,
            Err(msg) => {
                eprintln!("{}: {}", "error".red().bold(), msg);
                return ExitCode::FAILURE;
            }
        },
        None => EmitKinds { code: true, ..Default::default() },
    };
    if cli.print_tokens { emit.tokens = true; }
    if cli.print_ast    { emit.ast    = true; }
    // --check disables code output
    if cli.check        { emit.code   = false; }

    // Parse -C options
    let codegen_cfg = match parse_codegen_opts(&cli.codegen_opts) {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{}: {}", "error".red().bold(), msg);
            return ExitCode::FAILURE;
        }
    };

    // Validate lints
    validate_lints(&cli);

    if cli.verbose {
        eprintln!("{} tomc {} (edition {})", "info:".cyan().bold(), env!("CARGO_PKG_VERSION"), cli.edition);
        eprintln!("{} compiling `{}`", "info:".cyan().bold(), input.display());
        eprintln!("{} target={}, album-type={}, opt-level={}", "info:".cyan().bold(),
            cli.target, cli.album_type, codegen_cfg.opt_level);
    }

    let start = Instant::now();

    // -----------------------------------------------------------------
    // Read source
    // -----------------------------------------------------------------
    let source = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: cannot read `{}`: {}", "error".red().bold(), input.display(), e);
            return ExitCode::FAILURE;
        }
    };

    let filename = input.to_string_lossy().to_string();
    let reporter = ErrorReporter::new(&filename, &source);

    // -----------------------------------------------------------------
    // Lexing
    // -----------------------------------------------------------------
    if cli.verbose { eprintln!("{} lexing…", "info:".cyan().bold()); }
    let t0 = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(errors) => {
            for e in errors { reporter.report(&e); }
            return ExitCode::FAILURE;
        }
    };
    if cli.verbose { eprintln!("{} lexed {} tokens ({:?})", "info:".cyan().bold(), tokens.len(), t0.elapsed()); }

    if emit.tokens {
        println!("{}", "=== Tokens ===".cyan().bold());
        for tok in &tokens { println!("  {tok:?}"); }
        println!();
    }

    // -----------------------------------------------------------------
    // Parsing
    // -----------------------------------------------------------------
    if cli.verbose { eprintln!("{} parsing…", "info:".cyan().bold()); }
    let t1 = Instant::now();
    let mut parser = TomiParser::new(tokens).with_source(source.clone());
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(errors) => {
            for e in errors { reporter.report(&e); }
            return ExitCode::FAILURE;
        }
    };
    if cli.verbose { eprintln!("{} parsed AST ({:?})", "info:".cyan().bold(), t1.elapsed()); }

    if emit.ast {
        println!("{}", "=== AST ===".cyan().bold());
        println!("{ast:#?}");
        println!();
    }

    if cli.check {
        println!("{}", "✓ syntax check passed".green().bold());
        if cli.verbose { eprintln!("{} total: {:?}", "info:".cyan().bold(), start.elapsed()); }
        return ExitCode::SUCCESS;
    }

    if !emit.code {
        if cli.verbose { eprintln!("{} total: {:?}", "info:".cyan().bold(), start.elapsed()); }
        return ExitCode::SUCCESS;
    }

    // -----------------------------------------------------------------
    // Code generation
    // -----------------------------------------------------------------
    if cli.verbose { eprintln!("{} generating {} code…", "info:".cyan().bold(), cli.target); }
    let t2 = Instant::now();
    let backend: Backend = cli.target.into();
    let generator = CodeGenerator::new(backend);
    let output_code = match generator.generate(&ast) {
        Ok(code) => code,
        Err(err) => {
            reporter.report(&err);
            return ExitCode::FAILURE;
        }
    };
    if cli.verbose { eprintln!("{} codegen done ({:?})", "info:".cyan().bold(), t2.elapsed()); }

    // -----------------------------------------------------------------
    // Write output
    // -----------------------------------------------------------------
    let output_path = cli.output.clone().unwrap_or_else(|| {
        let stem = input.file_stem().unwrap_or_default();
        let ext = match cli.target { Target::Rust => "rs" };
        PathBuf::from(format!("{}.{}", stem.to_string_lossy(), ext))
    });

    if let Err(e) = std::fs::write(&output_path, &output_code) {
        eprintln!("{}: cannot write `{}`: {}", "error".red().bold(), output_path.display(), e);
        return ExitCode::FAILURE;
    }

    println!("{} {} → {}", "✓".green().bold(), input.display(), output_path.display());
    if cli.verbose { eprintln!("{} total: {:?}", "info:".cyan().bold(), start.elapsed()); }

    if emit.metadata {
        let name = input.file_stem().unwrap_or_default().to_string_lossy();
        println!();
        println!("{}", "=== Metadata ===".cyan().bold());
        println!("  name:      {name}");
        println!("  edition:   {}", cli.edition);
        println!("  album-type: {}", cli.album_type);
        println!("  target:    {}", cli.target);
        println!("  opt-level: {}", codegen_cfg.opt_level);
        println!("  tomc:      {}", env!("CARGO_PKG_VERSION"));
    }

    ExitCode::SUCCESS
}
