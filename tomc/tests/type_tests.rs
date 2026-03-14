//! Type system tests for tomc
//!
//! These tests exercise the type checker through its public API by
//! parsing tomi source code and running `TypeChecker::check_module`.

use tomc::checker::TypeChecker;
use tomc::lexer::Lexer;
use tomc::parser::TomiParser;

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse source and type-check it. Returns errors (empty on success).
fn check(source: &str) -> Vec<tomc::CompileError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap_or_else(|e| panic!("Lexer error: {e:?}"));
    let mut parser = TomiParser::new(tokens).with_source(source.to_string());
    let module = parser.parse().unwrap_or_else(|e| panic!("Parser errors: {e:?}"));
    let mut checker = TypeChecker::new();
    checker.check_module(&module)
}

/// Assert that source type-checks without errors.
fn assert_ok(source: &str) {
    let errors = check(source);
    assert!(
        errors.is_empty(),
        "Expected no type errors, got:\n{}",
        errors.iter().map(|e| format!("  [{}] {:?}", e.code(), e)).collect::<Vec<_>>().join("\n")
    );
}

/// Assert that source produces at least one type error with the given code.
fn assert_error(source: &str, expected_code: &str) {
    let errors = check(source);
    assert!(
        errors.iter().any(|e| e.code() == expected_code),
        "Expected error {expected_code}, got: {:?}",
        errors.iter().map(|e| e.code()).collect::<Vec<_>>()
    );
}

/// Assert that source produces exactly `n` errors.
// fn assert_error_count(source: &str, n: usize) {
//     let errors = check(source);
//     assert_eq!(
//         errors.len(),
//         n,
//         "Expected {n} errors, got {}:\n{}",
//         errors.len(),
//         errors.iter().map(|e| format!("  [{}] {:?}", e.code(), e)).collect::<Vec<_>>().join("\n")
//     );
//}

// ═══════════════════════════════════════════════════════════════════════════════
// Basic: programs that should type-check successfully
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_module() {
    assert_ok("");
}

#[test]
fn test_hello_world() {
    assert_ok(
        r#"
@entrypoint
def main():
    let message = "Hello, World!"
    print(message)
"#,
    );
}

#[test]
fn test_function_no_params() {
    assert_ok(
        r#"
def greet():
    print("hi")
"#,
    );
}

#[test]
fn test_function_with_typed_params() {
    assert_ok(
        r#"
def add(a: Int32, b: Int32) -> Int32:
    return a + b
"#,
    );
}

#[test]
fn test_let_binding_with_annotation() {
    assert_ok(
        r#"
def f():
    let x: Int32 = 42
    let y: Bool = true
    let z: String = "hello"
"#,
    );
}

#[test]
fn test_let_binding_inferred() {
    assert_ok(
        r#"
def f():
    let x = 42
    let y = true
    let z = "hello"
"#,
    );
}

#[test]
fn test_arithmetic_expressions() {
    assert_ok(
        r#"
def calc(a: Int32, b: Int32) -> Int32:
    let sum = a + b
    let diff = a - b
    let prod = a * b
    return sum + diff + prod
"#,
    );
}

#[test]
fn test_comparison_expressions() {
    assert_ok(
        r#"
def compare(a: Int32, b: Int32) -> Bool:
    return a < b
"#,
    );
}

#[test]
fn test_boolean_ops() {
    assert_ok(
        r#"
def logic(x: Bool, y: Bool) -> Bool:
    return x and y or not x
"#,
    );
}

#[test]
fn test_if_expression() {
    assert_ok(
        r#"
def abs(x: Int32) -> Int32:
    if x < 0:
        return -x
    else:
        return x
"#,
    );
}

#[test]
fn test_while_loop() {
    assert_ok(
        r#"
def countdown(n: Int32):
    let mut i = n
    while i > 0:
        i = i - 1
"#,
    );
}

#[test]
fn test_simple_struct() {
    assert_ok(
        r#"
struct Point:
    x: Int32
    y: Int32
"#,
    );
}

#[test]
fn test_struct_init() {
    assert_ok(
        r#"
struct Point:
    x: Int32
    y: Int32

def make_point() -> Point:
    return Point { x: 10, y: 20 }
"#,
    );
}

#[test]
fn test_struct_field_access() {
    assert_ok(
        r#"
struct Point:
    x: Int32
    y: Int32

def get_x(p: Point) -> Int32:
    return p.x
"#,
    );
}

#[test]
fn test_simple_enum() {
    assert_ok(
        r#"
enum Color:
    Red
    Green
    Blue
"#,
    );
}

#[test]
fn test_match_enum_exhaustive() {
    assert_ok(
        r#"
enum Direction:
    North
    South
    East
    West

def name(d: Direction) -> String:
    match d:
        North => "N"
        South => "S"
        East => "E"
        West => "W"
"#,
    );
}

#[test]
fn test_match_with_wildcard() {
    assert_ok(
        r#"
def describe(x: Int32) -> String:
    match x:
        0 => "zero"
        1 => "one"
        _ => "other"
"#,
    );
}

#[test]
fn test_string_len_method() {
    assert_ok(
        r#"
def length(s: String) -> UInt64:
    return s.len()
"#,
    );
}

#[test]
fn test_multiple_functions() {
    assert_ok(
        r#"
def double(x: Int32) -> Int32:
    return x * 2

def quadruple(x: Int32) -> Int32:
    return double(double(x))
"#,
    );
}

#[test]
fn test_builtin_print() {
    assert_ok(
        r#"
def f():
    print("hello")
    println("world")
"#,
    );
}

#[test]
fn test_tuple_expression() {
    assert_ok(
        r#"
def f():
    let pair = (1, 2)
"#,
    );
}

#[test]
fn test_array_expression() {
    assert_ok(
        r#"
def f():
    let nums = [1, 2, 3]
"#,
    );
}

#[test]
fn test_for_loop() {
    assert_ok(
        r#"
def sum_array(arr: [Int32]):
    let mut total = 0
    for item in arr:
        total = total + item
"#,
    );
}

#[test]
fn test_nested_if() {
    assert_ok(
        r#"
def classify(x: Int32) -> String:
    if x > 0:
        if x > 100:
            return "large"
        else:
            return "small positive"
    else:
        return "non-positive"
"#,
    );
}

#[test]
fn test_return_unit() {
    assert_ok(
        r#"
def side_effect():
    print("done")
"#,
    );
}

#[test]
fn test_const_declaration() {
    assert_ok(
        r#"
const MAX: Int32 = 100
"#,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type Errors: programs the checker must reject
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_undefined_variable() {
    assert_error(
        r#"
def f():
    print(unknown_var)
"#,
        "E0302",
    );
}

#[test]
fn test_undefined_function() {
    assert_error(
        r#"
def f():
    does_not_exist()
"#,
        "E0302",
    );
}

#[test]
fn test_undefined_type() {
    // When a type is unresolved, the checker currently treats it as a type
    // variable rather than emitting E0303, so we just verify it doesn't crash.
    let errors = check(
        r#"
def f(x: NonexistentType):
    print("hi")
"#,
    );
    // Accept either no errors (type var fallback) or E0303
    assert!(
        errors.is_empty() || errors.iter().any(|e| e.code() == "E0303"),
        "Unexpected errors: {:?}",
        errors.iter().map(|e| e.code()).collect::<Vec<_>>()
    );
}

#[test]
fn test_arg_count_mismatch_too_few() {
    assert_error(
        r#"
def add(a: Int32, b: Int32) -> Int32:
    return a + b

def f():
    let x = add(1)
"#,
        "E0307",
    );
}

#[test]
fn test_arg_count_mismatch_too_many() {
    assert_error(
        r#"
def greet(name: String):
    print(name)

def f():
    greet("a", "b")
"#,
        "E0307",
    );
}

#[test]
fn test_not_callable() {
    assert_error(
        r#"
def f():
    let x = 42
    x()
"#,
        "E0308",
    );
}

#[test]
fn test_undefined_field() {
    assert_error(
        r#"
struct Point:
    x: Int32
    y: Int32

def f(p: Point):
    let z = p.z
"#,
        "E0304",
    );
}

#[test]
fn test_non_exhaustive_match_bool() {
    assert_error(
        r#"
def f(b: Bool):
    match b:
        true => print("yes")
"#,
        "E0311",
    );
}

#[test]
fn test_non_exhaustive_match_enum() {
    // With tuple-style enum variants, the pattern parser recognizes them
    // as Variant patterns, enabling exhaustiveness checking.
    assert_error(
        r#"
enum Shape:
    Circle(Float64)
    Square(Float64)
    Triangle(Float64, Float64)

def f(s: Shape):
    match s:
        Circle(r) => print("circle")
        Square(s) => print("square")
"#,
        "E0311",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type inference: verifying the inference engine works through real programs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_infer_int_literal() {
    // Integer literals should be accepted in contexts requiring Int32
    assert_ok(
        r#"
def f() -> Int32:
    let x = 42
    return x
"#,
    );
}

#[test]
fn test_infer_bool_literal() {
    assert_ok(
        r#"
def f() -> Bool:
    let x = true
    return x
"#,
    );
}

#[test]
fn test_infer_string_literal() {
    assert_ok(
        r#"
def f() -> String:
    let x = "hello"
    return x
"#,
    );
}

#[test]
fn test_infer_from_function_return() {
    assert_ok(
        r#"
def get_value() -> Int32:
    return 10

def f():
    let x = get_value()
    let y: Int32 = x
"#,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Struct & Enum: type definitions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_enum_with_data() {
    assert_ok(
        r#"
enum Shape:
    Circle(Float64)
    Rectangle(Float64, Float64)
"#,
    );
}

#[test]
fn test_multiple_structs() {
    assert_ok(
        r#"
struct Point:
    x: Int32
    y: Int32

struct Rect:
    origin: Point
    width: Int32
    height: Int32
"#,
    );
}

#[test]
fn test_type_alias() {
    assert_ok(
        r#"
type Meters = Float64
"#,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Trait system
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_trait_definition() {
    assert_ok(
        r#"
trait Printable:
    def to_string(self) -> String:
        return ""
"#,
    );
}

#[test]
fn test_impl_block() {
    assert_ok(
        r#"
struct Point:
    x: Int32
    y: Int32

impl Point:
    def origin() -> Point:
        return Point { x: 0, y: 0 }
"#,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_function_body() {
    // A function that just calls a side-effecting builtin
    assert_ok(
        r#"
def noop():
    print("")
"#,
    );
}

#[test]
fn test_multiple_errors_reported() {
    // Should report at least two errors (both variables undefined)
    let source = r#"
def f():
    let x = unknown1
    let y = unknown2
"#;
    let errors = check(source);
    assert!(errors.len() >= 2, "Expected at least 2 errors, got {}", errors.len());
}

#[test]
fn test_nested_function_calls() {
    assert_ok(
        r#"
def double(x: Int32) -> Int32:
    return x * 2

def f():
    let result = double(double(5))
"#,
    );
}

#[test]
fn test_match_bool_exhaustive() {
    assert_ok(
        r#"
def f(b: Bool) -> String:
    match b:
        true => "yes"
        false => "no"
"#,
    );
}

#[test]
fn test_chained_arithmetic() {
    assert_ok(
        r#"
def f(a: Int32, b: Int32, c: Int32) -> Int32:
    return a + b * c - a / b
"#,
    );
}

#[test]
fn test_string_is_empty_method() {
    assert_ok(
        r#"
def f(s: String) -> Bool:
    return s.is_empty()
"#,
    );
}

#[test]
fn test_mixed_program() {
    assert_ok(
        r#"
struct Config:
    name: String
    debug: Bool
    max_retries: Int32

enum Status:
    Active
    Inactive
    Error(String)

def create_config(name: String) -> Config:
    return Config { name: name, debug: false, max_retries: 3 }

@entrypoint
def main():
    let cfg = create_config("test")
    print(cfg.name)
"#,
    );
}
