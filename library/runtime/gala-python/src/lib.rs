//! Python interoperability bridge for Gala.
//!
//! Provides a bridge for calling Gala-compiled quantum kernels from Python
//! and for invoking Python ML frameworks (PyTorch, JAX) from Gala.
//! This enables incremental adoption — teams keep their existing Python
//! pipelines while dropping Gala kernels into critical paths.

use gala_core::int::Int;
use gala_core::float::Float;
use gala_core::bool::Bool;
use std::collections::HashMap;

/// A value that can be passed across the Python-Gala boundary.
#[derive(Debug, Clone)]
pub enum PyValue {
    Int(Int),
    Float(Float),
    Bool(Bool),
    String(String),
    List(Vec<PyValue>),
    Dict(HashMap<String, PyValue>),
    None,
}

/// Describes a Gala kernel that has been exported for Python use.
pub struct KernelExport {
    pub name: String,
    pub input_types: Vec<String>,
    pub output_type: String,
    pub num_qubits: usize,
    pub num_params: usize,
}

/// Registry of Gala kernels exported to Python.
impl Default for PyBridge {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PyBridge {
    exports: HashMap<String, KernelExport>,
}

impl PyBridge {
    pub fn new() -> Self {
        PyBridge {
            exports: HashMap::new(),
        }
    }

    /// Registers an exported Gala kernel.
    pub fn register(&mut self, export: KernelExport) {
        log::info!("registering Python-exported kernel: {}", export.name);
        self.exports.insert(export.name.clone(), export);
    }

    /// Looks up an exported kernel by name.
    pub fn lookup(&self, name: &str) -> Option<&KernelExport> {
        self.exports.get(name)
    }

    /// Calls a registered kernel with the given parameters.
    ///
    /// In a full implementation, this would JIT-compile and execute
    /// the kernel, returning measurement results.
    pub fn call_kernel(&self, name: &str, params: &[f64], shots: u32) -> Result<HashMap<String, u32>, String> {
        let _export = self
            .exports
            .get(name)
            .ok_or_else(|| format!("kernel '{name}' not registered"))?;

        // Simulated execution
        let mut results = HashMap::new();
        results.insert("0".to_string(), shots / 2);
        results.insert("1".to_string(), shots - shots / 2);

        log::info!("executed kernel '{name}' with {} params, {} shots", params.len(), shots);
        Ok(results)
    }

    /// Lists all registered kernels.
    pub fn list_kernels(&self) -> Vec<&KernelExport> {
        self.exports.values().collect()
    }

    /// Returns the number of registered kernels.
    pub fn kernel_count(&self) -> usize {
        self.exports.len()
    }
}

/// A handle for converting PyTorch tensors to Gala params and back.
/// In a full implementation, this links to libtorch or uses the
/// Python C API for zero-copy tensor sharing.
pub struct TensorConversion;

impl TensorConversion {
    /// Converts a flat parameter array to a Gala Params representation.
    pub fn params_to_tensor(params: &[Float]) -> Vec<f64> {
        params.to_vec()
    }

    /// Converts measurement results into a tensor-like structure.
    pub fn results_to_tensor(results: &HashMap<String, u32>, num_bits: usize) -> Vec<f64> {
        let size = 1 << num_bits;
        let mut tensor = vec![0.0_f64; size];
        for (bitstring, count) in results {
            if let Ok(idx) = usize::from_str_radix(bitstring, 2) {
                if idx < size {
                    tensor[idx] = *count as f64;
                }
            }
        }
        tensor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_register_and_lookup() {
        let mut bridge = PyBridge::new();
        let export = KernelExport {
            name: "ansatz".to_string(),
            input_types: vec!["Float".to_string()],
            output_type: "Measured<Bool>".to_string(),
            num_qubits: 4,
            num_params: 8,
        };
        bridge.register(export);
        assert_eq!(bridge.kernel_count(), 1);
        let found = bridge.lookup("ansatz");
        assert!(found.is_some());
        assert_eq!(found.unwrap().num_qubits, 4);
    }

    #[test]
    fn test_call_kernel() {
        let mut bridge = PyBridge::new();
        bridge.register(KernelExport {
            name: "test_kernel".to_string(),
            input_types: vec![],
            output_type: "Measured<Bool>".to_string(),
            num_qubits: 2,
            num_params: 0,
        });
        let result = bridge.call_kernel("test_kernel", &[], 1000);
        assert!(result.is_ok());
        let counts = result.unwrap();
        let total: u32 = counts.values().sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn test_tensor_conversion() {
        let params = vec![0.5, 0.3, 0.8, 1.0];
        let tensor = TensorConversion::params_to_tensor(&params);
        assert_eq!(tensor.len(), 4);
        assert!((tensor[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_call_nonexistent_kernel() {
        let bridge = PyBridge::new();
        let result = bridge.call_kernel("nonexistent", &[], 100);
        assert!(result.is_err());
    }
}
