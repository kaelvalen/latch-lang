use std::fmt;

use crate::ast::Type;

/// Structured error context — every error carries location info when available.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub source_line: Option<String>,
    pub hint: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self { Self::default() }

    #[allow(dead_code)]
    pub fn at(line: usize, col: usize) -> Self {
        Self { line: Some(line), col: Some(col), ..Default::default() }
    }

    pub fn with_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string()); self
    }

    #[allow(dead_code)]
    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string()); self
    }

    pub fn with_source(mut self, src: &str) -> Self {
        self.source_line = Some(src.to_string()); self
    }
}

/// Every error Latch can produce – from lexing through runtime.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LatchError {
    // ── Lexer ────────────────────────────────────────────────
    UnexpectedChar { ch: char, line: usize, col: usize },
    UnterminatedString { line: usize, col: usize },

    // ── Parser ───────────────────────────────────────────────
    UnexpectedToken { expected: String, found: String, line: usize },
    UnexpectedEOF,

    // ── Semantic ─────────────────────────────────────────────
    UndefinedVariable(String),
    UndefinedFunction(String),
    UndeclaredAssign(String),
    ReturnOutsideFn,
    DuplicateFn(String),
    ArgCountMismatch { name: String, expected: usize, found: usize },
    TypeAnnotationMismatch { name: String, expected: Type, found: Type },
    ImportNotFound(String),

    // ── Runtime ──────────────────────────────────────────────
    TypeMismatch { expected: String, found: String },
    UnknownModule(String),
    UnknownMethod { module: String, method: String },
    IoError(String),
    HttpError(String),
    AiError(String),
    ProcessFailed { code: i32, stderr: String },
    DivisionByZero,
    IndexOutOfBounds { index: i64, len: usize },
    KeyNotFound(String),

    // ── Internal signals (not user-facing) ───────────────────
    ReturnSignal(crate::env::Value),
    StopSignal(i32),

    GenericError(String),
}

// ── Formatting ───────────────────────────────────────────────

/// Resolve the source line from source text, given a 1-based line number.
pub fn get_source_line(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line.saturating_sub(1)).map(|s| s.to_string())
}

/// Format a LatchError with full context into the standard format:
///
/// ```text
/// [latch] IO Error
///   file: examples/test.lt
///   line: 12  col: 5
///   → fs.read("x")
///   reason: No such file
///   hint: Use `or` to provide a default
/// ```
pub fn format_error(err: &LatchError, ctx: &ErrorContext) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("[latch] {}\n", err.category()));

    // File
    if let Some(file) = &ctx.file {
        out.push_str(&format!("  file: {file}\n"));
    }

    // Line / Col
    match (err.line_number().or(ctx.line), err.col_number().or(ctx.col)) {
        (Some(line), Some(col)) => out.push_str(&format!("  line: {line}  col: {col}\n")),
        (Some(line), None)      => out.push_str(&format!("  line: {line}\n")),
        _ => {}
    }

    // Source line arrow
    if let Some(src) = &ctx.source_line {
        out.push_str(&format!("  → {}\n", src.trim()));
    }

    // Reason
    out.push_str(&format!("  reason: {}\n", err.reason()));

    // Hint
    let hint = ctx.hint.as_deref().unwrap_or_else(|| err.default_hint());
    if !hint.is_empty() {
        out.push_str(&format!("  hint: {hint}\n"));
    }

    out.trim_end().to_string()
}

impl LatchError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::UnexpectedChar { .. } | Self::UnterminatedString { .. } => "Lexer Error",
            Self::UnexpectedToken { .. } | Self::UnexpectedEOF => "Parser Error",
            Self::UndefinedVariable(_) | Self::UndefinedFunction(_) |
            Self::UndeclaredAssign(_) | Self::ReturnOutsideFn |
            Self::DuplicateFn(_) | Self::ArgCountMismatch { .. } |
            Self::TypeAnnotationMismatch { .. } | Self::ImportNotFound(_) => "Semantic Error",
            Self::IoError(_) => "IO Error",
            Self::HttpError(_) => "HTTP Error",
            Self::AiError(_) => "AI Error",
            Self::ProcessFailed { .. } => "Process Error",
            _ => "Runtime Error",
        }
    }

    pub fn line_number(&self) -> Option<usize> {
        match self {
            Self::UnexpectedChar { line, .. } => Some(*line),
            Self::UnterminatedString { line, .. } => Some(*line),
            Self::UnexpectedToken { line, .. } => Some(*line),
            _ => None,
        }
    }

    pub fn col_number(&self) -> Option<usize> {
        match self {
            Self::UnexpectedChar { col, .. } => Some(*col),
            Self::UnterminatedString { col, .. } => Some(*col),
            _ => None,
        }
    }

    pub fn reason(&self) -> String {
        match self {
            Self::UnexpectedChar { ch, .. } => format!("Unexpected character '{ch}'"),
            Self::UnterminatedString { .. } => "Unterminated string literal".into(),
            Self::UnexpectedToken { expected, found, .. } => format!("Expected {expected}, found {found}"),
            Self::UnexpectedEOF => "Unexpected end of file".into(),
            Self::UndefinedVariable(n) => format!("Undefined variable '{n}'"),
            Self::UndefinedFunction(n) => format!("Undefined function '{n}'"),
            Self::UndeclaredAssign(n) => format!("Assignment to undeclared variable '{n}'"),
            Self::ReturnOutsideFn => "'return' used outside of a function".into(),
            Self::DuplicateFn(n) => format!("Duplicate function definition '{n}'"),
            Self::ArgCountMismatch { name, expected, found } =>
                format!("Function '{name}' expects {expected} argument(s), got {found}"),
            Self::TypeAnnotationMismatch { name, expected, found } =>
                format!("Variable '{name}' declared as {expected:?} but assigned {found:?}"),
            Self::ImportNotFound(p) => format!("Import not found: '{p}'"),
            Self::TypeMismatch { expected, found } => format!("Type mismatch: expected {expected}, found {found}"),
            Self::UnknownModule(m) => format!("Unknown module '{m}'"),
            Self::UnknownMethod { module, method } => format!("Unknown method '{module}.{method}'"),
            Self::IoError(msg) => msg.clone(),
            Self::HttpError(msg) => msg.clone(),
            Self::AiError(msg) => msg.clone(),
            Self::ProcessFailed { code, stderr } => format!("Process exited with code {code}: {stderr}"),
            Self::DivisionByZero => "Division by zero".into(),
            Self::IndexOutOfBounds { index, len } => format!("Index {index} out of bounds (length {len})"),
            Self::KeyNotFound(k) => format!("Key '{k}' not found in map"),
            Self::ReturnSignal(_) => "internal return signal".into(),
            Self::StopSignal(code) => format!("Script stopped with exit code {code}"),
            Self::GenericError(msg) => msg.clone(),
        }
    }

    pub fn default_hint(&self) -> &'static str {
        match self {
            Self::UnexpectedChar { .. } => "Check for typos or unsupported characters",
            Self::UnterminatedString { .. } => "Close the string with a double quote",
            Self::UnexpectedToken { .. } => "Check the syntax around this token",
            Self::UnexpectedEOF => "You may have an unclosed block or missing expression",
            Self::UndefinedVariable(_) => "Declare the variable first with ':='",
            Self::UndefinedFunction(_) => "Define the function with 'fn name(...)' before calling it",
            Self::UndeclaredAssign(_) => "Declare the variable first with ':='",
            Self::ReturnOutsideFn => "'return' can only appear inside a 'fn' block",
            Self::DuplicateFn(_) => "Each function name must be unique in its scope",
            Self::ArgCountMismatch { .. } => "Check the function signature",
            Self::TypeAnnotationMismatch { .. } => "Change the annotation or the value",
            Self::ImportNotFound(_) => "Check that the file exists and the path is correct",
            Self::UnknownModule(_) => "Available modules: fs, proc, http, time, ai",
            Self::IoError(_) => "Use 'or' to provide a fallback: fs.read(\"file\") or \"\"",
            Self::AiError(_) => "Set LATCH_AI_KEY environment variable",
            Self::DivisionByZero => "Check the divisor is not zero",
            Self::IndexOutOfBounds { .. } => "Use len() to check bounds first",
            _ => "",
        }
    }
}

/// Legacy Display — used when no ErrorContext is available.
impl fmt::Display for LatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ctx = ErrorContext::new();
        write!(f, "{}", format_error(self, &ctx))
    }
}

impl std::error::Error for LatchError {}

pub type Result<T> = std::result::Result<T, LatchError>;
