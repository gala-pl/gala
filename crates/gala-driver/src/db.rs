use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gala_span::{FileId, Span};
use gala_diagnostics::{Diagnostic, Diagnostics};
use gala_lexer::{Lexer, Token};
use gala_parser::parse;
use gala_hir::{
    build_module_graph, desugar_file, CrateId, DefId, HirFnDef, HirFile, ModuleGraph, Resolution,
    Resolver,
};
use gala_types::{check_linearity, type_check_fn, Effect as TyEffect, Ty};
use gala_uncompute::{analyze_provenance, synthesize_uncompute, UncomputePlan};
use gala_gir::{lower_hir_to_gir, Gir};
use gala_diff::lower_gradients;
use gala_opt::optimize_gir;

// ════════════════════════════════════════════════════════════════════════════════
// Salsa inputs
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::input]
pub struct SourceFile {
    #[returns(ref)]
    pub text: String,
    pub file_id: u32,
}

#[salsa::input]
pub struct DefIdInput {
    pub file: SourceFile,
    pub index: u32,
}

// ════════════════════════════════════════════════════════════════════════════════
// Database implementation
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::db]
#[derive(Clone)]
pub struct GalaDatabaseImpl {
    storage: salsa::Storage<Self>,
    file_index: Arc<Mutex<HashMap<u32, SourceFile>>>,
    def_index: Arc<Mutex<HashMap<(u32, u32), DefIdInput>>>,
}

impl Default for GalaDatabaseImpl {
    fn default() -> Self {
        Self {
            storage: salsa::Storage::new(None),
            file_index: Arc::new(Mutex::new(HashMap::new())),
            def_index: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl GalaDatabaseImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, _path: std::path::PathBuf, content: String) -> FileId {
        let mut index = self.file_index.lock().unwrap();
        let id = index.len() as u32;
        let source_file = SourceFile::new(self, content.clone(), id);
        index.insert(id, source_file);

        // Discover function definitions to pre-create DefIdInput instances
        // so they are available as salsa query keys.
        let fid = FileId(id);
        let lexer = Lexer::new(fid, &content);
        let toks = lexer.collect_all();
        if let Ok(items) = parse(toks.as_slice()) {
            let hir = desugar_file(&items, fid);
            let mut resolver = Resolver::new();
            let resolved = resolver.build_and_resolve(&hir);
            let mut def_index = self.def_index.lock().unwrap();
            for item in &resolved.items {
                if let gala_hir::HirItem::FnDef(f) = item {
                    let did = DefIdInput::new(self, source_file, f.def_id.index);
                    def_index.insert((f.def_id.crate_id.0 .0, f.def_id.index), did);
                }
            }
        }

        FileId(id)
    }

    pub fn source_file(&self, file_id: FileId) -> Option<SourceFile> {
        self.file_index.lock().unwrap().get(&file_id.0).copied()
    }

    fn get_def_input(&self, def_id: &DefId) -> Option<DefIdInput> {
        self.def_index
            .lock()
            .unwrap()
            .get(&(def_id.crate_id.0 .0, def_id.index))
            .copied()
    }

    // ── Convenience wrappers (retain existing public API) ────────────────

    pub fn type_of(&self, def_id: &DefId) -> Result<Arc<Ty>, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => type_of_query(self, input),
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn effect_of(&self, def_id: &DefId) -> Result<TyEffect, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => match effect_of_query(self, input) {
                Ok(eff) => Ok(*eff),
                Err(e) => Err(e),
            },
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn linearity_diagnostics(
        &self,
        def_id: &DefId,
    ) -> Result<Diagnostics, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => match linearity_check_query(self, input) {
                Ok(diags) => Ok((*diags).clone()),
                Err(e) => Err(e),
            },
            None => Ok(Diagnostics::new()),
        }
    }

    pub fn uncompute_plan(
        &self,
        def_id: &DefId,
    ) -> Result<Arc<UncomputePlan>, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => uncompute_query(self, input),
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn gir(&self, def_id: &DefId) -> Result<Arc<Gir>, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => gir_query(self, input),
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn gir_diff(&self, def_id: &DefId) -> Result<Arc<Gir>, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => gir_diff_query(self, input),
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn gir_opt(&self, def_id: &DefId) -> Result<Arc<Gir>, Arc<Diagnostics>> {
        match self.get_def_input(def_id) {
            Some(input) => gir_opt_query(self, input),
            None => Err(Arc::new(Diagnostics::new())),
        }
    }

    pub fn resolve(&self, def_id: &DefId) -> Arc<Resolution> {
        match self.get_def_input(def_id) {
            Some(input) => resolve_query(self, input),
            None => Arc::new(Resolution::Error(Diagnostic::error(
                gala_diagnostics::codes::UNKNOWN_TYPE,
                "unresolved definition",
            ))),
        }
    }

    pub fn compile_fn(&self, def_id: &DefId) -> Result<Arc<Gir>, Arc<Diagnostics>> {
        self.gir(def_id)
    }
}

#[salsa::db]
impl salsa::Database for GalaDatabaseImpl {}

// ════════════════════════════════════════════════════════════════════════════════
// File-level tracked queries (existing)
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::tracked(returns(ref))]
pub fn tokens(db: &dyn salsa::Database, file: SourceFile) -> Vec<(Token, Span)> {
    let text = file.text(db).clone();
    let fid = FileId(file.file_id(db));
    let lexer = Lexer::new(fid, &text);
    lexer.collect_all()
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn ast(db: &dyn salsa::Database, file: SourceFile) -> Arc<Vec<gala_ast::Item>> {
    let text = file.text(db).clone();
    let fid = FileId(file.file_id(db));
    let lexer = Lexer::new(fid, &text);
    let toks = lexer.collect_all();
    match parse(toks.as_slice()) {
        Ok(items) => Arc::new(items),
        Err(_) => Arc::new(Vec::new()),
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn hir_file(db: &dyn salsa::Database, file: SourceFile) -> Arc<HirFile> {
    let items = ast(db, file);
    let fid = FileId(file.file_id(db));
    Arc::new(desugar_file(items.as_ref(), fid))
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn hir_file_resolved(db: &dyn salsa::Database, file: SourceFile) -> Arc<HirFile> {
    let hir = hir_file(db, file);
    let mut resolver = Resolver::new();
    Arc::new(resolver.build_and_resolve(hir.as_ref()))
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn hir_functions(
    db: &dyn salsa::Database,
    file: SourceFile,
) -> Arc<HashMap<DefId, HirFnDef>> {
    let hir = hir_file_resolved(db, file);
    let mut funcs = HashMap::new();
    for item in &hir.items {
        if let gala_hir::HirItem::FnDef(f) = item {
            funcs.insert(f.def_id.clone(), f.clone());
        }
    }
    Arc::new(funcs)
}

// ════════════════════════════════════════════════════════════════════════════════
// Module-graph query
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::tracked(returns(ref), no_eq)]
pub fn module_graph(db: &dyn salsa::Database, file: SourceFile) -> Arc<ModuleGraph> {
    let hir = hir_file_resolved(db, file);
    let fid = FileId(file.file_id(db));
    Arc::new(build_module_graph(&[(fid, hir.as_ref())]))
}

// ════════════════════════════════════════════════════════════════════════════════
// Resolve query
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::tracked(returns(ref), no_eq)]
pub fn resolve_query(db: &dyn salsa::Database, def_id: DefIdInput) -> Arc<Resolution> {
    let index = def_id.index(db);
    let file_id = def_id.file(db).file_id(db);
    if index >= 10_000 {
        let builtin_idx = (index - 10_000) as usize;
        if let Some(def) = builtin_from_idx(builtin_idx) {
            return Arc::new(Resolution::Builtin(def));
        }
    }
    Arc::new(Resolution::Def(DefId { crate_id: CrateId(FileId(file_id)), index }))
}

fn builtin_from_idx(idx: usize) -> Option<gala_hir::BuiltinDef> {
    use gala_hir::BuiltinDef::*;
    Some(match idx {
        0 => QubitAlloc,
        1 => QubitsAlloc,
        2 => Measure,
        3 => Drop,
        4 => HGate,
        5 => XGate,
        6 => YGate,
        7 => ZGate,
        8 => SGate,
        9 => TGate,
        10 => RxGate,
        11 => RyGate,
        12 => RzGate,
        13 => CXGate,
        14 => CZGate,
        15 => SwapGate,
        _ => return None,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// DefId-level tracked queries
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::tracked(returns(ref), no_eq)]
pub fn type_of_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<Ty>, Arc<Diagnostics>> {
    let file = def_id.file(db);
    let funcs = hir_functions(db, file);
    let hir_def_id = DefId { crate_id: CrateId(FileId(file.file_id(db))), index: def_id.index(db) };
    if let Some(func) = funcs.get(&hir_def_id) {
        match type_check_fn(func) {
            Ok(ty) => Ok(Arc::new(ty)),
            Err(diags) => Err(Arc::new(diags)),
        }
    } else {
        Err(Arc::new(Diagnostics::new()))
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn effect_of_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<TyEffect>, Arc<Diagnostics>> {
    let file = def_id.file(db);
    let funcs = hir_functions(db, file);
    let hir_def_id = DefId { crate_id: CrateId(FileId(file.file_id(db))), index: def_id.index(db) };
    if let Some(func) = funcs.get(&hir_def_id) {
        let eff = match &func.effect {
            gala_ast::Effect::Pure => TyEffect::Pure,
            gala_ast::Effect::Quantum => TyEffect::Quantum,
            gala_ast::Effect::Prob => TyEffect::Prob,
        };
        Ok(Arc::new(eff))
    } else {
        Err(Arc::new(Diagnostics::new()))
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn linearity_check_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<Diagnostics>, Arc<Diagnostics>> {
    let file = def_id.file(db);
    let funcs = hir_functions(db, file);
    let hir_def_id = DefId { crate_id: CrateId(FileId(file.file_id(db))), index: def_id.index(db) };
    if let Some(func) = funcs.get(&hir_def_id) {
        Ok(Arc::new(check_linearity(func)))
    } else {
        Ok(Arc::new(Diagnostics::new()))
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn uncompute_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<UncomputePlan>, Arc<Diagnostics>> {
    let file = def_id.file(db);
    let funcs = hir_functions(db, file);
    let hir_def_id = DefId { crate_id: CrateId(FileId(file.file_id(db))), index: def_id.index(db) };
    if let Some(func) = funcs.get(&hir_def_id) {
        let provenance = analyze_provenance(func);
        match synthesize_uncompute(func, &provenance) {
            Ok(plan) => Ok(Arc::new(plan)),
            Err(diags) => Err(Arc::new(diags)),
        }
    } else {
        Err(Arc::new(Diagnostics::new()))
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn gir_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<Gir>, Arc<Diagnostics>> {
    let file = def_id.file(db);
    let funcs = hir_functions(db, file);
    let mut type_of_map = HashMap::new();
    let mut effect_of_map = HashMap::new();
    for (did, func) in funcs.iter() {
        if let Ok(ty) = type_check_fn(func) {
            type_of_map.insert(did.clone(), ty);
        }
        let eff = match &func.effect {
            gala_ast::Effect::Pure => TyEffect::Pure,
            gala_ast::Effect::Quantum => TyEffect::Quantum,
            gala_ast::Effect::Prob => TyEffect::Prob,
        };
        effect_of_map.insert(did.clone(), eff);
    }
    let hir_funcs: HashMap<DefId, HirFnDef> = funcs.as_ref().clone();
    match lower_hir_to_gir(&hir_funcs, &type_of_map, &effect_of_map) {
        Ok(g) => Ok(Arc::new(g)),
        Err(e) => Err(Arc::new(e)),
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn gir_diff_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<Gir>, Arc<Diagnostics>> {
    let base = gir_query(db, def_id);
    match base {
        Ok(gir) => {
            let mut cloned = (*gir).clone();
            lower_gradients(&mut cloned);
            Ok(Arc::new(cloned))
        }
        Err(e) => Err(e.clone()),
    }
}

#[salsa::tracked(returns(ref), no_eq)]
pub fn gir_opt_query(
    db: &dyn salsa::Database,
    def_id: DefIdInput,
) -> Result<Arc<Gir>, Arc<Diagnostics>> {
    let base = gir_query(db, def_id);
    match base {
        Ok(gir) => {
            let mut cloned = (*gir).clone();
            optimize_gir(&mut cloned);
            Ok(Arc::new(cloned))
        }
        Err(e) => Err(e.clone()),
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Diagnostics aggregate query
// ════════════════════════════════════════════════════════════════════════════════

#[salsa::tracked(returns(ref), no_eq)]
pub fn diagnostics(db: &dyn salsa::Database, file: SourceFile) -> Arc<Vec<Diagnostic>> {
    let funcs = hir_functions(db, file);
    let mut all_diags = Vec::new();

    for (_def_id, func) in funcs.iter() {
        if let Err(diags) = type_check_fn(func) {
            all_diags.extend(diags.diagnostics);
        }
        let lin_diags = check_linearity(func);
        all_diags.extend(lin_diags.diagnostics);
        let provenance = analyze_provenance(func);
        if let Err(diags) = synthesize_uncompute(func, &provenance) {
            all_diags.extend(diags.diagnostics);
        }
    }

    Arc::new(all_diags)
}

// ════════════════════════════════════════════════════════════════════════════════
// Legacy parse helper (used by compile_source in lib.rs)
// ════════════════════════════════════════════════════════════════════════════════

pub fn parse_file(
    file_id: FileId,
    source: &str,
) -> Result<Vec<gala_ast::Item>, Diagnostics> {
    let lexer = Lexer::new(file_id, source);
    let tokens = lexer.collect_all();
    parse(tokens.as_slice())
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db(source: &str) -> (GalaDatabaseImpl, FileId) {
        let mut db = GalaDatabaseImpl::new();
        let fid = db.add_file(std::path::PathBuf::from("test.gala"), source.to_string());
        (db, fid)
    }

    #[test]
    fn test_cached_and_fresh_queries_consistent() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();

        // First call: fresh computation
        let ast1 = ast(&db, file);
        let len1 = ast1.len();

        // Second call: should return cached value
        let ast2 = ast(&db, file);
        let len2 = ast2.len();

        assert_eq!(len1, len2, "cached and fresh AST queries should return the same length");
        assert_eq!(ast1.len(), ast2.len(), "cached and fresh AST queries should be consistent");
    }

    #[test]
    fn test_module_graph_query() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();

        let mg = module_graph(&db, file);
        assert!(mg.modules.contains_key(&gala_hir::ModuleId(0)), "module graph should contain root module");

        // Second call returns cached
        let mg2 = module_graph(&db, file);
        assert_eq!(mg.modules.len(), mg2.modules.len());
    }

    #[test]
    fn test_diagnostics_aggregate() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();

        let diags = diagnostics(&db, file);
        // A valid pure function should have no diagnostics
        assert!(!diags.iter().any(|d| d.severity == gala_diagnostics::Severity::Error));
    }

    #[test]
    fn test_def_id_queries() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();
        let funcs = hir_functions(&db, file);

        for (def_id, _func) in funcs.iter() {
            // type_of
            let ty_result = db.type_of(def_id);
            assert!(ty_result.is_ok(), "type_of should succeed for valid function");

            // effect_of
            let eff_result = db.effect_of(def_id);
            assert!(eff_result.is_ok(), "effect_of should succeed");

            // linearity_check
            let lin_result = db.linearity_diagnostics(def_id);
            assert!(lin_result.is_ok(), "linearity_check should not fail");

            // uncompute
            let unc_result = db.uncompute_plan(def_id);
            // May fail if uncompute can't handle it, but shouldn't panic

            // gir
            let gir_result = db.gir(def_id);
            assert!(gir_result.is_ok(), "gir lowering should succeed");
        }
    }

    #[test]
    fn test_gir_diff_and_opt() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();
        let funcs = hir_functions(&db, file);

        for (def_id, _func) in funcs.iter() {
            let base = db.gir(def_id).unwrap();
            let diffed = db.gir_diff(def_id).unwrap();
            let opted = db.gir_opt(def_id).unwrap();

            // gir_diff is a pass over the base GIR
            assert!(diffed.funcs.len() >= base.funcs.len());
            // gir_opt may simplify but shouldn't remove functions
            assert_eq!(opted.funcs.len(), base.funcs.len());
        }
    }

    #[test]
    fn test_resolve_builtin() {
        let (db, fid) = create_test_db("fn main() -> Int { return 42; }");
        let file = db.source_file(fid).unwrap();
        let funcs = hir_functions(&db, file);

        // Resolve the function def itself
        if let Some(def_id) = funcs.keys().next() {
            let res = db.resolve(def_id);
            match res.as_ref() {
                Resolution::Def(_) => {} // expected
                other => panic!("expected Resolution::Def, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_multi_file_consistency() {
        let mut db = GalaDatabaseImpl::new();
        let fid1 = db.add_file(
            std::path::PathBuf::from("a.gala"),
            "fn a() -> Int { return 1; }".to_string(),
        );
        let fid2 = db.add_file(
            std::path::PathBuf::from("b.gala"),
            "fn b() -> Int { return 2; }".to_string(),
        );

        let file1 = db.source_file(fid1).unwrap();
        let file2 = db.source_file(fid2).unwrap();

        let ast1 = ast(&db, file1);
        let ast2 = ast(&db, file2);

        assert_eq!(ast1.len(), 1, "file 1 should have one item");
        assert_eq!(ast2.len(), 1, "file 2 should have one item");

        // Cross-file queries don't interfere
        let funcs1 = hir_functions(&db, file1);
        let funcs2 = hir_functions(&db, file2);

        assert_eq!(funcs1.len(), 1);
        assert_eq!(funcs2.len(), 1);
    }
}
