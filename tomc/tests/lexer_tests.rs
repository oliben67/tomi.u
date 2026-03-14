//! Lexer tests for tomc

use tomc::lexer::Lexer;
use tomc::lexer::TokenKind;

/// Helper to tokenize and return token kinds only
fn tokenize(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(source);
    lexer
        .tokenize()
        .expect("Tokenization should succeed")
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Helper to get text of tokens
fn tokenize_with_text(source: &str) -> Vec<(TokenKind, String)> {
    let mut lexer = Lexer::new(source);
    lexer
        .tokenize()
        .expect("Tokenization should succeed")
        .into_iter()
        .map(|t| (t.kind, source[t.span.start..t.span.end].to_string()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// Basic Token Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_source() {
    let tokens = tokenize("");
    assert_eq!(tokens, vec![TokenKind::Eof]);
}

#[test]
fn test_whitespace_only() {
    let tokens = tokenize("   \t  ");
    // Whitespace-only input may produce Indent/Dedent tokens depending on lexer implementation
    assert!(tokens.contains(&TokenKind::Eof));
}

#[test]
fn test_single_newline() {
    let tokens = tokenize("\n");
    assert_eq!(tokens, vec![TokenKind::Newline, TokenKind::Eof]);
}

// ═══════════════════════════════════════════════════════════════════════
// Keyword Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_keywords() {
    let keywords = vec![
        ("let", TokenKind::Let),
        ("mut", TokenKind::Mut),
        ("const", TokenKind::Const),
        ("def", TokenKind::Def),
        ("return", TokenKind::Return),
        ("if", TokenKind::If),
        ("elif", TokenKind::Elif),
        ("else", TokenKind::Else),
        ("for", TokenKind::For),
        ("while", TokenKind::While),
        ("loop", TokenKind::Loop),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("match", TokenKind::Match),
        ("struct", TokenKind::Struct),
        ("enum", TokenKind::Enum),
        ("trait", TokenKind::Trait),
        ("impl", TokenKind::Impl),
        ("type", TokenKind::Type),
        ("module", TokenKind::Module),
        ("import", TokenKind::Import),
        ("pub", TokenKind::Pub),
        ("async", TokenKind::Async),
        ("await", TokenKind::Await),
        ("try", TokenKind::Try),
        ("catch", TokenKind::Catch),
        ("except", TokenKind::Except),
        ("finally", TokenKind::Finally),
        ("raise", TokenKind::Raise),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
    ];

    for (text, expected_kind) in keywords {
        let tokens = tokenize(text);
        assert_eq!(
            tokens[0], expected_kind,
            "Expected keyword '{text}' to produce {:?}",
            expected_kind
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Identifier Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_simple_identifier() {
    let tokens = tokenize_with_text("foo");
    assert_eq!(tokens[0], (TokenKind::Identifier, "foo".to_string()));
}

#[test]
fn test_identifier_with_underscore() {
    let tokens = tokenize_with_text("foo_bar");
    assert_eq!(tokens[0], (TokenKind::Identifier, "foo_bar".to_string()));
}

#[test]
fn test_identifier_starting_with_underscore() {
    let tokens = tokenize_with_text("_private");
    assert_eq!(tokens[0], (TokenKind::Identifier, "_private".to_string()));
}

#[test]
fn test_identifier_with_numbers() {
    let tokens = tokenize_with_text("var123");
    assert_eq!(tokens[0], (TokenKind::Identifier, "var123".to_string()));
}

#[test]
fn test_unicode_identifier() {
    let tokens = tokenize_with_text("变量");
    assert_eq!(tokens[0], (TokenKind::Identifier, "变量".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// Literal Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_integer_literal() {
    let tokens = tokenize_with_text("42");
    assert_eq!(tokens[0], (TokenKind::IntLiteral, "42".to_string()));
}

#[test]
fn test_hex_literal() {
    let tokens = tokenize_with_text("0x2A");
    assert_eq!(tokens[0], (TokenKind::IntLiteral, "0x2A".to_string()));
}

#[test]
fn test_binary_literal() {
    let tokens = tokenize_with_text("0b101010");
    assert_eq!(tokens[0], (TokenKind::IntLiteral, "0b101010".to_string()));
}

#[test]
fn test_float_literal() {
    let tokens = tokenize_with_text("3.14");
    assert_eq!(tokens[0], (TokenKind::FloatLiteral, "3.14".to_string()));
}

#[test]
fn test_string_literal() {
    let tokens = tokenize_with_text("\"hello world\"");
    assert_eq!(tokens[0], (TokenKind::String, "\"hello world\"".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// Operator Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_arithmetic_operators() {
    let tokens = tokenize("+ - * / %");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_comparison_operators() {
    let tokens = tokenize("== != < > <= >=");
    assert_eq!(
        tokens,
        vec![
            TokenKind::EqEq,
            TokenKind::BangEq,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_logical_operators() {
    let tokens = tokenize("&& || !");
    assert_eq!(
        tokens,
        vec![
            TokenKind::AmpAmp,
            TokenKind::PipePipe,
            TokenKind::Bang,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_assignment_operators() {
    let tokens = tokenize("= += -= *= /=");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Eq,
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::Eof
        ]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Delimiter Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_delimiters() {
    let tokens = tokenize("( ) [ ] { }");
    assert_eq!(
        tokens,
        vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_punctuation() {
    let tokens = tokenize(": , . ;");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Colon,
            TokenKind::Comma,
            TokenKind::Dot,
            TokenKind::Semicolon,
            TokenKind::Eof
        ]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Comment Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_line_comment() {
    let tokens = tokenize("# This is a comment\nlet x = 1");
    // Comments are tokenized but filtered by parser
    assert!(tokens.contains(&TokenKind::Comment));
    assert!(tokens.contains(&TokenKind::Let));
}

// ═══════════════════════════════════════════════════════════════════════
// Indentation Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_indent_dedent() {
    let source = "def main():\n    return 1\n";
    let tokens = tokenize(source);
    
    assert!(tokens.contains(&TokenKind::Def));
    assert!(tokens.contains(&TokenKind::Indent));
    assert!(tokens.contains(&TokenKind::Return));
    assert!(tokens.contains(&TokenKind::Dedent));
}

// ═══════════════════════════════════════════════════════════════════════
// Arrow Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_arrows() {
    let tokens = tokenize("-> =>");
    assert_eq!(
        tokens,
        vec![TokenKind::Arrow, TokenKind::FatArrow, TokenKind::Eof]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Decorator Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_decorator() {
    let tokens = tokenize("@entrypoint");
    assert_eq!(
        tokens,
        vec![TokenKind::At, TokenKind::Identifier, TokenKind::Eof]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Complex Expression Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_function_call() {
    let tokens = tokenize("print(x, y)");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Identifier, // print
            TokenKind::LParen,
            TokenKind::Identifier, // x
            TokenKind::Comma,
            TokenKind::Identifier, // y
            TokenKind::RParen,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_struct_literal() {
    let tokens = tokenize("Point { x: 10, y: 20 }");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Identifier, // Point
            TokenKind::LBrace,
            TokenKind::Identifier, // x
            TokenKind::Colon,
            TokenKind::IntLiteral, // 10
            TokenKind::Comma,
            TokenKind::Identifier, // y
            TokenKind::Colon,
            TokenKind::IntLiteral, // 20
            TokenKind::RBrace,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_let_statement() {
    let tokens = tokenize("let x = 42");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Let,
            TokenKind::Identifier, // x
            TokenKind::Eq,
            TokenKind::IntLiteral, // 42
            TokenKind::Eof
        ]
    );
}
