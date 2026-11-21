//! Phase 3: SpeedCo I floating-binary arithmetic and Set 1 elementary mathematical functions.

use crate::core_types::{MachineStop, Op1, SpeedError, SpeedFloat};

pub const MAX_ADD_SUB_DEC_EXP_DIFF: u32 = 76;
pub const MAX_SQRT_EXP_LN_DEC_EXP: i32 = 39_456;
pub const MAX_SINE_ARTAN_DEC_EXP: i32 = 76;

/// Outcome of executing an arithmetic `OP1` instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOutcome {
    Value(SpeedFloat),
    Stop(MachineStop),
}

/// Executes binary `OP1` arithmetic (`ADD`, `SUB`, `ADDAB`, `ABADD`, `SUBAB`, `ABSUB`, `MPY`, `NGMPY`, `DIV`, `NGDIV`).
pub fn execute_op1_arithmetic(
    _op: Op1,
    _a: SpeedFloat,
    _b: SpeedFloat,
) -> Result<ArithmeticOutcome, SpeedError> {
    todo!()
}

/// Executes Set 1 unary elementary mathematical `OP1` operations (`SQRT`, `SINE`, `ARTAN`, `EXP`, `LN`).
pub fn execute_op1_math(_op: Op1, _a: SpeedFloat) -> Result<SpeedFloat, SpeedError> {
    todo!()
}
