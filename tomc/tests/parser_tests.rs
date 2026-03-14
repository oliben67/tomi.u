//! Parser tests for tomc

use tomc::ast::*;
use tomc::lexer::Lexer;
use tomc::parser::TomiParser;

/// Helper to parse source code and return the AST module
fn parse(source: &str) -> Module {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = TomiParser::new(tokens).with_source(source.to_string());
    parser.parse().expect("Parser should succeed")
}

/// Helper to parse and expect an error
fn parse_should_fail(source: &str) -> Vec<tomc::CompileError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = TomiParser::new(tokens).with_source(source.to_string());
    parser.parse().expect_err("Parser should fail")
}

// ═══════════════════════════════════════════════════════════════════════
// Empty Module Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_module() {
    let module = parse("");
    assert!(module.items.is_empty());
    assert!(module.imports.is_empty());
}

#[test]
fn test_whitespace_only_module() {
    // Pure whitespace may parse differently based on lexer behavior
    let source = "   \n\n  ";
    // Just verify parsing doesn't crash
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = TomiParser::new(tokens).with_source(source.to_string());
    // The parser may succeed with empty module or may produce errors
    // depending on lexer's indentation handling
    let _ = parser.parse(); // Just make sure it doesn't panic
}

// ═══════════════════════════════════════════════════════════════════════
// Function Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_simple_function() {
    let source = "def hello():\n    return 0\n";
    let module = parse(source);

    assert_eq!(module.items.len(), 1);
    match &module.items[0] {
        Item::Function(func) => {
            assert_eq!(func.name.node, "hello");
            assert!(func.params.is_empty());
            assert!(func.return_type.is_none());
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_function_with_params() {
    let source = "def add(a: i32, b: i32) -> i32:\n    return a + b\n";
    let module = parse(source);

    assert_eq!(module.items.len(), 1);
    match &module.items[0] {
        Item::Function(func) => {
            assert_eq!(func.name.node, "add");
            assert_eq!(func.params.len(), 2);
            assert_eq!(func.params[0].name.node, "a");
            assert_eq!(func.params[1].name.node, "b");
            assert!(func.return_type.is_some());
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_public_function() {
    let source = "pub def public_fn():\n    return 1\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            assert!(matches!(func.visibility, Visibility::Public));
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_async_function() {
    let source = "async def fetch():\n    return 1\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            assert!(func.is_async);
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_decorated_function() {
    let source = "@entrypoint\ndef main():\n    return 0\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            assert_eq!(func.decorators.len(), 1);
            assert_eq!(func.decorators[0].name.node, "entrypoint");
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Struct Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_simple_struct() {
    let source = "struct Point:\n    x: i32\n    y: i32\n";
    let module = parse(source);

    assert_eq!(module.items.len(), 1);
    match &module.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.name.node, "Point");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name.node, "x");
            assert_eq!(s.fields[1].name.node, "y");
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_public_struct() {
    let source = "pub struct Public:\n    value: i32\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Struct(s) => {
            assert!(matches!(s.visibility, Visibility::Public));
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
#[ignore = "Generic struct syntax not yet fully implemented"]
fn test_generic_struct() {
    let source = "struct Container<T>:\n    value: T\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.type_params.len(), 1);
            assert_eq!(s.type_params[0].name.node, "T");
        }
        _ => panic!("Expected struct"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Enum Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_simple_enum() {
    let source = "enum Color:\n    Red\n    Green\n    Blue\n";
    let module = parse(source);

    assert_eq!(module.items.len(), 1);
    match &module.items[0] {
        Item::Enum(e) => {
            assert_eq!(e.name.node, "Color");
            assert_eq!(e.variants.len(), 3);
            assert_eq!(e.variants[0].name.node, "Red");
            assert_eq!(e.variants[1].name.node, "Green");
            assert_eq!(e.variants[2].name.node, "Blue");
        }
        _ => panic!("Expected enum"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Statement Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_let_statement() {
    let source = "def test():\n    let x = 42\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            assert_eq!(func.body.stmts.len(), 1);
            match &func.body.stmts[0] {
                Stmt::Let { pattern, value, .. } => {
                    match pattern {
                        Pattern::Identifier { name, .. } => {
                            assert_eq!(name.node, "x");
                        }
                        _ => panic!("Expected identifier pattern"),
                    }
                    assert!(value.is_some());
                }
                _ => panic!("Expected let statement"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_let_mut_statement() {
    // Note: The parser supports `mut x = 0` syntax for mutable bindings
    // A standalone `mut` keyword triggers the mutable let path
    let source = "def test():\n    mut x = 0\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Let { is_mut, .. } => {
                assert!(*is_mut);
            }
            _ => panic!("Expected let statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_let_with_type() {
    let source = "def test():\n    let x: i32 = 42\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Let { ty, .. } => {
                assert!(ty.is_some());
            }
            _ => panic!("Expected let statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_return_statement() {
    let source = "def test():\n    return 42\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Return { value, .. } => {
                assert!(value.is_some());
            }
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Expression Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_integer_literal() {
    let source = "def test():\n    42\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => {
                assert!(matches!(expr, Expr::IntLiteral { value: 42, .. }));
            }
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_binary_expression() {
    let source = "def test():\n    1 + 2\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => match expr {
                Expr::Binary { op, .. } => {
                    assert!(matches!(op, BinaryOp::Add));
                }
                _ => panic!("Expected binary expression"),
            },
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_function_call() {
    let source = "def test():\n    print(x, y)\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => match expr {
                Expr::Call { args, .. } => {
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected call expression"),
            },
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_struct_init() {
    let source = "def test():\n    Point { x: 10, y: 20 }\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => match expr {
                Expr::StructInit { path, fields, .. } => {
                    assert_eq!(path.segments[0].node, "Point");
                    assert_eq!(fields.len(), 2);
                }
                _ => panic!("Expected struct init expression"),
            },
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_field_access() {
    let source = "def test():\n    point.x\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => match expr {
                Expr::FieldAccess { field, .. } => {
                    assert_eq!(field.node, "x");
                }
                _ => panic!("Expected field access expression"),
            },
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Control Flow Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_if_expression() {
    let source = "def test():\n    if x > 0:\n        return 1\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::Expr(expr) => match expr {
                Expr::If { condition, then_block, else_block, .. } => {
                    assert!(matches!(condition.as_ref(), Expr::Binary { .. }));
                    assert!(!then_block.stmts.is_empty());
                    assert!(else_block.is_none());
                }
                _ => panic!("Expected if expression"),
            },
            _ => panic!("Expected expression statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_while_loop() {
    let source = "def test():\n    while true:\n        break\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => match &func.body.stmts[0] {
            Stmt::While { condition, body, .. } => {
                assert!(matches!(condition, Expr::BoolLiteral { value: true, .. }));
                assert!(!body.stmts.is_empty());
            }
            _ => panic!("Expected while statement"),
        },
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Type Parsing Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_primitive_types() {
    let source = "def test(a: i32, b: f64, c: bool, d: str):\n    return 0\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 4);
            // Types should be parsed correctly - ty is always present in params
            assert!(matches!(&func.params[0].ty, Type::Named { .. }));
        }
        _ => panic!("Expected function"),
    }
}

#[test]
#[ignore = "Generic type syntax Optional<T> not yet implemented"]
fn test_optional_type() {
    let source = "def test(x: Optional<i32>):\n    return 0\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            let ty = &func.params[0].ty;
            assert!(matches!(ty, Type::Optional { .. }));
        }
        _ => panic!("Expected function"),
    }
}

#[test]
#[ignore = "Array type syntax [T] not yet implemented"]
fn test_array_type() {
    let source = "def test(arr: [i32]):\n    return 0\n";
    let module = parse(source);

    match &module.items[0] {
        Item::Function(func) => {
            let ty = &func.params[0].ty;
            assert!(matches!(ty, Type::Array { .. }));
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Error Case Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_missing_function_body() {
    let errors = parse_should_fail("def test()");
    assert!(!errors.is_empty());
}

#[test]
fn test_invalid_expression() {
    let errors = parse_should_fail("def test():\n    + + +\n");
    assert!(!errors.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// Multiple Items Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_functions() {
    let source = "def a():\n    return 1\n\ndef b():\n    return 2\n";
    let module = parse(source);

    assert_eq!(module.items.len(), 2);
}

#[test]
fn test_mixed_items() {
    let source = r#"
struct Point:
    x: i32
    y: i32

def new_point(x: i32, y: i32) -> Point:
    return Point { x: x, y: y }
"#;
    let module = parse(source);

    assert_eq!(module.items.len(), 2);
    assert!(matches!(&module.items[0], Item::Struct(_)));
    assert!(matches!(&module.items[1], Item::Function(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// Exception Handling Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_try_catch() {
    let source = r#"
def test():
    try:
        let x = risky()
    catch:
        return 0
"#;
    let module = parse(source);
    assert_eq!(module.items.len(), 1);

    if let Item::Function(func) = &module.items[0] {
        assert_eq!(func.body.stmts.len(), 1);
        assert!(matches!(&func.body.stmts[0], Stmt::TryCatch { .. }));
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_try_except() {
    let source = r#"
def test():
    try:
        let x = risky()
    except:
        return 0
"#;
    let module = parse(source);

    if let Item::Function(func) = &module.items[0] {
        assert!(matches!(&func.body.stmts[0], Stmt::TryCatch { .. }));
    } else {
        panic!("Expected function");
    }
}

#[test]
fn test_try_catch_with_type() {
    let source = r#"
def test():
    try:
        let x = risky()
    catch ValueError:
        return 1
"#;
    let module = parse(source);

    if let Item::Function(func) = &module.items[0] {
        if let Stmt::TryCatch { handlers, .. } = &func.body.stmts[0] {
            assert_eq!(handlers.len(), 1);
            assert!(handlers[0].exception_type.is_some());
        } else {
            panic!("Expected TryCatch statement");
        }
    }
}

#[test]
fn test_try_catch_with_binding() {
    let source = r#"
def test():
    try:
        let x = risky()
    catch ValueError as e:
        print(e)
"#;
    let module = parse(source);

    if let Item::Function(func) = &module.items[0] {
        if let Stmt::TryCatch { handlers, .. } = &func.body.stmts[0] {
            assert_eq!(handlers.len(), 1);
            assert!(handlers[0].binding.is_some());
        } else {
            panic!("Expected TryCatch statement");
        }
    }
}

#[test]
fn test_try_finally() {
    let source = r#"
def test():
    try:
        let x = risky()
    catch:
        return 0
    finally:
        cleanup()
"#;
    let module = parse(source);

    if let Item::Function(func) = &module.items[0] {
        if let Stmt::TryCatch { finally_block, .. } = &func.body.stmts[0] {
            assert!(finally_block.is_some());
        } else {
            panic!("Expected TryCatch statement");
        }
    }
}

#[test]
fn test_raise_statement() {
    let source = r#"
def test():
    raise "error message"
"#;
    let module = parse(source);

    if let Item::Function(func) = &module.items[0] {
        assert!(matches!(&func.body.stmts[0], Stmt::Raise { .. }));
    } else {
        panic!("Expected function");
    }
}
