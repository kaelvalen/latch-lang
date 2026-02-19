use crate::error::{LatchError, Result};

// ── Token ────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    InterpolatedStr(Vec<StringPart>),

    // Identifier
    Ident(String),

    // Operators
    ColonEq,  // :=
    Eq,       // =
    Plus,     // +
    Minus,    // -
    Star,     // *
    Slash,    // /
    Percent,  // %
    EqEq,     // ==
    NotEq,    // !=
    Lt,       // <
    Gt,       // >
    LtEq,     // <=
    GtEq,     // >=
    And,      // &&
    Or,       // ||
    Bang,     // !
    Arrow,    // ->
    Dot,      // .
    DotDot,   // ..
    Comma,    // ,
    Colon,    // :
    PlusEq,   // +=
    MinusEq,  // -=
    StarEq,   // *=
    SlashEq,  // /=
    PercentEq,// %=
    QuestionQuestion, // ??
    QuestionDot,      // ?.
    PipeGt,   // |>

    // Grouping
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    LParen,   // (
    RParen,   // )

    // Keywords
    KwIf,
    KwElse,
    KwFor,
    KwIn,
    KwParallel,
    KwWorkers,
    KwFn,
    KwReturn,
    KwTry,
    KwCatch,
    KwUse,
    KwOr,
    KwStop,
    KwNull,

    // Other
    Newline,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(String), // raw source inside ${}
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Spanned<T> {
    pub node: T,
    pub line: usize,
    pub col: usize,
}

pub type TokenStream = Vec<Spanned<Token>>;

// ── Lexer ────────────────────────────────────────────────────
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<TokenStream> {
        let mut tokens = Vec::new();

        while !self.at_end() {
            let ch = self.peek();

            match ch {
                // Skip spaces and tabs (not newlines)
                ' ' | '\t' | '\r' => {
                    self.advance();
                }

                '\n' => {
                    // Collapse consecutive newlines into one token
                    let line = self.line;
                    let col = self.col;
                    while !self.at_end() && self.peek() == '\n' {
                        self.advance_newline();
                    }
                    // Only push if last token isn't already a newline
                    if tokens.last().map_or(true, |t: &Spanned<Token>| t.node != Token::Newline) {
                        tokens.push(Spanned { node: Token::Newline, line, col });
                    }
                }

                '#' => {
                    // Comment — skip until end of line
                    while !self.at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }

                '"' => {
                    let tok = self.lex_string()?;
                    tokens.push(tok);
                }

                '0'..='9' => {
                    let tok = self.lex_number();
                    tokens.push(tok);
                }

                'a'..='z' | 'A'..='Z' | '_' => {
                    let tok = self.lex_ident_or_keyword();
                    tokens.push(tok);
                }

                ':' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::ColonEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Colon, line, col });
                    }
                }

                '=' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::EqEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Eq, line, col });
                    }
                }

                '!' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::NotEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Bang, line, col });
                    }
                }

                '<' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::LtEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Lt, line, col });
                    }
                }

                '>' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::GtEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Gt, line, col });
                    }
                }

                '&' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '&' {
                        self.advance();
                        tokens.push(Spanned { node: Token::And, line, col });
                    } else {
                        return Err(LatchError::UnexpectedChar { ch: '&', line, col });
                    }
                }

                '|' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '|' {
                        self.advance();
                        tokens.push(Spanned { node: Token::Or, line, col });
                    } else if !self.at_end() && self.peek() == '>' {
                        self.advance();
                        tokens.push(Spanned { node: Token::PipeGt, line, col });
                    } else {
                        return Err(LatchError::UnexpectedChar { ch: '|', line, col });
                    }
                }

                '-' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '>' {
                        self.advance();
                        tokens.push(Spanned { node: Token::Arrow, line, col });
                    } else if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::MinusEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Minus, line, col });
                    }
                }

                '+' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::PlusEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Plus, line, col });
                    }
                }
                '*' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::StarEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Star, line, col });
                    }
                }
                '/' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance(); // consume first /
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::SlashEq, line, col });
                        continue;
                    }
                    if !self.at_end() && self.peek() == '/' {
                        // // line comment — skip to end of line
                        self.advance(); // consume second /
                        while !self.at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    } else {
                        tokens.push(Spanned { node: Token::Slash, line, col });
                    }
                }
                '.' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '.' {
                        self.advance();
                        tokens.push(Spanned { node: Token::DotDot, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Dot, line, col });
                    }
                }
                ',' => { let s = self.simple(Token::Comma); tokens.push(s); }
                '%' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '=' {
                        self.advance();
                        tokens.push(Spanned { node: Token::PercentEq, line, col });
                    } else {
                        tokens.push(Spanned { node: Token::Percent, line, col });
                    }
                }
                '?' => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    if !self.at_end() && self.peek() == '?' {
                        self.advance();
                        tokens.push(Spanned { node: Token::QuestionQuestion, line, col });
                    } else if !self.at_end() && self.peek() == '.' {
                        self.advance();
                        tokens.push(Spanned { node: Token::QuestionDot, line, col });
                    } else {
                        return Err(LatchError::UnexpectedChar { ch: '?', line, col });
                    }
                }
                '{' => { let s = self.simple(Token::LBrace); tokens.push(s); }
                '}' => { let s = self.simple(Token::RBrace); tokens.push(s); }
                '[' => { let s = self.simple(Token::LBracket); tokens.push(s); }
                ']' => { let s = self.simple(Token::RBracket); tokens.push(s); }
                '(' => { let s = self.simple(Token::LParen); tokens.push(s); }
                ')' => { let s = self.simple(Token::RParen); tokens.push(s); }

                _ => {
                    let line = self.line;
                    let col = self.col;
                    self.advance();
                    return Err(LatchError::UnexpectedChar { ch, line, col });
                }
            }
        }

        // Remove trailing newline
        if let Some(last) = tokens.last() {
            if last.node == Token::Newline {
                tokens.pop();
            }
        }

        tokens.push(Spanned { node: Token::EOF, line: self.line, col: self.col });
        Ok(tokens)
    }

    // ── Helpers ──────────────────────────────────────────────

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> char {
        self.chars[self.pos]
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.pos];
        self.pos += 1;
        self.col += 1;
        ch
    }

    fn advance_newline(&mut self) {
        self.pos += 1;
        self.line += 1;
        self.col = 1;
    }

    fn simple(&mut self, tok: Token) -> Spanned<Token> {
        let line = self.line;
        let col = self.col;
        self.advance();
        Spanned { node: tok, line, col }
    }

    fn lex_number(&mut self) -> Spanned<Token> {
        let line = self.line;
        let col = self.col;
        let mut s = String::new();
        let mut is_float = false;

        while !self.at_end() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            if self.peek() == '.' {
                // Look-ahead: only treat as decimal if next char is a digit
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_ascii_digit() {
                    is_float = true;
                    s.push(self.advance());
                } else {
                    break;
                }
            } else {
                s.push(self.advance());
            }
        }

        if is_float {
            Spanned { node: Token::Float(s.parse().unwrap()), line, col }
        } else {
            Spanned { node: Token::Int(s.parse().unwrap()), line, col }
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Spanned<Token> {
        let line = self.line;
        let col = self.col;
        let mut s = String::new();

        while !self.at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            s.push(self.advance());
        }

        let tok = match s.as_str() {
            "if"       => Token::KwIf,
            "else"     => Token::KwElse,
            "for"      => Token::KwFor,
            "in"       => Token::KwIn,
            "parallel" => Token::KwParallel,
            "workers"  => Token::KwWorkers,
            "fn"       => Token::KwFn,
            "return"   => Token::KwReturn,
            "try"      => Token::KwTry,
            "catch"    => Token::KwCatch,
            "use"      => Token::KwUse,
            "or"       => Token::KwOr,
            "stop"     => Token::KwStop,
            "true"     => Token::Bool(true),
            "false"    => Token::Bool(false),
            "null"     => Token::KwNull,
            _          => Token::Ident(s),
        };

        Spanned { node: tok, line, col }
    }

    fn lex_string(&mut self) -> Result<Spanned<Token>> {
        let line = self.line;
        let col = self.col;
        self.advance(); // skip opening "

        let mut parts: Vec<StringPart> = Vec::new();
        let mut current = String::new();

        loop {
            if self.at_end() {
                return Err(LatchError::UnterminatedString { line, col });
            }

            let ch = self.peek();

            if ch == '"' {
                self.advance(); // skip closing "
                break;
            }

            if ch == '\\' {
                self.advance();
                if self.at_end() {
                    return Err(LatchError::UnterminatedString { line, col });
                }
                let escaped = self.advance();
                match escaped {
                    'n'  => current.push('\n'),
                    't'  => current.push('\t'),
                    '\\' => current.push('\\'),
                    '"'  => current.push('"'),
                    '$'  => current.push('$'),
                    _    => {
                        current.push('\\');
                        current.push(escaped);
                    }
                }
                continue;
            }

            if ch == '$' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '{' {
                // String interpolation
                if !current.is_empty() {
                    parts.push(StringPart::Literal(std::mem::take(&mut current)));
                }
                self.advance(); // skip $
                self.advance(); // skip {

                let mut expr_src = String::new();
                let mut depth = 1;
                while !self.at_end() && depth > 0 {
                    let c = self.advance();
                    if c == '{' { depth += 1; }
                    if c == '}' { depth -= 1; }
                    if depth > 0 { expr_src.push(c); }
                }

                parts.push(StringPart::Expr(expr_src));
                continue;
            }

            if ch == '\n' {
                self.advance_newline();
                current.push('\n');
            } else {
                current.push(self.advance());
            }
        }

        // If no interpolation happened, produce a plain Str token
        if parts.is_empty() {
            Ok(Spanned { node: Token::Str(current), line, col })
        } else {
            if !current.is_empty() {
                parts.push(StringPart::Literal(current));
            }
            Ok(Spanned { node: Token::InterpolatedStr(parts), line, col })
        }
    }
}
