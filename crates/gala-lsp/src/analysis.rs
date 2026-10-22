//! Per-document analysis: parse and compile the text, then index the HIR by source position so the
//! request handlers can answer hover, definition, and the `gala/*` queries.

use gala_ast::Item;
use gala_diagnostics::{Diagnostic, Severity, SuggestedFix};
use gala_driver::{compile_source, Sources};
use gala_gir::Gir;
use gala_hir::{
    lower_file, DefId, ExprId, HirBlock, HirExpr, HirExprKind, HirFn, HirPackage, HirPat, HirStmt,
    LocalId,
};
use gala_parser::parse_source;
use gala_span::{ByteSpan, FileId, Span};
use gala_types::{check_package, Ty, TypeInfo};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::position::lsp_range;

/// File id of the document. A fresh `Sources` gives the root file id 0.
pub const ROOT_FILE: FileId = FileId(0);

/// Everything the server knows about one document version.
pub struct Analysis {
    /// Document text this analysis was computed from.
    pub text: String,
    /// Diagnostics for the document (parse errors, or the compile errors).
    pub diagnostics: Vec<Diagnostic>,
    /// Names of top-level functions declared in the document, for completion.
    pub declared_fns: Vec<String>,
    /// Name-resolved and type-checked program. `None` when the file has syntax errors.
    pub semantic: Option<Semantic>,
}

/// Compiler results for a document that parsed.
pub struct Semantic {
    /// Name-resolved HIR.
    pub hir: HirPackage,
    /// Expression, local and function types.
    pub types: TypeInfo,
    /// Lowered GIR. Only present when the whole pipeline succeeded.
    pub gir: Option<Gir>,
    index: Index,
}

/// What the cursor is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Function(DefId),
    Local(LocalId),
    Expr(ExprId),
}

impl Analysis {
    /// An analysis with no results, used when the analysis itself fails.
    pub fn empty(text: &str) -> Self {
        Analysis {
            text: text.to_string(),
            diagnostics: Vec::new(),
            declared_fns: Vec::new(),
            semantic: None,
        }
    }

    /// The symbol under a byte offset. Among overlapping occurrences the narrowest wins, and a
    /// name beats a plain expression with the same span.
    pub fn symbol_at(&self, offset: usize) -> Option<Symbol> {
        let sem = self.semantic.as_ref()?;
        let off = u32::try_from(offset).ok()?;
        let occ = sem
            .index
            .occs
            .iter()
            .filter(|o| {
                o.span.file_id == ROOT_FILE
                    && o.span.byte_span.start <= off
                    && off <= o.span.byte_span.end
            })
            .min_by_key(|o| (o.span.byte_span.len(), matches!(o.kind, Occ::Expr(_))))?;
        Some(match occ.kind {
            Occ::Expr(id) => Symbol::Expr(id),
            Occ::Fn(def) => Symbol::Function(def),
            Occ::Local(local) => Symbol::Local(local),
        })
    }

    /// Source span of the name a symbol is defined at. Only spans in the document are returned.
    pub fn definition_span(&self, symbol: Symbol) -> Option<Span> {
        let index = &self.semantic.as_ref()?.index;
        match symbol {
            Symbol::Function(def) => index.fn_names.get(&def).copied(),
            Symbol::Local(local) => index.binds.get(&local).copied(),
            Symbol::Expr(_) => None,
        }
    }

    /// Markdown-free text shown on hover for a symbol.
    pub fn hover_text(&self, symbol: Symbol) -> Option<String> {
        let sem = self.semantic.as_ref()?;
        match symbol {
            Symbol::Function(def) => self.signature(def),
            Symbol::Local(local) => {
                let ty = known(sem.types.local_ty.get(&local)?)?;
                Some(format!("{}: {ty}", sem.hir.local_name(local)))
            }
            Symbol::Expr(id) => known(sem.types.expr_ty.get(&id)?),
        }
    }

    /// The type of a symbol. A function's type is `fn(params) -> ret`, without name or effect.
    pub fn type_text(&self, symbol: Symbol) -> Option<String> {
        let sem = self.semantic.as_ref()?;
        match symbol {
            Symbol::Function(def) => {
                let sig = sem.types.sigs.get(def.0 as usize)?;
                let params: Vec<String> = sig.params.iter().map(ToString::to_string).collect();
                Some(format!("fn({}) -> {}", params.join(", "), sig.ret))
            }
            Symbol::Local(local) => known(sem.types.local_ty.get(&local)?),
            Symbol::Expr(id) => known(sem.types.expr_ty.get(&id)?),
        }
    }

    /// `fn name(p: T, ...) -> R effect` for a function.
    pub fn signature(&self, def: DefId) -> Option<String> {
        let sem = self.semantic.as_ref()?;
        let f = sem.hir.func(def)?;
        let sig = sem.types.sigs.get(def.0 as usize)?;
        let params: Vec<String> =
            f.params.iter().zip(&sig.params).map(|(p, t)| format!("{}: {t}", p.name)).collect();
        let effect =
            sem.types.effects.get(def.0 as usize).map_or_else(String::new, |e| format!(" {e}"));
        Some(format!("fn {}({}) -> {}{effect}", f.name.as_str(), params.join(", "), sig.ret))
    }

    /// The function whose span contains the offset (the innermost one).
    pub fn enclosing_fn(&self, offset: usize) -> Option<DefId> {
        let sem = self.semantic.as_ref()?;
        let off = u32::try_from(offset).ok()?;
        sem.hir
            .fns
            .iter()
            .filter(|f| contains(f.span, off))
            .min_by_key(|f| f.span.byte_span.len())
            .map(|f| f.def)
    }

    /// Effect of a function, as its lattice name (`pure`, `quantum`, `prob`).
    pub fn effect_of(&self, def: DefId) -> Option<String> {
        let sem = self.semantic.as_ref()?;
        sem.types.effects.get(def.0 as usize).map(ToString::to_string)
    }

    /// Name and GIR dump section of the function enclosing the offset.
    pub fn gir_at(&self, offset: usize) -> Option<(String, String)> {
        let def = self.enclosing_fn(offset)?;
        let gir = self.semantic.as_ref()?.gir.as_ref()?;
        let i = def.0 as usize;
        let func = gir.funcs.get(i)?;
        let section = gir_section(&gir.dump(), i, &func.name)?;
        Some((func.name.clone(), section))
    }

    /// Diagnostics as LSP `Diagnostic` objects.
    pub fn diagnostics_json(&self) -> Vec<Value> {
        self.diagnostics
            .iter()
            .map(|d| {
                let (range, code) = (diag_range(&self.text, d.primary_span()), d.code.to_string());
                json!({
                    "range": range,
                    "severity": severity_number(d.severity),
                    "code": code,
                    "source": "gala",
                    "message": d.message,
                })
            })
            .collect()
    }

    /// Suggested fixes from every diagnostic, as `{code, title, edits}` objects.
    pub fn fixes_json(&self) -> Vec<Value> {
        let mut out = Vec::new();
        for d in &self.diagnostics {
            for fix in &d.fixes {
                out.push(self.fix_json(d, fix));
            }
        }
        out
    }

    fn fix_json(&self, d: &Diagnostic, fix: &SuggestedFix) -> Value {
        let edits: Vec<Value> = fix
            .edits
            .iter()
            .map(|e| json!({"range": diag_range(&self.text, Some(e.span)), "newText": e.new_text}))
            .collect();
        json!({"code": d.code.to_string(), "title": fix.description, "edits": edits})
    }
}

impl Semantic {
    fn new(text: &str, hir: HirPackage, types: TypeInfo, gir: Option<Gir>) -> Self {
        let index = Index::build(text, &hir);
        Semantic { hir, types, gir, index }
    }
}

/// Analyze a document. Syntax errors stop the pipeline and are returned alone. Otherwise the
/// full compile runs, and on failure the HIR and types are still computed from the parsed items
/// so that hover and definition keep working on a file with, say, a linearity error.
pub fn analyze(text: &str) -> Analysis {
    let parsed = parse_source(ROOT_FILE, text);
    let declared_fns = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            Item::FnDef(f) => Some(f.ident.as_str().to_string()),
            _ => None,
        })
        .collect();
    if parsed.diagnostics.has_errors() {
        return Analysis {
            text: text.to_string(),
            diagnostics: parsed.diagnostics.diagnostics,
            declared_fns,
            semantic: None,
        };
    }
    let mut sources = Sources::default();
    let (diagnostics, semantic) = match compile_source(text, &mut sources) {
        Ok(program) => {
            let sem = Semantic::new(text, program.hir, program.types, Some(program.gir));
            (Vec::new(), sem)
        }
        Err(diags) => {
            let (hir, _lower_diags) = lower_file(ROOT_FILE, parsed.items);
            let (types, _check_diags) = check_package(&hir);
            (diags.diagnostics, Semantic::new(text, hir, types, None))
        }
    };
    Analysis { text: text.to_string(), diagnostics, declared_fns, semantic: Some(semantic) }
}

/// The section of a GIR dump for function `index`, from its header line to the closing `}`.
pub fn gir_section(dump: &str, index: usize, name: &str) -> Option<String> {
    let header = format!("fn {name} #{index} ");
    let mut out = String::new();
    let mut inside = false;
    for line in dump.lines() {
        if !inside {
            if !line.starts_with(&header) {
                continue;
            }
            inside = true;
        }
        out.push_str(line);
        out.push('\n');
        if line == "}" {
            break;
        }
    }
    inside.then_some(out)
}

fn known(ty: &Ty) -> Option<String> {
    if matches!(ty, Ty::Error) {
        None
    } else {
        Some(ty.to_string())
    }
}

fn severity_number(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 1,
        Severity::Warning => 2,
        Severity::Note => 3,
        Severity::Help => 4,
    }
}

/// LSP range for a span. Spans outside the document (bundled library code) map to 0:0.
fn diag_range(text: &str, span: Option<Span>) -> Value {
    match span {
        Some(s) if s.file_id == ROOT_FILE => {
            lsp_range(text, s.byte_span.start as usize, s.byte_span.end as usize)
        }
        _ => lsp_range(text, 0, 0),
    }
}

fn contains(span: Span, off: u32) -> bool {
    span.file_id == ROOT_FILE && span.byte_span.start <= off && off <= span.byte_span.end
}

/// Position index over the document's HIR.
#[derive(Default)]
struct Index {
    occs: Vec<Occurrence>,
    binds: HashMap<LocalId, Span>,
    fn_names: HashMap<DefId, Span>,
}

#[derive(Debug, Clone, Copy)]
struct Occurrence {
    span: Span,
    kind: Occ,
}

#[derive(Debug, Clone, Copy)]
enum Occ {
    Expr(ExprId),
    Fn(DefId),
    Local(LocalId),
}

impl Index {
    fn build(text: &str, hir: &HirPackage) -> Self {
        let mut walker = Walker { text, ix: Index::default() };
        for f in &hir.fns {
            walker.function(f);
        }
        for c in &hir.consts {
            walker.expr(&c.value);
        }
        walker.ix
    }
}

struct Walker<'a> {
    text: &'a str,
    ix: Index,
}

impl Walker<'_> {
    fn push(&mut self, span: Span, kind: Occ) {
        self.ix.occs.push(Occurrence { span, kind });
    }

    fn bind(&mut self, local: LocalId, span: Span) {
        self.ix.binds.insert(local, span);
        self.push(span, Occ::Local(local));
    }

    fn function(&mut self, f: &HirFn) {
        if f.span.file_id != ROOT_FILE {
            return;
        }
        if let Some(span) = name_span(self.text, f.span, f.name.as_str(), Some("fn")) {
            self.ix.fn_names.insert(f.def, span);
            self.push(span, Occ::Fn(f.def));
        }
        for p in &f.params {
            if let Some(span) = name_span(self.text, p.span, &p.name, None) {
                self.bind(p.local, span);
            }
        }
        if let Some(body) = &f.body {
            self.block(body);
        }
    }

    fn block(&mut self, b: &HirBlock) {
        for stmt in &b.stmts {
            match stmt {
                HirStmt::Let { pat, init, .. } => {
                    self.pat(pat);
                    self.expr(init);
                }
                HirStmt::Expr(e) => self.expr(e),
                HirStmt::Return { value, .. } => {
                    if let Some(v) = value {
                        self.expr(v);
                    }
                }
            }
        }
        if let Some(tail) = &b.tail {
            self.expr(tail);
        }
    }

    fn pat(&mut self, pat: &HirPat) {
        match pat {
            HirPat::Bind { local, name, span, .. } => {
                if let Some(s) = name_span(self.text, *span, name, None) {
                    self.bind(*local, s);
                }
            }
            HirPat::Tuple(pats, _) => {
                for p in pats {
                    self.pat(p);
                }
            }
            HirPat::Wild | HirPat::Lit(..) => {}
        }
    }

    fn expr(&mut self, e: &HirExpr) {
        if e.span.file_id == ROOT_FILE {
            match &e.kind {
                HirExprKind::Local(id, _) => self.push(e.span, Occ::Local(*id)),
                HirExprKind::Fn(def) => self.push(e.span, Occ::Fn(*def)),
                _ => {}
            }
            self.push(e.span, Occ::Expr(e.id));
        }
        match &e.kind {
            HirExprKind::Lit(_)
            | HirExprKind::Local(..)
            | HirExprKind::Fn(_)
            | HirExprKind::Builtin(_)
            | HirExprKind::Const(_)
            | HirExprKind::Error => {}
            HirExprKind::Call { callee, args, .. } => {
                self.expr(callee);
                for a in args {
                    self.expr(a);
                }
            }
            HirExprKind::Unary { expr, .. } => self.expr(expr),
            HirExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            HirExprKind::If { cond, then_block, else_expr } => {
                self.expr(cond);
                self.block(then_block);
                if let Some(other) = else_expr {
                    self.expr(other);
                }
            }
            HirExprKind::For { pat, iterable, body } => {
                self.pat(pat);
                self.expr(iterable);
                self.block(body);
            }
            HirExprKind::While { cond, body } => {
                self.expr(cond);
                self.block(body);
            }
            HirExprKind::Block(b) => self.block(b),
            HirExprKind::Tuple(items) | HirExprKind::Array(items) => {
                for item in items {
                    self.expr(item);
                }
            }
            HirExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            HirExprKind::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            HirExprKind::Match { scrutinee, arms } => {
                self.expr(scrutinee);
                for arm in arms {
                    self.pat(&arm.pat);
                    if let Some(guard) = &arm.guard {
                        self.expr(guard);
                    }
                    self.expr(&arm.body);
                }
            }
        }
    }
}

/// Span of `name` inside `within`. When `after_kw` is given, the search starts after that keyword
/// (so `fn f` finds the `f`, not an earlier mention).
fn name_span(text: &str, within: Span, name: &str, after_kw: Option<&str>) -> Option<Span> {
    if within.file_id != ROOT_FILE {
        return None;
    }
    let (start, end) = (within.byte_span.start as usize, within.byte_span.end as usize);
    let slice = text.get(start..end)?;
    let from = match after_kw {
        Some(kw) => find_word(slice, kw, 0)? + kw.len(),
        None => 0,
    };
    let rel = find_word(slice, name, from)?;
    let s = start + rel;
    Some(Span::new(
        ROOT_FILE,
        ByteSpan::new(u32::try_from(s).ok()?, u32::try_from(s + name.len()).ok()?),
    ))
}

/// Byte index of the first whole-word occurrence of `word` at or after `from`.
fn find_word(haystack: &str, word: &str, from: usize) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let mut i = from;
    while i <= haystack.len() {
        let rel = haystack.get(i..)?.find(word)?;
        let start = i + rel;
        let end = start + word.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return Some(start);
        }
        i = start + 1;
        while i < haystack.len() && !haystack.is_char_boundary(i) {
            i += 1;
        }
    }
    None
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_word_respects_boundaries() {
        assert_eq!(find_word("fnord fn f", "fn", 0), Some(6));
        assert_eq!(find_word("xf f", "f", 0), Some(3));
        assert_eq!(find_word("abc", "d", 0), None);
    }

    #[test]
    fn gir_section_extracts_one_function() {
        let dump = "fn a #0 effect=pure params=[] slots=1 {\nb0:\n}\nfn b #1 effect=pure params=[] slots=1 {\nb0:\n  ret\n}\n";
        assert_eq!(
            gir_section(dump, 1, "b").as_deref(),
            Some("fn b #1 effect=pure params=[] slots=1 {\nb0:\n  ret\n}\n")
        );
        assert_eq!(gir_section(dump, 2, "b"), None);
    }

    #[test]
    fn syntax_errors_are_reported_without_semantics() {
        let a = analyze("fn f( {\n");
        assert!(a.semantic.is_none());
        assert!(!a.diagnostics.is_empty());
        assert!(a.diagnostics.iter().all(|d| d.code.to_string().starts_with('E')));
    }

    #[test]
    fn linearity_error_keeps_hir_for_hover() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/conformance/linearity/use_after_consume.gala"
        ))
        .expect("conformance file exists");
        let a = analyze(&text);
        assert!(a.diagnostics.iter().any(|d| d.code.to_string() == "E0412"));
        let def = a.enclosing_fn(text.find("fn f").unwrap() + 3);
        assert!(def.is_some(), "function is indexed despite the compile error");
    }
}
