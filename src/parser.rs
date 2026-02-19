use crate::ast::*;
use crate::error::{LatchError, Result};
use crate::lexer::{Lexer, Spanned, StringPart as LexStringPart, Token, TokenStream};

/// Recursive-descent parser: token stream → AST.
pub struct Parser {
    tokens: TokenStream,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: TokenStream) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── Helpers ──────────────────────────────────────────────

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].node
    }

    fn peek_spanned(&self) -> &Spanned<Token> {
        &self.tokens[self.pos]
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    fn advance(&mut self) -> &Spanned<Token> {
        let tok = &self.tokens[self.pos];
        if !self.at_end() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<&Spanned<Token>> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            let sp = self.peek_spanned();
            Err(LatchError::UnexpectedToken {
                expected: format!("{expected:?}"),
                found: format!("{:?}", sp.node),
                line: sp.line,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    fn line(&self) -> usize {
        self.tokens[self.pos].line
    }

    // ── Program ──────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_end() {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    // ── Statements ───────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt> {
        self.skip_newlines();
        match self.peek().clone() {
            Token::KwIf       => self.parse_if(),
            Token::KwFor      => self.parse_for(),
            Token::KwParallel => self.parse_parallel(),
            Token::KwFn       => self.parse_fn(),
            Token::KwReturn   => self.parse_return(),
            Token::KwStop     => self.parse_stop(),
            Token::KwTry      => self.parse_try(),
            Token::KwUse      => self.parse_use(),
            Token::Ident(_)   => self.parse_ident_stmt(),
            _                 => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    /// An identifier at statement position can be:
    /// - `name := value`       (let)
    /// - `name: type := value` (let with annotation)
    /// - `name = value`        (assign)
    /// - `name[idx] = value`   (index assign)
    /// - `name(...)` or `mod.method(...)` (expression statement)
    fn parse_ident_stmt(&mut self) -> Result<Stmt> {
        let name = match self.advance().node.clone() {
            Token::Ident(n) => n,
            _ => unreachable!(),
        };

        match self.peek().clone() {
            Token::ColonEq => {
                self.advance(); // skip :=
                let value = self.parse_expr()?;
                Ok(Stmt::Let { name, type_ann: None, value })
            }

            Token::Colon => {
                // name: type := value
                self.advance(); // skip :
                let type_ann = self.parse_type()?;
                self.expect(&Token::ColonEq)?;
                let value = self.parse_expr()?;
                Ok(Stmt::Let { name, type_ann: Some(type_ann), value })
            }

            Token::Eq => {
                self.advance(); // skip =
                let value = self.parse_expr()?;
                Ok(Stmt::Assign { name, value })
            }

            // Compound assignments: +=, -=, *=, /=, %=
            Token::PlusEq | Token::MinusEq | Token::StarEq | Token::SlashEq | Token::PercentEq => {
                let op = match self.advance().node.clone() {
                    Token::PlusEq    => BinOp::Add,
                    Token::MinusEq   => BinOp::Sub,
                    Token::StarEq    => BinOp::Mul,
                    Token::SlashEq   => BinOp::Div,
                    Token::PercentEq => BinOp::Mod,
                    _ => unreachable!(),
                };
                let value = self.parse_expr()?;
                Ok(Stmt::CompoundAssign { name, op, value })
            }

            Token::LBracket => {
                // name[idx] = value  or  name[a][b] = value  (index assignment)
                self.advance(); // skip [
                let first_index = self.parse_expr()?;
                self.expect(&Token::RBracket)?;

                if matches!(self.peek(), Token::Eq) {
                    // Simple: name[idx] = value
                    self.advance(); // skip =
                    let value = self.parse_expr()?;
                    Ok(Stmt::IndexAssign { target: Expr::Ident(name), index: first_index, value })
                } else {
                    // Build Expr::Index and continue postfix
                    let base = Expr::Index {
                        expr: Box::new(Expr::Ident(name)),
                        index: Box::new(first_index),
                    };
                    let expr = self.continue_postfix(base)?;

                    // Check if this is a nested index assignment: expr[...][...] = value
                    if matches!(self.peek(), Token::Eq) {
                        self.advance(); // skip =
                        let value = self.parse_expr()?;
                        // Decompose: the outermost Expr::Index gives us target + index
                        if let Expr::Index { expr: target, index } = expr {
                            Ok(Stmt::IndexAssign { target: *target, index: *index, value })
                        } else {
                            Err(crate::error::LatchError::GenericError("Invalid assignment target".into()))
                        }
                    } else {
                        Ok(Stmt::Expr(expr))
                    }
                }
            }

            // Anything else: treat as expression starting with this ident
            _ => {
                // Rewind so we can re-parse as expression
                self.pos -= 1;
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'if'
        let cond = self.parse_expr()?;
        let then = self.parse_block()?;

        self.skip_newlines();
        let else_ = if matches!(self.peek(), Token::KwElse) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If { cond, then, else_ })
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'for'
        let var = match self.advance().node.clone() {
            Token::Ident(n) => n,
            other => return Err(LatchError::UnexpectedToken {
                expected: "identifier".into(), found: format!("{other:?}"), line: self.line(),
            }),
        };
        self.expect(&Token::KwIn)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iter, body })
    }

    fn parse_parallel(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'parallel'
        let var = match self.advance().node.clone() {
            Token::Ident(n) => n,
            other => return Err(LatchError::UnexpectedToken {
                expected: "identifier".into(), found: format!("{other:?}"), line: self.line(),
            }),
        };
        self.expect(&Token::KwIn)?;
        let iter = self.parse_expr()?;

        // Optional: workers=N
        let workers = if matches!(self.peek(), Token::KwWorkers) {
            self.advance(); // skip 'workers'
            self.expect(&Token::Eq)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Ok(Stmt::Parallel { var, iter, workers, body })
    }

    fn parse_fn(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'fn'
        let name = match self.advance().node.clone() {
            Token::Ident(n) => n,
            other => return Err(LatchError::UnexpectedToken {
                expected: "function name".into(), found: format!("{other:?}"), line: self.line(),
            }),
        };

        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;

        let return_type = if matches!(self.peek(), Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Ok(Stmt::Fn { name, params, return_type, body })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(params);
        }
        loop {
            let name = match self.advance().node.clone() {
                Token::Ident(n) => n,
                other => return Err(LatchError::UnexpectedToken {
                    expected: "parameter name".into(), found: format!("{other:?}"), line: self.line(),
                }),
            };
            let type_ann = if matches!(self.peek(), Token::Colon) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(Param { name, type_ann });
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        match self.advance().node.clone() {
            Token::Ident(s) => match s.as_str() {
                "int"     => Ok(Type::Int),
                "float"   => Ok(Type::Float),
                "bool"    => Ok(Type::Bool),
                "string"  => Ok(Type::Str),
                "list"    => Ok(Type::List),
                "dict"    => Ok(Type::Dict),
                "process" => Ok(Type::Process),
                "file"    => Ok(Type::File),
                "any"     => Ok(Type::Any),
                _ => Err(LatchError::UnexpectedToken {
                    expected: "type".into(), found: s, line: self.line(),
                }),
            },
            other => Err(LatchError::UnexpectedToken {
                expected: "type".into(), found: format!("{other:?}"), line: self.line(),
            }),
        }
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'return'
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(expr))
    }

    fn parse_stop(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'stop'
        let expr = self.parse_expr()?;
        Ok(Stmt::Stop(expr))
    }

    fn parse_try(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'try'
        let body = self.parse_block()?;
        self.skip_newlines();
        self.expect(&Token::KwCatch)?;
        let catch_var = match self.advance().node.clone() {
            Token::Ident(n) => n,
            other => return Err(LatchError::UnexpectedToken {
                expected: "catch variable".into(), found: format!("{other:?}"), line: self.line(),
            }),
        };
        let catch_body = self.parse_block()?;
        Ok(Stmt::Try { body, catch_var, catch_body })
    }

    fn parse_use(&mut self) -> Result<Stmt> {
        self.advance(); // skip 'use'
        match self.advance().node.clone() {
            Token::Str(path) => Ok(Stmt::Use(path)),
            other => Err(LatchError::UnexpectedToken {
                expected: "string path".into(), found: format!("{other:?}"), line: self.line(),
            }),
        }
    }

    fn parse_block(&mut self) -> Result<Block> {
        self.skip_newlines();
        self.expect(&Token::LBrace)?;
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Token::RBrace | Token::EOF) {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        Ok(stmts)
    }

    // ── Expressions (precedence climbing) ────────────────────

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or_default()
    }

    fn parse_or_default(&mut self) -> Result<Expr> {
        let expr = self.parse_pipe()?;

        // Handle `or` default: `expr or default`
        if matches!(self.peek(), Token::KwOr) {
            self.advance();
            let default = self.parse_pipe()?;
            return Ok(Expr::OrDefault {
                expr: Box::new(expr),
                default: Box::new(default),
            });
        }

        Ok(expr)
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let expr = self.parse_null_coalesce()?;

        // Handle `|>` pipe: `expr |> func(args)` (supports multi-line)
        let saved = self.pos;
        self.skip_newlines();
        if matches!(self.peek(), Token::PipeGt) {
            let mut result = expr;
            while matches!(self.peek(), Token::PipeGt) {
                self.advance();
                let func_expr = self.parse_null_coalesce()?;
                result = Expr::Pipe {
                    expr: Box::new(result),
                    func: Box::new(func_expr),
                };
                // Allow multi-line continuation
                let saved_inner = self.pos;
                self.skip_newlines();
                if !matches!(self.peek(), Token::PipeGt) {
                    self.pos = saved_inner;
                }
            }
            return Ok(result);
        }
        self.pos = saved; // backtrack if no |> found after newlines

        Ok(expr)
    }

    fn parse_null_coalesce(&mut self) -> Result<Expr> {
        let mut left = self.parse_or_expr()?;
        while matches!(self.peek(), Token::QuestionQuestion) {
            self.advance();
            let right = self.parse_or_expr()?;
            left = Expr::NullCoalesce {
                expr: Box::new(left),
                default: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_and_expr()?;
        while matches!(self.peek(), Token::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while matches!(self.peek(), Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinOp { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Token::EqEq  => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_range()?;
        loop {
            let op = match self.peek() {
                Token::Lt   => BinOp::Lt,
                Token::Gt   => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                Token::KwIn => BinOp::In,
                _ => break,
            };
            self.advance();
            let right = self.parse_range()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_range(&mut self) -> Result<Expr> {
        let left = self.parse_additive()?;
        if matches!(self.peek(), Token::DotDot) {
            self.advance();
            let right = self.parse_additive()?;
            return Ok(Expr::Range {
                start: Box::new(left),
                end: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star    => BinOp::Mul,
                Token::Slash   => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Token::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let expr = self.parse_primary()?;
        self.continue_postfix(expr)
    }

    /// Continue parsing postfix operations from an already-parsed base expression.
    fn continue_postfix(&mut self, mut expr: Expr) -> Result<Expr> {

        loop {
            match self.peek() {
                // field access: expr.field or module call: mod.method(args)
                Token::Dot => {
                    self.advance();
                    let field = match self.advance().node.clone() {
                        Token::Ident(n) => n,
                        other => return Err(LatchError::UnexpectedToken {
                            expected: "field name".into(), found: format!("{other:?}"), line: self.line(),
                        }),
                    };

                    if matches!(self.peek(), Token::LParen) {
                        // This is a method/module call: expr.method(args)
                        // We only support: ident.method(args) for module calls
                        self.advance(); // skip (
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;

                        if let Expr::Ident(module) = expr {
                            expr = Expr::ModuleCall { module, method: field, args };
                        } else {
                            return Err(LatchError::GenericError(
                                "Method calls are only supported on module names".into(),
                            ));
                        }
                    } else {
                        expr = Expr::FieldAccess { expr: Box::new(expr), field };
                    }
                }

                // index: expr[index]
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBracket)?;
                    expr = Expr::Index { expr: Box::new(expr), index: Box::new(index) };
                }

                // safe access: expr?.field
                Token::QuestionDot => {
                    self.advance();
                    let field = match self.advance().node.clone() {
                        Token::Ident(n) => n,
                        other => return Err(LatchError::UnexpectedToken {
                            expected: "field name".into(), found: format!("{other:?}"), line: self.line(),
                        }),
                    };
                    expr = Expr::SafeAccess { expr: Box::new(expr), field };
                }

                // call: expr(args) — only for Ident
                Token::LParen if matches!(expr, Expr::Ident(_)) => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    if let Expr::Ident(name) = expr {
                        expr = Expr::Call { name, args };
                    }
                }

                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let tok = self.peek().clone();
        match tok {
            Token::Int(n)    => { self.advance(); Ok(Expr::Int(n)) }
            Token::Float(n)  => { self.advance(); Ok(Expr::Float(n)) }
            Token::Bool(b)   => { self.advance(); Ok(Expr::Bool(b)) }
            Token::Str(s)    => { self.advance(); Ok(Expr::Str(s)) }
            Token::KwNull    => { self.advance(); Ok(Expr::Null) }
            Token::Ident(n)  => { self.advance(); Ok(Expr::Ident(n)) }

            Token::InterpolatedStr(parts) => {
                self.advance();
                let ast_parts = self.convert_interpolation(parts)?;
                Ok(Expr::Interpolated(ast_parts))
            }

            Token::LBracket => {
                self.advance(); // skip [
                let mut elems = Vec::new();
                self.skip_newlines();
                while !matches!(self.peek(), Token::RBracket | Token::EOF) {
                    elems.push(self.parse_expr()?);
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        self.skip_newlines();
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::List(elems))
            }

            Token::LBrace => {
                // Map literal: {"key": value, "key2": value2}
                self.advance(); // skip {
                let mut entries = Vec::new();
                self.skip_newlines();
                while !matches!(self.peek(), Token::RBrace | Token::EOF) {
                    let key = match self.advance().node.clone() {
                        Token::Str(s) => s,
                        Token::Ident(s) => s,
                        other => return Err(LatchError::UnexpectedToken {
                            expected: "string or identifier key".into(),
                            found: format!("{other:?}"),
                            line: self.line(),
                        }),
                    };
                    self.expect(&Token::Colon)?;
                    let value = self.parse_expr()?;
                    entries.push((key, value));
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        self.skip_newlines();
                    }
                }
                self.expect(&Token::RBrace)?;
                Ok(Expr::Map(entries))
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }

            // Anonymous function: fn(x, y) { ... }
            Token::KwFn => {
                self.advance(); // skip 'fn'
                self.expect(&Token::LParen)?;
                let params = self.parse_params()?;
                self.expect(&Token::RParen)?;
                let body = self.parse_block()?;
                Ok(Expr::Fn { params, body })
            }

            _ => {
                let sp = self.peek_spanned();
                Err(LatchError::UnexpectedToken {
                    expected: "expression".into(),
                    found: format!("{:?}", sp.node),
                    line: sp.line,
                })
            }
        }
    }

    /// Convert lexer StringParts into AST StringParts by
    /// sub-parsing each Expr fragment.
    fn convert_interpolation(&self, parts: Vec<LexStringPart>) -> Result<Vec<StringPart>> {
        let mut out = Vec::new();
        for part in parts {
            match part {
                LexStringPart::Literal(s) => out.push(StringPart::Literal(s)),
                LexStringPart::Expr(src) => {
                    let mut lexer = Lexer::new(&src);
                    let tokens = lexer.tokenize()?;
                    out.push(StringPart::Expr(tokens));
                }
            }
        }
        Ok(out)
    }
}
