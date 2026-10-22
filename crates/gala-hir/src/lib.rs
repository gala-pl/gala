//! High-level IR: name-resolved, desugared Gala.
//!
//! Lowering from the AST does three things:
//! - resolves every name to a local, a function definition, or a compiler builtin, reporting
//!   `E0202` for names that do not resolve;
//! - gives every local binding and every expression a stable ID so later passes can attach
//!   types, effects, and linearity facts in side tables;
//! - turns imports into bindings, looking modules up in a [`ModuleTable`].

use gala_ast::{self as ast, BinOp, ConstExpr, Effect, Ident, Literal, TypeArg, UnOp};
use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_span::{FileId, Span};
use std::collections::HashMap;

/// A local variable binding within one function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);

/// A top-level function definition in the package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub u32);

/// An expression node, unique across the package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprId(pub u32);

/// Compiler-provided operations. Library declarations without a body bind to these by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Builtin {
    /// `qubit() -> Qubit quantum`: allocate one qubit in |0⟩.
    Qubit,
    /// `qubits<N>() -> Qubits<N> quantum`: allocate a register in |0…0⟩.
    Qubits,
    H,
    X,
    Y,
    Z,
    S,
    T,
    Rx,
    Ry,
    Rz,
    Cx,
    Cz,
    Swap,
    /// `cp(c, t, θ)`: controlled phase.
    Cp,
    /// `measure(q) -> Measured<Bool> prob` (or `Measured<Vec<Bool>>` for a register).
    Measure,
    /// `drop(q) quantum`: release a qubit after synthesized uncomputation.
    Drop,
    /// `print(x)`: classical output, effect `pure`.
    Print,
    /// `clone(q)`: always an error; qubits cannot be copied (E0401).
    Clone,
    /// `grad(f, wrt: p)`: declared, not implemented yet.
    Grad,
}

impl Builtin {
    /// The builtin bound to a prelude or library name, if any.
    pub fn from_name(name: &str) -> Option<Builtin> {
        Some(match name {
            "qubit" => Builtin::Qubit,
            "qubits" => Builtin::Qubits,
            "h" => Builtin::H,
            "x" => Builtin::X,
            "y" => Builtin::Y,
            "z" => Builtin::Z,
            "s" => Builtin::S,
            "t" => Builtin::T,
            "rx" => Builtin::Rx,
            "ry" => Builtin::Ry,
            "rz" => Builtin::Rz,
            "cx" => Builtin::Cx,
            "cz" => Builtin::Cz,
            "swap" => Builtin::Swap,
            "cp" => Builtin::Cp,
            "measure" => Builtin::Measure,
            "drop" => Builtin::Drop,
            "print" => Builtin::Print,
            "clone" => Builtin::Clone,
            "grad" => Builtin::Grad,
            _ => return None,
        })
    }

    /// Source name of the builtin.
    pub fn name(&self) -> &'static str {
        match self {
            Builtin::Qubit => "qubit",
            Builtin::Qubits => "qubits",
            Builtin::H => "h",
            Builtin::X => "x",
            Builtin::Y => "y",
            Builtin::Z => "z",
            Builtin::S => "s",
            Builtin::T => "t",
            Builtin::Rx => "rx",
            Builtin::Ry => "ry",
            Builtin::Rz => "rz",
            Builtin::Cx => "cx",
            Builtin::Cz => "cz",
            Builtin::Swap => "swap",
            Builtin::Cp => "cp",
            Builtin::Measure => "measure",
            Builtin::Drop => "drop",
            Builtin::Print => "print",
            Builtin::Clone => "clone",
            Builtin::Grad => "grad",
        }
    }

    /// True for unitary gates that take qubits and return the same qubits.
    pub fn is_gate(&self) -> bool {
        matches!(
            self,
            Builtin::H
                | Builtin::X
                | Builtin::Y
                | Builtin::Z
                | Builtin::S
                | Builtin::T
                | Builtin::Rx
                | Builtin::Ry
                | Builtin::Rz
                | Builtin::Cx
                | Builtin::Cz
                | Builtin::Swap
                | Builtin::Cp
        )
    }

    /// Number of qubit arguments a gate takes (0 for non-gates).
    pub fn qubit_arity(&self) -> usize {
        match self {
            Builtin::H
            | Builtin::X
            | Builtin::Y
            | Builtin::Z
            | Builtin::S
            | Builtin::T
            | Builtin::Rx
            | Builtin::Ry
            | Builtin::Rz => 1,
            Builtin::Cx | Builtin::Cz | Builtin::Swap | Builtin::Cp => 2,
            _ => 0,
        }
    }

    /// Number of classical angle arguments a gate takes after its qubits.
    pub fn angle_arity(&self) -> usize {
        match self {
            Builtin::Rx | Builtin::Ry | Builtin::Rz | Builtin::Cp => 1,
            _ => 0,
        }
    }
}

/// A module in the package, identified by its dotted path (`""` for the root file).
#[derive(Debug, Clone)]
pub struct SourceModule {
    /// Dotted module path, for example `gala.algorithms` or `mypkg.math`.
    pub path: String,
    /// File the items came from.
    pub file: FileId,
    /// Parsed items.
    pub items: Vec<ast::Item>,
}

/// Every module the program can import, keyed by dotted path.
#[derive(Debug, Clone, Default)]
pub struct ModuleTable {
    modules: HashMap<String, SourceModule>,
}

impl ModuleTable {
    /// Build a table from a list of modules.
    pub fn new(modules: Vec<SourceModule>) -> Self {
        ModuleTable { modules: modules.into_iter().map(|m| (m.path.clone(), m)).collect() }
    }

    /// Look up a module by dotted path.
    pub fn get(&self, path: &str) -> Option<&SourceModule> {
        self.modules.get(path)
    }
}

/// A resolved function, lowered to HIR.
#[derive(Debug, Clone)]
pub struct HirFn {
    /// This function's ID.
    pub def: DefId,
    /// Module the function was declared in.
    pub module: String,
    /// Source name.
    pub name: Ident,
    /// Attributes (`#[property(...)]`), kept for the test harness.
    pub attributes: Vec<ast::Attribute>,
    /// Parameters, in order.
    pub params: Vec<HirParam>,
    /// Declared return type, if written.
    pub ret_ty: Option<ast::Type>,
    /// Declared effect, if written.
    pub declared_effect: Option<Effect>,
    /// Body; `None` for intrinsic declarations.
    pub body: Option<HirBlock>,
    /// The builtin this declaration maps to when it has no body.
    pub intrinsic: Option<Builtin>,
    /// Generic parameters (const generics are recorded but not solved yet).
    pub generics: Vec<ast::GenericParam>,
    /// Source span of the declaration.
    pub span: Span,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct HirParam {
    /// The binding this parameter introduces.
    pub local: LocalId,
    /// Source name (for diagnostics).
    pub name: String,
    /// Declared type.
    pub ty: ast::Type,
    /// Span of the parameter.
    pub span: Span,
}

/// A block of statements with an optional tail expression.
#[derive(Debug, Clone)]
pub struct HirBlock {
    /// Statements in order.
    pub stmts: Vec<HirStmt>,
    /// Tail expression, whose value is the block's value.
    pub tail: Option<Box<HirExpr>>,
    /// Span of the block.
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum HirStmt {
    /// `let pat [: ty] = init`.
    Let { pat: HirPat, ty: Option<ast::Type>, init: HirExpr, span: Span },
    /// An expression evaluated for effect.
    Expr(HirExpr),
    /// `return value`.
    Return { value: Option<HirExpr>, span: Span },
}

/// An expression with its ID and span.
#[derive(Debug, Clone)]
pub struct HirExpr {
    /// Unique ID; keys the side tables computed by later passes.
    pub id: ExprId,
    /// What the expression is.
    pub kind: HirExprKind,
    /// Source span.
    pub span: Span,
}

/// Resolved expression kinds.
#[derive(Debug, Clone)]
pub enum HirExprKind {
    Lit(Literal),
    Local(LocalId, String),
    Fn(DefId),
    Builtin(Builtin),
    /// A top-level constant, by index into [`HirPackage::consts`].
    Const(usize),
    Call {
        callee: Box<HirExpr>,
        generics: Vec<TypeArg>,
        args: Vec<HirExpr>,
        names: Vec<Option<Ident>>,
    },
    Unary {
        op: UnOp,
        expr: Box<HirExpr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
    },
    If {
        cond: Box<HirExpr>,
        then_block: HirBlock,
        else_expr: Option<Box<HirExpr>>,
    },
    For {
        pat: HirPat,
        iterable: Box<HirExpr>,
        body: HirBlock,
    },
    While {
        cond: Box<HirExpr>,
        body: HirBlock,
    },
    Block(HirBlock),
    Tuple(Vec<HirExpr>),
    Array(Vec<HirExpr>),
    Index {
        base: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    Assign {
        target: Box<HirExpr>,
        op: Option<BinOp>,
        value: Box<HirExpr>,
    },
    Match {
        scrutinee: Box<HirExpr>,
        arms: Vec<HirArm>,
    },
    /// Placeholder for an expression that failed to resolve. Compilation has already failed.
    Error,
}

/// A `match` arm.
#[derive(Debug, Clone)]
pub struct HirArm {
    /// Pattern.
    pub pat: HirPat,
    /// Optional guard.
    pub guard: Option<HirExpr>,
    /// Arm body.
    pub body: HirExpr,
}

/// A binding pattern.
#[derive(Debug, Clone)]
pub enum HirPat {
    /// Binds a new local.
    Bind { local: LocalId, name: String, mutable: bool, span: Span },
    /// `_`.
    Wild,
    /// `(p1, p2, …)`.
    Tuple(Vec<HirPat>, Span),
    /// A literal to match against.
    Lit(Literal, Span),
}

/// The output of lowering a whole package.
#[derive(Debug, Clone, Default)]
pub struct HirPackage {
    /// Every function in every module, indexed by `DefId`.
    pub fns: Vec<HirFn>,
    /// Name of each local, indexed by `LocalId`.
    pub local_names: Vec<String>,
    /// Number of expression IDs allocated.
    pub expr_count: u32,
    /// The `main` function of the root module, if any.
    pub main: Option<DefId>,
    /// Top-level constants: name → (module, initializer).
    pub consts: Vec<HirConst>,
}

/// A top-level constant (`let x = …` or `const x: T = …`).
#[derive(Debug, Clone)]
pub struct HirConst {
    /// Module the constant was declared in.
    pub module: String,
    /// Source name.
    pub name: String,
    /// Declared type, if written.
    pub ty: Option<ast::Type>,
    /// Initializer, lowered as a body with no parameters.
    pub value: HirExpr,
    /// Span of the declaration.
    pub span: Span,
}

impl HirPackage {
    /// Function by ID.
    pub fn func(&self, def: DefId) -> Option<&HirFn> {
        self.fns.get(def.0 as usize)
    }

    /// Name of a local.
    pub fn local_name(&self, local: LocalId) -> &str {
        self.local_names.get(local.0 as usize).map(|s| s.as_str()).unwrap_or("?")
    }
}

/// Lower every module of a package to HIR. `root` is the module whose `main` becomes the entry.
pub fn lower_package(modules: &[SourceModule], root: &str) -> (HirPackage, Diagnostics) {
    let mut lowerer = Lowerer::new(modules);
    lowerer.run(root);
    (lowerer.pkg, lowerer.diags)
}

/// Lower a single file as the root module.
pub fn lower_file(file: FileId, items: Vec<ast::Item>) -> (HirPackage, Diagnostics) {
    lower_package(&[SourceModule { path: String::new(), file, items }], "")
}

/// Per-module symbol table: which names are defined or imported here.
#[derive(Debug, Default, Clone)]
struct ModuleScope {
    fns: HashMap<String, DefId>,
    consts: HashMap<String, usize>,
    imports: HashMap<String, Binding>,
}

#[derive(Debug, Clone, Copy)]
enum Binding {
    Def(DefId),
    Builtin(Builtin),
}

struct Lowerer<'a> {
    modules: &'a [SourceModule],
    pkg: HirPackage,
    diags: Diagnostics,
    scopes: HashMap<String, ModuleScope>,
    /// Stack of local-name scopes for the function being lowered.
    locals: Vec<HashMap<String, LocalId>>,
    next_expr: u32,
    current_module: String,
}

impl<'a> Lowerer<'a> {
    fn new(modules: &'a [SourceModule]) -> Self {
        Lowerer {
            modules,
            pkg: HirPackage::default(),
            diags: Diagnostics::new(),
            scopes: HashMap::new(),
            locals: Vec::new(),
            next_expr: 0,
            current_module: String::new(),
        }
    }

    fn run(&mut self, root: &str) {
        // Pass 1: allocate DefIds for every function so calls can refer forward.
        for module in self.modules {
            let mut scope = ModuleScope::default();
            for item in &module.items {
                if let ast::Item::FnDef(f) = item {
                    let def = DefId(self.pkg.fns.len() as u32);
                    scope.fns.insert(f.ident.as_str().to_string(), def);
                    // Placeholder, replaced in pass 2.
                    self.pkg.fns.push(HirFn {
                        def,
                        module: module.path.clone(),
                        name: f.ident.clone(),
                        attributes: f.attributes.clone(),
                        params: Vec::new(),
                        ret_ty: f.ret_ty.clone(),
                        declared_effect: f.effect,
                        body: None,
                        // A signature-only declaration whose name is a builtin is an intrinsic.
                        intrinsic: if f.body.is_none() {
                            Builtin::from_name(f.ident.as_str())
                        } else {
                            None
                        },
                        generics: f.generics.clone(),
                        span: f.span,
                    });
                }
            }
            self.scopes.insert(module.path.clone(), scope);
        }

        // Pass 2: imports, then constants, then function bodies.
        for module in self.modules {
            self.resolve_imports(module);
        }
        for module in self.modules {
            self.current_module = module.path.clone();
            self.lower_consts(module);
        }
        let mut fn_index = 0usize;
        for module in self.modules {
            self.current_module = module.path.clone();
            for item in &module.items {
                if let ast::Item::FnDef(f) = item {
                    let def = DefId(fn_index as u32);
                    fn_index += 1;
                    let lowered = self.lower_fn(def, f);
                    self.pkg.fns[def.0 as usize] = lowered;
                }
            }
        }

        if let Some(scope) = self.scopes.get(root) {
            self.pkg.main = scope.fns.get("main").copied();
        }
        self.pkg.expr_count = self.next_expr;
    }

    fn resolve_imports(&mut self, module: &SourceModule) {
        for item in &module.items {
            let ast::Item::Import(imp) = item else { continue };
            let segs: Vec<String> =
                imp.path.segments.iter().map(|s| s.ident.as_str().to_string()).collect();
            let names: Vec<String> = match (&imp.items, imp.glob) {
                (Some(items), _) => items.iter().map(|i| i.as_str().to_string()).collect(),
                (None, true) => Vec::new(),
                (None, false) => {
                    // `import a.b.c` binds the last segment (or the alias).
                    segs.last().cloned().into_iter().collect()
                }
            };
            // The module path is everything before the imported names.
            let module_path = if imp.items.is_some() || imp.glob {
                segs.join(".")
            } else {
                segs[..segs.len().saturating_sub(1)].join(".")
            };

            let is_builtin_module =
                module_path.starts_with("gala") || module_path.starts_with("gala_std");
            let known = self.modules.iter().find(|m| m.path == module_path);

            if imp.glob {
                if let Some(target) = known {
                    let defs = self.scope_fns(&target.path);
                    for (name, def) in defs {
                        self.bind_import(module, &name, Binding::Def(def));
                    }
                } else if !is_builtin_module {
                    self.diags.push(
                        Diagnostic::error(
                            codes::UNRESOLVED_NAME,
                            format!("unresolved module `{module_path}`"),
                        )
                        .with_primary_label(imp.span, "no such module")
                        .with_help("check the module path or add the module to the package"),
                    );
                } else {
                    // Glob over a builtin library: everything in the prelude is already visible.
                }
                continue;
            }

            for name in names {
                let binding = if known.is_some() {
                    self.scope_fns(&module_path).get(&name).copied().map(Binding::Def)
                } else {
                    None
                };
                let binding = binding.or_else(|| {
                    if is_builtin_module {
                        Builtin::from_name(&name).map(Binding::Builtin)
                    } else {
                        None
                    }
                });
                match binding {
                    Some(b) => {
                        let local_name = imp
                            .alias
                            .as_ref()
                            .filter(|_| imp.items.is_none())
                            .map(|a| a.as_str().to_string())
                            .unwrap_or_else(|| name.clone());
                        self.bind_import(module, &local_name, b);
                    }
                    None => {
                        self.diags.push(
                            Diagnostic::error(
                                codes::UNRESOLVED_NAME,
                                format!("unresolved import `{name}` from `{module_path}`"),
                            )
                            .with_primary_label(imp.span, "not found")
                            .with_help("check the spelling and that the module exports this name"),
                        );
                    }
                }
            }
        }
    }

    /// The value a function name denotes: intrinsic declarations are their builtin.
    fn def_value(&self, def: DefId) -> HirExprKind {
        match self.pkg.fns.get(def.0 as usize).and_then(|f| f.intrinsic) {
            Some(b) => HirExprKind::Builtin(b),
            None => HirExprKind::Fn(def),
        }
    }

    fn scope_fns(&self, module: &str) -> HashMap<String, DefId> {
        self.scopes.get(module).map(|s| s.fns.clone()).unwrap_or_default()
    }

    fn bind_import(&mut self, module: &SourceModule, name: &str, binding: Binding) {
        if let Some(scope) = self.scopes.get_mut(&module.path) {
            scope.imports.insert(name.to_string(), binding);
        }
    }

    fn lower_consts(&mut self, module: &SourceModule) {
        for item in &module.items {
            let ast::Item::ConstDef(c) = item else { continue };
            self.locals.push(HashMap::new());
            let value = self.lower_expr(&c.value);
            self.locals.pop();
            let idx = self.pkg.consts.len();
            self.pkg.consts.push(HirConst {
                module: module.path.clone(),
                name: c.ident.as_str().to_string(),
                ty: c.ty.clone(),
                value,
                span: c.span,
            });
            if let Some(scope) = self.scopes.get_mut(&module.path) {
                scope.consts.insert(c.ident.as_str().to_string(), idx);
            }
        }
    }

    fn lower_fn(&mut self, def: DefId, f: &ast::FnDef) -> HirFn {
        self.locals.clear();
        self.locals.push(HashMap::new());
        let mut params = Vec::new();
        for p in &f.params {
            let (name, span) = pattern_name(&p.pattern, p.span);
            let local = self.fresh_local(&name);
            if let Some(scope) = self.locals.last_mut() {
                scope.insert(name.clone(), local);
            }
            params.push(HirParam { local, name, ty: p.ty.clone(), span });
        }
        let body = f.body.as_ref().map(|b| self.lower_block(b));

        // A signature-only declaration in a library module is an intrinsic.
        let intrinsic = if body.is_none() {
            let builtin = Builtin::from_name(f.ident.as_str());
            if builtin.is_none() {
                self.diags.push(
                    Diagnostic::error(
                        codes::UNRESOLVED_NAME,
                        format!(
                            "function `{}` has no body and is not a compiler intrinsic",
                            f.ident
                        ),
                    )
                    .with_primary_label(f.span, "declared without a body"),
                );
            }
            builtin
        } else {
            None
        };
        HirFn {
            def,
            module: self.current_module.clone(),
            name: f.ident.clone(),
            attributes: f.attributes.clone(),
            params,
            ret_ty: f.ret_ty.clone(),
            declared_effect: f.effect,
            body,
            intrinsic,
            generics: f.generics.clone(),
            span: f.span,
        }
    }

    fn fresh_local(&mut self, name: &str) -> LocalId {
        let id = LocalId(self.pkg.local_names.len() as u32);
        self.pkg.local_names.push(name.to_string());
        id
    }

    fn fresh_expr(&mut self) -> ExprId {
        let id = ExprId(self.next_expr);
        self.next_expr += 1;
        id
    }

    fn lower_block(&mut self, block: &ast::Block) -> HirBlock {
        self.locals.push(HashMap::new());
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            match stmt {
                ast::Stmt::Let(l) => {
                    // Lower the initializer before binding the pattern, so `let a = h(a)` refers
                    // to the old `a` on the right-hand side.
                    let init = match &l.init {
                        Some(e) => self.lower_expr(e),
                        None => {
                            self.diags.push(
                                Diagnostic::error(
                                    codes::UNEXPECTED_TOKEN,
                                    "`let` requires an initializer",
                                )
                                .with_primary_label(l.span, "missing `= value`"),
                            );
                            self.error_expr(l.span)
                        }
                    };
                    let pat = self.lower_pattern(&l.pattern, l.mutable, l.span);
                    stmts.push(HirStmt::Let { pat, ty: l.ty.clone(), init, span: l.span });
                }
                ast::Stmt::Expr(e) => stmts.push(HirStmt::Expr(self.lower_expr(e))),
                ast::Stmt::Return(v) => {
                    let value = v.as_ref().map(|e| self.lower_expr(e));
                    stmts.push(HirStmt::Return { value, span: block.span });
                }
                ast::Stmt::Item(_) => {}
            }
        }
        let tail = block.tail.as_ref().map(|e| Box::new(self.lower_expr(e)));
        self.locals.pop();
        HirBlock { stmts, tail, span: block.span }
    }

    fn error_expr(&mut self, span: Span) -> HirExpr {
        HirExpr { id: self.fresh_expr(), kind: HirExprKind::Error, span }
    }

    fn lower_pattern(&mut self, pat: &ast::Pattern, mutable: bool, span: Span) -> HirPat {
        match pat {
            ast::Pattern::Ident(name) => {
                let local = self.fresh_local(name.as_str());
                if let Some(scope) = self.locals.last_mut() {
                    scope.insert(name.as_str().to_string(), local);
                }
                HirPat::Bind { local, name: name.as_str().to_string(), mutable, span }
            }
            ast::Pattern::Wildcard => HirPat::Wild,
            ast::Pattern::Tuple(items) => HirPat::Tuple(
                items.iter().map(|p| self.lower_pattern(p, mutable, span)).collect(),
                span,
            ),
            ast::Pattern::Literal(l) => HirPat::Lit(l.clone(), span),
            ast::Pattern::Struct { .. } => {
                self.diags.push(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        "struct patterns are not supported yet",
                    )
                    .with_primary_label(span, "here"),
                );
                HirPat::Wild
            }
        }
    }

    /// Resolve a single-name reference.
    fn resolve_name(&mut self, name: &str, span: Span) -> HirExprKind {
        for scope in self.locals.iter().rev() {
            if let Some(local) = scope.get(name) {
                return HirExprKind::Local(*local, name.to_string());
            }
        }
        let module = self.current_module.clone();
        if let Some(scope) = self.scopes.get(&module) {
            if let Some(def) = scope.fns.get(name) {
                return self.def_value(*def);
            }
            if let Some(binding) = scope.imports.get(name) {
                return match binding {
                    Binding::Def(d) => self.def_value(*d),
                    Binding::Builtin(b) => HirExprKind::Builtin(*b),
                };
            }
            if let Some(idx) = scope.consts.get(name) {
                return HirExprKind::Const(*idx);
            }
        }
        if let Some(builtin) = Builtin::from_name(name) {
            return HirExprKind::Builtin(builtin);
        }
        self.diags.push(
            Diagnostic::error(
                codes::UNRESOLVED_NAME,
                format!("cannot find `{name}` in this scope"),
            )
            .with_primary_label(span, "not found here")
            .with_help("declare it with `let`, define a `fn`, or import it"),
        );
        HirExprKind::Error
    }

    fn lower_expr(&mut self, e: &ast::Expr) -> HirExpr {
        let span = e.span;
        let kind = match &e.kind {
            ast::ExprKind::Literal(l) => HirExprKind::Lit(l.clone()),
            ast::ExprKind::Ident(name) => self.resolve_name(name.as_str(), span),
            ast::ExprKind::Binary(b) => HirExprKind::Binary {
                op: b.op,
                lhs: Box::new(self.lower_expr(&b.lhs)),
                rhs: Box::new(self.lower_expr(&b.rhs)),
            },
            ast::ExprKind::Unary(u) => {
                HirExprKind::Unary { op: u.op, expr: Box::new(self.lower_expr(&u.expr)) }
            }
            ast::ExprKind::Call(c) => {
                let callee = self.lower_expr(&c.callee);
                let args = c.args.iter().map(|a| self.lower_expr(a)).collect();
                HirExprKind::Call {
                    callee: Box::new(callee),
                    generics: c.generics.clone(),
                    args,
                    names: c.names.clone(),
                }
            }
            ast::ExprKind::MethodCall(m) => {
                self.diags.push(
                    Diagnostic::error(
                        codes::UNRESOLVED_NAME,
                        format!("method `{}` is not supported yet", m.method),
                    )
                    .with_primary_label(span, "here"),
                );
                HirExprKind::Error
            }
            ast::ExprKind::If(i) => HirExprKind::If {
                cond: Box::new(self.lower_expr(&i.cond)),
                then_block: self.lower_block(&i.then_branch),
                else_expr: i.else_branch.as_ref().map(|e| Box::new(self.lower_expr(e))),
            },
            ast::ExprKind::Match(m) => {
                let scrutinee = Box::new(self.lower_expr(&m.scrutinee));
                let arms = m
                    .arms
                    .iter()
                    .map(|arm| {
                        self.locals.push(HashMap::new());
                        let pat = self.lower_pattern(&arm.pattern, false, arm.span);
                        let guard = arm.guard.as_ref().map(|g| self.lower_expr(g));
                        let body = self.lower_expr(&arm.body);
                        self.locals.pop();
                        HirArm { pat, guard, body }
                    })
                    .collect();
                HirExprKind::Match { scrutinee, arms }
            }
            ast::ExprKind::For(f) => {
                let iterable = Box::new(self.lower_expr(&f.iterable));
                self.locals.push(HashMap::new());
                let pat = self.lower_pattern(&f.pattern, false, f.span);
                let body = self.lower_block(&f.body);
                self.locals.pop();
                HirExprKind::For { pat, iterable, body }
            }
            ast::ExprKind::While(w) => HirExprKind::While {
                cond: Box::new(self.lower_expr(&w.cond)),
                body: self.lower_block(&w.body),
            },
            ast::ExprKind::Block(b) => HirExprKind::Block(self.lower_block(b)),
            ast::ExprKind::Tuple(items) => {
                HirExprKind::Tuple(items.iter().map(|x| self.lower_expr(x)).collect())
            }
            ast::ExprKind::Array(items) => {
                HirExprKind::Array(items.iter().map(|x| self.lower_expr(x)).collect())
            }
            ast::ExprKind::Index(ix) => HirExprKind::Index {
                base: Box::new(self.lower_expr(&ix.base)),
                index: Box::new(self.lower_expr(&ix.index)),
            },
            ast::ExprKind::Assign(a) => HirExprKind::Assign {
                target: Box::new(self.lower_expr(&a.target)),
                op: a.op,
                value: Box::new(self.lower_expr(&a.value)),
            },
            ast::ExprKind::Field(_) | ast::ExprKind::Lambda(_) | ast::ExprKind::Let(_) => {
                self.diags.push(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        "this expression form is not supported yet",
                    )
                    .with_primary_label(span, "here"),
                );
                HirExprKind::Error
            }
        };
        HirExpr { id: self.fresh_expr(), kind, span }
    }
}

fn pattern_name(p: &ast::Pattern, fallback: Span) -> (String, Span) {
    match p {
        ast::Pattern::Ident(i) => (i.as_str().to_string(), fallback),
        _ => ("_".to_string(), fallback),
    }
}

/// Convenience for callers that only need a constant's integer value (for `Qubits<N>`).
pub fn const_int(c: &ConstExpr) -> Option<i64> {
    match c {
        ConstExpr::Int(v) => Some(*v),
        ConstExpr::Binary { lhs, op, rhs } => {
            let (a, b) = (const_int(lhs)?, const_int(rhs)?);
            match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                _ => None,
            }
        }
        ConstExpr::Ident(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gala_parser::parse_source;

    fn lower(src: &str) -> (HirPackage, Diagnostics) {
        let out = parse_source(FileId(0), src);
        assert!(!out.diagnostics.has_errors(), "parse failed: {:?}", out.diagnostics);
        lower_file(FileId(0), out.items)
    }

    #[test]
    fn resolves_locals_and_builtins() {
        let (pkg, diags) = lower("fn f() -> Int { let x = 1\n x }");
        assert!(!diags.has_errors(), "{:?}", diags);
        let body = pkg.fns[0].body.as_ref().unwrap();
        let tail = body.tail.as_ref().unwrap();
        assert!(matches!(tail.kind, HirExprKind::Local(_, _)));
    }

    #[test]
    fn unresolved_name_is_e0202() {
        let (_, diags) = lower("fn f() -> Int { missing + 1 }");
        assert!(diags.errors().iter().any(|d| d.code == codes::UNRESOLVED_NAME), "{diags:?}");
    }

    #[test]
    fn forward_call_resolves_to_fn() {
        let (pkg, diags) = lower("fn a() -> Int { b() }\nfn b() -> Int { 1 }");
        assert!(!diags.has_errors(), "{diags:?}");
        let body = pkg.fns[0].body.as_ref().unwrap();
        let HirExprKind::Call { callee, .. } = &body.tail.as_ref().unwrap().kind else { panic!() };
        assert!(matches!(callee.kind, HirExprKind::Fn(DefId(1))));
    }

    #[test]
    fn let_rhs_sees_previous_binding() {
        // `let a = h(a)` must refer to the old `a` on the right-hand side.
        let (pkg, diags) = lower("fn f(a: Qubit) -> Qubit quantum { let a = h(a)\n a }");
        assert!(!diags.has_errors(), "{diags:?}");
        let body = pkg.fns[0].body.as_ref().unwrap();
        let HirStmt::Let { init, pat, .. } = &body.stmts[0] else { panic!() };
        let HirExprKind::Call { args, .. } = &init.kind else { panic!() };
        let HirExprKind::Local(old, _) = args[0].kind else { panic!() };
        let HirPat::Bind { local: new, .. } = pat else { panic!() };
        assert_ne!(old, *new, "new binding must be a fresh local");
        assert_eq!(old, pkg.fns[0].params[0].local);
    }

    #[test]
    fn intrinsic_declaration_binds_to_builtin() {
        let (pkg, diags) = lower("fn h(q: Qubit) -> Qubit quantum;");
        assert!(!diags.has_errors(), "{diags:?}");
        assert_eq!(pkg.fns[0].intrinsic, Some(Builtin::H));
    }

    #[test]
    fn bodyless_non_intrinsic_is_an_error() {
        let (_, diags) = lower("fn notabuiltin(q: Qubit) -> Qubit quantum;");
        assert!(diags.has_errors());
    }

    #[test]
    fn import_from_module_table_binds_function() {
        let lib_src = "fn helper() -> Int { 7 }";
        let main_src = "import mylib.{ helper }\nfn main() -> Int { helper() }";
        let lib = parse_source(FileId(1), lib_src);
        let main = parse_source(FileId(0), main_src);
        assert!(!lib.diagnostics.has_errors() && !main.diagnostics.has_errors());
        let modules = vec![
            SourceModule { path: String::new(), file: FileId(0), items: main.items },
            SourceModule { path: "mylib".into(), file: FileId(1), items: lib.items },
        ];
        let (pkg, diags) = lower_package(&modules, "");
        assert!(!diags.has_errors(), "{diags:?}");
        assert!(pkg.main.is_some());
    }

    #[test]
    fn unknown_import_is_reported() {
        let (_, diags) = lower("import nope.{ thing }\nfn main() -> Int { 1 }");
        assert!(diags.errors().iter().any(|d| d.code == codes::UNRESOLVED_NAME));
    }

    #[test]
    fn expression_ids_are_unique() {
        let (pkg, _) = lower("fn f() -> Int { 1 + 2 * 3 }");
        assert!(pkg.expr_count >= 5);
    }
}
