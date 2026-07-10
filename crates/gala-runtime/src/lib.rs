//! Hybrid runtime for Gala (dispatch, mid-circuit measurement, batching).

use gala_gir::{Gir, GirFunc};
use gala_diagnostics::{Diagnostic, Diagnostics, codes};
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

impl SimBackend {
    pub fn new() -> Self {
        SimBackend {
            capability: BackendCapability {
                name: "gala-sim".to_string(),
                num_qubits: 32,
                native_gates: vec!["h", "x", "y", "z", "cx", "rz", "rx", "ry"].into_iter().map(String::from).collect(),
                connectivity: (0..31).map(|i| (i, i+1)).collect(),
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

    fn execute(&self, _program: &RuntimeProgram) -> Result<RuntimeResult, Diagnostics> {
        Err(Diagnostics::new())
    }
}

/// Hybrid runtime orchestrator.
pub struct Runtime {
    pub backends: HashMap<String, Box<dyn Backend>>,
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

    pub fn run(&self, gir: &Gir, backend_name: &str, shots: u64) -> Result<RuntimeResult, Diagnostics> {
        let _ = (gir, backend_name, shots);
        Err(Diagnostics::new())
    }
}

/// Batch multiple gradient evaluations.
pub struct GradientBatcher {
    pub pending: Vec<(RuntimeProgram, std::sync::mpsc::Sender<RuntimeResult>)>,
}

impl GradientBatcher {
    pub fn new() -> Self {
        GradientBatcher { pending: Vec::new() }
    }

    pub fn submit(&mut self, program: RuntimeProgram, sender: std::sync::mpsc::Sender<RuntimeResult>) {
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
        let result = RuntimeResult {
            counts: HashMap::new(),
            memory: Vec::new(),
        };
        assert!(result.counts.is_empty());
        assert!(result.memory.is_empty());
    }

    #[test]
    fn test_runtime_program_creation() {
        let program = RuntimeProgram {
            operations: Vec::new(),
            num_qubits: 2,
            num_clbits: 2,
            shots: 1024,
        };
        assert_eq!(program.num_qubits, 2);
        assert_eq!(program.shots, 1024);
    }

    #[test]
    fn test_backend_execute_returns_err_for_now() {
        let backend = SimBackend::new();
        let program = RuntimeProgram {
            operations: Vec::new(),
            num_qubits: 1,
            num_clbits: 1,
            shots: 1,
        };
        let result = backend.execute(&program);
        // Currently returns error since simulator is not hooked up
        assert!(result.is_err());
    }
}