//! HIR with desugaring and name resolution for Gala.

use gala_ast::*;
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_span::{FileId, Span};
use std::collections::{HashMap, HashSet};

/// A crate in the compilation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CrateId(pub FileId);

/// Module graph representing the crate structure.
#[derive(Debug, Clone, Default)]
pub struct ModuleGraph {
    pub modules: HashMap<ModuleId, ModuleData>,
    pub root: ModuleId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModuleId(pub u32);

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub id: ModuleId,
    pub name: Ident,
    pub parent: Option<ModuleId>,
    pub children: Vec<ModuleId>,
    pub items: Vec<DefId>,
}

/// Definition ID - stable identifier for a definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefId {
    pub crate_id: CrateId,
    pub index: u32,
}

/// Resolution result.
#[derive(Debug, Clone)]
pub enum Resolution {
    Def(DefId),
    Local(LocalId),
    Builtin(BuiltinDef),
    Error(Diagnostic),
}

/// Local variable ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

/// Built-in definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinDef {
    QubitAlloc,
    QubitsAlloc,
    Measure,
    Drop,
    HGate,
    XGate,
    YGate,
    ZGate,
    SGate,
    TGate,
    RxGate,
    RyGate,
    RzGate,
    CXGate,
    CZGate,
    SwapGate,
}

/// HIR file with desugared AST.
#[derive(Debug, Clone)]
pub struct HirFile {
    pub file_id: FileId,
    pub items: Vec<HirItem>,
}

/// HIR item (desugared).
#[derive(Debug, Clone)]
pub enum HirItem {
    FnDef(HirFnDef),
    StructDef(HirStructDef),
    EnumDef(HirEnumDef),
    TraitDef(HirTraitDef),
    ImplBlock(HirImplBlock),
    TypeAlias(HirTypeAlias),
    ConstDef(HirConstDef),
    Import(HirImport),
}

/// Desugared function definition.
#[derive(Debug, Clone)]
pub struct HirFnDef {
    pub def_id: DefId,
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<HirParam>,
    pub ret_ty: Option<Type>,
    pub effect: Effect,
    pub is_extern: bool,
    pub body: HirBlock,
    pub span: Span,
}

/// HIR parameter.
#[derive(Debug, Clone)]
pub struct HirParam {
    pub local_id: LocalId,
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Type,
    pub span: Span,
}

/// HIR block.
#[derive(Debug, Clone)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub tail: Option<Box<HirExpr>>,
    pub span: Span,
}

/// HIR statement.
#[derive(Debug, Clone)]
pub enum HirStmt {
    Let(HirLetStmt),
    Expr(HirExpr),
    Item(HirItem),
    Return(Option<Box<HirExpr>>),
}

/// HIR let statement.
#[derive(Debug, Clone)]
pub struct HirLetStmt {
    pub local_id: LocalId,
    pub mutable: bool,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: HirExpr,
    pub span: Span,
}

/// HIR expression (desugared).
#[derive(Debug, Clone)]
pub enum HirExpr {
    Literal(Literal),
    Ident(Ident, DefId, bool),
    Binary(HirBinaryExpr),
    Unary(HirUnaryExpr),
    Call(HirCallExpr),
    MethodCall(HirMethodCallExpr),
    If(HirIfExpr),
    Match(HirMatchExpr),
    For(HirForExpr),
    While(HirWhileExpr),
    Block(HirBlock),
    Tuple(Vec<HirExpr>),
    Array(Vec<HirExpr>),
    Lambda(HirLambdaExpr),
    Field(HirFieldExpr),
    Index(HirIndexExpr),
    Let(HirLetExpr),
    Return(Option<Box<HirExpr>>),
}

#[derive(Debug, Clone)]
pub struct HirBinaryExpr {
    pub lhs: Box<HirExpr>,
    pub op: BinOp,
    pub rhs: Box<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirUnaryExpr {
    pub op: UnOp,
    pub expr: Box<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirCallExpr {
    pub callee: Box<HirExpr>,
    pub args: Vec<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirMethodCallExpr {
    pub receiver: Box<HirExpr>,
    pub method: Ident,
    pub args: Vec<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirIfExpr {
    pub cond: Box<HirExpr>,
    pub then_branch: HirBlock,
    pub else_branch: Option<Box<HirExpr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirMatchExpr {
    pub scrutinee: Box<HirExpr>,
    pub arms: Vec<HirMatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirMatchArm {
    pub pattern: Pattern,
    pub guard: Option<HirExpr>,
    pub body: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirForExpr {
    pub pattern: Pattern,
    pub iterable: Box<HirExpr>,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirWhileExpr {
    pub cond: Box<HirExpr>,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirLambdaExpr {
    pub params: Vec<HirParam>,
    pub ret_ty: Option<Type>,
    pub effect: Option<Effect>,
    pub body: Box<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFieldExpr {
    pub base: Box<HirExpr>,
    pub field: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirIndexExpr {
    pub base: Box<HirExpr>,
    pub index: Box<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirLetExpr {
    pub local_id: LocalId,
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Box<HirExpr>,
    pub span: Span,
}

/// Desugar the AST to HIR.
pub fn desugar_file(ast: &[Item], file_id: FileId) -> HirFile {
    let mut desugarer = Desugarer::new(file_id);
    let items = ast.iter().map(|item| desugarer.desugar_item(item)).collect();
    HirFile { file_id, items }
}

struct Desugarer {
    file_id: FileId,
    local_counter: u32,
    in_place_vars: HashMap<Ident, LocalId>,
}

impl Desugarer {
    fn new(file_id: FileId) -> Self {
        Desugarer { file_id, local_counter: 0, in_place_vars: HashMap::new() }
    }

    fn fresh_local(&mut self) -> LocalId {
        let id = LocalId(self.local_counter);
        self.local_counter += 1;
        id
    }

    fn desugar_item(&mut self, item: &Item) -> HirItem {
        match item {
            Item::FnDef(f) => HirItem::FnDef(self.desugar_fn_def(f)),
            Item::StructDef(s) => HirItem::StructDef(self.desugar_struct_def(s)),
            Item::EnumDef(e) => HirItem::EnumDef(self.desugar_enum_def(e)),
            Item::TraitDef(t) => HirItem::TraitDef(self.desugar_trait_def(t)),
            Item::ImplBlock(i) => HirItem::ImplBlock(self.desugar_impl_block(i)),
            Item::TypeAlias(t) => HirItem::TypeAlias(self.desugar_type_alias(t)),
            Item::ConstDef(c) => HirItem::ConstDef(self.desugar_const_def(c)),
            Item::Import(i) => HirItem::Import(self.desugar_import(i)),
        }
    }

    fn desugar_fn_def(&mut self, f: &FnDef) -> HirFnDef {
        let def_id = DefId { crate_id: CrateId(self.file_id), index: 0 };
        let mut params = Vec::new();
        for p in &f.params {
            params.push(HirParam {
                local_id: self.fresh_local(),
                mutable: p.mutable,
                pattern: p.pattern.clone(),
                ty: p.ty.clone(),
                span: p.span,
            });
        }
        let body = self.desugar_block(&f.body);
        HirFnDef {
            def_id,
            ident: f.ident.clone(),
            generics: f.generics.clone(),
            params,
            ret_ty: f.ret_ty.clone(),
            effect: f.effect.unwrap_or(Effect::Pure),
            body,
            is_extern: false,
            span: f.span,
        }
    }

    fn desugar_block(&mut self, block: &Block) -> HirBlock {
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.push(self.desugar_stmt(stmt));
        }
        let tail = block.tail.as_ref().map(|e| Box::new(self.desugar_expr(e)));
        HirBlock { stmts, tail, span: block.span }
    }

    fn desugar_stmt(&mut self, stmt: &Stmt) -> HirStmt {
        match stmt {
            Stmt::Let(l) => HirStmt::Let(self.desugar_let_stmt(l)),
            Stmt::Expr(e) => HirStmt::Expr(self.desugar_expr(e)),
            Stmt::Item(i) => HirStmt::Item(self.desugar_item(i)),
            Stmt::Return(e) => HirStmt::Return(e.as_ref().map(|e| Box::new(self.desugar_expr(e)))),
        }
    }

    fn desugar_let_stmt(&mut self, l: &LetStmt) -> HirLetStmt {
        let local_id = self.fresh_local();
        let init = l.init.as_ref().map(|e| self.desugar_expr(e)).unwrap();
        HirLetStmt {
            local_id,
            mutable: l.mutable,
            pattern: l.pattern.clone(),
            ty: l.ty.clone(),
            init,
            span: l.span,
        }
    }

    fn desugar_expr(&mut self, expr: &Expr) -> HirExpr {
        match expr {
            Expr::Literal(l) => HirExpr::Literal(l.clone()),
            Expr::Ident(ident) => {
                if let Some(local_id) = self.in_place_vars.get(ident) {
                    HirExpr::Ident(
                        ident.clone(),
                        DefId { crate_id: CrateId(self.file_id), index: local_id.0 },
                        false,
                    )
                } else {
                    HirExpr::Ident(
                        ident.clone(),
                        DefId { crate_id: CrateId(self.file_id), index: 0 },
                        false,
                    )
                }
            }
            Expr::Binary(b) => HirExpr::Binary(HirBinaryExpr {
                lhs: Box::new(self.desugar_expr(&b.lhs)),
                op: b.op,
                rhs: Box::new(self.desugar_expr(&b.rhs)),
                span: b.span,
            }),
            Expr::Unary(u) => HirExpr::Unary(HirUnaryExpr {
                op: u.op,
                expr: Box::new(self.desugar_expr(&u.expr)),
                span: u.span,
            }),
            Expr::Call(c) => HirExpr::Call(HirCallExpr {
                callee: Box::new(self.desugar_expr(&c.callee)),
                args: c.args.iter().map(|a| self.desugar_expr(a)).collect(),
                span: c.span,
            }),
            Expr::MethodCall(m) => HirExpr::MethodCall(HirMethodCallExpr {
                receiver: Box::new(self.desugar_expr(&m.receiver)),
                method: m.method.clone(),
                args: m.args.iter().map(|a| self.desugar_expr(a)).collect(),
                span: m.span,
            }),
            Expr::If(i) => HirExpr::If(HirIfExpr {
                cond: Box::new(self.desugar_expr(&i.cond)),
                then_branch: self.desugar_block(&i.then_branch),
                else_branch: i.else_branch.as_ref().map(|e| Box::new(self.desugar_expr(e))),
                span: i.span,
            }),
            Expr::Match(m) => HirExpr::Match(HirMatchExpr {
                scrutinee: Box::new(self.desugar_expr(&m.scrutinee)),
                arms: m.arms.iter().map(|a| self.desugar_match_arm(a)).collect(),
                span: m.span,
            }),
            Expr::For(f) => HirExpr::For(HirForExpr {
                pattern: f.pattern.clone(),
                iterable: Box::new(self.desugar_expr(&f.iterable)),
                body: self.desugar_block(&f.body),
                span: f.span,
            }),
            Expr::While(w) => HirExpr::While(HirWhileExpr {
                cond: Box::new(self.desugar_expr(&w.cond)),
                body: self.desugar_block(&w.body),
                span: w.span,
            }),
            Expr::Block(b) => HirExpr::Block(self.desugar_block(b)),
            Expr::Tuple(es) => HirExpr::Tuple(es.iter().map(|e| self.desugar_expr(e)).collect()),
            Expr::Array(es) => HirExpr::Array(es.iter().map(|e| self.desugar_expr(e)).collect()),
            Expr::Lambda(l) => HirExpr::Lambda(self.desugar_lambda(l)),
            Expr::Field(f) => HirExpr::Field(HirFieldExpr {
                base: Box::new(self.desugar_expr(&f.base)),
                field: f.field.clone(),
                span: f.span,
            }),
            Expr::Index(i) => HirExpr::Index(HirIndexExpr {
                base: Box::new(self.desugar_expr(&i.base)),
                index: Box::new(self.desugar_expr(&i.index)),
                span: i.span,
            }),
            Expr::Let(l) => HirExpr::Let(self.desugar_let_expr(l)),
        }
    }

    fn desugar_match_arm(&mut self, arm: &MatchArm) -> HirMatchArm {
        HirMatchArm {
            pattern: arm.pattern.clone(),
            guard: arm.guard.as_ref().map(|g| self.desugar_expr(g)),
            body: self.desugar_expr(&arm.body),
            span: arm.span,
        }
    }

    fn desugar_lambda(&mut self, l: &LambdaExpr) -> HirLambdaExpr {
        let mut params = Vec::new();
        for p in &l.params {
            params.push(HirParam {
                local_id: self.fresh_local(),
                mutable: p.mutable,
                pattern: p.pattern.clone(),
                ty: p.ty.clone(),
                span: p.span,
            });
        }
        let body = Box::new(self.desugar_expr(&l.body));
        HirLambdaExpr { params, ret_ty: l.ret_ty.clone(), effect: l.effect, body, span: l.span }
    }

    fn desugar_let_expr(&mut self, l: &LetExpr) -> HirLetExpr {
        let local_id = self.fresh_local();
        HirLetExpr {
            local_id,
            pattern: l.pattern.clone(),
            ty: l.ty.clone(),
            init: Box::new(self.desugar_expr(&l.init)),
            span: l.span,
        }
    }

    fn desugar_struct_def(&mut self, s: &StructDef) -> HirStructDef {
        let def_id = DefId { crate_id: CrateId(self.file_id), index: 0 };
        HirStructDef {
            def_id,
            ident: s.ident.clone(),
            generics: s.generics.clone(),
            fields: s
                .fields
                .iter()
                .map(|f| StructField { ident: f.ident.clone(), ty: f.ty.clone(), span: f.span })
                .collect(),
            span: s.span,
        }
    }

    fn desugar_enum_def(&mut self, e: &EnumDef) -> HirEnumDef {
        let def_id = DefId { crate_id: CrateId(self.file_id), index: 0 };
        HirEnumDef {
            def_id,
            ident: e.ident.clone(),
            generics: e.generics.clone(),
            variants: e
                .variants
                .iter()
                .map(|v| EnumVariant {
                    ident: v.ident.clone(),
                    fields: v.fields.clone(),
                    span: v.span,
                })
                .collect(),
            span: e.span,
        }
    }

    fn desugar_trait_def(&mut self, t: &TraitDef) -> HirTraitDef {
        let def_id = DefId { crate_id: CrateId(self.file_id), index: 0 };
        HirTraitDef {
            def_id,
            ident: t.ident.clone(),
            generics: t.generics.clone(),
            items: t
                .items
                .iter()
                .map(|i| match i {
                    gala_ast::TraitItem::Fn(sig) => gala_ast::TraitItem::Fn(gala_ast::FnSig {
                        ident: sig.ident.clone(),
                        generics: sig.generics.clone(),
                        params: sig
                            .params
                            .iter()
                            .map(|p| gala_ast::Param {
                                mutable: p.mutable,
                                pattern: p.pattern.clone(),
                                ty: p.ty.clone(),
                                span: p.span,
                            })
                            .collect(),
                        ret_ty: sig.ret_ty.clone(),
                        effect: sig.effect,
                        span: sig.span,
                    }),
                })
                .collect(),
            span: t.span,
        }
    }

    fn desugar_impl_block(&mut self, i: &ImplBlock) -> HirImplBlock {
        HirImplBlock {
            ty: i.ty.clone(),
            effect: i.effect,
            items: i
                .items
                .iter()
                .filter_map(|item| {
                    if let Item::FnDef(f) = item {
                        Some(self.desugar_fn_def(f))
                    } else {
                        None
                    }
                })
                .collect(),
            span: i.span,
        }
    }

    fn desugar_type_alias(&mut self, t: &TypeAlias) -> HirTypeAlias {
        HirTypeAlias {
            ident: t.ident.clone(),
            generics: t.generics.clone(),
            ty: t.ty.clone(),
            span: t.span,
        }
    }

    fn desugar_const_def(&mut self, c: &ConstDef) -> HirConstDef {
        HirConstDef {
            ident: c.ident.clone(),
            ty: c.ty.clone(),
            value: self.desugar_expr(&c.value),
            span: c.span,
        }
    }

    fn desugar_import(&mut self, i: &Import) -> HirImport {
        HirImport {
            path: i.path.clone(),
            alias: i.alias.clone(),
            items: i.items.clone(),
            glob: i.glob,
            span: i.span,
        }
    }
}

// HIR item definitions
#[derive(Debug, Clone)]
pub struct HirStructDef {
    pub def_id: DefId,
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirEnumDef {
    pub def_id: DefId,
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirTraitDef {
    pub def_id: DefId,
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub items: Vec<gala_ast::TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirImplBlock {
    pub ty: Type,
    pub effect: Option<Effect>,
    pub items: Vec<HirFnDef>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirTypeAlias {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirConstDef {
    pub ident: Ident,
    pub ty: Type,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirImport {
    pub path: Path,
    pub alias: Option<Ident>,
    pub items: Option<Vec<Ident>>,
    pub glob: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirFnSig {
    pub ident: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub effect: Option<Effect>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Name Resolution
// ═══════════════════════════════════════════════════════════════════════════════

/// A scope for name resolution.
#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<usize>,
    pub bindings: HashMap<String, ScopeEntry>,
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub enum ScopeEntry {
    Local(LocalId),
    Def(DefId),
    Builtin(BuiltinDef),
}

/// Name resolution context.
#[derive(Debug, Clone)]
pub struct Resolver {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
    pub def_counter: u32,
    pub module_graph: ModuleGraph,
    pub next_module_id: u32,
    pub diagnostics: Diagnostics,
    pub extern_defids: HashSet<DefId>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        let mut resolver = Resolver {
            scopes: vec![Scope { parent: None, bindings: HashMap::new(), depth: 0 }],
            current_scope: 0,
            def_counter: 0,
            module_graph: ModuleGraph::default(),
            next_module_id: 1,
            diagnostics: Diagnostics::new(),
            extern_defids: HashSet::new(),
        };
        resolver.populate_builtins();
        resolver
    }

    fn populate_builtins(&mut self) {
        let builtins = [
            ("qubit", BuiltinDef::QubitAlloc),
            ("qubits", BuiltinDef::QubitsAlloc),
            ("measure", BuiltinDef::Measure),
            ("drop", BuiltinDef::Drop),
            ("h", BuiltinDef::HGate),
            ("x", BuiltinDef::XGate),
            ("y", BuiltinDef::YGate),
            ("z", BuiltinDef::ZGate),
            ("s", BuiltinDef::SGate),
            ("t", BuiltinDef::TGate),
            ("rx", BuiltinDef::RxGate),
            ("ry", BuiltinDef::RyGate),
            ("rz", BuiltinDef::RzGate),
            ("cx", BuiltinDef::CXGate),
            ("cz", BuiltinDef::CZGate),
            ("swap", BuiltinDef::SwapGate),
        ];
        for (name, def) in &builtins {
            self.scopes[0].bindings.insert(name.to_string(), ScopeEntry::Builtin(*def));
        }
    }

    pub fn push_scope(&mut self) -> usize {
        let id = self.scopes.len();
        self.scopes.push(Scope {
            parent: Some(self.current_scope),
            bindings: HashMap::new(),
            depth: self.scopes[self.current_scope].depth + 1,
        });
        self.current_scope = id;
        id
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    pub fn insert_local(&mut self, name: &str, local_id: LocalId) {
        self.scopes[self.current_scope]
            .bindings
            .insert(name.to_string(), ScopeEntry::Local(local_id));
    }

    pub fn fresh_def_id(&mut self, file_id: FileId) -> DefId {
        let id = DefId { crate_id: CrateId(file_id), index: self.def_counter };
        self.def_counter += 1;
        id
    }

    pub fn resolve_name(&self, name: &str) -> Option<ScopeEntry> {
        let mut scope = self.current_scope;
        loop {
            if let Some(entry) = self.scopes[scope].bindings.get(name) {
                return Some(entry.clone());
            }
            if let Some(parent) = self.scopes[scope].parent {
                scope = parent;
            } else {
                break;
            }
        }

        // If not found in scopes, check for extern names
        if self.is_extern_name(name) {
            // Return a special marker for extern names using DefId with max values
            return Some(ScopeEntry::Def(DefId {
                crate_id: CrateId(FileId(u32::MAX)),
                index: u32::MAX,
            }));
        }

        None
    }

    fn is_extern_name(&self, name: &str) -> bool {
        // Extern names come from known crates (e.g. "gala_std.io.print").
        name.starts_with("gala_std.")
    }

    fn is_extern_def_id(&self, def_id: &DefId) -> bool {
        // Extern names resolve to the sentinel DefId with max crate/index values.
        def_id.crate_id.0 .0 == u32::MAX
    }

    /// Resolve all names in a HIR file, returning a new HIR file with resolved DefIds.
    pub fn resolve_file(&mut self, hir_file: &HirFile) -> HirFile {
        let mut resolved_items = Vec::new();
        for item in &hir_file.items {
            let resolved = self.resolve_item(item, hir_file.file_id);
            // Skip import items as they've been processed during resolution
            if !matches!(resolved, HirItem::Import(_)) {
                resolved_items.push(resolved);
            }
        }
        HirFile { file_id: hir_file.file_id, items: resolved_items }
    }

    fn resolve_item(&mut self, item: &HirItem, file_id: FileId) -> HirItem {
        match item {
            HirItem::FnDef(f) => {
                let def_id = self.fresh_def_id(file_id);
                self.scopes[0].bindings.insert(f.ident.0.clone(), ScopeEntry::Def(def_id.clone()));
                let resolved = self.resolve_fn_def(f, def_id, file_id);
                HirItem::FnDef(resolved)
            }
            HirItem::StructDef(s) => {
                let def_id = self.fresh_def_id(file_id);
                self.scopes[0].bindings.insert(s.ident.0.clone(), ScopeEntry::Def(def_id));
                HirItem::StructDef(s.clone())
            }
            HirItem::EnumDef(e) => {
                let def_id = self.fresh_def_id(file_id);
                self.scopes[0].bindings.insert(e.ident.0.clone(), ScopeEntry::Def(def_id));
                HirItem::EnumDef(e.clone())
            }
            HirItem::Import(import) => {
                // Resolve the import so its names become available in scope.
                self.resolve_import(import, file_id);
                HirItem::Import(import.clone())
            }
            _ => item.clone(),
        }
    }

    fn resolve_fn_def(&mut self, f: &HirFnDef, def_id: DefId, file_id: FileId) -> HirFnDef {
        let _ = self.push_scope();

        // Insert parameters into scope
        let mut resolved_params = Vec::new();
        for p in &f.params {
            if let Pattern::Ident(ref name) = p.pattern {
                self.insert_local(&name.0, p.local_id);
            }
            resolved_params.push(p.clone());
        }

        let resolved_body = self.resolve_block(&f.body, file_id);
        self.pop_scope();

        HirFnDef {
            def_id,
            ident: f.ident.clone(),
            generics: f.generics.clone(),
            params: resolved_params,
            ret_ty: f.ret_ty.clone(),
            effect: f.effect,
            body: resolved_body,
            is_extern: f.is_extern,
            span: f.span,
        }
    }

    fn resolve_import(&mut self, import: &HirImport, file_id: FileId) {
        // Resolve the import path to find the referenced module/items
        // For now, we only support imports from known extern crates (like gala_std)
        // In the future, this would be extended to support local modules and other crates

        // Check if this is an import from a known extern crate
        if !import.path.segments.is_empty() {
            let first_segment = &import.path.segments[0];
            let crate_name = &first_segment.ident.0;

            // Check if this is a known extern crate
            if crate_name == "gala_std" {
                // Import from gala_std - resolve the items
                if import.glob {
                    // Import everything: import gala_std.*;
                    // For now, we'll import a predefined set of items
                    self.add_glob_import_from_gala_std(file_id);
                } else if let Some(items) = &import.items {
                    // Import specific items: import gala_std.{ item1, item2 };
                    self.add_named_imports_from_gala_std(items, file_id);
                } else {
                    // Import module: import gala_std;
                    // This makes the module name available, but not its contents
                    // For simplicity, we'll treat this as importing the module itself
                    self.add_module_import("gala_std", file_id);
                }
            }
        }
    }

    fn add_glob_import_from_gala_std(&mut self, _file_id: FileId) {
        // Add all public items from gala_std to the current scope
        // These are the known public items from gala_std
        let gala_std_items = [
            ("io", "gala_std.io"),   // module
            ("str", "gala_std.str"), // module
            ("vec", "gala_std.vec"), // module
            // io module items
            ("print", "gala_std.io.print"),
        ];

        for (local_name, _full_path) in gala_std_items.iter() {
            // Check if this name is already in scope to avoid warnings/errors
            if !self.scopes[0].bindings.contains_key(*local_name) {
                // Create a DefId pointing to the extern location
                let extern_def_id = DefId {
                    crate_id: CrateId(FileId(u32::MAX)),
                    index: self.extern_defids.len() as u32, // Simple counter for now
                };
                self.extern_defids.insert(extern_def_id.clone());

                // Add to scope
                self.scopes[0]
                    .bindings
                    .insert(local_name.to_string(), ScopeEntry::Def(extern_def_id));
            }
        }
    }

    fn add_named_imports_from_gala_std(&mut self, items: &[Ident], _file_id: FileId) {
        // Map of simple names to their full paths in gala_std
        let item_map = [
            ("io", "gala_std.io"),
            ("str", "gala_std.str"),
            ("vec", "gala_std.vec"),
            ("print", "gala_std.io.print"),
        ];

        for item in items {
            let item_name = &item.0;
            if let Some(_full_path) =
                item_map.iter().find_map(
                    |(simple, full)| if *simple == *item_name { Some(*full) } else { None },
                )
            {
                // Check if this name is already in scope to avoid warnings/errors
                if !self.scopes[0].bindings.contains_key(item_name) {
                    // Create a DefId pointing to the extern location
                    let extern_def_id = DefId {
                        crate_id: CrateId(FileId(u32::MAX)),
                        index: self.extern_defids.len() as u32, // Simple counter for now
                    };
                    self.extern_defids.insert(extern_def_id.clone());

                    // Add to scope
                    self.scopes[0]
                        .bindings
                        .insert(item_name.to_string(), ScopeEntry::Def(extern_def_id));
                }
            }
        }
    }

    fn add_module_import(&mut self, module_name: &str, _file_id: FileId) {
        // For module imports like "import gala_std;", we make the module name available
        // but not its contents. This is useful for qualified access like gala_std.io.print
        if !self.scopes[0].bindings.contains_key(module_name) {
            // Create a DefId for the module itself
            let extern_def_id = DefId {
                crate_id: CrateId(FileId(u32::MAX)),
                index: self.extern_defids.len() as u32,
            };
            self.extern_defids.insert(extern_def_id.clone());

            // Add to scope
            self.scopes[0].bindings.insert(module_name.to_string(), ScopeEntry::Def(extern_def_id));
        }
    }

    fn resolve_block(&mut self, block: &HirBlock, file_id: FileId) -> HirBlock {
        let _scope = self.push_scope();
        let mut resolved_stmts = Vec::new();
        for stmt in &block.stmts {
            match stmt {
                HirStmt::Let(l) => {
                    let resolved_init = self.resolve_expr(&l.init, file_id);
                    self.insert_local(&pattern_name(&l.pattern), l.local_id);
                    resolved_stmts.push(HirStmt::Let(HirLetStmt {
                        local_id: l.local_id,
                        mutable: l.mutable,
                        pattern: l.pattern.clone(),
                        ty: l.ty.clone(),
                        init: resolved_init,
                        span: l.span,
                    }));
                }
                HirStmt::Expr(e) => {
                    resolved_stmts.push(HirStmt::Expr(self.resolve_expr(e, file_id)));
                }
                HirStmt::Return(e) => {
                    resolved_stmts.push(HirStmt::Return(
                        e.as_ref().map(|e| Box::new(self.resolve_expr(e, file_id))),
                    ));
                }
                _ => {
                    resolved_stmts.push(stmt.clone());
                }
            }
        }
        let tail = block.tail.as_ref().map(|e| Box::new(self.resolve_expr(e, file_id)));
        self.pop_scope();
        HirBlock { stmts: resolved_stmts, tail, span: block.span }
    }

    fn resolve_expr(&mut self, expr: &HirExpr, file_id: FileId) -> HirExpr {
        match expr {
            HirExpr::Ident(name, _, _) => {
                let resolved_def = match self.resolve_name(&name.0) {
                    Some(entry) => match entry {
                        ScopeEntry::Local(local_id) => {
                            DefId { crate_id: CrateId(file_id), index: local_id.0 }
                        }
                        ScopeEntry::Def(def_id) => def_id,
                        ScopeEntry::Builtin(_) => DefId { crate_id: CrateId(file_id), index: 0 },
                    },
                    None => {
                        let msg = format!("unresolved name: {}", name.0);
                        self.diagnostics.push(Diagnostic::error(codes::UNKNOWN_TYPE, msg));
                        DefId { crate_id: CrateId(file_id), index: 0 }
                    }
                };
                // Check if this is an extern function
                let is_extern = self.is_extern_def_id(&resolved_def);
                HirExpr::Ident(name.clone(), resolved_def, is_extern)
            }
            HirExpr::Binary(b) => HirExpr::Binary(HirBinaryExpr {
                lhs: Box::new(self.resolve_expr(&b.lhs, file_id)),
                op: b.op,
                rhs: Box::new(self.resolve_expr(&b.rhs, file_id)),
                span: b.span,
            }),
            HirExpr::Block(b) => {
                let resolved = self.resolve_block(b, file_id);
                HirExpr::Block(resolved)
            }
            HirExpr::Call(c) => HirExpr::Call(HirCallExpr {
                callee: Box::new(self.resolve_expr(&c.callee, file_id)),
                args: c.args.iter().map(|a| self.resolve_expr(a, file_id)).collect(),
                span: c.span,
            }),
            HirExpr::If(i) => HirExpr::If(HirIfExpr {
                cond: Box::new(self.resolve_expr(&i.cond, file_id)),
                then_branch: self.resolve_block(&i.then_branch, file_id),
                else_branch: i
                    .else_branch
                    .as_ref()
                    .map(|e| Box::new(self.resolve_expr(e, file_id))),
                span: i.span,
            }),
            _ => expr.clone(),
        }
    }

    pub fn build_and_resolve(&mut self, hir_file: &HirFile) -> HirFile {
        self.populate_builtins();

        // First pass: register all top-level definitions
        for item in &hir_file.items {
            match item {
                HirItem::FnDef(f) => {
                    let def_id = self.fresh_def_id(hir_file.file_id);
                    self.scopes[0].bindings.insert(f.ident.0.clone(), ScopeEntry::Def(def_id));
                }
                HirItem::StructDef(s) => {
                    let def_id = self.fresh_def_id(hir_file.file_id);
                    self.scopes[0].bindings.insert(s.ident.0.clone(), ScopeEntry::Def(def_id));
                }
                HirItem::EnumDef(e) => {
                    let def_id = self.fresh_def_id(hir_file.file_id);
                    self.scopes[0].bindings.insert(e.ident.0.clone(), ScopeEntry::Def(def_id));
                }
                _ => {}
            }
        }

        // Second pass: resolve names in each item
        let mut resolved_items = Vec::new();
        for item in &hir_file.items {
            resolved_items.push(self.resolve_item(item, hir_file.file_id));
        }

        // Build module graph
        let root_id = ModuleId(0);
        self.module_graph.root = root_id;
        self.module_graph.modules.insert(
            root_id,
            ModuleData {
                id: root_id,
                name: Ident::new("root"),
                parent: None,
                children: Vec::new(),
                items: Vec::new(),
            },
        );

        HirFile { file_id: hir_file.file_id, items: resolved_items }
    }
}

fn pattern_name(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Ident(i) => i.0.clone(),
        Pattern::Wildcard => "_".to_string(),
        _ => "_".to_string(),
    }
}

/// Build a module graph from a list of files.
pub fn build_module_graph(files: &[(FileId, &HirFile)]) -> ModuleGraph {
    let mut graph = ModuleGraph::default();
    let root_id = ModuleId(0);

    graph.root = root_id;
    graph.modules.insert(
        root_id,
        ModuleData {
            id: root_id,
            name: Ident::new("root"),
            parent: None,
            children: Vec::new(),
            items: Vec::new(),
        },
    );

    for (idx, (file_id, hir_file)) in files.iter().enumerate() {
        let module_id = ModuleId(idx as u32 + 1);

        let mut module_items = Vec::new();
        for item in &hir_file.items {
            if let HirItem::FnDef(f) = item {
                module_items.push(DefId { crate_id: CrateId(*file_id), index: f.def_id.index });
            }
        }

        graph.modules.insert(
            module_id,
            ModuleData {
                id: module_id,
                name: Ident::new(format!("file_{}", file_id.0)),
                parent: Some(root_id),
                children: Vec::new(),
                items: module_items,
            },
        );

        if let Some(root) = graph.modules.get_mut(&root_id) {
            root.children.push(module_id);
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_span::SourceMap;

    #[test]
    fn test_desugar_simple_fn_def() {
        let ast = vec![Item::FnDef(gala_ast::FnDef {
            ident: Ident::new("main"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: Some(gala_ast::Type::Path(gala_ast::Path {
                segments: vec![gala_ast::PathSegment {
                    ident: Ident::new("Int"),
                    type_args: Vec::new(),
                }],
                span: Span::dummy(),
            })),
            effect: Some(gala_ast::Effect::Pure),
            body: gala_ast::Block {
                stmts: vec![gala_ast::Stmt::Return(Some(gala_ast::Expr::Literal(
                    gala_ast::Literal::Int(42),
                )))],
                tail: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        assert_eq!(hir.items.len(), 1);
    }

    #[test]
    fn test_desugar_with_params() {
        let ast = vec![Item::FnDef(gala_ast::FnDef {
            ident: Ident::new("add"),
            generics: Vec::new(),
            params: vec![
                gala_ast::Param {
                    mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("x")),
                    ty: gala_ast::Type::Path(gala_ast::Path {
                        segments: vec![gala_ast::PathSegment {
                            ident: Ident::new("Int"),
                            type_args: Vec::new(),
                        }],
                        span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                },
                gala_ast::Param {
                    mutable: false,
                    pattern: gala_ast::Pattern::Ident(Ident::new("y")),
                    ty: gala_ast::Type::Path(gala_ast::Path {
                        segments: vec![gala_ast::PathSegment {
                            ident: Ident::new("Int"),
                            type_args: Vec::new(),
                        }],
                        span: Span::dummy(),
                    }),
                    span: Span::dummy(),
                },
            ],
            ret_ty: None,
            effect: None,
            body: gala_ast::Block {
                stmts: Vec::new(),
                tail: Some(Box::new(gala_ast::Expr::Literal(gala_ast::Literal::Int(0)))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        if let HirItem::FnDef(f) = &hir.items[0] {
            assert_eq!(f.params.len(), 2);
            assert!(f.ret_ty.is_none());
        } else {
            panic!("expected FnDef");
        }
    }

    #[test]
    fn test_desugar_struct_def() {
        let ast = vec![Item::StructDef(gala_ast::StructDef {
            ident: Ident::new("Point"),
            generics: Vec::new(),
            fields: vec![gala_ast::StructField {
                ident: Ident::new("x"),
                ty: gala_ast::Type::Path(gala_ast::Path {
                    segments: vec![gala_ast::PathSegment {
                        ident: Ident::new("Float"),
                        type_args: Vec::new(),
                    }],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        assert_eq!(hir.items.len(), 1);
    }

    #[test]
    fn test_desugar_local_id_assignment() {
        let ast = vec![Item::FnDef(gala_ast::FnDef {
            ident: Ident::new("f"),
            generics: Vec::new(),
            params: vec![gala_ast::Param {
                mutable: false,
                pattern: gala_ast::Pattern::Ident(Ident::new("a")),
                ty: gala_ast::Type::Path(gala_ast::Path {
                    segments: vec![gala_ast::PathSegment {
                        ident: Ident::new("Int"),
                        type_args: Vec::new(),
                    }],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            ret_ty: None,
            effect: None,
            body: gala_ast::Block {
                stmts: Vec::new(),
                tail: Some(Box::new(gala_ast::Expr::Ident(Ident::new("a")))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        if let HirItem::FnDef(f) = &hir.items[0] {
            assert_eq!(f.params[0].local_id, LocalId(0));
        }
    }

    #[test]
    fn test_def_id_equality() {
        let id1 = DefId { crate_id: CrateId(FileId(0)), index: 0 };
        let id2 = DefId { crate_id: CrateId(FileId(0)), index: 0 };
        let id3 = DefId { crate_id: CrateId(FileId(0)), index: 1 };
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_module_id_default() {
        let m = ModuleId::default();
        assert_eq!(m.0, 0);
    }

    #[test]
    fn test_resolver_builtin_qubit() {
        let resolver = Resolver::new();
        let entry = resolver.resolve_name("qubit");
        assert!(entry.is_some());
        match entry.unwrap() {
            ScopeEntry::Builtin(b) => assert!(matches!(b, BuiltinDef::QubitAlloc)),
            _ => panic!("expected Builtin"),
        }
    }

    #[test]
    fn test_resolver_builtin_cx() {
        let resolver = Resolver::new();
        let entry = resolver.resolve_name("cx");
        assert!(entry.is_some());
        match entry.unwrap() {
            ScopeEntry::Builtin(b) => assert!(matches!(b, BuiltinDef::CXGate)),
            _ => panic!("expected Builtin"),
        }
    }

    #[test]
    fn test_resolver_unknown_name() {
        let resolver = Resolver::new();
        let entry = resolver.resolve_name("nonexistent");
        assert!(entry.is_none());
    }

    #[test]
    fn test_resolver_scope_push_pop() {
        // Directly test scope mechanics without using Resolver methods
        let mut scopes: Vec<Scope> = Vec::new();
        scopes.push(Scope { parent: None, bindings: HashMap::new(), depth: 0 });
        let mut current = 0;

        // Push
        let id = scopes.len();
        scopes.push(Scope {
            parent: Some(current),
            bindings: HashMap::new(),
            depth: scopes[current].depth + 1,
        });
        current = id;

        assert_eq!(current, 1);
        scopes[1].bindings.insert("x".to_string(), ScopeEntry::Local(LocalId(42)));

        assert!(scopes[1].bindings.contains_key("x"), "scope 1 should have 'x'");
        assert!(!scopes[0].bindings.contains_key("x"), "scope 0 should NOT have 'x'");

        // Pop
        if let Some(parent) = scopes[current].parent {
            current = parent;
        }
        assert_eq!(current, 0);

        // Verify resolvable from scope 0
        let entry = resolve_name_at_scope(0, &scopes, "x");
        assert!(entry.is_none(), "'x' should not be visible after scope pop");
    }

    fn resolve_name_at_scope(start: usize, scopes: &[Scope], name: &str) -> Option<ScopeEntry> {
        let mut scope = start;
        loop {
            if let Some(entry) = scopes[scope].bindings.get(name) {
                return Some(entry.clone());
            }
            if let Some(parent) = scopes[scope].parent {
                scope = parent;
            } else {
                return None;
            }
        }
    }

    #[test]
    fn test_resolver_build_and_resolve_fn() {
        let ast = vec![Item::FnDef(gala_ast::FnDef {
            ident: Ident::new("main"),
            generics: Vec::new(),
            params: vec![gala_ast::Param {
                mutable: false,
                pattern: gala_ast::Pattern::Ident(Ident::new("x")),
                ty: gala_ast::Type::Path(gala_ast::Path {
                    segments: vec![gala_ast::PathSegment {
                        ident: Ident::new("Int"),
                        type_args: Vec::new(),
                    }],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            ret_ty: None,
            effect: None,
            body: gala_ast::Block {
                stmts: Vec::new(),
                tail: Some(Box::new(gala_ast::Expr::Ident(Ident::new("x")))),
                span: Span::dummy(),
            },
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        let mut resolver = Resolver::new();
        let resolved = resolver.build_and_resolve(&hir);

        assert_eq!(resolved.items.len(), 1);
        // Verify no resolution errors
        assert!(!resolver.diagnostics.has_errors());
    }

    #[test]
    fn test_build_module_graph() {
        let ast = vec![Item::FnDef(gala_ast::FnDef {
            ident: Ident::new("main"),
            generics: Vec::new(),
            params: Vec::new(),
            ret_ty: None,
            effect: None,
            body: gala_ast::Block { stmts: Vec::new(), tail: None, span: Span::dummy() },
            span: Span::dummy(),
        })];

        let hir = desugar_file(&ast, FileId(0));
        let graph = build_module_graph(&[(FileId(0), &hir)]);
        assert!(graph.modules.contains_key(&ModuleId(0)));
        assert!(graph.modules.contains_key(&ModuleId(1)));
    }

    #[test]
    fn test_resolver_scope_depth() {
        let mut resolver = Resolver::new();
        let s1 = resolver.push_scope();
        assert_eq!(resolver.scopes[s1].depth, 1);
        let s2 = resolver.push_scope();
        assert_eq!(resolver.scopes[s2].depth, 2);
    }
}
