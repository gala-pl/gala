//! Simulator backends for Gala (roqoqo/QuEST).

use gala_diagnostics::{codes, Diagnostic, Diagnostics};
use gala_gir::{
    AllocKind, BlockId, ClassicalOp, Constant, GateKind, Gir, GirFunc, GirNode, NodeKind,
    Terminator, TypeKind, ValueId,
};
use std::collections::{HashMap, HashSet};

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

impl Default for NoiseModel {
    fn default() -> Self {
        Self {
            depolarizing_prob: Some(0.0),
            amplitude_damping: Some(0.0),
            phase_damping: Some(0.0),
            readout_error: Some(0.0),
        }
    }
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig { backend: SimBackend::StateVector, shots: 1024, seed: None, noise_model: None }
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
    run_state_vector_with_noise(gir, config, None)
}

fn run_state_vector_with_noise(
    gir: &Gir,
    config: SimConfig,
    noise_model: Option<NoiseModel>,
) -> Result<SimResult, Diagnostics> {
    let Some(func) = gir
        .funcs
        .values()
        .find(|func| func.name.0 == "main")
        .or_else(|| gir.funcs.values().min_by_key(|func| func.id.0))
    else {
        return Ok(SimResult {
            counts: HashMap::new(),
            probabilities: HashMap::new(),
            state_vector: Some(vec![1.0, 0.0]),
            shots: config.shots,
        });
    };

    let num_qubits = count_qubits(gir, func);
    if num_qubits > 20 {
        return Err(unsupported("state-vector simulation is limited to 20 qubits"));
    }
    let state_len = 1usize
        .checked_shl(num_qubits as u32)
        .ok_or_else(|| unsupported("the circuit has too many qubits"))?;
    let mut rng = Rng::new(config.seed.unwrap_or(0x6A1A_2026));
    let iterations = config.shots.max(1);
    let mut counts = HashMap::new();
    let mut last_state = vec![(0.0, 0.0); state_len];
    for _shot in 0..iterations {
        let mut state = vec![(0.0, 0.0); state_len];
        state[0] = (1.0, 0.0);
        let mut executor = Executor {
            gir,
            func,
            state: &mut state,
            values: HashMap::new(),
            measurements: Vec::new(),
            rng: &mut rng,
            noise_model: noise_model.clone(),
        };
        executor.execute()?;

        let measurements = std::mem::take(&mut executor.measurements);
        drop(executor);
        let key = if measurements.is_empty() {
            let basis = sample_basis(&state, &mut rng);
            basis_key(basis, num_qubits)
        } else {
            measurements.iter().map(|bit| if *bit { '1' } else { '0' }).collect()
        };
        if config.shots > 0 {
            *counts.entry(key).or_insert(0) += 1;
        }
        last_state = state;
    }

    let probabilities = if config.shots == 0 {
        HashMap::new()
    } else {
        counts
            .iter()
            .map(|(key, count)| (key.clone(), *count as f64 / config.shots as f64))
            .collect()
    };
    Ok(SimResult {
        counts,
        probabilities,
        state_vector: Some(last_state.into_iter().flat_map(|(re, im)| [re, im]).collect()),
        shots: config.shots,
    })
}

fn run_noisy(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    let noise_model = config.noise_model.clone().unwrap_or_default();
    run_state_vector_with_noise(gir, config, Some(noise_model))
}

fn run_tensor_network(gir: &Gir, config: SimConfig) -> Result<SimResult, Diagnostics> {
    // The reference implementation uses the same exact state evolution for
    // small circuits; the backend boundary is still functional and preserves
    // the tensor-network API until a contracted MPS engine is introduced.
    run_state_vector_with_noise(gir, config, None)
}

#[derive(Clone, Debug)]
enum Value {
    Qubit(usize),
    Bool(bool),
    Int(i64),
    Float(f64),
    Tuple(Vec<Value>),
}

#[derive(Clone, Copy)]
enum Numeric {
    Int(i64),
    Float(f64),
}

struct Executor<'a> {
    gir: &'a Gir,
    func: &'a GirFunc,
    state: &'a mut [(f64, f64)],
    values: HashMap<ValueId, Value>,
    measurements: Vec<bool>,
    rng: &'a mut Rng,
    noise_model: Option<NoiseModel>,
}

impl<'a> Executor<'a> {
    fn execute(&mut self) -> Result<Option<Value>, Diagnostics> {
        let quantum_params = self.quantum_params();
        let angle_params = self.angle_params();
        let mut next_qubit = 0;
        for value in &self.func.params {
            let parameter_value = if quantum_params.contains(value) {
                let qubit = next_qubit;
                next_qubit += 1;
                Value::Qubit(qubit)
            } else if angle_params.contains(value) {
                Value::Float(0.0)
            } else {
                Value::Int(0)
            };
            self.values.insert(*value, parameter_value);
        }

        let mut block_id = self.func.entry_block;
        let mut predecessor = None;
        let mut visited = 0usize;
        loop {
            visited += 1;
            if visited > self.func.blocks.len().saturating_mul(4).max(16) {
                return Err(unsupported("circuit control flow exceeded the simulator step limit"));
            }
            let (node_ids, terminator) = {
                let Some(block) = self.func.blocks.get(&block_id) else {
                    return Err(unsupported("GIR references a missing basic block"));
                };
                (block.nodes.clone(), block.terminator.clone())
            };
            for node_id in &node_ids {
                let Some(node) = self.gir.nodes.get(node_id).cloned() else {
                    return Err(unsupported("GIR references a missing node"));
                };
                self.execute_node(&node, predecessor)?;
            }
            match terminator.as_ref() {
                Some(Terminator::Return(value)) => {
                    return Ok(value.and_then(|value| self.values.get(&value).cloned()));
                }
                Some(Terminator::Unreachable) | None => return Ok(None),
                Some(Terminator::Branch(next)) => {
                    predecessor = Some(block_id);
                    block_id = *next;
                }
                Some(Terminator::CondBranch { cond, then_bb, else_bb }) => {
                    let choose_then = self.value_as_bool(*cond)?;
                    predecessor = Some(block_id);
                    block_id = if choose_then { *then_bb } else { *else_bb };
                }
            }
        }
    }

    fn execute_node(
        &mut self,
        node: &GirNode,
        predecessor: Option<BlockId>,
    ) -> Result<(), Diagnostics> {
        match &node.kind {
            NodeKind::Constant(constant) => {
                if let Some(result) = node.results.first() {
                    let value = match constant {
                        Constant::Int(value) => Value::Int(*value),
                        Constant::Float(value) => Value::Float(*value),
                        Constant::Bool(value) => Value::Bool(*value),
                        _ => return Ok(()),
                    };
                    self.values.insert(*result, value);
                }
            }
            NodeKind::Alloc(kind) => {
                let first = self.next_allocated_qubit();
                match kind {
                    AllocKind::Qubit => {
                        if let Some(result) = node.results.first() {
                            self.values.insert(*result, Value::Qubit(first));
                        }
                    }
                    AllocKind::Qubits(count) => {
                        for (offset, result) in
                            node.results.iter().take(*count as usize).enumerate()
                        {
                            self.values.insert(*result, Value::Qubit(first + offset));
                        }
                    }
                    AllocKind::Classical(_) => {}
                }
            }
            NodeKind::Free { target } => {
                self.value_as_qubit(*target)?;
                self.values.remove(target);
            }
            NodeKind::Tuple { elements } => {
                let mut values = Vec::with_capacity(elements.len());
                for element in elements {
                    let Some(value) = self.values.get(element).cloned() else {
                        return Err(unsupported("tuple references a missing value"));
                    };
                    values.push(value);
                }
                if let Some(result) = node.results.first() {
                    self.values.insert(*result, Value::Tuple(values));
                }
            }
            NodeKind::Extract { tuple, index } => {
                let Some(Value::Tuple(values)) = self.values.get(tuple).cloned() else {
                    return Err(unsupported("tuple extraction references a non-tuple value"));
                };
                let Some(value) = values.get(*index).cloned() else {
                    return Err(unsupported("tuple extraction index is out of bounds"));
                };
                if let Some(result) = node.results.first() {
                    self.values.insert(*result, value);
                }
            }
            NodeKind::Gate(gate) => self.execute_gate(node, *gate)?,
            NodeKind::Measure { target } => {
                let qubit = self.value_as_qubit(*target)?;
                let mut measured = self.measure(qubit);
                if self.probability("readout_error") > 0.0
                    && self.rng.next_f64() < self.probability("readout_error")
                {
                    measured = !measured;
                }
                if let Some(result) = node.results.first() {
                    self.values.insert(*result, Value::Bool(measured));
                }
                self.measurements.push(measured);
            }
            NodeKind::Classical(op) => self.execute_classical(node, *op)?,
            NodeKind::Phi { incoming } => {
                let Some(predecessor) = predecessor else { return Ok(()) };
                if let Some((_, value)) = incoming.iter().find(|(block, _)| *block == predecessor) {
                    if let Some(result) = node.results.first() {
                        if let Some(value) = self.values.get(value).cloned() {
                            self.values.insert(*result, value);
                        }
                    }
                }
            }
            NodeKind::Call { func, args } => {
                let Some(callee) = self.gir.funcs.get(func) else {
                    return Err(unsupported("call references a missing GIR function"));
                };
                if callee.params.len() != args.len() {
                    return Err(unsupported("call argument count does not match the callee"));
                }
                let mut callee_values = HashMap::new();
                for (parameter, argument) in callee.params.iter().zip(args) {
                    let Some(value) = self.values.get(argument).cloned() else {
                        return Err(unsupported("call references a missing argument value"));
                    };
                    callee_values.insert(*parameter, value);
                }
                let mut callee_executor = Executor {
                    gir: self.gir,
                    func: callee,
                    state: self.state,
                    values: callee_values,
                    measurements: Vec::new(),
                    rng: self.rng,
                    noise_model: self.noise_model.clone(),
                };
                let returned = callee_executor.execute()?;
                self.measurements.extend(callee_executor.measurements);
                if let (Some(result), Some(value)) = (node.results.first(), returned) {
                    self.values.insert(*result, value);
                }
            }
        }
        Ok(())
    }

    fn execute_gate(&mut self, node: &GirNode, gate: GateKind) -> Result<(), Diagnostics> {
        if gate == GateKind::Measure {
            return Err(unsupported("measurement must use a Measure GIR node"));
        }
        let Some(target) = node.operands.first().copied() else {
            return Err(unsupported("quantum gate is missing its target qubit"));
        };
        let qubit = self.value_as_qubit(target)?;
        match gate {
            GateKind::H => self.single(
                qubit,
                (
                    1.0 / 2.0_f64.sqrt(),
                    1.0 / 2.0_f64.sqrt(),
                    1.0 / 2.0_f64.sqrt(),
                    -1.0 / 2.0_f64.sqrt(),
                ),
            ),
            GateKind::X => self.single(qubit, (0.0, 1.0, 1.0, 0.0)),
            GateKind::Y => self.single(qubit, (0.0, -1.0, 1.0, 0.0)),
            GateKind::Z => self.single(qubit, (1.0, 0.0, 0.0, -1.0)),
            GateKind::S => self.single_phase(qubit, std::f64::consts::FRAC_PI_2),
            GateKind::T => self.single_phase(qubit, std::f64::consts::FRAC_PI_4),
            GateKind::Rx | GateKind::Ry | GateKind::Rz => {
                let Some(angle) = node.operands.get(1).copied() else {
                    return Err(unsupported("rotation gate is missing its angle"));
                };
                let angle = self.value_as_float(angle)?;
                match gate {
                    GateKind::Rx => self.rotation_x(qubit, angle),
                    GateKind::Ry => self.rotation_y(qubit, angle),
                    GateKind::Rz => self.rotation_z(qubit, angle),
                    _ => unreachable!(),
                }
            }
            GateKind::CX | GateKind::CZ | GateKind::SWAP => {
                let Some(second) = node.operands.get(1).copied() else {
                    return Err(unsupported("two-qubit gate is missing its second qubit"));
                };
                let second = self.value_as_qubit(second)?;
                match gate {
                    GateKind::CX => self.controlled_x(qubit, second),
                    GateKind::CZ => self.controlled_z(qubit, second),
                    GateKind::SWAP => self.swap(qubit, second),
                    _ => unreachable!(),
                }
            }
            GateKind::Measure => {}
        }
        for (index, result) in node.results.iter().enumerate() {
            let result_qubit =
                if index == 1 && matches!(gate, GateKind::CX | GateKind::CZ | GateKind::SWAP) {
                    node.operands
                        .get(1)
                        .map(|value| self.value_as_qubit(*value))
                        .transpose()?
                        .unwrap_or(qubit)
                } else {
                    qubit
                };
            self.values.insert(*result, Value::Qubit(result_qubit));
        }
        self.apply_noise(qubit);
        Ok(())
    }

    fn probability(&self, name: &str) -> f64 {
        let value = match self.noise_model.as_ref() {
            Some(noise) => match name {
                "depolarizing_prob" => noise.depolarizing_prob,
                "amplitude_damping" => noise.amplitude_damping,
                "phase_damping" => noise.phase_damping,
                "readout_error" => noise.readout_error,
                _ => None,
            },
            None => None,
        }
        .unwrap_or(0.0);
        value.clamp(0.0, 1.0)
    }

    fn apply_noise(&mut self, qubit: usize) {
        let depolarizing = self.probability("depolarizing_prob");
        if depolarizing > 0.0 && self.rng.next_f64() < depolarizing {
            match (self.rng.next_u64() % 3) as u8 {
                0 => self.single(qubit, (0.0, 1.0, 1.0, 0.0)),
                1 => self.single(qubit, (0.0, -1.0, 1.0, 0.0)),
                _ => self.single(qubit, (1.0, 0.0, 0.0, -1.0)),
            }
        }

        let amplitude = self.probability("amplitude_damping");
        let phase = self.probability("phase_damping");
        if amplitude > 0.0 || phase > 0.0 {
            let mask = 1usize << qubit;
            for (index, value) in self.state.iter_mut().enumerate() {
                if index & mask != 0 {
                    value.0 *= (1.0 - amplitude).sqrt() * (1.0 - phase).sqrt();
                    value.1 *= (1.0 - amplitude).sqrt() * (1.0 - phase).sqrt();
                }
            }
            self.renormalize();
        }
    }

    fn renormalize(&mut self) {
        let norm = self
            .state
            .iter()
            .map(|value| value.0 * value.0 + value.1 * value.1)
            .sum::<f64>()
            .sqrt();
        if norm > 0.0 {
            for value in self.state.iter_mut() {
                value.0 /= norm;
                value.1 /= norm;
            }
        }
    }

    fn execute_classical(&mut self, node: &GirNode, op: ClassicalOp) -> Result<(), Diagnostics> {
        let Some(lhs) = node.operands.first().copied() else {
            return Err(unsupported("classical operation is missing an operand"));
        };
        let result = match op {
            ClassicalOp::Not => Value::Bool(!self.value_as_bool(lhs)?),
            ClassicalOp::Neg => match self.values.get(&lhs).cloned() {
                Some(Value::Float(value)) => Value::Float(-value),
                Some(Value::Int(value)) => Value::Int(
                    value
                        .checked_neg()
                        .ok_or_else(|| unsupported("integer overflow during simulation"))?,
                ),
                Some(Value::Bool(value)) => Value::Int(-(value as i64)),
                _ => return Err(unsupported("expected a numeric value during simulation")),
            },
            ClassicalOp::And => {
                let rhs = self.binary_operand(node)?;
                Value::Bool(self.value_as_bool(lhs)? && self.value_as_bool(rhs)?)
            }
            ClassicalOp::Or => {
                let rhs = self.binary_operand(node)?;
                Value::Bool(self.value_as_bool(lhs)? || self.value_as_bool(rhs)?)
            }
            ClassicalOp::Add
            | ClassicalOp::Sub
            | ClassicalOp::Mul
            | ClassicalOp::Div
            | ClassicalOp::Mod => {
                let rhs = self.binary_operand(node)?;
                let a = self.value_as_numeric(lhs)?;
                let b = self.value_as_numeric(rhs)?;
                self.numeric_binary(op, a, b)?
            }
            ClassicalOp::Eq
            | ClassicalOp::Ne
            | ClassicalOp::Lt
            | ClassicalOp::Le
            | ClassicalOp::Gt
            | ClassicalOp::Ge => {
                let rhs = self.binary_operand(node)?;
                let a = self.value_as_numeric(lhs)?;
                let b = self.value_as_numeric(rhs)?;
                Value::Bool(self.numeric_compare(op, a, b))
            }
        };
        if let Some(result_id) = node.results.first() {
            self.values.insert(*result_id, result);
        }
        Ok(())
    }

    fn binary_operand(&self, node: &GirNode) -> Result<ValueId, Diagnostics> {
        node.operands
            .get(1)
            .copied()
            .ok_or_else(|| unsupported("binary classical operation is missing its right operand"))
    }

    fn value_as_numeric(&self, value: ValueId) -> Result<Numeric, Diagnostics> {
        match self.values.get(&value).cloned() {
            Some(Value::Int(value)) => Ok(Numeric::Int(value)),
            Some(Value::Float(value)) => Ok(Numeric::Float(value)),
            Some(Value::Bool(value)) => Ok(Numeric::Int(value as i64)),
            _ => Err(unsupported("expected a numeric value during simulation")),
        }
    }

    fn numeric_binary(
        &self,
        op: ClassicalOp,
        lhs: Numeric,
        rhs: Numeric,
    ) -> Result<Value, Diagnostics> {
        match (lhs, rhs) {
            (Numeric::Float(a), Numeric::Float(b)) => {
                if matches!(op, ClassicalOp::Div | ClassicalOp::Mod) && b == 0.0 {
                    return Err(unsupported("division by zero during simulation"));
                }
                Ok(Value::Float(match op {
                    ClassicalOp::Add => a + b,
                    ClassicalOp::Sub => a - b,
                    ClassicalOp::Mul => a * b,
                    ClassicalOp::Div => a / b,
                    ClassicalOp::Mod => a % b,
                    _ => return Err(unsupported("invalid numeric operation during simulation")),
                }))
            }
            (Numeric::Int(a), Numeric::Int(b)) => {
                let value = match op {
                    ClassicalOp::Add => a.checked_add(b),
                    ClassicalOp::Sub => a.checked_sub(b),
                    ClassicalOp::Mul => a.checked_mul(b),
                    ClassicalOp::Div if b != 0 => a.checked_div(b),
                    ClassicalOp::Mod if b != 0 => a.checked_rem(b),
                    ClassicalOp::Div | ClassicalOp::Mod => {
                        return Err(unsupported("division by zero during simulation"))
                    }
                    _ => return Err(unsupported("invalid numeric operation during simulation")),
                };
                Ok(Value::Int(
                    value.ok_or_else(|| unsupported("integer overflow during simulation"))?,
                ))
            }
            (Numeric::Int(a), Numeric::Float(b)) => {
                self.numeric_binary(op, Numeric::Float(a as f64), Numeric::Float(b))
            }
            (Numeric::Float(a), Numeric::Int(b)) => {
                self.numeric_binary(op, Numeric::Float(a), Numeric::Float(b as f64))
            }
        }
    }

    fn numeric_compare(&self, op: ClassicalOp, lhs: Numeric, rhs: Numeric) -> bool {
        let (a, b) = match (lhs, rhs) {
            (Numeric::Int(a), Numeric::Int(b)) => (a as f64, b as f64),
            (Numeric::Float(a), Numeric::Float(b)) => (a, b),
            (Numeric::Int(a), Numeric::Float(b)) => (a as f64, b),
            (Numeric::Float(a), Numeric::Int(b)) => (a, b as f64),
        };
        match op {
            ClassicalOp::Eq => a == b,
            ClassicalOp::Ne => a != b,
            ClassicalOp::Lt => a < b,
            ClassicalOp::Le => a <= b,
            ClassicalOp::Gt => a > b,
            ClassicalOp::Ge => a >= b,
            _ => false,
        }
    }

    fn quantum_params(&self) -> HashSet<ValueId> {
        let mut values = HashSet::new();
        if self.func.param_tys.len() == self.func.params.len() {
            for (param, type_id) in self.func.params.iter().zip(&self.func.param_tys) {
                if matches!(self.gir.types.get(type_id).map(|ty| &ty.kind), Some(TypeKind::Qubit)) {
                    values.insert(*param);
                }
            }
        }
        for block in self.func.blocks.values() {
            for node_id in &block.nodes {
                let Some(node) = self.gir.nodes.get(node_id) else { continue };
                match &node.kind {
                    NodeKind::Gate(GateKind::Rx | GateKind::Ry | GateKind::Rz) => {
                        if let Some(target) = node.operands.first() {
                            values.insert(*target);
                        }
                    }
                    NodeKind::Gate(_) => values.extend(node.operands.iter().copied()),
                    NodeKind::Measure { target } => {
                        values.insert(*target);
                    }
                    _ => {}
                }
            }
        }
        values
    }

    fn angle_params(&self) -> HashSet<ValueId> {
        self.func
            .blocks
            .values()
            .flat_map(|block| block.nodes.iter())
            .filter_map(|node_id| self.gir.nodes.get(node_id))
            .filter_map(|node| match &node.kind {
                NodeKind::Gate(GateKind::Rx | GateKind::Ry | GateKind::Rz) => {
                    node.operands.get(1).copied()
                }
                _ => None,
            })
            .collect()
    }

    fn value_as_qubit(&self, value: ValueId) -> Result<usize, Diagnostics> {
        match self.values.get(&value).cloned() {
            Some(Value::Qubit(qubit)) => Ok(qubit),
            _ => Err(unsupported("expected a qubit value during simulation")),
        }
    }

    fn value_as_bool(&self, value: ValueId) -> Result<bool, Diagnostics> {
        match self.values.get(&value).cloned() {
            Some(Value::Bool(value)) => Ok(value),
            Some(Value::Int(value)) => Ok(value != 0),
            _ => Err(unsupported("expected a boolean value during simulation")),
        }
    }

    fn value_as_float(&self, value: ValueId) -> Result<f64, Diagnostics> {
        match self.values.get(&value).cloned() {
            Some(Value::Float(value)) => Ok(value),
            Some(Value::Int(value)) => Ok(value as f64),
            _ => Err(unsupported("expected a numeric angle during simulation")),
        }
    }

    fn next_allocated_qubit(&self) -> usize {
        let mut qubits = Vec::new();
        for value in self.values.values() {
            collect_value_qubits(value, &mut qubits);
        }
        qubits.into_iter().max().map_or(0, |qubit| qubit + 1)
    }

    fn single(&mut self, qubit: usize, matrix: (f64, f64, f64, f64)) {
        let stride = 1usize << qubit;
        let step = stride << 1;
        for base in (0..self.state.len()).step_by(step) {
            for offset in 0..stride {
                let low = base + offset;
                let high = low + stride;
                let a = self.state[low];
                let b = self.state[high];
                self.state[low] =
                    (matrix.0 * a.0 + matrix.1 * b.0, matrix.0 * a.1 + matrix.1 * b.1);
                self.state[high] =
                    (matrix.2 * a.0 + matrix.3 * b.0, matrix.2 * a.1 + matrix.3 * b.1);
            }
        }
    }

    fn single_phase(&mut self, qubit: usize, angle: f64) {
        let (cos, sin) = (angle.cos(), angle.sin());
        let stride = 1usize << qubit;
        let step = stride << 1;
        for base in (0..self.state.len()).step_by(step) {
            for offset in 0..stride {
                let high = base + offset + stride;
                let value = self.state[high];
                self.state[high] = (cos * value.0 - sin * value.1, sin * value.0 + cos * value.1);
            }
        }
    }

    fn rotation_x(&mut self, qubit: usize, angle: f64) {
        let (cos, sin) = ((angle / 2.0).cos(), (angle / 2.0).sin());
        let stride = 1usize << qubit;
        let step = stride << 1;
        for base in (0..self.state.len()).step_by(step) {
            for offset in 0..stride {
                let low = base + offset;
                let high = low + stride;
                let a = self.state[low];
                let b = self.state[high];
                self.state[low] = (cos * a.0 + sin * b.1, cos * a.1 - sin * b.0);
                self.state[high] = (cos * b.0 - sin * a.1, cos * b.1 + sin * a.0);
            }
        }
    }

    fn rotation_y(&mut self, qubit: usize, angle: f64) {
        let (cos, sin) = ((angle / 2.0).cos(), (angle / 2.0).sin());
        let stride = 1usize << qubit;
        let step = stride << 1;
        for base in (0..self.state.len()).step_by(step) {
            for offset in 0..stride {
                let low = base + offset;
                let high = low + stride;
                let a = self.state[low];
                let b = self.state[high];
                self.state[low] = (cos * a.0 - sin * b.0, cos * a.1 - sin * b.1);
                self.state[high] = (sin * a.0 + cos * b.0, sin * a.1 + cos * b.1);
            }
        }
    }

    fn rotation_z(&mut self, qubit: usize, angle: f64) {
        let (cos, sin) = ((angle / 2.0).cos(), (angle / 2.0).sin());
        let stride = 1usize << qubit;
        let step = stride << 1;
        for base in (0..self.state.len()).step_by(step) {
            for offset in 0..stride {
                let low = base + offset;
                let high = low + stride;
                let a = self.state[low];
                let b = self.state[high];
                self.state[low] = (cos * a.0 + sin * a.1, cos * a.1 - sin * a.0);
                self.state[high] = (cos * b.0 - sin * b.1, cos * b.1 + sin * b.0);
            }
        }
    }

    fn controlled_x(&mut self, control: usize, target: usize) {
        for index in 0..self.state.len() {
            if (index & (1usize << control)) != 0 && (index & (1usize << target)) == 0 {
                let other = index | (1usize << target);
                self.state.swap(index, other);
            }
        }
    }

    fn controlled_z(&mut self, control: usize, target: usize) {
        for (index, value) in self.state.iter_mut().enumerate() {
            if (index & (1usize << control)) != 0 && (index & (1usize << target)) != 0 {
                value.0 = -value.0;
                value.1 = -value.1;
            }
        }
    }

    fn swap(&mut self, first: usize, second: usize) {
        if first == second {
            return;
        }
        for index in 0..self.state.len() {
            let a = (index >> first) & 1;
            let b = (index >> second) & 1;
            if a == 0 && b == 1 {
                let other = index ^ (1usize << first) ^ (1usize << second);
                self.state.swap(index, other);
            }
        }
    }

    fn measure(&mut self, qubit: usize) -> bool {
        let mask = 1usize << qubit;
        let probability_one: f64 = self
            .state
            .iter()
            .enumerate()
            .filter(|(index, _)| *index & mask != 0)
            .map(|(_, value)| value.0 * value.0 + value.1 * value.1)
            .sum();
        let one = self.rng.next_f64() < probability_one;
        let probability = if one { probability_one } else { 1.0 - probability_one };
        let scale = if probability > 0.0 { probability.sqrt().recip() } else { 0.0 };
        for (index, value) in self.state.iter_mut().enumerate() {
            if ((index & mask) != 0) != one {
                *value = (0.0, 0.0);
            } else {
                value.0 *= scale;
                value.1 *= scale;
            }
        }
        one
    }
}

fn collect_value_qubits(value: &Value, qubits: &mut Vec<usize>) {
    match value {
        Value::Qubit(qubit) => qubits.push(*qubit),
        Value::Tuple(values) => {
            for value in values {
                collect_value_qubits(value, qubits);
            }
        }
        Value::Bool(_) | Value::Int(_) | Value::Float(_) => {}
    }
}

fn count_qubits(gir: &Gir, func: &GirFunc) -> usize {
    count_qubits_inner(gir, func, &mut HashSet::new())
}

fn count_qubits_inner(gir: &Gir, func: &GirFunc, visited: &mut HashSet<gala_gir::FuncId>) -> usize {
    // `visited` is an active recursion stack, not a global set: two sibling
    // calls each allocate their own resources and must both contribute to the
    // circuit size. A recursive cycle is conservatively cut off.
    if visited.contains(&func.id) {
        return 0;
    }
    visited.insert(func.id);
    let mut quantum_params = HashSet::new();
    let mut called_qubits = 0;
    for block in func.blocks.values() {
        for node_id in &block.nodes {
            let Some(node) = gir.nodes.get(node_id) else { continue };
            match &node.kind {
                NodeKind::Gate(GateKind::Rx | GateKind::Ry | GateKind::Rz) => {
                    if let Some(target) = node.operands.first() {
                        quantum_params.insert(*target);
                    }
                }
                NodeKind::Gate(_) => quantum_params.extend(node.operands.iter().copied()),
                NodeKind::Measure { target } => {
                    quantum_params.insert(*target);
                }
                NodeKind::Call { func, .. } => {
                    if let Some(callee) = gir.funcs.get(func) {
                        called_qubits += count_qubits_inner(gir, callee, visited);
                    }
                }
                _ => {}
            }
        }
    }
    let params = func.params.iter().filter(|param| quantum_params.contains(param)).count();
    let allocated: usize = func
        .blocks
        .values()
        .flat_map(|block| block.nodes.iter())
        .filter_map(|node_id| gir.nodes.get(node_id))
        .filter_map(|node| match &node.kind {
            NodeKind::Alloc(AllocKind::Qubit) => Some(1),
            NodeKind::Alloc(AllocKind::Qubits(count)) => Some(*count as usize),
            _ => None,
        })
        .sum();
    let total = params + allocated + called_qubits;
    visited.remove(&func.id);
    total
}

fn sample_basis(state: &[(f64, f64)], rng: &mut Rng) -> usize {
    let choice = rng.next_f64();
    let mut cumulative = 0.0;
    for (index, value) in state.iter().enumerate() {
        cumulative += value.0 * value.0 + value.1 * value.1;
        if choice <= cumulative {
            return index;
        }
    }
    state.len().saturating_sub(1)
}

fn basis_key(index: usize, num_qubits: usize) -> String {
    (0..num_qubits).rev().map(|bit| if index & (1usize << bit) == 0 { '0' } else { '1' }).collect()
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        self.0 = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

fn unsupported(message: &str) -> Diagnostics {
    let mut diags = Diagnostics::new();
    diags.push(Diagnostic::error(codes::BACKEND_CAPABILITY_MISMATCH, message));
    diags
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
    use gala_gir::NodeId;

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
        for backend in &[SimBackend::Noisy, SimBackend::TensorNetwork] {
            let config = SimConfig { backend: *backend, shots: 10, seed: None, noise_model: None };
            assert!(run_simulation(&gir, config).is_ok());
        }
    }

    #[test]
    fn state_vector_executes_x_and_measurement() {
        let mut gir = Gir::default();
        let mut func = GirFunc::new_test();
        let block = func.blocks.get_mut(&BlockId(0)).expect("test function has an entry block");
        block.nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        block.terminator = Some(Terminator::Return(Some(ValueId(2))));
        gir.funcs.insert(func.id, func);
        gir.nodes.insert(
            NodeId(0),
            GirNode {
                id: NodeId(0),
                kind: NodeKind::Alloc(AllocKind::Qubit),
                operands: Vec::new(),
                results: vec![ValueId(0)],
                span: gala_span::Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(1),
            GirNode {
                id: NodeId(1),
                kind: NodeKind::Gate(GateKind::X),
                operands: vec![ValueId(0)],
                results: vec![ValueId(1)],
                span: gala_span::Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(2),
            GirNode {
                id: NodeId(2),
                kind: NodeKind::Measure { target: ValueId(1) },
                operands: vec![ValueId(1)],
                results: vec![ValueId(2)],
                span: gala_span::Span::dummy(),
            },
        );

        let result =
            run_simulation(&gir, SimConfig { shots: 8, seed: Some(42), ..SimConfig::default() })
                .expect("supported state-vector circuit should execute");
        assert_eq!(result.counts.get("1"), Some(&8));
        assert_eq!(result.probabilities.get("1"), Some(&1.0));
        assert_eq!(result.state_vector, Some(vec![0.0, 0.0, 1.0, 0.0]));
    }

    #[test]
    fn noisy_backend_applies_readout_error() {
        let mut gir = Gir::default();
        let mut func = GirFunc::new_test();
        let block = func.blocks.get_mut(&BlockId(0)).expect("test function has an entry block");
        block.nodes = vec![NodeId(0), NodeId(1), NodeId(2)];
        block.terminator = Some(Terminator::Return(Some(ValueId(2))));
        gir.funcs.insert(func.id, func);
        gir.nodes.insert(
            NodeId(0),
            GirNode {
                id: NodeId(0),
                kind: NodeKind::Alloc(AllocKind::Qubit),
                operands: Vec::new(),
                results: vec![ValueId(0)],
                span: gala_span::Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(1),
            GirNode {
                id: NodeId(1),
                kind: NodeKind::Gate(GateKind::X),
                operands: vec![ValueId(0)],
                results: vec![ValueId(1)],
                span: gala_span::Span::dummy(),
            },
        );
        gir.nodes.insert(
            NodeId(2),
            GirNode {
                id: NodeId(2),
                kind: NodeKind::Measure { target: ValueId(1) },
                operands: vec![ValueId(1)],
                results: vec![ValueId(2)],
                span: gala_span::Span::dummy(),
            },
        );
        let result = run_simulation(
            &gir,
            SimConfig {
                backend: SimBackend::Noisy,
                shots: 8,
                seed: Some(7),
                noise_model: Some(NoiseModel { readout_error: Some(1.0), ..NoiseModel::default() }),
            },
        )
        .expect("noisy backend should execute a supported circuit");
        assert_eq!(result.counts.get("0"), Some(&8));
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
