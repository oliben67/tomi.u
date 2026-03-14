//! Lexer for tomi.u source code.
//!
//! The lexer converts source text into a stream of tokens, handling:
//! - Keywords and identifiers (Unicode-aware)
//! - Operators and punctuation
//! - String literals with interpolation
//! - Number literals (int, float, hex, binary)
//! - Comments (single-line `#`, multi-line `###`)
//! - Indentation-based blocks

mod token;

pub use token::{Token, TokenKind};

use crate::error::CompileError;
use crate::span::Span;

/// The tomi.u lexer.
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    /// Stack of indentation levels
    indent_stack: Vec<usize>,
    /// Pending indent/dedent tokens
    pending_tokens: Vec<Token>,
    /// Are we at the start of a line?
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
            at_line_start: true,
        }
    }

    /// Tokenize the entire source, returning all tokens or errors.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Vec<CompileError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        loop {
            match self.next_token() {
                Ok(token) => {
                    let is_eof = token.kind == TokenKind::Eof;
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(e) => {
                    errors.push(e);
                    // Try to recover by skipping the problematic character
                    self.advance();
                }
            }
        }

        // Add remaining dedents at EOF
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.insert(
                tokens.len() - 1, // Before EOF
                Token::new(TokenKind::Dedent, Span::new(self.current_pos, self.current_pos)),
            );
        }

        if errors.is_empty() { Ok(tokens) } else { Err(errors) }
    }

    /// Get the next token.
    fn next_token(&mut self) -> Result<Token, CompileError> {
        // Return any pending tokens first
        if let Some(token) = self.pending_tokens.pop() {
            return Ok(token);
        }

        // Handle indentation at line start
        if self.at_line_start {
            self.at_line_start = false;
            if let Some(token) = self.handle_indentation()? {
                return Ok(token);
            }
        }

        // Skip whitespace (but not newlines)
        self.skip_horizontal_whitespace();

        let start = self.current_pos;

        let Some((_, ch)) = self.peek() else {
            return Ok(Token::new(TokenKind::Eof, Span::new(start, start)));
        };

        // Handle different token types
        let token = match ch {
            // Newlines
            '\n' => {
                self.advance();
                self.at_line_start = true;
                Token::new(TokenKind::Newline, Span::new(start, self.current_pos))
            }
            '\r' => {
                self.advance();
                if self.peek_char() == Some('\n') {
                    self.advance();
                }
                self.at_line_start = true;
                Token::new(TokenKind::Newline, Span::new(start, self.current_pos))
            }

            // Comments
            '#' => self.lex_comment()?,

            // Strings
            '"' => self.lex_string()?,

            // Numbers
            '0'..='9' => self.lex_number()?,

            // Identifiers and keywords
            c if is_ident_start(c) => self.lex_identifier(),

            // Operators and punctuation
            '+' => self.single_or_double(TokenKind::Plus, '=', TokenKind::PlusEq),
            '-' => self.lex_minus_or_arrow(),
            '*' => self.single_or_double(TokenKind::Star, '=', TokenKind::StarEq),
            '/' => self.single_or_double(TokenKind::Slash, '=', TokenKind::SlashEq),
            '%' => self.single_or_double(TokenKind::Percent, '=', TokenKind::PercentEq),
            '=' => self.lex_equals(),
            '!' => self.single_or_double(TokenKind::Bang, '=', TokenKind::BangEq),
            '<' => self.lex_less_than(),
            '>' => self.lex_greater_than(),
            '&' => self.single_or_double(TokenKind::Ampersand, '&', TokenKind::AmpAmp),
            '|' => self.single_or_double(TokenKind::Pipe, '|', TokenKind::PipePipe),
            '^' => self.single(TokenKind::Caret),
            '~' => self.single(TokenKind::Tilde),
            '.' => self.lex_dot(),
            ',' => self.single(TokenKind::Comma),
            ':' => self.single(TokenKind::Colon),
            ';' => self.single(TokenKind::Semicolon),
            '(' => self.single(TokenKind::LParen),
            ')' => self.single(TokenKind::RParen),
            '[' => self.single(TokenKind::LBracket),
            ']' => self.single(TokenKind::RBracket),
            '{' => self.single(TokenKind::LBrace),
            '}' => self.single(TokenKind::RBrace),
            '@' => self.single(TokenKind::At),
            '?' => self.single(TokenKind::Question),

            // Unknown character
            c => {
                return Err(CompileError::UnexpectedChar {
                    ch: c,
                    span: Span::new(start, start + c.len_utf8()),
                });
            }
        };

        Ok(token)
    }

    /// Handle indentation at the start of a line.
    fn handle_indentation(&mut self) -> Result<Option<Token>, CompileError> {
        let start = self.current_pos;

        // Count spaces at line start
        let mut spaces = 0;
        while let Some((_, ' ')) = self.peek() {
            self.advance();
            spaces += 1;
        }

        // Skip blank lines and comment-only lines
        if let Some((_, ch)) = self.peek() {
            if ch == '\n' || ch == '\r' || ch == '#' {
                return Ok(None);
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if spaces > current_indent {
            // Indent
            self.indent_stack.push(spaces);
            Ok(Some(Token::new(TokenKind::Indent, Span::new(start, self.current_pos))))
        } else if spaces < current_indent {
            // Dedent (possibly multiple levels)
            while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > spaces {
                self.indent_stack.pop();
                self.pending_tokens
                    .push(Token::new(TokenKind::Dedent, Span::new(start, self.current_pos)));
            }

            // Check for misaligned dedent
            if *self.indent_stack.last().unwrap() != spaces {
                return Err(CompileError::InconsistentIndentation {
                    span: Span::new(start, self.current_pos),
                });
            }

            // Return the first dedent, rest are pending
            Ok(self.pending_tokens.pop())
        } else {
            Ok(None)
        }
    }

    /// Lex a comment (single-line or multi-line).
    fn lex_comment(&mut self) -> Result<Token, CompileError> {
        let start = self.current_pos;
        self.advance(); // consume first #

        // Check for multi-line comment ###
        if self.peek_char() == Some('#') {
            self.advance();
            if self.peek_char() == Some('#') {
                self.advance();
                // Multi-line comment
                let content_start = self.current_pos;
                loop {
                    match self.peek() {
                        None => {
                            return Err(CompileError::UnterminatedComment {
                                span: Span::new(start, self.current_pos),
                            });
                        }
                        Some((_, '#')) => {
                            self.advance();
                            if self.peek_char() == Some('#') {
                                self.advance();
                                if self.peek_char() == Some('#') {
                                    self.advance();
                                    break;
                                }
                            }
                        }
                        Some(_) => {
                            self.advance();
                        }
                    }
                }
                return Ok(Token::new(TokenKind::Comment, Span::new(start, self.current_pos)));
            }
        }

        // Single-line comment: consume until end of line
        while let Some((_, ch)) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance();
        }

        Ok(Token::new(TokenKind::Comment, Span::new(start, self.current_pos)))
    }

    /// Lex a string literal (with interpolation support).
    fn lex_string(&mut self) -> Result<Token, CompileError> {
        let start = self.current_pos;
        self.advance(); // consume opening quote

        let mut has_interpolation = false;

        loop {
            match self.peek() {
                None | Some((_, '\n')) => {
                    return Err(CompileError::UnterminatedString {
                        span: Span::new(start, self.current_pos),
                    });
                }
                Some((_, '"')) => {
                    self.advance();
                    break;
                }
                Some((_, '\\')) => {
                    self.advance();
                    if let Some((pos, ch)) = self.peek() {
                        if !matches!(
                            ch,
                            'n' | 't' | 'r' | '\\' | '"' | '\'' | '0' | '{' | 'x' | 'u'
                        ) {
                            return Err(CompileError::InvalidEscape {
                                ch,
                                span: Span::new(pos, pos + ch.len_utf8()),
                            });
                        }
                        self.advance();
                        // Handle \xHH and \u{HHHH}
                        if ch == 'x' {
                            self.advance();
                            self.advance();
                        } else if ch == 'u' && self.peek_char() == Some('{') {
                            while self.peek_char() != Some('}') && self.peek().is_some() {
                                self.advance();
                            }
                            self.advance(); // consume }
                        }
                    }
                }
                Some((_, '{')) => {
                    has_interpolation = true;
                    self.advance();
                    // TODO: Track interpolation depth for nested braces
                }
                Some(_) => {
                    self.advance();
                }
            }
        }

        let kind =
            if has_interpolation { TokenKind::InterpolatedString } else { TokenKind::String };

        Ok(Token::new(kind, Span::new(start, self.current_pos)))
    }

    /// Lex a number literal.
    fn lex_number(&mut self) -> Result<Token, CompileError> {
        let start = self.current_pos;

        // Check for hex/binary/octal
        if self.peek_char() == Some('0') {
            self.advance();
            match self.peek_char() {
                Some('x') | Some('X') => {
                    self.advance();
                    while matches!(self.peek_char(), Some('0'..='9' | 'a'..='f' | 'A'..='F' | '_'))
                    {
                        self.advance();
                    }
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.current_pos),
                    ));
                }
                Some('b') | Some('B') => {
                    self.advance();
                    while matches!(self.peek_char(), Some('0' | '1' | '_')) {
                        self.advance();
                    }
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.current_pos),
                    ));
                }
                Some('o') | Some('O') => {
                    self.advance();
                    while matches!(self.peek_char(), Some('0'..='7' | '_')) {
                        self.advance();
                    }
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.current_pos),
                    ));
                }
                _ => {}
            }
        }

        // Decimal integer or float
        while matches!(self.peek_char(), Some('0'..='9' | '_')) {
            self.advance();
        }

        // Check for float
        let mut is_float = false;
        if self.peek_char() == Some('.') {
            // Look ahead to distinguish from method call (e.g., 123.to_string())
            let mut lookahead = self.chars.clone();
            lookahead.next(); // skip '.'
            if let Some((_, ch)) = lookahead.peek() {
                if ch.is_ascii_digit() {
                    is_float = true;
                    self.advance(); // consume '.'
                    while matches!(self.peek_char(), Some('0'..='9' | '_')) {
                        self.advance();
                    }
                }
            }
        }

        // Exponent
        if matches!(self.peek_char(), Some('e' | 'E')) {
            is_float = true;
            self.advance();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.advance();
            }
            while matches!(self.peek_char(), Some('0'..='9' | '_')) {
                self.advance();
            }
        }

        // Type suffix (e.g., 123i64, 3.14f32)
        if matches!(self.peek_char(), Some('i' | 'u' | 'f')) {
            self.advance();
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.advance();
            }
        }

        let kind = if is_float { TokenKind::FloatLiteral } else { TokenKind::IntLiteral };

        Ok(Token::new(kind, Span::new(start, self.current_pos)))
    }

    /// Lex an identifier or keyword.
    fn lex_identifier(&mut self) -> Token {
        let start = self.current_pos;

        while let Some((_, ch)) = self.peek() {
            if is_ident_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.current_pos];
        let kind = keyword_or_ident(text);

        Token::new(kind, Span::new(start, self.current_pos))
    }

    /// Lex `-` or `->`.
    fn lex_minus_or_arrow(&mut self) -> Token {
        let start = self.current_pos;
        self.advance();
        if self.peek_char() == Some('>') {
            self.advance();
            Token::new(TokenKind::Arrow, Span::new(start, self.current_pos))
        } else if self.peek_char() == Some('=') {
            self.advance();
            Token::new(TokenKind::MinusEq, Span::new(start, self.current_pos))
        } else {
            Token::new(TokenKind::Minus, Span::new(start, self.current_pos))
        }
    }

    /// Lex `=`, `==`, or `=>`.
    fn lex_equals(&mut self) -> Token {
        let start = self.current_pos;
        self.advance();
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::EqEq, Span::new(start, self.current_pos))
            }
            Some('>') => {
                self.advance();
                Token::new(TokenKind::FatArrow, Span::new(start, self.current_pos))
            }
            _ => Token::new(TokenKind::Eq, Span::new(start, self.current_pos)),
        }
    }

    /// Lex `<`, `<=`, `<<`, or `<<=`.
    fn lex_less_than(&mut self) -> Token {
        let start = self.current_pos;
        self.advance();
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::LtEq, Span::new(start, self.current_pos))
            }
            Some('<') => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::ShlEq, Span::new(start, self.current_pos))
                } else {
                    Token::new(TokenKind::Shl, Span::new(start, self.current_pos))
                }
            }
            _ => Token::new(TokenKind::Lt, Span::new(start, self.current_pos)),
        }
    }

    /// Lex `>`, `>=`, `>>`, or `>>=`.
    fn lex_greater_than(&mut self) -> Token {
        let start = self.current_pos;
        self.advance();
        match self.peek_char() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::GtEq, Span::new(start, self.current_pos))
            }
            Some('>') => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::ShrEq, Span::new(start, self.current_pos))
                } else {
                    Token::new(TokenKind::Shr, Span::new(start, self.current_pos))
                }
            }
            _ => Token::new(TokenKind::Gt, Span::new(start, self.current_pos)),
        }
    }

    /// Lex `.`, `..`, or `..=`.
    fn lex_dot(&mut self) -> Token {
        let start = self.current_pos;
        self.advance();
        if self.peek_char() == Some('.') {
            self.advance();
            if self.peek_char() == Some('=') {
                self.advance();
                Token::new(TokenKind::DotDotEq, Span::new(start, self.current_pos))
            } else {
                Token::new(TokenKind::DotDot, Span::new(start, self.current_pos))
            }
        } else {
            Token::new(TokenKind::Dot, Span::new(start, self.current_pos))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Helper methods
    // ═══════════════════════════════════════════════════════════════════════

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.peek().copied()
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.current_pos = pos + ch.len_utf8();
        }
        result
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\t')) {
            self.advance();
        }
    }

    /// Create a single-character token.
    fn single(&mut self, kind: TokenKind) -> Token {
        let start = self.current_pos;
        self.advance();
        Token::new(kind, Span::new(start, self.current_pos))
    }

    /// Create a one or two character token.
    fn single_or_double(&mut self, single: TokenKind, next: char, double: TokenKind) -> Token {
        let start = self.current_pos;
        self.advance();
        if self.peek_char() == Some(next) {
            self.advance();
            Token::new(double, Span::new(start, self.current_pos))
        } else {
            Token::new(single, Span::new(start, self.current_pos))
        }
    }
}

/// Check if a character can start an identifier.
fn is_ident_start(ch: char) -> bool {
    ch == '_' || unicode_xid::UnicodeXID::is_xid_start(ch)
}

/// Check if a character can continue an identifier.
fn is_ident_continue(ch: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(ch)
}

/// Convert an identifier string to a keyword token or Identifier.
fn keyword_or_ident(text: &str) -> TokenKind {
    match text {
        // Keywords
        "let" => TokenKind::Let,
        "mut" => TokenKind::Mut,
        "const" => TokenKind::Const,
        "def" => TokenKind::Def,
        "return" => TokenKind::Return,
        "if" => TokenKind::If,
        "elif" => TokenKind::Elif,
        "else" => TokenKind::Else,
        "for" => TokenKind::For,
        "while" => TokenKind::While,
        "loop" => TokenKind::Loop,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "match" => TokenKind::Match,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "trait" => TokenKind::Trait,
        "impl" => TokenKind::Impl,
        "type" => TokenKind::Type,
        "module" => TokenKind::Module,
        "import" => TokenKind::Import,
        "pub" => TokenKind::Pub,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "actor" => TokenKind::Actor,
        "spawn" => TokenKind::Spawn,
        "in" => TokenKind::In,
        "as" => TokenKind::As,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        "try" => TokenKind::Try,
        "catch" => TokenKind::Catch,
        "except" => TokenKind::Except,
        "finally" => TokenKind::Finally,
        "raise" => TokenKind::Raise,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "None" => TokenKind::None,
        "Some" => TokenKind::Some,
        "Ok" => TokenKind::Ok,
        "Err" => TokenKind::Err,
        "Self" => TokenKind::SelfType,
        "self" => TokenKind::SelfValue,

        // Not a keyword
        _ => TokenKind::Identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize().unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_simple_tokens() {
        assert_eq!(
            lex("let x = 42"),
            vec![
                TokenKind::Let,
                TokenKind::Identifier,
                TokenKind::Eq,
                TokenKind::IntLiteral,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            lex("+ - * / -> =>"),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Arrow,
                TokenKind::FatArrow,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(lex(r#""hello""#), vec![TokenKind::String, TokenKind::Eof]);
    }

    #[test]
    fn test_interpolated_string() {
        assert_eq!(lex(r#""hello {name}""#), vec![TokenKind::InterpolatedString, TokenKind::Eof]);
    }

    #[test]
    fn test_indentation() {
        assert_eq!(
            lex("if x:\n    y"),
            vec![
                TokenKind::If,
                TokenKind::Identifier,
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Identifier,
                TokenKind::Dedent,
                TokenKind::Eof
            ]
        );
    }
}
