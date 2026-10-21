//! Hybrid runtime for Gala (dispatch, mid-circuit measurement, batching).

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{
    AllocKind, Constant, GateKind, Gir, GirFunc, GirNode, NodeKind, Terminator, TypeKind, ValueId,
};
use gala_sim::{run_simulation, SimBackend as SimulatorBackend, SimConfig};
use std::collections::HashMap;

/// Backend capability descriptor.
#[derive(Debug, Clone)]
pub struct BackendCapability {
    pub name: String,
    pub num_qubits: usize,
    pub native_gates: Vec<String>,
    pub connectivity: Vec<(usize, usize)>,
    pub supports_mid_circuit_measurement: bool,
    pub supports_reset: bool,
}

/// Runtime backend trait.
pub trait Backend: Send + Sync {
    fn capability(&self) -> &BackendCapability;
    fn execute(&self, program: &RuntimeProgram) -> Result<RuntimeResult, Diagnostics>;
}

/// Runtime program (lowered from GIR).
#[derive(Debug, Clone)]
pub struct RuntimeProgram {
    pub operations: Vec<RuntimeOp>,
    pub num_qubits: usize,
    pub num_clbits: usize,
    pub shots: u64,
}

/// Runtime operation.
#[derive(Debug, Clone)]
pub enum RuntimeOp {
    Gate { gate: String, qubits: Vec<usize>, params: Vec<f64> },
    Measure { qubit: usize, clbit: usize },
    Reset { qubit: usize },
    Barrier { qubits: Vec<usize> },
    Conditional { condition: RuntimeCondition, body: Vec<RuntimeOp> },
}

/// Condition for measurement-based control flow.
#[derive(Debug, Clone)]
pub struct RuntimeCondition {
    pub clbit: usize,
    pub value: bool,
}

/// Runtime result.
#[derive(Debug, Clone)]
pub struct RuntimeResult {
    pub counts: HashMap<String, u64>,
    pub memory: Vec<u8>,
}

/// Simulator backend implementation.
pub struct SimBackend {
    pub capability: BackendCapability,
}

impl Default for SimBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SimBackend {
    pub fn new() -> Self {
        SimBackend {
            capability: BackendCapability {
                name: "gala-sim".to_string(),
                num_qubits: 32,
                native_gates: vec!["h", "x", "y", "z", "cx", "rz", "rx", "ry"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                connectivity: (0..31).map(|i| (i, i + 1)).collect(),
                supports_mid_circuit_measurement: true,
                supports_reset: true,
            },
        }
    }
}

impl Backend for SimBackend {
    fn capability(&self) -> &BackendCapability {
        &self.capability
    }

    fn execute(&self, program: &RuntimeProgram) -> Result<RuntimeResult, Diagnostics> {
        let gir = runtime_program_to_gir(program)?;
        let result = run_simulation(
            &gir,
            SimConfig {
                backend: SimulatorBackend::StateVector,
                shots: program.shots,
                seed: None,
                noise_model: None,
            },
        )?;
        Ok(RuntimeResult { counts: result.counts, memory: Vec::new() })
    }
}

/// Hybrid runtime orchestrator.
pub struct Runtime {
    pub backends: HashMap<String, Box<dyn Backend>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        let mut backends: HashMap<String, Box<dyn Backend>> = HashMap::new();
        backends.insert("sim".to_string(), Box::new(SimBackend::new()));
        Runtime { backends }
    }

    pub fn register_backend(&mut self, name: String, backend: Box<dyn Backend>) {
        self.backends.insert(name, backend);
    }

    pub fn run(
        &self,
        gir: &Gir,
        backend_name: &str,
        shots: u64,
    ) -> Result<RuntimeResult, Diagnostics> {
        let Some(backend) = self.backends.get(backend_name) else {
            return Err(unsupported(&format!("unknown runtime backend '{backend_name}'")));
        };
        if backend_name == "sim" {
            let result = run_simulation(
                gir,
                SimConfig {
                    backend: SimulatorBackend::StateVector,
                    shots,
                    seed: None,
                    noise_model: None,
                },
            )?;
            return Ok(RuntimeResult { counts: result.counts, memory: Vec::new() });
        }
        let program = gir_to_runtime_program(gir, shots)?;
        backend.execute(&program)
    }
}

fn unsupported(message: &str) -> Diagnostics {
    let mut diags = Diagnostics::new();
    diags.push(Diagnostic::error(codes::BACKEND_CAPABILITY_MISMATCH, message));
    diags
}

fn gate_name(gate: GateKind) -> &'static str {
    match gate {
        GateKind::H => "h",
        GateKind::X => "x",
        GateKind::Y => "y",
        GateKind::Z => "z",
        GateKind::S => "s",
        GateKind::T => "t",
        GateKind::Rx => "rx",
        GateKind::Ry => "ry",
        GateKind::Rz => "rz",
        GateKind::CX => "cx",
        GateKind::CZ => "cz",
        GateKind::SWAP => "swap",
        GateKind::Measure => "measure",
    }
}

fn gir_to_runtime_program(gir: &Gir, shots: u64) -> Result<RuntimeProgram, Diagnostics> {
    let Some(func) = gir.funcs.values().min_by_key(|func| func.id.0) else {
        return Ok(RuntimeProgram { operations: Vec::new(), num_qubits: 0, num_clbits: 0, shots });
    };
    if func.blocks.len() != 1 {
        return Err(unsupported("runtime lowering currently requires one basic block"));
    }
    let block =
        func.blocks.values().next().ok_or_else(|| unsupported("runtime function has no block"))?;
    if !matches!(&block.terminator, Some(Terminator::Return(_)) | None) {
        return Err(unsupported("runtime lowering currently supports return terminators only"));
    }
    let mut operations = Vec::new();
    let mut values = HashMap::<ValueId, usize>::new();
    let mut angles = HashMap::<ValueId, f64>::new();
    let mut next_qubit = 0usize;
    let mut next_clbit = 0usize;
    for (index, param) in func.params.iter().enumerate() {
        if let Some(type_id) = func.param_tys.get(index) {
            let Some(ty) = gir.types.get(type_id) else {
                return Err(unsupported("runtime lowering references a missing parameter type"));
            };
            if !matches!(&ty.kind, TypeKind::Qubit) {
                return Err(unsupported(
                    "runtime operation programs currently accept only Qubit parameters",
                ));
            }
        }
        values.insert(*param, next_qubit);
        next_qubit += 1;
    }
    for node_id in &block.nodes {
        let Some(node) = gir.nodes.get(node_id) else {
            return Err(unsupported("runtime lowering encountered a missing GIR node"));
        };
        match &node.kind {
            NodeKind::Constant(Constant::Float(value)) => {
                if let Some(result) = node.results.first() {
                    angles.insert(*result, *value);
                }
            }
            NodeKind::Constant(_) => {}
            NodeKind::Alloc(AllocKind::Qubit) => {
                let Some(result) = node.results.first() else {
                    return Err(unsupported("runtime allocation is missing its result"));
                };
                values.insert(*result, next_qubit);
                next_qubit += 1;
            }
            NodeKind::Alloc(AllocKind::Qubits(_)) | NodeKind::Alloc(AllocKind::Classical(_)) => {
                return Err(unsupported("runtime lowering supports single-qubit allocation only"));
            }
            NodeKind::Free { target } => {
                if !values.contains_key(target) {
                    return Err(unsupported("runtime free references an invalid qubit"));
                }
                values.remove(target);
            }
            NodeKind::Gate(gate) => {
                let Some(first) = node.operands.first().and_then(|value| values.get(value)) else {
                    return Err(unsupported("runtime gate is missing its target"));
                };
                let mut qubits = vec![*first];
                let mut params = Vec::new();
                match gate {
                    GateKind::Rx | GateKind::Ry | GateKind::Rz => {
                        let Some(angle) = node.operands.get(1).and_then(|value| angles.get(value))
                        else {
                            return Err(unsupported(
                                "runtime rotation is missing a constant angle",
                            ));
                        };
                        params.push(*angle);
                    }
                    GateKind::CX | GateKind::CZ | GateKind::SWAP => {
                        let Some(second) = node.operands.get(1).and_then(|value| values.get(value))
                        else {
                            return Err(unsupported(
                                "runtime two-qubit gate is missing its second target",
                            ));
                        };
                        qubits.push(*second);
                    }
                    GateKind::Measure => {
                        return Err(unsupported("runtime measurement must use a Measure node"));
                    }
                    _ => {}
                }
                operations.push(RuntimeOp::Gate {
                    gate: gate_name(*gate).to_string(),
                    qubits,
                    params,
                });
                if let Some(result) = node.results.first() {
                    if let Some(value) =
                        node.operands.first().and_then(|operand| values.get(operand))
                    {
                        values.insert(*result, *value);
                    }
                }
            }
            NodeKind::Measure { target } => {
                let Some(qubit) = values.get(target).copied() else {
                    return Err(unsupported("runtime measurement is missing its target"));
                };
                operations.push(RuntimeOp::Measure { qubit, clbit: next_clbit });
                next_clbit += 1;
            }
            NodeKind::Classical(_)
            | NodeKind::Phi { .. }
            | NodeKind::Call { .. }
            | NodeKind::Tuple { .. }
            | NodeKind::Extract { .. } => {
                return Err(unsupported("runtime lowering does not support this GIR node"));
            }
        }
    }
    Ok(RuntimeProgram { operations, num_qubits: next_qubit, num_clbits: next_clbit, shots })
}

fn runtime_program_to_gir(program: &RuntimeProgram) -> Result<Gir, Diagnostics> {
    let mut gir = Gir::default();
    let mut func = GirFunc::new_test();
    let block = func
        .blocks
        .get_mut(&gala_gir::BlockId(0))
        .ok_or_else(|| unsupported("runtime test function has no entry block"))?;
    let mut qubit_values = Vec::with_capacity(program.num_qubits);
    let mut next_value = 0u32;
    let mut next_node = 0u32;
    for _ in 0..program.num_qubits {
        let value = ValueId(next_value);
        next_value += 1;
        let node_id = gala_gir::NodeId(next_node);
        next_node += 1;
        block.nodes.push(node_id);
        gir.nodes.insert(
            node_id,
            GirNode {
                id: node_id,
                kind: NodeKind::Alloc(AllocKind::Qubit),
                operands: Vec::new(),
                results: vec![value],
                span: gala_span::Span::dummy(),
            },
        );
        qubit_values.push(value);
    }
    let mut last_result = None;
    for operation in &program.operations {
        match operation {
            RuntimeOp::Gate { gate, qubits, params } => {
                let kind = match gate.as_str() {
                    "h" => GateKind::H,
                    "x" => GateKind::X,
                    "y" => GateKind::Y,
                    "z" => GateKind::Z,
                    "s" => GateKind::S,
                    "t" => GateKind::T,
                    "rx" => GateKind::Rx,
                    "ry" => GateKind::Ry,
                    "rz" => GateKind::Rz,
                    "cx" => GateKind::CX,
                    "cz" => GateKind::CZ,
                    "swap" => GateKind::SWAP,
                    _ => return Err(unsupported("runtime program contains an unknown gate")),
                };
                let Some(first) =
                    qubits.first().and_then(|qubit| qubit_values.get(*qubit)).copied()
                else {
                    return Err(unsupported("runtime gate references an invalid qubit"));
                };
                let mut operands = vec![first];
                if matches!(kind, GateKind::CX | GateKind::CZ | GateKind::SWAP) {
                    let Some(second) =
                        qubits.get(1).and_then(|qubit| qubit_values.get(*qubit)).copied()
                    else {
                        return Err(unsupported(
                            "runtime two-qubit gate references an invalid qubit",
                        ));
                    };
                    operands.push(second);
                }
                if matches!(kind, GateKind::Rx | GateKind::Ry | GateKind::Rz) {
                    let Some(angle) = params.first().copied() else {
                        return Err(unsupported("runtime rotation has no angle"));
                    };
                    let angle_value = ValueId(next_value);
                    next_value += 1;
                    let angle_node = gala_gir::NodeId(next_node);
                    next_node += 1;
                    block.nodes.push(angle_node);
                    gir.nodes.insert(
                        angle_node,
                        GirNode {
                            id: angle_node,
                            kind: NodeKind::Constant(Constant::Float(angle)),
                            operands: Vec::new(),
                            results: vec![angle_value],
                            span: gala_span::Span::dummy(),
                        },
                    );
                    operands.push(angle_value);
                }
                let result = ValueId(next_value);
                next_value += 1;
                let node_id = gala_gir::NodeId(next_node);
                next_node += 1;
                block.nodes.push(node_id);
                gir.nodes.insert(
                    node_id,
                    GirNode {
                        id: node_id,
                        kind: NodeKind::Gate(kind),
                        operands,
                        results: vec![result],
                        span: gala_span::Span::dummy(),
                    },
                );
                if let Some(qubit) = qubits.first().copied() {
                    if let Some(value) = qubit_values.get_mut(qubit) {
                        *value = result;
                    }
                }
                last_result = Some(result);
            }
            RuntimeOp::Measure { qubit, .. } => {
                let Some(target) = qubit_values.get(*qubit).copied() else {
                    return Err(unsupported("runtime measurement references an invalid qubit"));
                };
                let result = ValueId(next_value);
                next_value += 1;
                let node_id = gala_gir::NodeId(next_node);
                next_node += 1;
                block.nodes.push(node_id);
                gir.nodes.insert(
                    node_id,
                    GirNode {
                        id: node_id,
                        kind: NodeKind::Measure { target },
                        operands: vec![target],
                        results: vec![result],
                        span: gala_span::Span::dummy(),
                    },
                );
                last_result = Some(result);
            }
            RuntimeOp::Reset { .. } | RuntimeOp::Barrier { .. } | RuntimeOp::Conditional { .. } => {
                return Err(unsupported(
                    "runtime operation is not supported by the state-vector bridge",
                ));
            }
        }
    }
    block.terminator = Some(Terminator::Return(last_result));
    gir.funcs.insert(func.id, func);
    Ok(gir)
}

/// Batch multiple gradient evaluations.
pub struct GradientBatcher {
    pub pending: Vec<(RuntimeProgram, std::sync::mpsc::Sender<RuntimeResult>)>,
}

impl Default for GradientBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl GradientBatcher {
    pub fn new() -> Self {
        GradientBatcher { pending: Vec::new() }
    }

    pub fn submit(
        &mut self,
        program: RuntimeProgram,
        sender: std::sync::mpsc::Sender<RuntimeResult>,
    ) {
        self.pending.push((program, sender));
    }

    pub fn flush(&mut self, backend: &dyn Backend) {
        for (program, sender) in self.pending.drain(..) {
            let _ = backend.execute(&program).map(|r| sender.send(r));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert!(runtime.backends.contains_key("sim"));
    }

    #[test]
    fn test_sim_backend_capability() {
        let backend = SimBackend::new();
        assert_eq!(backend.capability.name, "gala-sim");
        assert_eq!(backend.capability.num_qubits, 32);
    }

    #[test]
    fn test_gradient_batcher_new() {
        let batcher = GradientBatcher::new();
        assert!(batcher.pending.is_empty());
    }

    #[test]
    fn test_runtime_register_backend() {
        let mut runtime = Runtime::new();
        let backend = SimBackend::new();
        runtime.register_backend("custom".to_string(), Box::new(backend));
        assert!(runtime.backends.contains_key("custom"));
    }

    #[test]
    fn test_runtime_result_creation() {
        let result = RuntimeResult { counts: HashMap::new(), memory: Vec::new() };
        assert!(result.counts.is_empty());
        assert!(result.memory.is_empty());
    }

    #[test]
    fn test_runtime_program_creation() {
        let program =
            RuntimeProgram { operations: Vec::new(), num_qubits: 2, num_clbits: 2, shots: 1024 };
        assert_eq!(program.num_qubits, 2);
        assert_eq!(program.shots, 1024);
    }

    #[test]
    fn test_backend_execute_runs_state_vector_program() {
        let backend = SimBackend::new();
        let program =
            RuntimeProgram { operations: Vec::new(), num_qubits: 1, num_clbits: 1, shots: 1 };
        let result = backend.execute(&program);
        let result = result.expect("runtime simulator should execute an empty program");
        assert_eq!(result.counts.get("0"), Some(&1));
    }
}
