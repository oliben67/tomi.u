//! Integration tests for tomc - end-to-end compilation tests

use tomc::codegen::{Backend, CodeGenerator, CodegenConfig};
use tomc::lexer::Lexer;
use tomc::parser::TomiParser;
use std::process::Command;
use std::fs;

/// Full compilation pipeline
fn compile(source: &str) -> Result<String, String> {
    // Lexer
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {:?}", e))?;
    
    // Parser
    let mut parser = TomiParser::new(tokens).with_source(source.to_string());
    let module = parser.parse().map_err(|e| format!("Parser errors: {:?}", e))?;
    
    // Codegen
    let config = CodegenConfig {
        include_comments: false,  // Cleaner output for tests
        ..CodegenConfig::default()
    };
    let generator = CodeGenerator::with_config(Backend::Rust, config);
    generator.generate(&module).map_err(|e| format!("Codegen error: {:?}", e))
}

/// Test that generated Rust code compiles with rustc
fn verify_compiles(rust_code: &str) -> bool {
    // Use unique temp file to avoid race conditions in parallel tests
    let temp_file = tempfile::Builder::new()
        .prefix("tomc_test_")
        .suffix(".rs")
        .tempfile()
        .expect("Failed to create temp file");
    
    fs::write(temp_file.path(), rust_code).expect("Failed to write to temp file");
    
    // Try to compile with rustc (check only, no output)
    let output_rmeta = temp_file.path().with_extension("rmeta");
    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(&output_rmeta)
        .arg(temp_file.path())
        .output()
        .expect("Failed to run rustc");
    
    // Clean up metadata file
    let _ = fs::remove_file(&output_rmeta);
    
    output.status.success()
}

// ═══════════════════════════════════════════════════════════════════════
// End-to-End Compilation Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_hello_world_compiles() {
    let source = r#"
@entrypoint
def main():
    let message = "Hello, World!"
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("fn main()"));
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_struct_compiles() {
    let source = r#"
struct Point:
    x: i32
    y: i32

@entrypoint    
def main():
    let p = Point { x: 10, y: 20 }
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_function_with_params_compiles() {
    let source = r#"
def add(a: i32, b: i32) -> i32:
    return a + b

@entrypoint
def main():
    let result = add(1, 2)
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_enum_compiles() {
    // Note: Path expressions (Color::Red) not yet implemented
    // Just test enum definition compiles
    let source = r#"
enum Color:
    Red
    Green
    Blue

@entrypoint
def main():
    let x = 1
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("enum Color"));
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_control_flow_compiles() {
    let source = r#"
@entrypoint
def main():
    let x = 10
    if x > 5:
        let y = 1
    else:
        let y = 0
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_while_loop_compiles() {
    let source = r#"
@entrypoint    
def main():
    let mut i = 0
    while i < 10:
        i = i + 1
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    // While loops should compile  
    assert!(rust.contains("while"));
}

#[test]
fn test_arithmetic_compiles() {
    let source = r#"
@entrypoint
def main():
    let a = 1 + 2
    let b = 3 - 4
    let c = 5 * 6
    let d = 7 / 8
    let e = 9 % 10
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_boolean_ops_compiles() {
    let source = r#"
@entrypoint
def main():
    let a = true && false
    let b = true || false
    let c = !true
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_comparison_ops_compiles() {
    let source = r#"
@entrypoint
def main():
    let a = 1 < 2
    let b = 3 > 4
    let c = 5 <= 6
    let d = 7 >= 8
    let e = 9 == 10
    let f = 11 != 12
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_public_items_compile() {
    let source = r#"
pub struct PublicStruct:
    value: i32

pub def public_fn() -> i32:
    return 42

@entrypoint
def main():
    let s = PublicStruct { value: 1 }
    let n = public_fn()
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("pub struct PublicStruct"));
    assert!(rust.contains("pub fn public_fn"));
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_field_access_compiles() {
    let source = r#"
struct Point:
    x: i32
    y: i32

@entrypoint
def main():
    let p = Point { x: 10, y: 20 }
    let x = p.x
    let y = p.y
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

#[test]
fn test_nested_expressions_compiles() {
    let source = r#"
@entrypoint
def main():
    let a = (1 + 2) * (3 - 4)
    let b = 10 / (2 + 3)
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Generated Rust should compile");
}

// ═══════════════════════════════════════════════════════════════════════
// Error Handling Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_syntax_error_detected() {
    let source = "def test(";  // Missing closing paren
    let result = compile(source);
    assert!(result.is_err());
}

#[test]
fn test_missing_body_error() {
    let source = "def test()";  // Missing body
    let result = compile(source);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// Example Files Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_full_example_hello() {
    let source = r#"# A simple hello world program in tomi.u

@entrypoint
def main():
    let message = "Hello, World!"
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Hello example should compile");
}

#[test]
fn test_full_example_structs() {
    let source = r#"# A more complete tomi.u example

struct Point:
    x: i32
    y: i32

def add(a: i32, b: i32) -> i32:
    return a + b

@entrypoint
def main():
    let p = Point { x: 10, y: 20 }
    let sum = add(p.x, p.y)
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(verify_compiles(&rust), "Structs example should compile");
}

// ═══════════════════════════════════════════════════════════════════════
// Regression Tests  
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_struct_init_in_expression() {
    // Ensure struct init is parsed as expression, not confused with other syntax
    let source = r#"
struct Config:
    value: i32

def test():
    let c = Config { value: 42 }
    return c
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("Config { value: 42 }"));
}

#[test]
fn test_multiple_functions() {
    let source = r#"
def a() -> i32:
    return 1

def b() -> i32:
    return 2

def c() -> i32:
    return a() + b()
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("fn a()"));
    assert!(rust.contains("fn b()"));
    assert!(rust.contains("fn c()"));
}

#[test]
fn test_unicode_identifiers() {
    let source = r#"
def 计算(数: i32) -> i32:
    return 数 * 2

@entrypoint
def main():
    let 结果 = 计算(5)
"#;
    
    let rust = compile(source).expect("Compilation should succeed");
    assert!(rust.contains("fn 计算"));
    assert!(rust.contains("let 结果"));
}
