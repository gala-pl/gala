//! Rich diagnostics with error codes for Gala.

use gala_span::Span;
use std::fmt;

/// A diagnostic code (e.g., E0101, E0401, E0501).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(pub u16);

impl DiagnosticCode {
    pub const fn new(code: u16) -> Self {
        DiagnosticCode(code)
    }

    pub fn category(&self) -> DiagnosticCategory {
        match self.0 / 100 {
            1 => DiagnosticCategory::Syntax,
            2 => DiagnosticCategory::Type,
            3 => DiagnosticCategory::Effect,
            4 => DiagnosticCategory::Linearity,
            5 => DiagnosticCategory::Uncomputation,
            6 => DiagnosticCategory::Backend,
            _ => DiagnosticCategory::Other,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

/// Category of diagnostic for filtering/grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCategory {
    Syntax,
    Type,
    Effect,
    Linearity,
    Uncomputation,
    Backend,
    Other,
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

/// A labeled span in source code.
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

/// A single diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity: Severity::Warning,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn note(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity: Severity::Note,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_primary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.insert(0, Label { span, message: message.into() });
        self
    }

    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label { span, message: message.into() });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Collection of diagnostics.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics::default()
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).collect()
    }

    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect()
    }
}

/// Emitter for rendering diagnostics (placeholder).
pub struct Emitter<'a> {
    _source_map: &'a gala_span::SourceMap,
}

impl<'a> Emitter<'a> {
    pub fn new(source_map: &'a gala_span::SourceMap) -> Self {
        Emitter { _source_map: source_map }
    }

    pub fn emit(&self, diag: &Diagnostic) {
        // Print basic diagnostic info
        eprintln!("{}: {}", diag.code, diag.message);
        for label in &diag.labels {
            eprintln!("  --> {}: {}", label.span, label.message);
        }
        for note in &diag.notes {
            eprintln!("  = note: {}", note);
        }
        if let Some(help) = &diag.help {
            eprintln!("  = help: {}", help);
        }
    }

    pub fn emit_all(&self, diagnostics: &Diagnostics) {
        for diag in &diagnostics.diagnostics {
            self.emit(diag);
        }
    }
}

/// Predefined error codes.
pub mod codes {
    use super::DiagnosticCode;

    // Syntax errors (E01xx)
    pub const UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode(101);
    pub const UNTERMINATED_STRING: DiagnosticCode = DiagnosticCode(102);
    pub const UNTERMINATED_COMMENT: DiagnosticCode = DiagnosticCode(103);
    pub const INVALID_NUMBER: DiagnosticCode = DiagnosticCode(104);
    pub const INVALID_CHARACTER: DiagnosticCode = DiagnosticCode(105);
    pub const EXPECTED_EXPRESSION: DiagnosticCode = DiagnosticCode(106);
    pub const EXPECTED_STATEMENT: DiagnosticCode = DiagnosticCode(107);
    pub const EXPECTED_TYPE: DiagnosticCode = DiagnosticCode(108);
    pub const EXPECTED_IDENTIFIER: DiagnosticCode = DiagnosticCode(109);
    pub const UNEXPECTED_EOF: DiagnosticCode = DiagnosticCode(110);

    // Type errors (E02xx)
    pub const TYPE_MISMATCH: DiagnosticCode = DiagnosticCode(201);
    pub const UNKNOWN_TYPE: DiagnosticCode = DiagnosticCode(202);
    pub const TYPE_ANNOTATION_REQUIRED: DiagnosticCode = DiagnosticCode(203);
    pub const RECURSIVE_TYPE: DiagnosticCode = DiagnosticCode(204);
    pub const CONST_GENERIC_MISMATCH: DiagnosticCode = DiagnosticCode(205);
    pub const QUBITS_SIZE_MISMATCH: DiagnosticCode = DiagnosticCode(206);

    // Effect errors (E03xx)
    pub const EFFECT_MISMATCH: DiagnosticCode = DiagnosticCode(301);
    pub const PURE_CALLS_QUANTUM: DiagnosticCode = DiagnosticCode(302);
    pub const QUANTUM_CALLS_PROB: DiagnosticCode = DiagnosticCode(303);
    pub const MEASUREMENT_IN_QUANTUM: DiagnosticCode = DiagnosticCode(304);
    pub const EFFECT_POLYMORPHISM_ERROR: DiagnosticCode = DiagnosticCode(305);

    // Linearity errors (E04xx)
    pub const USE_AFTER_CONSUME: DiagnosticCode = DiagnosticCode(401);
    pub const DUPLICATE_USE: DiagnosticCode = DiagnosticCode(402);
    pub const IMPLICIT_DROP: DiagnosticCode = DiagnosticCode(403);
    pub const ALIASING_QUBIT: DiagnosticCode = DiagnosticCode(404);
    pub const QUBIT_NOT_CONSUMED: DiagnosticCode = DiagnosticCode(405);

    // Uncomputation errors (E05xx)
    pub const CANNOT_UNCOMPUTE: DiagnosticCode = DiagnosticCode(501);
    pub const UNCOMPUTE_DEPENDS_ON_MEASUREMENT: DiagnosticCode = DiagnosticCode(502);
    pub const UNCOMPUTE_DEPENDS_ON_IRREVERSIBLE: DiagnosticCode = DiagnosticCode(503);
    pub const UNCOMPUTE_INPUT_NOT_LIVE: DiagnosticCode = DiagnosticCode(504);

    // Backend errors (E06xx)
    pub const BACKEND_CAPABILITY_MISMATCH: DiagnosticCode = DiagnosticCode(601);
    pub const QIR_EMISSION_FAILED: DiagnosticCode = DiagnosticCode(602);
    pub const HARDWARE_UNAVAILABLE: DiagnosticCode = DiagnosticCode(603);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::{ByteSpan, FileId, SourceMap};

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error(codes::TYPE_MISMATCH, "expected Int, found Float")
            .with_primary_label(Span::new(FileId(0), ByteSpan::new(0, 3)), "here")
            .with_note("this is a note")
            .with_help("try casting to Int");

        assert_eq!(diag.code, codes::TYPE_MISMATCH);
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.labels.len() == 1);
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(codes::USE_AFTER_CONSUME.category(), DiagnosticCategory::Linearity);
        assert_eq!(codes::CANNOT_UNCOMPUTE.category(), DiagnosticCategory::Uncomputation);
        assert_eq!(codes::TYPE_MISMATCH.category(), DiagnosticCategory::Type);
    }

    #[test]
    fn test_emitter() {
        let mut source_map = SourceMap::new();
        source_map.add_file("test.gala".into(), "fn main() -> Int { return 42; }".to_string());

        let emitter = Emitter::new(&source_map);
        let diag = Diagnostic::error(codes::TYPE_MISMATCH, "type mismatch")
            .with_primary_label(Span::new(FileId(0), ByteSpan::new(0, 3)), "here");

        // Just verify it doesn't panic
        emitter.emit(&diag);
    }
}
