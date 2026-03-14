//! Abstract Syntax Tree for tomi.u
//!
//! This module defines all AST node types produced by the parser.
//! Each node carries source location information via [`Span`] for error reporting.

use crate::span::{Span, Spanned};

// ═══════════════════════════════════════════════════════════════════════════════
// Top-Level Constructs
// ═══════════════════════════════════════════════════════════════════════════════

/// A complete tomi.u source file.
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name (from `module Name:`)
    pub name: Option<Spanned<String>>,
    /// Import statements
    pub imports: Vec<Import>,
    /// Top-level items
    pub items: Vec<Item>,
    pub span: Span,
}

/// An import statement.
#[derive(Debug, Clone)]
pub struct Import {
    /// The path being imported (e.g., ["std", "io"])
    pub path: Vec<Spanned<String>>,
    /// Optional alias (e.g., `import std.io as stdio`)
    pub alias: Option<Spanned<String>>,
    /// Specific items (e.g., `import std.io.{println, print}`)
    pub items: Option<Vec<Spanned<String>>>,
    pub span: Span,
}

/// A top-level item (function, struct, enum, etc.).
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    TypeAlias(TypeAlias),
    Const(Const),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Function(f) => f.span,
            Item::Struct(s) => s.span,
            Item::Enum(e) => e.span,
            Item::Trait(t) => t.span,
            Item::Impl(i) => i.span,
            Item::TypeAlias(t) => t.span,
            Item::Const(c) => c.span,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// A function definition.
#[derive(Debug, Clone)]
pub struct Function {
    /// Decorators (e.g., @entrypoint, @constructor)
    pub decorators: Vec<Decorator>,
    /// Visibility
    pub visibility: Visibility,
    /// Is this an async function?
    pub is_async: bool,
    /// Function name
    pub name: Spanned<String>,
    /// Generic type parameters
    pub type_params: Vec<TypeParam>,
    /// Function parameters
    pub params: Vec<Param>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function body
    pub body: Block,
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name
    pub name: Spanned<String>,
    /// Type annotation
    pub ty: Type,
    /// Is this a mutable parameter?
    pub is_mut: bool,
    /// Default value
    pub default: Option<Expr>,
    pub span: Span,
}

/// A decorator (e.g., @entrypoint, @constructor).
#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: Spanned<String>,
    /// Optional arguments (e.g., @timeout(30))
    pub args: Vec<Expr>,
    pub span: Span,
}

/// Visibility modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Private,
    Public,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A type reference.
#[derive(Debug, Clone)]
pub enum Type {
    /// Named type (e.g., Int32, String, MyStruct)
    Named(TypePath),
    /// Generic instantiation (e.g., List[Int32], Map[String, Value])
    Generic(TypePath, Vec<Type>),
    /// Function type (e.g., fn(Int32) -> String)
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Reference type (e.g., &String, &mut String)
    Reference {
        is_mut: bool,
        inner: Box<Type>,
    },
    /// Optional type (e.g., ?String shorthand for Option[String])
    Optional(Box<Type>),
    /// Tuple type (e.g., (Int32, String))
    Tuple(Vec<Type>),
    /// Array type (e.g., [Int32; 10])
    Array {
        element: Box<Type>,
        size: Option<Box<Expr>>,
    },
    /// Slice type (e.g., [Int32])
    Slice(Box<Type>),
    /// Unit type ()
    Unit,
    /// Never type (!)
    Never,
    /// Inferred type (_)
    Infer,
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Named(path) => path.span,
            Type::Generic(path, _) => path.span,
            // For other types, we'd need to track spans more carefully
            _ => Span::DUMMY,
        }
    }
}

/// A type path (e.g., std.collections.HashMap).
#[derive(Debug, Clone)]
pub struct TypePath {
    pub segments: Vec<Spanned<String>>,
    pub span: Span,
}

/// A generic type parameter (e.g., T, T: Trait).
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Spanned<String>,
    /// Trait bounds (e.g., T: Display + Clone)
    pub bounds: Vec<TypePath>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Structs and Enums
// ═══════════════════════════════════════════════════════════════════════════════

/// A struct definition.
#[derive(Debug, Clone)]
pub struct Struct {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub span: Span,
}

/// A struct field.
#[derive(Debug, Clone)]
pub struct Field {
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

/// An enum definition.
#[derive(Debug, Clone)]
pub struct Enum {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<Variant>,
    pub methods: Vec<Function>,
    pub span: Span,
}

/// An enum variant.
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Spanned<String>,
    /// Associated data (unit, tuple, or struct)
    pub data: VariantData,
    pub span: Span,
}

/// Data associated with an enum variant.
#[derive(Debug, Clone)]
pub enum VariantData {
    /// No data (e.g., `None`)
    Unit,
    /// Tuple data (e.g., `Some(T)`)
    Tuple(Vec<Type>),
    /// Struct data (e.g., `Point { x: Int32, y: Int32 }`)
    Struct(Vec<Field>),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Traits and Implementations
// ═══════════════════════════════════════════════════════════════════════════════

/// A trait definition.
#[derive(Debug, Clone)]
pub struct Trait {
    pub decorators: Vec<Decorator>,
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub type_params: Vec<TypeParam>,
    /// Super traits
    pub bounds: Vec<TypePath>,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

/// An item in a trait definition.
#[derive(Debug, Clone)]
pub enum TraitItem {
    /// A required or provided method
    Method(Function),
    /// An associated type
    Type {
        name: Spanned<String>,
        bounds: Vec<TypePath>,
        span: Span,
    },
}

/// A trait implementation.
#[derive(Debug, Clone)]
pub struct Impl {
    pub type_params: Vec<TypeParam>,
    /// The trait being implemented (None for inherent impl)
    pub trait_path: Option<TypePath>,
    /// The type implementing the trait
    pub target: Type,
    pub items: Vec<Function>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type Aliases and Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// A type alias.
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub type_params: Vec<TypeParam>,
    pub ty: Type,
    pub span: Span,
}

/// A constant definition.
#[derive(Debug, Clone)]
pub struct Const {
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════════════════════════════════════

/// A block of statements.
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Let binding (let x = expr)
    Let {
        is_mut: bool,
        pattern: Pattern,
        ty: Option<Type>,
        value: Option<Expr>,
        span: Span,
    },
    /// Expression statement
    Expr(Expr),
    /// Return statement
    Return {
        value: Option<Expr>,
        span: Span,
    },
    /// Break statement
    Break {
        label: Option<Spanned<String>>,
        value: Option<Expr>,
        span: Span,
    },
    /// Continue statement
    Continue {
        label: Option<Spanned<String>>,
        span: Span,
    },
    /// For loop
    For {
        pattern: Pattern,
        iterable: Expr,
        body: Block,
        span: Span,
    },
    /// While loop
    While {
        condition: Expr,
        body: Block,
        span: Span,
    },
    /// Loop (infinite)
    Loop {
        label: Option<Spanned<String>>,
        body: Block,
        span: Span,
    },
    /// Assignment (x = expr, x += expr, etc.)
    Assignment {
        target: Expr,
        op: AssignOp,
        value: Expr,
        span: Span,
    },
    /// Try-catch/except statement for exception handling
    /// Supports both `catch` (Java/JS style) and `except` (Python style)
    TryCatch {
        try_block: Block,
        /// Exception handlers - can use either `catch` or `except` keyword
        handlers: Vec<ExceptionHandler>,
        finally_block: Option<Block>,
        span: Span,
    },
    /// Raise/throw statement
    Raise {
        exception: Expr,
        span: Span,
    },
}

/// An exception handler for try-catch/except blocks
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// Optional exception type to catch
    pub exception_type: Option<Type>,
    /// Optional binding name for the caught exception
    pub binding: Option<Spanned<String>>,
    /// Handler body
    pub body: Block,
    pub span: Span,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let { span, .. } => *span,
            Stmt::Expr(e) => e.span(),
            Stmt::Return { span, .. } => *span,
            Stmt::Break { span, .. } => *span,
            Stmt::Continue { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::Loop { span, .. } => *span,
            Stmt::Assignment { span, .. } => *span,
            Stmt::TryCatch { span, .. } => *span,
            Stmt::Raise { span, .. } => *span,
        }
    }
}

/// Assignment operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
    ShlAssign, // <<=
    ShrAssign, // >>=
}

// ═══════════════════════════════════════════════════════════════════════════════
// Expressions
// ═══════════════════════════════════════════════════════════════════════════════

/// An expression.
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLiteral {
        value: i128,
        suffix: Option<String>,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        suffix: Option<String>,
        span: Span,
    },
    StringLiteral {
        value: String,
        span: Span,
    },
    InterpolatedString {
        parts: Vec<StringPart>,
        span: Span,
    },
    BoolLiteral {
        value: bool,
        span: Span,
    },
    CharLiteral {
        value: char,
        span: Span,
    },

    // Variables and paths
    Identifier {
        name: String,
        span: Span,
    },
    Path {
        segments: Vec<Spanned<String>>,
        span: Span,
    },

    // Compound expressions
    Tuple {
        elements: Vec<Expr>,
        span: Span,
    },
    Array {
        elements: Vec<Expr>,
        span: Span,
    },
    StructInit {
        path: TypePath,
        fields: Vec<FieldInit>,
        span: Span,
    },

    // Operators
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },

    // Access
    FieldAccess {
        object: Box<Expr>,
        field: Spanned<String>,
        span: Span,
    },
    MethodCall {
        object: Box<Expr>,
        method: Spanned<String>,
        type_args: Vec<Type>,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    // Calls
    Call {
        callee: Box<Expr>,
        type_args: Vec<Type>,
        args: Vec<Expr>,
        span: Span,
    },

    // Control flow
    If {
        condition: Box<Expr>,
        then_block: Block,
        elif_clauses: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    // References and dereferencing
    Reference {
        is_mut: bool,
        inner: Box<Expr>,
        span: Span,
    },
    Deref {
        inner: Box<Expr>,
        span: Span,
    },

    // Ranges
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },

    // Lambda
    Lambda {
        params: Vec<LambdaParam>,
        return_type: Option<Type>,
        body: Box<Expr>,
        span: Span,
    },

    // Async/await
    Await {
        inner: Box<Expr>,
        span: Span,
    },

    // Error propagation
    Try {
        inner: Box<Expr>,
        span: Span,
    },

    // Type cast
    Cast {
        expr: Box<Expr>,
        ty: Type,
        span: Span,
    },

    // Block expression
    Block {
        block: Block,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLiteral { span, .. } => *span,
            Expr::FloatLiteral { span, .. } => *span,
            Expr::StringLiteral { span, .. } => *span,
            Expr::InterpolatedString { span, .. } => *span,
            Expr::BoolLiteral { span, .. } => *span,
            Expr::CharLiteral { span, .. } => *span,
            Expr::Identifier { span, .. } => *span,
            Expr::Path { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::StructInit { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Reference { span, .. } => *span,
            Expr::Deref { span, .. } => *span,
            Expr::Range { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::Try { span, .. } => *span,
            Expr::Cast { span, .. } => *span,
            Expr::Block { span, .. } => *span,
        }
    }
}

/// A part of an interpolated string.
#[derive(Debug, Clone)]
pub enum StringPart {
    /// Literal text
    Literal(String),
    /// Interpolated expression
    Expr(Expr),
}

/// A field initializer in a struct literal.
#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: Spanned<String>,
    pub value: Option<Expr>, // None means shorthand (name: name)
    pub span: Span,
}

/// A lambda parameter.
#[derive(Debug, Clone)]
pub struct LambdaParam {
    pub name: Spanned<String>,
    pub ty: Option<Type>,
    pub span: Span,
}

/// A match arm.
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Operators
// ═══════════════════════════════════════════════════════════════════════════════

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,    // ! or not
    Neg,    // -
    BitNot, // ~
    Ref,    // &
    MutRef, // &mut
    Deref,  // *
}

// ═══════════════════════════════════════════════════════════════════════════════
// Patterns
// ═══════════════════════════════════════════════════════════════════════════════

/// A pattern for matching.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard (_)
    Wildcard { span: Span },
    /// Variable binding
    Identifier {
        is_mut: bool,
        name: Spanned<String>,
        span: Span,
    },
    /// Literal pattern
    Literal { value: Expr, span: Span },
    /// Tuple pattern
    Tuple { elements: Vec<Pattern>, span: Span },
    /// Struct pattern
    Struct {
        path: TypePath,
        fields: Vec<PatternField>,
        rest: bool, // .. at end
        span: Span,
    },
    /// Enum variant pattern
    Variant {
        path: TypePath,
        data: Option<Box<Pattern>>,
        span: Span,
    },
    /// Slice pattern
    Slice {
        elements: Vec<Pattern>,
        rest: Option<Box<Pattern>>,
        span: Span,
    },
    /// Or pattern (a | b)
    Or {
        patterns: Vec<Pattern>,
        span: Span,
    },
    /// Range pattern (1..10, 'a'..='z')
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
    /// Binding pattern (name @ pattern)
    Binding {
        name: Spanned<String>,
        pattern: Box<Pattern>,
        span: Span,
    },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span } => *span,
            Pattern::Identifier { span, .. } => *span,
            Pattern::Literal { span, .. } => *span,
            Pattern::Tuple { span, .. } => *span,
            Pattern::Struct { span, .. } => *span,
            Pattern::Variant { span, .. } => *span,
            Pattern::Slice { span, .. } => *span,
            Pattern::Or { span, .. } => *span,
            Pattern::Range { span, .. } => *span,
            Pattern::Binding { span, .. } => *span,
        }
    }
}

/// A field in a struct pattern.
#[derive(Debug, Clone)]
pub struct PatternField {
    pub name: Spanned<String>,
    pub pattern: Option<Pattern>, // None means shorthand
    pub span: Span,
}
