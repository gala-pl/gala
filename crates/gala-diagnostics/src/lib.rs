//! Rich diagnostics with stable error codes for Gala.
//!
//! Codes are `E` followed by four digits. The first two digits after `E0` select the category:
//! `E01xx` syntax, `E02xx` types, `E03xx` effects, `E04xx` linearity, `E05xx` uncomputation,
//! `E06xx` backend, `E07xx` runtime. The registry in [`explain`] holds the long-form text that
//! `gala explain` prints.

use gala_span::{SourceMap, Span};
use serde_json::{json, Value};
use std::fmt;
use std::ops::Range;

pub mod explain;

/// A stable diagnostic code such as `E0412`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(pub u16);

impl DiagnosticCode {
    /// Build a code from its numeric part (`412` for `E0412`).
    pub const fn new(code: u16) -> Self {
        DiagnosticCode(code)
    }

    /// The category this code belongs to.
    pub fn category(&self) -> DiagnosticCategory {
        match self.0 / 100 {
            1 => DiagnosticCategory::Syntax,
            2 => DiagnosticCategory::Type,
            3 => DiagnosticCategory::Effect,
            4 => DiagnosticCategory::Linearity,
            5 => DiagnosticCategory::Uncomputation,
            6 => DiagnosticCategory::Backend,
            7 => DiagnosticCategory::Runtime,
            _ => DiagnosticCategory::Other,
        }
    }

    /// The canonical `E0000` string form.
    pub fn as_string(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

/// Category of a diagnostic, for filtering and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCategory {
    Syntax,
    Type,
    Effect,
    Linearity,
    Uncomputation,
    Backend,
    Runtime,
    Other,
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl Severity {
    /// Lower-case name used in JSON output.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }
}

/// A labeled span in source code.
#[derive(Debug, Clone)]
pub struct Label {
    /// Where the label points.
    pub span: Span,
    /// Text shown next to the span.
    pub message: String,
}

/// A single replacement in a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// Region of the original source to replace.
    pub span: Span,
    /// Replacement text (may be empty).
    pub new_text: String,
}

/// A machine-applicable fix offered with a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedFix {
    /// Human-readable description of the fix.
    pub description: String,
    /// Edits that together apply the fix.
    pub edits: Vec<TextEdit>,
}

/// A single diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable code, for example `E0412`.
    pub code: DiagnosticCode,
    /// Error, warning, note, or help.
    pub severity: Severity,
    /// One-line summary.
    pub message: String,
    /// Labeled spans. The first label is the primary one.
    pub labels: Vec<Label>,
    /// Extra `= note:` lines.
    pub notes: Vec<String>,
    /// Optional `= help:` line.
    pub help: Option<String>,
    /// Machine-applicable fixes (used by `gala check --json` and `gala fix`).
    pub fixes: Vec<SuggestedFix>,
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::with_severity(code, Severity::Error, message)
    }

    /// Create a warning diagnostic.
    pub fn warning(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::with_severity(code, Severity::Warning, message)
    }

    /// Create a note diagnostic.
    pub fn note(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::with_severity(code, Severity::Note, message)
    }

    fn with_severity(code: DiagnosticCode, severity: Severity, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
            fixes: Vec::new(),
        }
    }

    /// Append a label.
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    /// Insert a primary label at the front.
    pub fn with_primary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.insert(0, Label { span, message: message.into() });
        self
    }

    /// Append a secondary label.
    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label { span, message: message.into() });
        self
    }

    /// Append a `= note:` line.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Set the `= help:` line.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Attach a machine-applicable fix.
    pub fn with_fix(mut self, description: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        self.fixes.push(SuggestedFix { description: description.into(), edits });
        self
    }

    /// The primary span, if the diagnostic has any label.
    pub fn primary_span(&self) -> Option<Span> {
        self.labels.first().map(|l| l.span)
    }
}

/// Collection of diagnostics produced by one or more passes.
#[derive(Debug, Default, Clone)]
pub struct Diagnostics {
    /// The diagnostics, in the order they were produced.
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    /// An empty collection.
    pub fn new() -> Self {
        Diagnostics::default()
    }

    /// Append one diagnostic.
    pub fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    /// Append every diagnostic from another collection.
    pub fn extend(&mut self, other: Diagnostics) {
        self.diagnostics.extend(other.diagnostics);
    }

    /// True when at least one error (not warning) is present.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// Only the error diagnostics.
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).collect()
    }

    /// Only the warning diagnostics.
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect()
    }

    /// True when nothing has been reported.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Render one diagnostic as plain text (no colors) with source snippets.
pub fn render(diag: &Diagnostic, source_map: &SourceMap) -> String {
    use ariadne::{Config, Label as ALabel, Report, ReportKind};

    let kind = match diag.severity {
        Severity::Error => ReportKind::Error,
        Severity::Warning => ReportKind::Warning,
        Severity::Note => ReportKind::Advice,
        Severity::Help => ReportKind::Advice,
    };

    // Use the first label's file as the report anchor; fall back to a placeholder.
    let anchor = match diag.primary_span() {
        Some(span) => (file_name(source_map, span), byte_range(span)),
        None => ("<gala>".to_string(), 0..0),
    };
    let mut builder = Report::build(kind, anchor.clone())
        .with_config(Config::default().with_color(false))
        .with_code(diag.code)
        .with_message(&diag.message);

    for label in &diag.labels {
        builder = builder.with_label(
            ALabel::new((file_name(source_map, label.span), byte_range(label.span)))
                .with_message(&label.message),
        );
    }
    for note in &diag.notes {
        builder = builder.with_note(note);
    }
    if let Some(help) = &diag.help {
        builder = builder.with_help(help);
    }
    let report = builder.finish();

    let sources: Vec<(String, String)> =
        source_map.files().map(|f| (f.path.display().to_string(), f.content.to_string())).collect();
    let mut buf = Vec::new();
    if report.write(ariadne::sources(sources), &mut buf).is_err() {
        return format!("{}: {}\n", diag.code, diag.message);
    }
    String::from_utf8_lossy(&buf).into_owned()
}

fn file_name(source_map: &SourceMap, span: Span) -> String {
    source_map
        .get_file(span.file_id)
        .map(|f| f.path.display().to_string())
        .unwrap_or_else(|| "<unknown>".to_string())
}

fn byte_range(span: Span) -> Range<usize> {
    span.byte_span.start as usize..span.byte_span.end as usize
}

/// Renders diagnostics to stderr.
pub struct Emitter<'a> {
    source_map: &'a SourceMap,
}

impl<'a> Emitter<'a> {
    /// Create an emitter that resolves spans through `source_map`.
    pub fn new(source_map: &'a SourceMap) -> Self {
        Emitter { source_map }
    }

    /// Render one diagnostic to stderr.
    pub fn emit(&self, diag: &Diagnostic) {
        eprint!("{}", render(diag, self.source_map));
    }

    /// Render every diagnostic to stderr.
    pub fn emit_all(&self, diagnostics: &Diagnostics) {
        for diag in &diagnostics.diagnostics {
            self.emit(diag);
        }
    }
}

fn span_json(span: Span, source_map: &SourceMap) -> Value {
    json!({
        "file_id": span.file_id.0,
        "start": span.byte_span.start,
        "end": span.byte_span.end,
        "file_path": file_name(source_map, span),
    })
}

/// Convert one diagnostic to the `diagnostic.v1` JSON shape (see
/// `tools/gala-mcp/src/schemas/diagnostic.v1.json`).
pub fn to_json(diag: &Diagnostic, source_map: &SourceMap) -> Value {
    let primary = match diag.primary_span() {
        Some(span) => span_json(span, source_map),
        None => json!({ "file_id": 0, "start": 0, "end": 0 }),
    };
    let labels: Vec<Value> = diag
        .labels
        .iter()
        .map(|l| {
            let mut v = span_json(l.span, source_map);
            v["message"] = json!(l.message);
            v
        })
        .collect();
    let fixes: Vec<Value> = diag
        .fixes
        .iter()
        .map(|fix| {
            let edits: Vec<Value> = fix
                .edits
                .iter()
                .map(|e| {
                    json!({
                        "file_id": e.span.file_id.0,
                        "start": e.span.byte_span.start,
                        "end": e.span.byte_span.end,
                        "new_text": e.new_text,
                    })
                })
                .collect();
            json!({ "description": fix.description, "edits": edits })
        })
        .collect();
    let mut out = json!({
        "code": diag.code.to_string(),
        "severity": diag.severity.as_str(),
        "message": diag.message,
        "primary_span": primary,
        "labels": labels,
        "notes": diag.notes,
        "suggested_fixes": fixes,
    });
    if let Some(help) = &diag.help {
        out["help"] = json!(help);
    }
    out["explain_url"] =
        json!(format!("https://gala-lang.org/errors/{}", diag.code.to_string().to_lowercase()));
    out
}

/// Predefined diagnostic codes. Keep in sync with [`explain::REGISTRY`].
pub mod codes {
    use super::DiagnosticCode;

    // Syntax (E01xx)
    /// An unexpected token was found where the grammar required something else.
    pub const UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode(101);
    /// A string literal was not closed.
    pub const UNTERMINATED_STRING: DiagnosticCode = DiagnosticCode(102);
    /// A block comment was not closed.
    pub const UNTERMINATED_COMMENT: DiagnosticCode = DiagnosticCode(103);
    /// A numeric literal could not be read.
    pub const INVALID_NUMBER: DiagnosticCode = DiagnosticCode(104);
    /// A character is not valid in Gala source.
    pub const INVALID_CHARACTER: DiagnosticCode = DiagnosticCode(105);
    /// An expression was expected.
    pub const EXPECTED_EXPRESSION: DiagnosticCode = DiagnosticCode(106);
    /// A statement was expected.
    pub const EXPECTED_STATEMENT: DiagnosticCode = DiagnosticCode(107);
    /// A type was expected.
    pub const EXPECTED_TYPE: DiagnosticCode = DiagnosticCode(108);
    /// An identifier was expected.
    pub const EXPECTED_IDENTIFIER: DiagnosticCode = DiagnosticCode(109);
    /// The file ended in the middle of a construct.
    pub const UNEXPECTED_EOF: DiagnosticCode = DiagnosticCode(110);

    // Types (E02xx)
    /// Two types that must agree do not.
    pub const TYPE_MISMATCH: DiagnosticCode = DiagnosticCode(201);
    /// A name does not resolve to any definition, local, or builtin.
    pub const UNRESOLVED_NAME: DiagnosticCode = DiagnosticCode(202);
    /// A call has the wrong number of arguments.
    pub const ARITY_MISMATCH: DiagnosticCode = DiagnosticCode(204);
    /// A const generic argument does not match.
    pub const CONST_GENERIC_MISMATCH: DiagnosticCode = DiagnosticCode(205);
    /// A register size does not match what the context requires.
    pub const QUBITS_SIZE_MISMATCH: DiagnosticCode = DiagnosticCode(206);
    /// An assignment targets a binding that was not declared `mut`.
    pub const IMMUTABLE_ASSIGN: DiagnosticCode = DiagnosticCode(203);
    /// The construct is valid Gala but not supported by this compiler yet.
    pub const UNSUPPORTED_FEATURE: DiagnosticCode = DiagnosticCode(207);

    // Effects (E03xx)
    /// A function's body performs more than its declared effect allows.
    pub const EFFECT_MISMATCH: DiagnosticCode = DiagnosticCode(301);
    /// A `pure` function calls a quantum or probabilistic function.
    pub const PURE_CALLS_QUANTUM: DiagnosticCode = DiagnosticCode(302);
    /// A `quantum` function calls a probabilistic function.
    pub const QUANTUM_CALLS_PROB: DiagnosticCode = DiagnosticCode(303);
    /// A measurement occurs in a function that is not declared `prob`.
    pub const MEASUREMENT_IN_QUANTUM: DiagnosticCode = DiagnosticCode(304);

    // Linearity (E04xx)
    /// A linear value is duplicated (no-cloning).
    pub const DUPLICATE_USE: DiagnosticCode = DiagnosticCode(401);
    /// A linear value goes out of scope without being consumed.
    pub const IMPLICIT_DROP: DiagnosticCode = DiagnosticCode(402);
    /// The same qubit is passed twice in one call.
    pub const ALIASING_QUBIT: DiagnosticCode = DiagnosticCode(403);
    /// A linear value is consumed on only some branches.
    pub const BRANCH_LINEARITY_MISMATCH: DiagnosticCode = DiagnosticCode(405);
    /// A linear value is used after it was consumed.
    pub const USE_AFTER_CONSUME: DiagnosticCode = DiagnosticCode(412);

    // Uncomputation (E05xx)
    /// An ancilla cannot be uncomputed automatically; the reason is in the message.
    pub const CANNOT_UNCOMPUTE: DiagnosticCode = DiagnosticCode(530);

    // Backend (E06xx)
    /// The program needs a capability the selected backend does not provide.
    pub const BACKEND_CAPABILITY_MISMATCH: DiagnosticCode = DiagnosticCode(601);
    /// Emitting QIR failed.
    pub const QIR_EMISSION_FAILED: DiagnosticCode = DiagnosticCode(602);
    /// The requested hardware backend is not reachable.
    pub const HARDWARE_UNAVAILABLE: DiagnosticCode = DiagnosticCode(603);

    // Runtime (E07xx)
    /// A runtime failure occurred while executing a program.
    pub const RUNTIME_ERROR: DiagnosticCode = DiagnosticCode(701);
    /// The program needs more qubits than the simulator can hold.
    pub const SIMULATOR_CAPACITY: DiagnosticCode = DiagnosticCode(702);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::{ByteSpan, FileId};

    fn sample_map() -> SourceMap {
        let mut sm = SourceMap::new();
        sm.add_file(
            "classifier.gala".into(),
            "fn f(q: Qubit) {\n    let a = q;\n    let b = q;\n}\n".to_string(),
        );
        sm
    }

    #[test]
    fn code_displays_with_four_digits() {
        assert_eq!(codes::USE_AFTER_CONSUME.to_string(), "E0412");
        assert_eq!(codes::UNEXPECTED_TOKEN.to_string(), "E0101");
    }

    #[test]
    fn category_follows_code_range() {
        assert_eq!(codes::USE_AFTER_CONSUME.category(), DiagnosticCategory::Linearity);
        assert_eq!(codes::CANNOT_UNCOMPUTE.category(), DiagnosticCategory::Uncomputation);
        assert_eq!(codes::TYPE_MISMATCH.category(), DiagnosticCategory::Type);
        assert_eq!(codes::QUANTUM_CALLS_PROB.category(), DiagnosticCategory::Effect);
    }

    #[test]
    fn builder_collects_labels_notes_and_help() {
        let span = Span::new(FileId(0), ByteSpan::new(40, 41));
        let diag = Diagnostic::error(codes::USE_AFTER_CONSUME, "qubit `q` used after consume")
            .with_primary_label(span, "used here")
            .with_secondary_label(span, "consumed here")
            .with_note("qubits are linear")
            .with_help("use the returned value");
        assert_eq!(diag.labels.len(), 2);
        assert_eq!(diag.labels[0].message, "used here");
        assert_eq!(diag.notes.len(), 1);
        assert_eq!(diag.help.as_deref(), Some("use the returned value"));
        assert_eq!(diag.primary_span(), Some(span));
    }

    #[test]
    fn render_contains_code_message_and_snippet() {
        let sm = sample_map();
        let start = 44u32; // the second `q` on line 3 (`    let b = q;`)
        let span = Span::new(FileId(0), ByteSpan::new(start, start + 1));
        let diag = Diagnostic::error(codes::USE_AFTER_CONSUME, "qubit `q` used after consume")
            .with_primary_label(span, "used here")
            .with_help("qubits are linear resources");
        let text = render(&diag, &sm);
        assert!(text.contains("E0412"), "missing code in:\n{text}");
        assert!(text.contains("qubit `q` used after consume"), "missing message in:\n{text}");
        assert!(text.contains("classifier.gala"), "missing file name in:\n{text}");
        assert!(text.contains("used here"), "missing label in:\n{text}");
        assert!(text.contains("Help: qubits are linear resources"), "missing help in:\n{text}");
        assert!(text.contains("3:13"), "wrong location in:\n{text}");
    }

    #[test]
    fn json_matches_schema_v1_shape() {
        let sm = sample_map();
        let span = Span::new(FileId(0), ByteSpan::new(44, 45));
        let diag = Diagnostic::error(codes::USE_AFTER_CONSUME, "boom")
            .with_primary_label(span, "here")
            .with_fix("delete the second use", vec![TextEdit { span, new_text: String::new() }]);
        let v = to_json(&diag, &sm);
        assert_eq!(v["code"], "E0412");
        assert_eq!(v["severity"], "error");
        assert_eq!(v["primary_span"]["start"], 44);
        assert_eq!(v["primary_span"]["end"], 45);
        assert_eq!(v["labels"][0]["message"], "here");
        assert_eq!(v["suggested_fixes"][0]["description"], "delete the second use");
        assert_eq!(v["suggested_fixes"][0]["edits"][0]["new_text"], "");
        for key in ["code", "severity", "message", "primary_span", "labels", "notes"] {
            assert!(v.get(key).is_some(), "missing key {key}");
        }
    }

    #[test]
    fn collection_reports_errors_and_warnings_separately() {
        let mut ds = Diagnostics::new();
        assert!(!ds.has_errors());
        ds.push(Diagnostic::warning(codes::TYPE_MISMATCH, "w"));
        assert!(!ds.has_errors());
        assert_eq!(ds.warnings().len(), 1);
        ds.push(Diagnostic::error(codes::TYPE_MISMATCH, "e"));
        assert!(ds.has_errors());
        assert_eq!(ds.errors().len(), 1);
    }

    #[test]
    fn every_registered_code_has_an_explanation() {
        for info in explain::REGISTRY {
            assert!(
                explain::lookup(DiagnosticCode(info.code)).is_some(),
                "code {} missing from registry",
                info.code
            );
        }
        for code in [
            codes::UNEXPECTED_TOKEN,
            codes::TYPE_MISMATCH,
            codes::UNRESOLVED_NAME,
            codes::EFFECT_MISMATCH,
            codes::PURE_CALLS_QUANTUM,
            codes::QUANTUM_CALLS_PROB,
            codes::DUPLICATE_USE,
            codes::IMPLICIT_DROP,
            codes::ALIASING_QUBIT,
            codes::USE_AFTER_CONSUME,
            codes::CANNOT_UNCOMPUTE,
            codes::BACKEND_CAPABILITY_MISMATCH,
            codes::RUNTIME_ERROR,
        ] {
            assert!(explain::lookup(code).is_some(), "{code} has no explain entry");
        }
    }
}
