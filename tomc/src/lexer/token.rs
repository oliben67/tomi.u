//! Token definitions for the tomi.u lexer.

use crate::span::Span;

/// A token from the lexer with its kind and source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Check if this is a specific token kind.
    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    /// Get the source text for this token.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}

/// All possible token types in tomi.u.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ═══════════════════════════════════════════════════════════════════════
    // Literals
    // ═══════════════════════════════════════════════════════════════════════
    /// Integer literal (42, 0x2A, 0b101010)
    IntLiteral,
    /// Floating-point literal (3.14, 1e10)
    FloatLiteral,
    /// String literal ("hello")
    String,
    /// Interpolated string ("hello {name}")
    InterpolatedString,
    /// Character literal ('a')
    Char,

    // ═══════════════════════════════════════════════════════════════════════
    // Identifiers and Keywords
    // ═══════════════════════════════════════════════════════════════════════
    /// User identifier
    Identifier,

    // Keywords
    Let,
    Mut,
    Const,
    Def,
    Return,
    If,
    Elif,
    Else,
    For,
    While,
    Loop,
    Break,
    Continue,
    Match,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Module,
    Import,
    Pub,
    Async,
    Await,
    Actor,
    Spawn,
    In,
    As,
    And,
    Or,
    Not,
    // Exception handling
    Try,
    Catch,
    Except,
    Finally,
    Raise,

    // Built-in type/value keywords
    True,
    False,
    None,
    Some,
    Ok,
    Err,
    SelfType,  // Self
    SelfValue, // self

    // ═══════════════════════════════════════════════════════════════════════
    // Operators
    // ═══════════════════════════════════════════════════════════════════════
    // Arithmetic
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %

    // Assignment
    Eq,        // =
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=
    PercentEq, // %=
    ShlEq,     // <<=
    ShrEq,     // >>=

    // Comparison
    EqEq,  // ==
    BangEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // Logical
    AmpAmp,   // &&
    PipePipe, // ||
    Bang,     // !

    // Bitwise
    Ampersand, // &
    Pipe,      // |
    Caret,     // ^
    Tilde,     // ~
    Shl,       // <<
    Shr,       // >>

    // ═══════════════════════════════════════════════════════════════════════
    // Punctuation
    // ═══════════════════════════════════════════════════════════════════════
    Dot,      // .
    DotDot,   // ..
    DotDotEq, // ..=
    Comma,    // ,
    Colon,    // :
    Semicolon, // ;
    Arrow,    // ->
    FatArrow, // =>
    At,       // @
    Question, // ?

    // Brackets
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    // ═══════════════════════════════════════════════════════════════════════
    // Special
    // ═══════════════════════════════════════════════════════════════════════
    /// Newline (for statement separation)
    Newline,
    /// Indentation increase
    Indent,
    /// Indentation decrease
    Dedent,
    /// Comment (skipped but tracked for formatting)
    Comment,
    /// End of file
    Eof,
}

impl TokenKind {
    /// Get a human-readable name for this token kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IntLiteral => "integer literal",
            Self::FloatLiteral => "float literal",
            Self::String => "string literal",
            Self::InterpolatedString => "interpolated string",
            Self::Char => "character literal",
            Self::Identifier => "identifier",
            Self::Let => "'let'",
            Self::Mut => "'mut'",
            Self::Const => "'const'",
            Self::Def => "'def'",
            Self::Return => "'return'",
            Self::If => "'if'",
            Self::Elif => "'elif'",
            Self::Else => "'else'",
            Self::For => "'for'",
            Self::While => "'while'",
            Self::Loop => "'loop'",
            Self::Break => "'break'",
            Self::Continue => "'continue'",
            Self::Match => "'match'",
            Self::Struct => "'struct'",
            Self::Enum => "'enum'",
            Self::Trait => "'trait'",
            Self::Impl => "'impl'",
            Self::Type => "'type'",
            Self::Module => "'module'",
            Self::Import => "'import'",
            Self::Pub => "'pub'",
            Self::Async => "'async'",
            Self::Await => "'await'",
            Self::Actor => "'actor'",
            Self::Spawn => "'spawn'",
            Self::In => "'in'",
            Self::As => "'as'",
            Self::And => "'and'",
            Self::Or => "'or'",
            Self::Not => "'not'",
            Self::Try => "'try'",
            Self::Catch => "'catch'",
            Self::Except => "'except'",
            Self::Finally => "'finally'",
            Self::Raise => "'raise'",
            Self::True => "'true'",
            Self::False => "'false'",
            Self::None => "'None'",
            Self::Some => "'Some'",
            Self::Ok => "'Ok'",
            Self::Err => "'Err'",
            Self::SelfType => "'Self'",
            Self::SelfValue => "'self'",
            Self::Plus => "'+'",
            Self::Minus => "'-'",
            Self::Star => "'*'",
            Self::Slash => "'/'",
            Self::Percent => "'%'",
            Self::Eq => "'='",
            Self::PlusEq => "'+='",
            Self::MinusEq => "'-='",
            Self::StarEq => "'*='",
            Self::SlashEq => "'/='",
            Self::PercentEq => "'%='",
            Self::ShlEq => "'<<='",
            Self::ShrEq => "'>>='",
            Self::EqEq => "'=='",
            Self::BangEq => "'!='",
            Self::Lt => "'<'",
            Self::LtEq => "'<='",
            Self::Gt => "'>'",
            Self::GtEq => "'>='",
            Self::AmpAmp => "'&&'",
            Self::PipePipe => "'||'",
            Self::Bang => "'!'",
            Self::Ampersand => "'&'",
            Self::Pipe => "'|'",
            Self::Caret => "'^'",
            Self::Tilde => "'~'",
            Self::Shl => "'<<'",
            Self::Shr => "'>>'",
            Self::Dot => "'.'",
            Self::DotDot => "'..'",
            Self::DotDotEq => "'..='",
            Self::Comma => "','",
            Self::Colon => "':'",
            Self::Semicolon => "';'",
            Self::Arrow => "'->'",
            Self::FatArrow => "'=>'",
            Self::At => "'@'",
            Self::Question => "'?'",
            Self::LParen => "'('",
            Self::RParen => "')'",
            Self::LBracket => "'['",
            Self::RBracket => "']'",
            Self::LBrace => "'{'",
            Self::RBrace => "'}'",
            Self::Newline => "newline",
            Self::Indent => "indent",
            Self::Dedent => "dedent",
            Self::Comment => "comment",
            Self::Eof => "end of file",
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
