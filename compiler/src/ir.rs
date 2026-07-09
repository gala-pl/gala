#[derive(Debug, Clone)]
pub enum IrValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone)]
pub enum IrInst {
    LoadConst(IrValue),
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Neg,
    Not,
    And,
    Or,
    Call(String, usize),
    Return,
    JmpIf(usize),
    Jmp(usize),
    Phi(Vec<usize>),
    Alloca(String),
    Store,
    Load,
}

#[derive(Debug, Default)]
pub struct IrBuilder {
    insts: Vec<IrInst>,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, inst: IrInst) -> usize {
        let idx = self.insts.len();
        self.insts.push(inst);
        idx
    }

    pub fn build(&self) -> &[IrInst] {
        &self.insts
    }
}
