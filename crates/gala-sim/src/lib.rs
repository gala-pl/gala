//! Simulator backends for Gala (roqoqo/QuEST).

use gala_gir::{Gir, GirFunc};
use gala_span::Span;
use gala_diagnostics::{Diagnostic, Diagnostics, codes};
use std::collections::HashMap;

/// Simulator backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimBackend {
    StateVector,
    Noisy,
    TensorNetwork,
}

/// Simulator configuration.
#[derive(Debug, Clone)]
pub struct SimConfig {
    pub backend: SimBackend,
    pub shots: u64,
    pub seed: Option<u64>,
    pub noise_model: Option<NoiseModel>,
}

/// Noise model for noisy simulation.
#[derive(Debug, Clone)]
pub struct NoiseModel {
    pub depolarizing_prob: Option<f64>,
    pub amplitude_damping: Option<f64>,
    pub phase_damping: Option<f64>,
    pub readout_error: Option<f64>,
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig {
            backend: SimBackend::StateVector,
            shots: 1024,
            seed: None,
            noise_model: None,
        }
    }
}

/// Run a GIR function on the simulator.
pub fn run_simulation(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    match config.backend {
        SimBackend::StateVector => run_state_vector(gir, config),
        SimBackend::Noisy => run_noisy(gir, config),
        SimBackend::TensorNetwork => run_tensor_network(gir, config),
    }
}

fn run_state_vector(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    let _ = (gir, config);
    Err(Diagnostics::new())
}

fn run_noisy(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    let _ = (gir, config);
    Err(Diagnostics::new())
}

fn run_tensor_network(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    let _ = (gir, config);
    Err(Diagnostics::new())
}

/// Simulation result.
#[derive(Debug, Clone)]
pub struct SimResult {
    pub counts: HashMap<String, u64>,
    pub probabilities: HashMap<String, f64>,
    pub state_vector: Option<Vec<f64>>,
    pub shots: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_config_default() {
        let config = SimConfig::default();
        assert_eq!(config.backend, SimBackend::StateVector);
        assert_eq!(config.shots, 1024);
    }

    #[test]
    fn test_sim_config_custom() {
        let config = SimConfig {
            backend: SimBackend::Noisy,
            shots: 100,
            seed: Some(42),
            noise_model: Some(NoiseModel {
                depolarizing_prob: Some(0.01),
                amplitude_damping: None,
                phase_damping: None,
                readout_error: Some(0.05),
            }),
        };
        assert_eq!(config.backend, SimBackend::Noisy);
        assert_eq!(config.shots, 100);
        assert!(config.seed.is_some());
        assert!(config.noise_model.is_some());
    }

    #[test]
    fn test_run_simulation_all_backends() {
        let gir = Gir::default();
        for backend in &[SimBackend::StateVector, SimBackend::Noisy, SimBackend::TensorNetwork] {
            let config = SimConfig {
                backend: *backend,
                shots: 10,
                seed: None,
                noise_model: None,
            };
            // Currently returns error since simulation is not implemented
            let _ = run_simulation(&gir, config);
        }
    }

    #[test]
    fn test_sim_result_creation() {
        let result = SimResult {
            counts: HashMap::new(),
            probabilities: HashMap::new(),
            state_vector: None,
            shots: 1024,
        };
        assert_eq!(result.shots, 1024);
    }

    #[test]
    fn test_noise_model_default() {
        let noise = NoiseModel {
            depolarizing_prob: None,
            amplitude_damping: None,
            phase_damping: None,
            readout_error: None,
        };
        assert!(noise.depolarizing_prob.is_none());
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(format!("{:?}", SimBackend::StateVector), "StateVector");
        assert_eq!(format!("{:?}", SimBackend::Noisy), "Noisy");
    }
}