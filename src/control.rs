//! Phase 4: Program counter, `OP2` control transfers, R-quantities, address counter,
//! and the top-level `SpeedCoMachine` interpreter.

use crate::core_types::{Address, ElectrostaticMemory, MachineStop, RCode, SpeedError};
use crate::diagnostics::{CheckingState, ListingBuffer, ListingSwitches};
use crate::io::{DrumId, DrumUnit, LinePrinter, TapeId, TapeUnit};
use crate::parser::PunchedCard;

pub const AC_MIN: i32 = -131_071;
pub const AC_MAX: i32 = 131_071;

/// Non-negative index quantities `R_A`, `R_B`, and `R_C` (manual pages 9-1..9-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RQuantities {
    pub r_a: u16,
    pub r_b: u16,
    pub r_c: u16,
}

impl RQuantities {
    /// Computes effective `(A, B, C)` addresses during instruction interpretation without mutating memory.
    pub fn effective_addresses(
        self,
        _r_code: RCode,
        _a: Address,
        _b: Address,
        _c: Address,
    ) -> Result<(Address, Address, Address), SpeedError> {
        todo!()
    }
}

/// Fixed-point 18-bit signed address counter (`-131_071..=131_071`, manual pages 9-7..9-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AddressCounter {
    _value: i32,
}

impl AddressCounter {
    /// Validates `value` lies within `-131_071..=131_071`.
    pub fn new(_value: i32) -> Result<Self, SpeedError> {
        todo!()
    }

    /// Returns the signed integer value currently in the address counter.
    pub fn value(self) -> i32 {
        todo!()
    }

    /// Resets the counter to zero and loads `addr` (`RADD*`).
    pub fn reset_and_add(&mut self, _addr: Address) {
        todo!()
    }

    /// Adds `addr` to the counter (`ADD*`).
    pub fn add(&mut self, _addr: Address) -> Result<(), SpeedError> {
        todo!()
    }

    /// Subtracts `addr` from the counter (`SUB*`).
    pub fn sub(&mut self, _addr: Address) -> Result<(), SpeedError> {
        todo!()
    }

    /// Returns `|value| mod 1024` stored by `STA`/`STB`/`STC`/`STD` (manual page 9-8).
    pub fn stored_address(self) -> Address {
        todo!()
    }
}

/// Operator console switches (`P` = switch 4, `Q` = switch 5, listing switches 1–3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SenseSwitches {
    pub p_switch_4: bool,
    pub q_switch_5: bool,
    pub listing: ListingSwitches,
}

/// Result of stepping a single SpeedCo I instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Continued { next_pc: Address },
    Stopped(MachineStop),
}

/// Complete simulated IBM 701 Speedcoding System (SpeedCo I) machine.
#[derive(Debug, Clone, Default)]
pub struct SpeedCoMachine {
    _marker: (),
}

impl SpeedCoMachine {
    /// Initializes a SpeedCo I machine with `PC = 300`, clean memory, 4 tapes, 2 drums, and printer.
    pub fn new() -> Self {
        todo!()
    }

    /// Returns the current program counter (`300..=1013`).
    pub fn pc(&self) -> Address {
        todo!()
    }

    /// Sets the program counter to a valid user storage cell (`300..=1013`).
    pub fn set_pc(&mut self, _addr: Address) -> Result<(), SpeedError> {
        todo!()
    }

    /// Returns a shared reference to electrostatic memory.
    pub fn memory(&self) -> &ElectrostaticMemory {
        todo!()
    }

    /// Returns a mutable reference to electrostatic memory.
    pub fn memory_mut(&mut self) -> &mut ElectrostaticMemory {
        todo!()
    }

    /// Returns the current `R_A`, `R_B`, `R_C` quantities.
    pub fn r_quantities(&self) -> RQuantities {
        todo!()
    }

    /// Returns a mutable reference to `R_A`, `R_B`, `R_C`.
    pub fn r_quantities_mut(&mut self) -> &mut RQuantities {
        todo!()
    }

    /// Returns the current address counter.
    pub fn address_counter(&self) -> AddressCounter {
        todo!()
    }

    /// Returns a mutable reference to the address counter.
    pub fn address_counter_mut(&mut self) -> &mut AddressCounter {
        todo!()
    }

    /// Returns a mutable reference to operator panel sense switches.
    pub fn switches_mut(&mut self) -> &mut SenseSwitches {
        todo!()
    }

    /// Returns a shared reference to tape unit `J`, `K`, `L`, or `M`.
    pub fn tape(&self, _id: TapeId) -> &TapeUnit {
        todo!()
    }

    /// Returns a mutable reference to tape unit `J`, `K`, `L`, or `M`.
    pub fn tape_mut(&mut self, _id: TapeId) -> &mut TapeUnit {
        todo!()
    }

    /// Returns a shared reference to magnetic drum `P` or `Q`.
    pub fn drum(&self, _id: DrumId) -> &DrumUnit {
        todo!()
    }

    /// Returns a mutable reference to magnetic drum `P` or `Q`.
    pub fn drum_mut(&mut self, _id: DrumId) -> &mut DrumUnit {
        todo!()
    }

    /// Returns a shared reference to the line printer output.
    pub fn printer(&self) -> &LinePrinter {
        todo!()
    }

    /// Returns a shared reference to checking-loop state.
    pub fn checking(&self) -> &CheckingState {
        todo!()
    }

    /// Returns a shared reference to the selective listing buffer.
    pub fn listing(&self) -> &ListingBuffer {
        todo!()
    }

    /// Rewinds all 4 tapes, processes instruction/data/loading-control cards, and sets `PC = 300`.
    pub fn load_deck(&mut self, _cards: &[PunchedCard]) -> Result<(), SpeedError> {
        todo!()
    }

    /// Fetches, interprets (`R`-code), executes `OP1` then `OP2`, logs listing if enabled, and advances `PC`.
    pub fn step(&mut self) -> Result<StepOutcome, SpeedError> {
        todo!()
    }

    /// Restarts execution after a `STOP` (`OP2 = 123`) by transferring control to its `D` address.
    pub fn restart_after_stop(&mut self) -> Result<Address, SpeedError> {
        todo!()
    }

    /// Runs instructions continuously until a `MachineStop` occurs or `max_steps` is exceeded.
    pub fn run_until_stop(&mut self, _max_steps: usize) -> Result<MachineStop, SpeedError> {
        todo!()
    }
}
