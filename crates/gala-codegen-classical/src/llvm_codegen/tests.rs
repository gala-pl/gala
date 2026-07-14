//! Unit + integration tests for the LLVM IR text backend (WP-1.7.1 / WP-1.7.2).

use super::*;
use gala_ast::Ident;
use gala_gir::{Block, BlockId, FuncId, Gir, GirFunc, GirNode, NodeId, ValueId};
use gala_span::Span;
use gala_types::Effect;
use std::collections::HashMap;

// ── GIR construction helpers ──────────────────────────────────────────────────

fn node(id: u32, kind: NodeKind, operands: &[u32], results: &[u32]) -> GirNode {
    GirNode {
        id: NodeId(id),
        kind,
        operands: operands.iter().map(|v| ValueId(*v)).collect(),
        results: results.iter().map(|v| ValueId(*v)).collect(),
        span: Span::dummy(),
    }
}

fn block(id: u32, nodes: &[GirNode], term: Terminator) -> Block {
    let nids: Vec<NodeId> = nodes.iter().map(|n| n.id).collect();
    Block { id: BlockId(id), nodes: nids, terminator: Some(term), span: Span::dummy() }
}

fn func(
    name: &str,
    effect: Effect,
    params: &[u32],
    ret_ty: Option<u32>,
    nodes: &[GirNode],
    blocks: HashMap<BlockId, Block>,
    entry: u32,
) -> (Gir, FuncId) {
    let mut gir = Gir::default();
    for n in nodes {
        gir.nodes.insert(n.id, n.clone());
    }
    let func = GirFunc {
        id: FuncId(0),
        name: Ident::new(name),
        params: params.iter().map(|p| ValueId(*p)).collect(),
        ret_ty: ret_ty.map(gala_gir::TypeId),
        effect,
        blocks,
        entry_block: BlockId(entry),
        gradient_param_indices: Vec::new(),
        span: Span::dummy(),
    };
    let fid = func.id;
    gir.funcs.insert(fid, func);
    (gir, fid)
}

fn emit_ok(gir: &Gir) -> String {
    emit_llvm(gir).expect("emit_llvm should succeed")
}

/// Run `llvm-as` (if available) on the IR to confirm it is syntactically valid.
fn assert_valid_llvm_ir(ir: &str) {
    let llvm_as = std::env::var("LLVM_AS")
        .ok()
        .or_else(|| find_binary(&["llvm-as", "llvm-as-17", "llvm-as-18"]));
    let Some(llvm_as) = llvm_as else {
        eprintln!("llvm-as not found; skipping IR validation");
        return;
    };
    let mut child = std::process::Command::new(&llvm_as)
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn llvm-as");
    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(ir.as_bytes()).unwrap();
    let output = child.wait_with_output().expect("wait llvm-as");
    assert!(
        output.status.success(),
        "llvm-as failed:\n{}\n--- IR ---\n{}",
        String::from_utf8_lossy(&output.stderr),
        ir
    );
}

fn find_binary(names: &[&str]) -> Option<String> {
    for name in names {
        if let Ok(path) = which(name) {
            return Some(path);
        }
    }
    None
}

fn which(name: &str) -> Result<String, ()> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output()
        .map_err(|_| ())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_classical_arithmetic() {
    // fn add(a: i64, b: i64) -> i64 { a + b }
    let n = node(2, NodeKind::Classical(ClassicalOp::Add), &[0, 1], &[2]);
    let nodes = vec![n];
    let blk = block(0, &nodes, Terminator::Return(Some(ValueId(2))));
    let mut blocks = HashMap::new();
    blocks.insert(blk.id, blk);
    let (gir, _) = func("add", Effect::Pure, &[0, 1], None, &nodes, blocks, 0);

    let ir = emit_ok(&gir);
    assert!(ir.contains("define i64 @add(i64 %v0, i64 %v1)"), "got:\n{ir}");
    assert!(ir.contains("add i64"), "got:\n{ir}");
    assert!(ir.contains("ret i64"), "got:\n{ir}");
    assert_valid_llvm_ir(&ir);
}

#[test]
fn test_classical_comparison() {
    // fn lt(a: i64, b: i64) -> i1 { a < b }
    let n = node(2, NodeKind::Classical(ClassicalOp::Lt), &[0, 1], &[2]);
    let nodes = vec![n];
    let blk = block(0, &nodes, Terminator::Return(Some(ValueId(2))));
    let mut blocks = HashMap::new();
    blocks.insert(blk.id, blk);
    let (gir, _) = func("lt", Effect::Pure, &[0, 1], None, &nodes, blocks, 0);

    let ir = emit_ok(&gir);
    assert!(ir.contains("icmp slt i64"), "got:\n{ir}");
    assert!(ir.contains("define i1 @lt"), "got:\n{ir}");
    assert!(ir.contains("ret i1"), "got:\n{ir}");
    assert_valid_llvm_ir(&ir);
}

#[test]
fn test_control_flow() {
    // if (true) { 1 } else { 2 }
    let c = node(0, NodeKind::Constant(Constant::Bool(true)), &[], &[0]);
    let t = node(1, NodeKind::Constant(Constant::Int(1)), &[], &[1]);
    let f = node(2, NodeKind::Constant(Constant::Int(2)), &[], &[2]);
    let nodes = vec![c, t, f];
    let entry = block(
        0,
        &nodes[0..1],
        Terminator::CondBranch { cond: ValueId(0), then_bb: BlockId(1), else_bb: BlockId(2) },
    );
    let blk_t = block(1, &nodes[1..2], Terminator::Return(Some(ValueId(1))));
    let blk_f = block(2, &nodes[2..3], Terminator::Return(Some(ValueId(2))));

    let mut blocks = HashMap::new();
    blocks.insert(entry.id, entry);
    blocks.insert(blk_t.id, blk_t);
    blocks.insert(blk_f.id, blk_f);
    let (gir, _) = func("sel", Effect::Pure, &[], None, &nodes, blocks, 0);

    let ir = emit_ok(&gir);
    assert!(ir.contains("br i1 %v0, label %block_1, label %block_2"), "got:\n{ir}");
    assert!(ir.contains("block_1:"), "got:\n{ir}");
    assert!(ir.contains("block_2:"), "got:\n{ir}");
    assert!(ir.contains("ret i64"), "got:\n{ir}");
    assert_valid_llvm_ir(&ir);
}

#[test]
fn test_quantum_intrinsics() {
    // qubit q = allocate; h(q); m = measure(q)
    let alloc = node(0, NodeKind::Alloc(AllocKind::Qubit), &[], &[0]);
    let h = node(1, NodeKind::Gate(GateKind::H), &[0], &[1]);
    let m = node(2, NodeKind::Measure { target: ValueId(1) }, &[], &[2]);
    let nodes = vec![alloc, h, m];
    let blk = block(0, &nodes, Terminator::Return(None));
    let mut blocks = HashMap::new();
    blocks.insert(blk.id, blk);
    let (gir, _) = func("q", Effect::Quantum, &[], None, &nodes, blocks, 0);

    let ir = emit_ok(&gir);
    assert!(ir.contains("%Qubit = type opaque"), "got:\n{ir}");
    assert!(ir.contains("declare %Qubit* @__quantum__qis__qubit__allocate()"), "got:\n{ir}");
    assert!(ir.contains("declare void @__quantum__qis__h__body(%Qubit*)"), "got:\n{ir}");
    assert!(ir.contains("declare i1 @__quantum__qis__mz__body(%Qubit*)"), "got:\n{ir}");
    assert!(ir.contains("call %Qubit* @__quantum__qis__qubit__allocate()"), "got:\n{ir}");
    assert!(ir.contains("call void @__quantum__qis__h__body(%Qubit*"), "got:\n{ir}");
    assert!(ir.contains("call i1 @__quantum__qis__mz__body(%Qubit*"), "got:\n{ir}");
    assert_valid_llvm_ir(&ir);
}

#[test]
fn test_two_qubit_gate() {
    // cx(q0, q1)
    let a = node(0, NodeKind::Alloc(AllocKind::Qubit), &[], &[0]);
    let b = node(1, NodeKind::Alloc(AllocKind::Qubit), &[], &[1]);
    let cx = node(2, NodeKind::Gate(GateKind::CX), &[0, 1], &[2, 3]);
    let nodes = vec![a, b, cx];
    let blk = block(0, &nodes, Terminator::Return(None));
    let mut blocks = HashMap::new();
    blocks.insert(blk.id, blk);
    let (gir, _) = func("bell", Effect::Quantum, &[], None, &nodes, blocks, 0);

    let ir = emit_ok(&gir);
    assert!(
        ir.contains("call void @__quantum__qis__cx__body(%Qubit* %v0, %Qubit* %v1)"),
        "got:\n{ir}"
    );
    assert_valid_llvm_ir(&ir);
}

#[test]
fn test_function_call() {
    // fn callee() -> i64 { 7 }
    // fn caller() -> i64 { callee() }
    let seven = node(0, NodeKind::Constant(Constant::Int(7)), &[], &[0]);
    let callee_nodes = vec![seven];
    let blk_c = block(0, &callee_nodes, Terminator::Return(Some(ValueId(0))));
    let mut callee_blocks = HashMap::new();
    callee_blocks.insert(blk_c.id, blk_c);
    let (mut gir, _) = func("callee", Effect::Pure, &[], None, &callee_nodes, callee_blocks, 0);

    // caller uses value ids 10+ to avoid collision with callee
    let call = node(10, NodeKind::Call { func: FuncId(0), args: vec![] }, &[], &[10]);
    let caller_nodes = vec![call];
    let blk_caller = block(0, &caller_nodes, Terminator::Return(Some(ValueId(10))));
    let mut caller_blocks = HashMap::new();
    caller_blocks.insert(blk_caller.id, blk_caller);
    let caller = GirFunc {
        id: FuncId(1),
        name: Ident::new("caller"),
        params: Vec::new(),
        ret_ty: None,
        effect: Effect::Pure,
        blocks: caller_blocks,
        entry_block: BlockId(0),
        gradient_param_indices: Vec::new(),
        span: Span::dummy(),
    };
    gir.funcs.insert(FuncId(1), caller);
    for n in &caller_nodes {
        gir.nodes.insert(n.id, n.clone());
    }

    let ir = emit_ok(&gir);
    assert!(ir.contains("call i64 @callee()"), "got:\n{ir}");
    assert_valid_llvm_ir(&ir);
}

// ── WP-1.6: native object / executable emission ───────────────────────────────

/// Build a classical `main` that returns `value` (as i64). Linking this into an
/// executable and running it must exit with that code.
fn main_returning(value: i64) -> (Gir, FuncId) {
    let c = node(0, NodeKind::Constant(Constant::Int(value)), &[], &[0]);
    let nodes = vec![c];
    let blk = block(0, &nodes, Terminator::Return(Some(ValueId(0))));
    let mut blocks = HashMap::new();
    blocks.insert(blk.id, blk);
    func("main", Effect::Pure, &[], None, &nodes, blocks, 0)
}

#[test]
fn test_emit_object_file() {
    if find_binary(&["llc"]).is_none() {
        eprintln!("llc not found; skipping object emission test");
        return;
    }
    let (gir, _) = main_returning(42);

    let obj = std::env::temp_dir().join(format!("gala_test_obj_{}.o", std::process::id()));
    let _ = std::fs::remove_file(&obj);

    match super::emit_native_object(&gir, &obj) {
        Ok(()) => {}
        Err(e) => {
            let _ = std::fs::remove_file(&obj);
            panic!("emit_native_object failed: {:?}", e);
        }
    }

    assert!(obj.exists(), "object file should exist at {:?}", obj);
    let bytes = std::fs::read(&obj).unwrap();
    assert!(!bytes.is_empty(), "object file must not be empty");
    let is_object = bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe]) // Mach-O (little-endian)
        || bytes.starts_with(&[0xfe, 0xed, 0xfa, 0xcf]) // Mach-O (big-endian)
        || bytes.starts_with(b"\x7fELF") // ELF
        || bytes.starts_with(&[0x4c, 0x01, 0x00, 0x00]); // COFF
    assert!(is_object, "output is not a recognized object file");
    let _ = std::fs::remove_file(&obj);
}

#[test]
fn test_emit_and_run_executable() {
    if find_binary(&["llc"]).is_none() || find_binary(&["cc", "clang"]).is_none() {
        eprintln!("llc/cc not found; skipping executable emission test");
        return;
    }
    let (gir, _) = main_returning(42);

    let exe = std::env::temp_dir().join(format!("gala_test_exe_{}", std::process::id()));
    let _ = std::fs::remove_file(&exe);

    match super::emit_native_executable(&gir, &exe) {
        Ok(()) => {}
        Err(e) => {
            let _ = std::fs::remove_file(&exe);
            panic!("emit_native_executable failed: {:?}", e);
        }
    }

    assert!(exe.exists(), "executable should exist at {:?}", exe);
    let status = std::process::Command::new(&exe).status().expect("run executable");
    assert_eq!(status.code(), Some(42), "main should exit with 42");
    let _ = std::fs::remove_file(&exe);
}
