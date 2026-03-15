//! Recursive descent parser for tomi.u
//!
//! This parser converts a token stream into an AST. It uses recursive descent
//! with operator precedence parsing for expressions.

use crate::ast::*;
use crate::error::CompileError;
use crate::lexer::{Token, TokenKind};
use crate::span::{Span, Spanned};

/// The tomi.u parser.
pub struct TomiParser {
    tokens: Vec<Token>,
    position: usize,
    /// Source code for error messages
    source: Option<String>,
}

impl TomiParser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        // Filter out comments and newlines at non-significant positions
        let tokens: Vec<_> = tokens
            .into_iter()
            .filter(|t| t.kind != TokenKind::Comment)
            .collect();

        Self {
            tokens,
            position: 0,
            source: None,
        }
    }

    /// Set the source code (for extracting literal values).
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Parse the entire module.
    pub fn parse(&mut self) -> Result<Module, Vec<CompileError>> {
        let mut errors = Vec::new();
        let start = self.current_span();

        // Skip leading newlines
        self.skip_newlines();

        // Parse module declaration if present
        let name = if self.check(TokenKind::Module) {
            match self.parse_module_declaration() {
                Ok(name) => Some(name),
                Err(e) => {
                    errors.push(e);
                    None
                }
            }
        } else {
            None
        };

        // Parse imports
        let mut imports = Vec::new();
        while self.check(TokenKind::Import) {
            match self.parse_import() {
                Ok(import) => imports.push(import),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        // Parse items
        let mut items = Vec::new();
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        let end = self.previous_span();

        if errors.is_empty() {
            Ok(Module {
                name,
                imports,
                items,
                span: start.merge(end),
            })
        } else {
            Err(errors)
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Top-Level Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_module_declaration(&mut self) -> Result<Spanned<String>, CompileError> {
        let _start = self.current_span();
        self.expect(TokenKind::Module)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        self.expect_newline_or_eof()?;
        Ok(name)
    }

    fn parse_import(&mut self) -> Result<Import, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Import)?;

        // Parse path segments
        let mut path = vec![self.parse_identifier()?];
        while self.check(TokenKind::Dot) {
            self.advance();
            path.push(self.parse_identifier()?);
        }

        // Check for alias or specific imports
        let alias = if self.check(TokenKind::As) {
            self.advance();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        // TODO: Parse `import std.io.{println, print}` syntax

        self.expect_newline_or_eof()?;

        let end = self.previous_span();
        Ok(Import {
            path,
            alias,
            items: None,
            span: start.merge(end),
        })
    }

    fn parse_item(&mut self) -> Result<Item, CompileError> {
        // Parse decorators
        let decorators = self.parse_decorators()?;

        // Parse visibility
        let visibility = self.parse_visibility();

        // Parse is_async for functions
        let is_async = self.check(TokenKind::Async);
        if is_async {
            self.advance();
        }

        match self.current_kind() {
            TokenKind::Def => {
                let func = self.parse_function(decorators, visibility, is_async)?;
                Ok(Item::Function(func))
            }
            TokenKind::Struct => {
                let s = self.parse_struct(decorators, visibility)?;
                Ok(Item::Struct(s))
            }
            TokenKind::Enum => {
                let e = self.parse_enum(decorators, visibility)?;
                Ok(Item::Enum(e))
            }
            TokenKind::Trait => {
                let t = self.parse_trait(decorators, visibility)?;
                Ok(Item::Trait(t))
            }
            TokenKind::Impl => {
                let i = self.parse_impl()?;
                Ok(Item::Impl(i))
            }
            TokenKind::Type => {
                let t = self.parse_type_alias(visibility)?;
                Ok(Item::TypeAlias(t))
            }
            TokenKind::Const => {
                let c = self.parse_const(visibility)?;
                Ok(Item::Const(c))
            }
            _ => Err(CompileError::ExpectedToken {
                expected: "item (def, struct, enum, trait, impl, type, const)".into(),
                found: self.current_kind().as_str().into(),
                span: self.current_span(),
            }),
        }
    }

    fn parse_decorators(&mut self) -> Result<Vec<Decorator>, CompileError> {
        let mut decorators = Vec::new();

        while self.check(TokenKind::At) {
            let start = self.current_span();
            self.advance();

            let name = self.parse_identifier()?;

            // Parse optional arguments
            let args = if self.check(TokenKind::LParen) {
                self.advance();
                let args =
                    self.parse_comma_separated(TokenKind::RParen, |p| p.parse_expression())?;
                self.expect(TokenKind::RParen)?;
                args
            } else {
                Vec::new()
            };

            let end = self.previous_span();
            decorators.push(Decorator {
                name,
                args,
                span: start.merge(end),
            });

            self.skip_newlines();
        }

        Ok(decorators)
    }

    fn parse_visibility(&mut self) -> Visibility {
        if self.check(TokenKind::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Function Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_function(
        &mut self,
        decorators: Vec<Decorator>,
        visibility: Visibility,
        is_async: bool,
    ) -> Result<Function, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Def)?;

        let name = self.parse_identifier()?;

        // Parse type parameters
        let type_params = self.parse_type_params()?;

        // Parse parameters
        self.expect(TokenKind::LParen)?;
        let params = self.parse_comma_separated(TokenKind::RParen, |p| p.parse_param())?;
        self.expect(TokenKind::RParen)?;

        // Parse return type
        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse body — trait method declarations may omit the colon and body
        let body = if self.check(TokenKind::Colon) {
            self.advance();
            self.parse_block()?
        } else {
            // Bodyless declaration (e.g. trait method signature)
            self.expect_newline()?;
            Block {
                stmts: Vec::new(),
                span: self.previous_span(),
            }
        };

        let end = self.previous_span();

        Ok(Function {
            decorators,
            visibility,
            is_async,
            name,
            type_params,
            params,
            return_type,
            body,
            span: start.merge(end),
        })
    }

    fn parse_param(&mut self) -> Result<Param, CompileError> {
        let start = self.current_span();

        // Check for mut
        let is_mut = if self.check(TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        // Check for self
        if self.check(TokenKind::SelfValue) {
            let name_span = self.current_span();
            self.advance();
            return Ok(Param {
                name: Spanned::new("self".into(), name_span),
                ty: Type::Named(TypePath {
                    segments: vec![Spanned::new("Self".into(), name_span)],
                    span: name_span,
                }),
                is_mut,
                default: None,
                span: start.merge(self.previous_span()),
            });
        }

        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        // Parse default value
        let default = if self.check(TokenKind::Eq) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous_span();
        Ok(Param {
            name,
            ty,
            is_mut,
            default,
            span: start.merge(end),
        })
    }

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, CompileError> {
        if !self.check(TokenKind::LBracket) {
            return Ok(Vec::new());
        }
        self.advance();

        let params = self.parse_comma_separated(TokenKind::RBracket, |p| p.parse_type_param())?;
        self.expect(TokenKind::RBracket)?;

        Ok(params)
    }

    fn parse_type_param(&mut self) -> Result<TypeParam, CompileError> {
        let start = self.current_span();
        let name = self.parse_identifier()?;

        // Parse bounds
        let bounds = if self.check(TokenKind::Colon) {
            self.advance();
            let mut bounds = vec![self.parse_type_path()?];
            while self.check(TokenKind::Plus) {
                self.advance();
                bounds.push(self.parse_type_path()?);
            }
            bounds
        } else {
            Vec::new()
        };

        let end = self.previous_span();
        Ok(TypeParam {
            name,
            bounds,
            span: start.merge(end),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Struct/Enum Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_struct(
        &mut self,
        decorators: Vec<Decorator>,
        visibility: Visibility,
    ) -> Result<Struct, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Struct)?;

        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::Colon)?;

        // Parse body (fields and methods)
        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }

            let item_decorators = self.parse_decorators()?;
            let item_visibility = self.parse_visibility();

            if self.check(TokenKind::Def) || self.check(TokenKind::Async) {
                let is_async = self.check(TokenKind::Async);
                if is_async {
                    self.advance();
                }
                methods.push(self.parse_function(item_decorators, item_visibility, is_async)?);
            } else {
                // Field
                let field_start = self.current_span();
                let field_name = self.parse_identifier()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type()?;

                let default = if self.check(TokenKind::Eq) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                self.expect_newline_or_dedent()?;

                let field_end = self.previous_span();
                fields.push(Field {
                    visibility: item_visibility,
                    name: field_name,
                    ty,
                    default,
                    span: field_start.merge(field_end),
                });
            }
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Struct {
            decorators,
            visibility,
            name,
            type_params,
            fields,
            methods,
            span: start.merge(end),
        })
    }

    fn parse_enum(
        &mut self,
        decorators: Vec<Decorator>,
        visibility: Visibility,
    ) -> Result<Enum, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Enum)?;

        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::Colon)?;

        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut variants = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }

            if self.check(TokenKind::Def) || self.check(TokenKind::At) || self.check(TokenKind::Pub)
            {
                let item_decorators = self.parse_decorators()?;
                let item_visibility = self.parse_visibility();
                methods.push(self.parse_function(item_decorators, item_visibility, false)?);
            } else {
                // Variant
                let var_start = self.current_span();
                let var_name = self.parse_identifier()?;

                let data = if self.check(TokenKind::LParen) {
                    self.advance();
                    let types =
                        self.parse_comma_separated(TokenKind::RParen, |p| p.parse_type())?;
                    self.expect(TokenKind::RParen)?;
                    VariantData::Tuple(types)
                } else {
                    VariantData::Unit
                };

                self.expect_newline_or_dedent()?;

                let var_end = self.previous_span();
                variants.push(Variant {
                    name: var_name,
                    data,
                    span: var_start.merge(var_end),
                });
            }
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Enum {
            decorators,
            visibility,
            name,
            type_params,
            variants,
            methods,
            span: start.merge(end),
        })
    }

    fn parse_trait(
        &mut self,
        decorators: Vec<Decorator>,
        visibility: Visibility,
    ) -> Result<Trait, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Trait)?;

        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;

        // Parse supertraits
        let bounds = if self.check(TokenKind::Colon) {
            // Actually need to check if it's bounds or just block start
            // For now, simplified
            Vec::new()
        } else {
            Vec::new()
        };

        self.expect(TokenKind::Colon)?;
        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut items = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }

            let item_decorators = self.parse_decorators()?;
            let method = self.parse_function(item_decorators, Visibility::Public, false)?;
            items.push(TraitItem::Method(method));
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Trait {
            decorators,
            visibility,
            name,
            type_params,
            bounds,
            items,
            span: start.merge(end),
        })
    }

    fn parse_impl(&mut self) -> Result<Impl, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Impl)?;

        let type_params = self.parse_type_params()?;

        // Parse trait path or target type
        let first_type = self.parse_type()?;

        // Check for "for Type" (trait impl)
        let (trait_path, target) = if self.check(TokenKind::For) {
            self.advance();
            let target = self.parse_type()?;
            let trait_path = match first_type {
                Type::Named(path) => Some(path),
                _ => {
                    return Err(CompileError::ExpectedToken {
                        expected: "trait name".into(),
                        found: "type".into(),
                        span: start,
                    });
                }
            };
            (trait_path, target)
        } else {
            (None, first_type)
        };

        self.expect(TokenKind::Colon)?;
        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut items = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }

            let decorators = self.parse_decorators()?;
            let visibility = self.parse_visibility();
            let is_async = self.check(TokenKind::Async);
            if is_async {
                self.advance();
            }
            items.push(self.parse_function(decorators, visibility, is_async)?);
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Impl {
            type_params,
            trait_path,
            target,
            items,
            span: start.merge(end),
        })
    }

    fn parse_type_alias(&mut self, visibility: Visibility) -> Result<TypeAlias, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Type)?;

        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        self.expect(TokenKind::Eq)?;
        let ty = self.parse_type()?;

        self.expect_newline_or_eof()?;

        let end = self.previous_span();
        Ok(TypeAlias {
            visibility,
            name,
            type_params,
            ty,
            span: start.merge(end),
        })
    }

    fn parse_const(&mut self, visibility: Visibility) -> Result<Const, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Const)?;

        let name = self.parse_identifier()?;

        let ty = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression()?;

        self.expect_newline_or_eof()?;

        let end = self.previous_span();
        Ok(Const {
            visibility,
            name,
            ty,
            value,
            span: start.merge(end),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Type Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_type(&mut self) -> Result<Type, CompileError> {
        // Check for reference types
        if self.check(TokenKind::Ampersand) {
            let _start = self.current_span();
            self.advance();
            let is_mut = self.check(TokenKind::Mut);
            if is_mut {
                self.advance();
            }
            let inner = self.parse_type()?;
            return Ok(Type::Reference {
                is_mut,
                inner: Box::new(inner),
            });
        }

        // Check for optional types
        if self.check(TokenKind::Question) {
            self.advance();
            let inner = self.parse_type()?;
            return Ok(Type::Optional(Box::new(inner)));
        }

        // Check for tuple/unit
        if self.check(TokenKind::LParen) {
            self.advance();
            if self.check(TokenKind::RParen) {
                self.advance();
                return Ok(Type::Unit);
            }

            let types = self.parse_comma_separated(TokenKind::RParen, |p| p.parse_type())?;
            self.expect(TokenKind::RParen)?;

            if types.len() == 1 {
                return Ok(types.into_iter().next().unwrap());
            }
            return Ok(Type::Tuple(types));
        }

        // Check for array/slice
        if self.check(TokenKind::LBracket) {
            self.advance();
            let element = self.parse_type()?;

            if self.check(TokenKind::Semicolon) {
                self.advance();
                let size = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                return Ok(Type::Array {
                    element: Box::new(element),
                    size: Some(Box::new(size)),
                });
            }

            self.expect(TokenKind::RBracket)?;
            return Ok(Type::Slice(Box::new(element)));
        }

        // Check for function type
        if self.check(TokenKind::Def) {
            self.advance();
            self.expect(TokenKind::LParen)?;
            let params = self.parse_comma_separated(TokenKind::RParen, |p| p.parse_type())?;
            self.expect(TokenKind::RParen)?;
            self.expect(TokenKind::Arrow)?;
            let return_type = self.parse_type()?;
            return Ok(Type::Function {
                params,
                return_type: Box::new(return_type),
            });
        }

        // Named type (possibly generic)
        let path = self.parse_type_path()?;

        // Check for generics
        if self.check(TokenKind::LBracket) {
            self.advance();
            let args = self.parse_comma_separated(TokenKind::RBracket, |p| p.parse_type())?;
            self.expect(TokenKind::RBracket)?;
            return Ok(Type::Generic(path, args));
        }

        Ok(Type::Named(path))
    }

    fn parse_type_path(&mut self) -> Result<TypePath, CompileError> {
        let start = self.current_span();
        let mut segments = vec![self.parse_identifier()?];

        while self.check(TokenKind::Dot) {
            self.advance();
            segments.push(self.parse_identifier()?);
        }

        let end = self.previous_span();
        Ok(TypePath {
            segments,
            span: start.merge(end),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Statement Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_block(&mut self) -> Result<Block, CompileError> {
        let start = self.current_span();

        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut stmts = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Block {
            stmts,
            span: start.merge(end),
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, CompileError> {
        let result = match self.current_kind() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Mut => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Loop => self.parse_loop(),
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Raise => self.parse_raise(),
            _ => {
                // Expression statement or assignment
                let expr = self.parse_expression()?;

                // Check for assignment
                if let Some(op) = self.parse_assign_op() {
                    let start = expr.span();
                    let value = self.parse_expression()?;
                    let end = self.previous_span();

                    return Ok(Stmt::Assignment {
                        target: expr,
                        op,
                        value,
                        span: start.merge(end),
                    });
                }

                Ok(Stmt::Expr(expr))
            }
        };

        // Consume trailing newline
        if result.is_ok() {
            self.skip_newlines();
        }

        result
    }

    fn parse_let(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();

        // Check for mut
        let is_mut = if self.check(TokenKind::Mut) {
            self.advance();
            true
        } else {
            self.expect(TokenKind::Let)?;
            false
        };

        let pattern = self.parse_pattern()?;

        let ty = if self.check(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.check(TokenKind::Eq) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous_span();
        Ok(Stmt::Let {
            is_mut,
            pattern,
            ty,
            value,
            span: start.merge(end),
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Return)?;

        let value = if !self.check(TokenKind::Newline)
            && !self.check(TokenKind::Dedent)
            && !self.is_at_end()
        {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous_span();
        Ok(Stmt::Return {
            value,
            span: start.merge(end),
        })
    }

    fn parse_break(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Break)?;

        // TODO: Parse label and value
        let end = self.previous_span();
        Ok(Stmt::Break {
            label: None,
            value: None,
            span: start.merge(end),
        })
    }

    fn parse_continue(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Continue)?;

        let end = self.previous_span();
        Ok(Stmt::Continue {
            label: None,
            span: start.merge(end),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::For)?;

        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;

        let end = self.previous_span();
        Ok(Stmt::For {
            pattern,
            iterable,
            body,
            span: start.merge(end),
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::While)?;

        let condition = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;

        let end = self.previous_span();
        Ok(Stmt::While {
            condition,
            body,
            span: start.merge(end),
        })
    }

    fn parse_loop(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Loop)?;

        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;

        let end = self.previous_span();
        Ok(Stmt::Loop {
            label: None,
            body,
            span: start.merge(end),
        })
    }

    /// Parse a try-catch/except statement
    /// Supports both syntaxes:
    /// - try: ... catch ExceptionType as e: ... finally: ...
    /// - try: ... except ExceptionType as e: ... finally: ...
    fn parse_try_catch(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Try)?;
        self.expect(TokenKind::Colon)?;

        let try_block = self.parse_block()?;

        let mut handlers = Vec::new();

        // Parse catch or except handlers
        while self.check(TokenKind::Catch) || self.check(TokenKind::Except) {
            let handler_start = self.current_span();
            self.advance(); // consume 'catch' or 'except'

            // Optional exception type
            let exception_type = if !self.check(TokenKind::Colon) && !self.check(TokenKind::As) {
                Some(self.parse_type()?)
            } else {
                None
            };

            // Optional binding (as e)
            let binding = if self.check(TokenKind::As) {
                self.advance();
                Some(self.parse_identifier()?)
            } else {
                None
            };

            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            let handler_end = self.previous_span();

            handlers.push(ExceptionHandler {
                exception_type,
                binding,
                body,
                span: handler_start.merge(handler_end),
            });
        }

        // Optional finally block
        let finally_block = if self.check(TokenKind::Finally) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = self.previous_span();
        Ok(Stmt::TryCatch {
            try_block,
            handlers,
            finally_block,
            span: start.merge(end),
        })
    }

    /// Parse a raise statement
    fn parse_raise(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Raise)?;

        let exception = self.parse_expression()?;

        let end = self.previous_span();
        Ok(Stmt::Raise {
            exception,
            span: start.merge(end),
        })
    }

    fn parse_assign_op(&mut self) -> Option<AssignOp> {
        let op = match self.current_kind() {
            TokenKind::Eq => AssignOp::Assign,
            TokenKind::PlusEq => AssignOp::AddAssign,
            TokenKind::MinusEq => AssignOp::SubAssign,
            TokenKind::StarEq => AssignOp::MulAssign,
            TokenKind::SlashEq => AssignOp::DivAssign,
            TokenKind::PercentEq => AssignOp::ModAssign,
            TokenKind::ShlEq => AssignOp::ShlAssign,
            TokenKind::ShrEq => AssignOp::ShrAssign,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Expression Parsing (Pratt parser)
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_expression(&mut self) -> Result<Expr, CompileError> {
        let expr = self.parse_expr_with_precedence(0)?;

        // Handle range operators (lowest precedence, outside Pratt loop)
        if self.check(TokenKind::DotDot) || self.check(TokenKind::DotDotEq) {
            let inclusive = self.check(TokenKind::DotDotEq);
            self.advance();
            let start_span = expr.span();
            // Parse the end of the range (if present)
            let end = if !self.check(TokenKind::Colon)
                && !self.check(TokenKind::Newline)
                && !self.check(TokenKind::Eof)
                && !self.check(TokenKind::RParen)
                && !self.check(TokenKind::RBracket)
                && !self.check(TokenKind::RBrace)
                && !self.check(TokenKind::Comma)
            {
                Some(Box::new(self.parse_expr_with_precedence(0)?))
            } else {
                None
            };
            let end_span = end.as_ref().map(|e| e.span()).unwrap_or(start_span);
            let span = start_span.merge(end_span);
            return Ok(Expr::Range {
                start: Some(Box::new(expr)),
                end,
                inclusive,
                span,
            });
        }

        Ok(expr)
    }

    fn parse_expr_with_precedence(&mut self, min_prec: u8) -> Result<Expr, CompileError> {
        let mut left = self.parse_unary()?;

        while let Some((op, prec, assoc)) = self.binary_op_info() {
            if prec < min_prec {
                break;
            }

            self.advance();

            let right_prec = if assoc == Assoc::Left { prec + 1 } else { prec };
            let right = self.parse_expr_with_precedence(right_prec)?;

            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();

        // Unary operators
        let op = match self.current_kind() {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Bang | TokenKind::Not => Some(UnaryOp::Not),
            TokenKind::Tilde => Some(UnaryOp::BitNot),
            TokenKind::Star => Some(UnaryOp::Deref),
            TokenKind::Ampersand => {
                self.advance();
                let is_mut = self.check(TokenKind::Mut);
                if is_mut {
                    self.advance();
                }
                let inner = self.parse_unary()?;
                let span = start.merge(inner.span());
                return Ok(Expr::Reference {
                    is_mut,
                    inner: Box::new(inner),
                    span,
                });
            }
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            let span = start.merge(operand.span());
            return Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
                span,
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current_kind() {
                // Field access or method call
                TokenKind::Dot => {
                    self.advance();
                    let field = self.parse_identifier()?;

                    if self.check(TokenKind::LParen) {
                        // Method call
                        self.advance();
                        let args = self
                            .parse_comma_separated(TokenKind::RParen, |p| p.parse_expression())?;
                        self.expect(TokenKind::RParen)?;

                        let span = expr.span().merge(self.previous_span());
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: field,
                            type_args: Vec::new(),
                            args,
                            span,
                        };
                    } else {
                        // Field access
                        let span = expr.span().merge(field.span);
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field,
                            span,
                        };
                    }
                }

                // Index
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;

                    let span = expr.span().merge(self.previous_span());
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span,
                    };
                }

                // Function call
                TokenKind::LParen => {
                    self.advance();
                    let args =
                        self.parse_comma_separated(TokenKind::RParen, |p| p.parse_expression())?;
                    self.expect(TokenKind::RParen)?;

                    let span = expr.span().merge(self.previous_span());
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        type_args: Vec::new(),
                        args,
                        span,
                    };
                }

                // Try operator
                TokenKind::Question => {
                    self.advance();
                    let span = expr.span().merge(self.previous_span());
                    expr = Expr::Try {
                        inner: Box::new(expr),
                        span,
                    };
                }

                // Await
                TokenKind::Await if !matches!(expr, Expr::Identifier { .. }) => {
                    // This handles expr.await syntax - not standard but could be supported
                    break;
                }

                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();

        match self.current_kind() {
            // Literals
            TokenKind::IntLiteral => {
                let text = self.current_text();
                let value = self.parse_int_literal(&text)?;
                self.advance();
                Ok(Expr::IntLiteral {
                    value,
                    suffix: None,
                    span: start,
                })
            }

            TokenKind::FloatLiteral => {
                let text = self.current_text();
                let value: f64 = text
                    .replace('_', "")
                    .parse()
                    .map_err(|_| CompileError::InvalidNumber { span: start })?;
                self.advance();
                Ok(Expr::FloatLiteral {
                    value,
                    suffix: None,
                    span: start,
                })
            }

            TokenKind::String => {
                let text = self.current_text();
                // Remove quotes and unescape
                let value = text[1..text.len() - 1].to_string();
                self.advance();
                Ok(Expr::StringLiteral { value, span: start })
            }

            TokenKind::InterpolatedString => {
                // TODO: Parse interpolated string properly
                let text = self.current_text();
                let value = text[1..text.len() - 1].to_string();
                self.advance();
                Ok(Expr::StringLiteral { value, span: start })
            }

            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLiteral {
                    value: true,
                    span: start,
                })
            }

            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLiteral {
                    value: false,
                    span: start,
                })
            }

            // Identifiers and paths
            TokenKind::Identifier | TokenKind::SelfValue | TokenKind::SelfType => {
                let name = self.current_text().to_string();
                self.advance();

                // Check for path syntax: Name::Variant or Name::Variant(args)
                if self.check(TokenKind::ColonColon) {
                    let mut segments = vec![Spanned::new(name, start)];
                    while self.check(TokenKind::ColonColon) {
                        self.advance(); // consume '::'
                        let seg = self.parse_identifier()?;
                        segments.push(seg);
                    }
                    let end_span = segments.last().map(|s| s.span).unwrap_or(start);
                    let span = start.merge(end_span);

                    // Check for constructor call: Path::Variant(args)
                    if self.check(TokenKind::LParen) {
                        self.advance();
                        let args = self
                            .parse_comma_separated(TokenKind::RParen, |p| p.parse_expression())?;
                        self.expect(TokenKind::RParen)?;
                        let call_span = start.merge(self.previous_span());
                        return Ok(Expr::Call {
                            callee: Box::new(Expr::Path { segments, span }),
                            type_args: Vec::new(),
                            args,
                            span: call_span,
                        });
                    }

                    // Check for struct init: Path::Variant { field: value }
                    if self.check(TokenKind::LBrace) {
                        self.advance();
                        let fields = self.parse_comma_separated(TokenKind::RBrace, |p| {
                            let field_start = p.current_span();
                            let field_name = p.parse_identifier()?;
                            p.expect(TokenKind::Colon)?;
                            let value = p.parse_expression()?;
                            let field_span = field_start.merge(p.previous_span());
                            Ok(FieldInit {
                                name: field_name,
                                value: Some(value),
                                span: field_span,
                            })
                        })?;
                        self.expect(TokenKind::RBrace)?;
                        let init_span = start.merge(self.previous_span());
                        return Ok(Expr::StructInit {
                            path: TypePath {
                                segments: segments.clone(),
                                span,
                            },
                            fields,
                            span: init_span,
                        });
                    }

                    return Ok(Expr::Path { segments, span });
                }

                // Check for struct init: Name { field: value }
                if self.check(TokenKind::LBrace) {
                    self.advance(); // consume '{'
                    let fields = self.parse_comma_separated(TokenKind::RBrace, |p| {
                        let field_start = p.current_span();
                        let field_name = p.parse_identifier()?;
                        p.expect(TokenKind::Colon)?;
                        let value = p.parse_expression()?;
                        let field_span = field_start.merge(p.previous_span());
                        Ok(FieldInit {
                            name: field_name,
                            value: Some(value),
                            span: field_span,
                        })
                    })?;
                    self.expect(TokenKind::RBrace)?;
                    let span = start.merge(self.previous_span());
                    Ok(Expr::StructInit {
                        path: TypePath {
                            segments: vec![Spanned::new(name, start)],
                            span: start,
                        },
                        fields,
                        span,
                    })
                } else {
                    Ok(Expr::Identifier { name, span: start })
                }
            }

            // Result constructors
            TokenKind::Ok | TokenKind::Err | TokenKind::Some | TokenKind::None => {
                let name = self.current_text().to_string();
                self.advance();

                if self.check(TokenKind::LParen) {
                    self.advance();
                    let args =
                        self.parse_comma_separated(TokenKind::RParen, |p| p.parse_expression())?;
                    self.expect(TokenKind::RParen)?;

                    let span = start.merge(self.previous_span());
                    Ok(Expr::Call {
                        callee: Box::new(Expr::Identifier { name, span: start }),
                        type_args: Vec::new(),
                        args,
                        span,
                    })
                } else {
                    Ok(Expr::Identifier { name, span: start })
                }
            }

            // Parenthesized expression or tuple
            TokenKind::LParen => {
                self.advance();

                if self.check(TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr::Tuple {
                        elements: Vec::new(),
                        span: start.merge(self.previous_span()),
                    });
                }

                let first = self.parse_expression()?;

                if self.check(TokenKind::Comma) {
                    // Tuple
                    self.advance();
                    let mut elements = vec![first];
                    while !self.check(TokenKind::RParen) && !self.is_at_end() {
                        elements.push(self.parse_expression()?);
                        if !self.check(TokenKind::RParen) {
                            self.expect(TokenKind::Comma)?;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    let span = start.merge(self.previous_span());
                    Ok(Expr::Tuple { elements, span })
                } else {
                    self.expect(TokenKind::RParen)?;
                    Ok(first)
                }
            }

            // Array literal
            TokenKind::LBracket => {
                self.advance();
                let elements =
                    self.parse_comma_separated(TokenKind::RBracket, |p| p.parse_expression())?;
                self.expect(TokenKind::RBracket)?;

                let span = start.merge(self.previous_span());
                Ok(Expr::Array { elements, span })
            }

            // If expression
            TokenKind::If => self.parse_if_expression(),

            // Match expression
            TokenKind::Match => self.parse_match_expression(),

            // Lambda
            TokenKind::Pipe => self.parse_lambda(),

            // Await
            TokenKind::Await => {
                self.advance();
                let inner = self.parse_expression()?;
                let span = start.merge(inner.span());
                Ok(Expr::Await {
                    inner: Box::new(inner),
                    span,
                })
            }

            _ => Err(CompileError::ExpectedExpression { span: start }),
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::If)?;

        let condition = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        let then_block = self.parse_block()?;

        // Parse elif clauses
        let mut elif_clauses = Vec::new();
        while self.check(TokenKind::Elif) {
            self.advance();
            let elif_cond = self.parse_expression()?;
            self.expect(TokenKind::Colon)?;
            let elif_block = self.parse_block()?;
            elif_clauses.push((elif_cond, elif_block));
        }

        // Parse else
        let else_block = if self.check(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = self.previous_span();
        Ok(Expr::If {
            condition: Box::new(condition),
            then_block,
            elif_clauses,
            else_block,
            span: start.merge(end),
        })
    }

    fn parse_match_expression(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Match)?;

        let scrutinee = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        self.expect_newline()?;
        self.expect(TokenKind::Indent)?;

        let mut arms = Vec::new();

        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(TokenKind::Dedent) {
                break;
            }

            let arm_start = self.current_span();
            let pattern = self.parse_pattern()?;

            // Optional guard
            let guard = if self.check(TokenKind::If) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expression()?;

            let arm_end = self.previous_span();
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_start.merge(arm_end),
            });

            self.skip_newlines();
        }

        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end = self.previous_span();
        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: start.merge(end),
        })
    }

    fn parse_lambda(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        self.expect(TokenKind::Pipe)?;

        let params = self.parse_comma_separated(TokenKind::Pipe, |p| {
            let name = p.parse_identifier()?;
            let ty = if p.check(TokenKind::Colon) {
                p.advance();
                Some(p.parse_type()?)
            } else {
                None
            };
            let span = name.span;
            Ok(LambdaParam { name, ty, span })
        })?;
        self.expect(TokenKind::Pipe)?;

        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Colon)?;
        let body = self.parse_expression()?;

        let span = start.merge(body.span());
        Ok(Expr::Lambda {
            params,
            return_type,
            body: Box::new(body),
            span,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Pattern Parsing
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_pattern(&mut self) -> Result<Pattern, CompileError> {
        let start = self.current_span();

        // Wildcard
        if self.current_text() == "_" {
            self.advance();
            return Ok(Pattern::Wildcard { span: start });
        }

        // Mut binding
        let is_mut = self.check(TokenKind::Mut);
        if is_mut {
            self.advance();
        }

        match self.current_kind() {
            TokenKind::Identifier => {
                let name = self.parse_identifier()?;

                // Check for struct pattern or variant
                if self.check(TokenKind::LBrace) {
                    // Struct pattern
                    self.advance();
                    let mut fields = Vec::new();
                    let mut rest = false;

                    while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                        if self.check(TokenKind::DotDot) {
                            self.advance();
                            rest = true;
                            break;
                        }

                        let field_start = self.current_span();
                        let field_name = self.parse_identifier()?;
                        let pattern = if self.check(TokenKind::Colon) {
                            self.advance();
                            Some(self.parse_pattern()?)
                        } else {
                            None
                        };

                        fields.push(PatternField {
                            name: field_name,
                            pattern,
                            span: field_start.merge(self.previous_span()),
                        });

                        if !self.check(TokenKind::RBrace) {
                            self.expect(TokenKind::Comma)?;
                        }
                    }

                    self.expect(TokenKind::RBrace)?;
                    let span = start.merge(self.previous_span());

                    return Ok(Pattern::Struct {
                        path: TypePath {
                            segments: vec![name],
                            span,
                        },
                        fields,
                        rest,
                        span,
                    });
                }

                if self.check(TokenKind::LParen) {
                    // Variant pattern with tuple data
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect(TokenKind::RParen)?;

                    let span = start.merge(self.previous_span());
                    return Ok(Pattern::Variant {
                        path: TypePath {
                            segments: vec![name],
                            span,
                        },
                        data: Some(Box::new(inner)),
                        span,
                    });
                }

                // Simple identifier binding
                Ok(Pattern::Identifier {
                    is_mut,
                    name,
                    span: start.merge(self.previous_span()),
                })
            }

            TokenKind::LParen => {
                self.advance();
                let patterns =
                    self.parse_comma_separated(TokenKind::RParen, |p| p.parse_pattern())?;
                self.expect(TokenKind::RParen)?;

                let span = start.merge(self.previous_span());
                Ok(Pattern::Tuple {
                    elements: patterns,
                    span,
                })
            }

            TokenKind::IntLiteral | TokenKind::String | TokenKind::True | TokenKind::False => {
                let value = self.parse_primary()?;
                Ok(Pattern::Literal {
                    value,
                    span: start.merge(self.previous_span()),
                })
            }

            _ => Err(CompileError::ExpectedToken {
                expected: "pattern".into(),
                found: self.current_kind().as_str().into(),
                span: start,
            }),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Helper Methods
    // ═══════════════════════════════════════════════════════════════════════

    fn binary_op_info(&self) -> Option<(BinaryOp, u8, Assoc)> {
        match self.current_kind() {
            TokenKind::Or | TokenKind::PipePipe => Some((BinaryOp::Or, 1, Assoc::Left)),
            TokenKind::And | TokenKind::AmpAmp => Some((BinaryOp::And, 2, Assoc::Left)),
            TokenKind::Pipe => Some((BinaryOp::BitOr, 3, Assoc::Left)),
            TokenKind::Caret => Some((BinaryOp::BitXor, 4, Assoc::Left)),
            TokenKind::Ampersand => Some((BinaryOp::BitAnd, 5, Assoc::Left)),
            TokenKind::EqEq => Some((BinaryOp::Eq, 6, Assoc::Left)),
            TokenKind::BangEq => Some((BinaryOp::Ne, 6, Assoc::Left)),
            TokenKind::Lt => Some((BinaryOp::Lt, 7, Assoc::Left)),
            TokenKind::LtEq => Some((BinaryOp::Le, 7, Assoc::Left)),
            TokenKind::Gt => Some((BinaryOp::Gt, 7, Assoc::Left)),
            TokenKind::GtEq => Some((BinaryOp::Ge, 7, Assoc::Left)),
            TokenKind::Shl => Some((BinaryOp::Shl, 8, Assoc::Left)),
            TokenKind::Shr => Some((BinaryOp::Shr, 8, Assoc::Left)),
            TokenKind::Plus => Some((BinaryOp::Add, 9, Assoc::Left)),
            TokenKind::Minus => Some((BinaryOp::Sub, 9, Assoc::Left)),
            TokenKind::Star => Some((BinaryOp::Mul, 10, Assoc::Left)),
            TokenKind::Slash => Some((BinaryOp::Div, 10, Assoc::Left)),
            TokenKind::Percent => Some((BinaryOp::Mod, 10, Assoc::Left)),
            _ => None,
        }
    }

    fn parse_int_literal(&self, text: &str) -> Result<i128, CompileError> {
        let text = text.replace('_', "");
        let (text, radix) = if text.starts_with("0x") || text.starts_with("0X") {
            (&text[2..], 16)
        } else if text.starts_with("0b") || text.starts_with("0B") {
            (&text[2..], 2)
        } else if text.starts_with("0o") || text.starts_with("0O") {
            (&text[2..], 8)
        } else {
            (text.as_str(), 10)
        };

        // Strip type suffix
        let text = text
            .trim_end_matches(|c: char| c.is_alphabetic())
            .to_string();

        i128::from_str_radix(&text, radix).map_err(|_| CompileError::InvalidNumber {
            span: self.current_span(),
        })
    }

    fn parse_identifier(&mut self) -> Result<Spanned<String>, CompileError> {
        let span = self.current_span();
        match self.current_kind() {
            TokenKind::Identifier
            | TokenKind::SelfType
            | TokenKind::SelfValue
            | TokenKind::Ok
            | TokenKind::Err
            | TokenKind::Some
            | TokenKind::None => {
                let name = self.current_text().to_string();
                self.advance();
                Ok(Spanned::new(name, span))
            }
            _ => Err(CompileError::ExpectedIdentifier { span }),
        }
    }

    fn parse_comma_separated<T>(
        &mut self,
        end: TokenKind,
        mut parse_item: impl FnMut(&mut Self) -> Result<T, CompileError>,
    ) -> Result<Vec<T>, CompileError> {
        let mut items = Vec::new();

        while !self.check(end) && !self.is_at_end() {
            items.push(parse_item(self)?);

            if !self.check(end) {
                self.expect(TokenKind::Comma)?;
                self.skip_newlines();
            }
        }

        Ok(items)
    }

    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.position)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.position)
            .map(|t| t.span)
            .unwrap_or(Span::DUMMY)
    }

    fn previous_span(&self) -> Span {
        if self.position > 0 {
            self.tokens
                .get(self.position - 1)
                .map(|t| t.span)
                .unwrap_or(Span::DUMMY)
        } else {
            Span::DUMMY
        }
    }

    fn current_text(&self) -> String {
        if let (Some(token), Some(source)) = (self.tokens.get(self.position), &self.source) {
            source[token.span.start..token.span.end].to_string()
        } else {
            self.current_kind().as_str().to_string()
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current_kind() == TokenKind::Eof
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), CompileError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: kind.as_str().into(),
                found: self.current_kind().as_str().into(),
                span: self.current_span(),
            })
        }
    }

    fn expect_newline(&mut self) -> Result<(), CompileError> {
        if self.check(TokenKind::Newline) {
            self.advance();
            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: "newline".into(),
                found: self.current_kind().as_str().into(),
                span: self.current_span(),
            })
        }
    }

    fn expect_newline_or_eof(&mut self) -> Result<(), CompileError> {
        if self.check(TokenKind::Newline) || self.check(TokenKind::Eof) {
            if self.check(TokenKind::Newline) {
                self.advance();
            }
            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: "newline or end of file".into(),
                found: self.current_kind().as_str().into(),
                span: self.current_span(),
            })
        }
    }

    fn expect_newline_or_dedent(&mut self) -> Result<(), CompileError> {
        if self.check(TokenKind::Newline) || self.check(TokenKind::Dedent) {
            if self.check(TokenKind::Newline) {
                self.advance();
            }
            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: "newline".into(),
                found: self.current_kind().as_str().into(),
                span: self.current_span(),
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Sync on newline followed by non-indentation
            if self.check(TokenKind::Newline) {
                self.advance();
                self.skip_newlines();

                // Check for item start
                match self.current_kind() {
                    TokenKind::Def
                    | TokenKind::Struct
                    | TokenKind::Enum
                    | TokenKind::Trait
                    | TokenKind::Impl
                    | TokenKind::Type
                    | TokenKind::Const
                    | TokenKind::Pub
                    | TokenKind::At => return,
                    _ => {}
                }
            }

            self.advance();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Assoc {
    Left,
    #[allow(dead_code)]
    Right,
}
