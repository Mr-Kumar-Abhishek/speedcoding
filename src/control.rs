//! Phase 4: Program counter, `OP2` control transfers, R-quantities, address counter,
//! and the top-level `SpeedCoMachine` interpreter.

use crate::arithmetic::{ArithmeticOutcome, execute_op1_arithmetic, execute_op1_math};
use crate::core_types::{
    Address, ElectrostaticMemory, INITIAL_PC, MachineStop, Op1, Op2, RCode, SPECIAL_R_CONTROL_ADDR,
    SpeedError, Word72,
};
use crate::diagnostics::{CheckingState, ListingBuffer, ListingSwitches};
use crate::io::{
    DrumId, DrumUnit, DrumWriteOutcome, LinePrinter, TapeId, TapeReadOutcome, TapeSkipOutcome,
    TapeUnit,
};
use crate::parser::{PunchedCard, StorageUnit};

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
        r_code: RCode,
        a: Address,
        b: Address,
        c: Address,
    ) -> Result<(Address, Address, Address), SpeedError> {
        let ea = if r_code.modifies_a() {
            Address::user(a.raw().saturating_add(self.r_a))?
        } else {
            a
        };
        let eb = if r_code.modifies_b() {
            Address::user(b.raw().saturating_add(self.r_b))?
        } else {
            b
        };
        let ec = if r_code.modifies_c() {
            Address::user(c.raw().saturating_add(self.r_c))?
        } else {
            c
        };
        Ok((ea, eb, ec))
    }

    fn apply_ti(&mut self, inc_a: bool, inc_b: bool, inc_c: bool) -> Result<(), SpeedError> {
        let next_a = if inc_a {
            self.r_a
                .checked_add(1)
                .ok_or(SpeedError::AddressOutOfBounds(u16::MAX))?
        } else {
            self.r_a
        };
        let next_b = if inc_b {
            self.r_b
                .checked_add(1)
                .ok_or(SpeedError::AddressOutOfBounds(u16::MAX))?
        } else {
            self.r_b
        };
        let next_c = if inc_c {
            self.r_c
                .checked_add(1)
                .ok_or(SpeedError::AddressOutOfBounds(u16::MAX))?
        } else {
            self.r_c
        };
        self.r_a = next_a;
        self.r_b = next_b;
        self.r_c = next_c;
        Ok(())
    }

    fn apply_td(&mut self, dec_a: bool, dec_b: bool, dec_c: bool) -> bool {
        if (dec_a && self.r_a == 0) || (dec_b && self.r_b == 0) || (dec_c && self.r_c == 0) {
            return false;
        }
        if dec_a {
            self.r_a -= 1;
        }
        if dec_b {
            self.r_b -= 1;
        }
        if dec_c {
            self.r_c -= 1;
        }
        true
    }
}

/// Fixed-point 18-bit signed address counter (`-131_071..=131_071`, manual pages 9-7..9-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AddressCounter {
    value: i32,
}

impl AddressCounter {
    /// Validates `value` lies within `-131_071..=131_071`.
    pub fn new(value: i32) -> Result<Self, SpeedError> {
        if !(AC_MIN..=AC_MAX).contains(&value) {
            return Err(SpeedError::BinaryExponentOutOfRange(value));
        }
        Ok(Self { value })
    }

    /// Returns the signed integer value currently in the address counter.
    pub fn value(self) -> i32 {
        self.value
    }

    /// Resets the counter to zero and loads `addr` (`RADD*`).
    pub fn reset_and_add(&mut self, addr: Address) {
        self.value = i32::from(addr.raw());
    }

    /// Adds `addr` to the counter (`ADD*`).
    pub fn add(&mut self, addr: Address) -> Result<(), SpeedError> {
        let next = self.value + i32::from(addr.raw());
        if !(AC_MIN..=AC_MAX).contains(&next) {
            return Err(SpeedError::BinaryExponentOutOfRange(next));
        }
        self.value = next;
        Ok(())
    }

    /// Subtracts `addr` from the counter (`SUB*`).
    pub fn sub(&mut self, addr: Address) -> Result<(), SpeedError> {
        let next = self.value - i32::from(addr.raw());
        if !(AC_MIN..=AC_MAX).contains(&next) {
            return Err(SpeedError::BinaryExponentOutOfRange(next));
        }
        self.value = next;
        Ok(())
    }

    /// Returns `|value| mod 1024` stored by `STA`/`STB`/`STC`/`STD` (manual page 9-8).
    pub fn stored_address(self) -> Address {
        Address::new((self.value.unsigned_abs() % 1024) as u16)
            .expect("modulo-1024 address fits in 0..=1023")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddrField {
    A,
    B,
    C,
    D,
}

fn ti_flags(op2: Op2) -> Option<(bool, bool, bool)> {
    match op2 {
        Op2::TiA => Some((true, false, false)),
        Op2::TiB => Some((false, true, false)),
        Op2::TiC => Some((false, false, true)),
        Op2::TiAB => Some((true, true, false)),
        Op2::TiBC => Some((false, true, true)),
        Op2::TiAC => Some((true, false, true)),
        Op2::TiABC => Some((true, true, true)),
        _ => None,
    }
}

fn td_flags(op2: Op2) -> Option<(bool, bool, bool)> {
    match op2 {
        Op2::TdA => Some((true, false, false)),
        Op2::TdB => Some((false, true, false)),
        Op2::TdC => Some((false, false, true)),
        Op2::TdAB => Some((true, true, false)),
        Op2::TdBC => Some((false, true, true)),
        Op2::TdAC => Some((true, false, true)),
        Op2::TdABC => Some((true, true, true)),
        _ => None,
    }
}

fn tape_index(id: TapeId) -> usize {
    match id {
        TapeId::J => 0,
        TapeId::K => 1,
        TapeId::L => 2,
        TapeId::M => 3,
    }
}

fn drum_index(id: DrumId) -> usize {
    match id {
        DrumId::P => 0,
        DrumId::Q => 1,
    }
}

fn tape_id_for_op1(op1: Op1) -> TapeId {
    match op1 {
        Op1::WrTpJ
        | Op1::RfTpJ
        | Op1::RbTpJ
        | Op1::SfTpJ
        | Op1::SbTpJ
        | Op1::RwTpJ
        | Op1::EfTpJ => TapeId::J,
        Op1::WrTpK
        | Op1::RfTpK
        | Op1::RbTpK
        | Op1::SfTpK
        | Op1::SbTpK
        | Op1::RwTpK
        | Op1::EfTpK => TapeId::K,
        Op1::WrTpL
        | Op1::RfTpL
        | Op1::RbTpL
        | Op1::SfTpL
        | Op1::SbTpL
        | Op1::RwTpL
        | Op1::EfTpL => TapeId::L,
        Op1::WrTpM
        | Op1::RfTpM
        | Op1::RbTpM
        | Op1::SfTpM
        | Op1::SbTpM
        | Op1::RwTpM
        | Op1::EfTpM => TapeId::M,
        other => unreachable!("not a tape OP1 operation: {other:?}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op1Outcome {
    Normal,
    SkipInstructions(u16),
    Stop(MachineStop),
}

/// Complete simulated IBM 701 Speedcoding System (SpeedCo I) machine.
#[derive(Debug, Clone)]
pub struct SpeedCoMachine {
    pc: Address,
    memory: ElectrostaticMemory,
    r_quantities: RQuantities,
    address_counter: AddressCounter,
    switches: SenseSwitches,
    tapes: [TapeUnit; 4],
    drums: [DrumUnit; 2],
    printer: LinePrinter,
    checking: CheckingState,
    listing: ListingBuffer,
    pending_restart_addr: Option<Address>,
}

impl Default for SpeedCoMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedCoMachine {
    /// Initializes a SpeedCo I machine with `PC = 300`, clean memory, 4 tapes, 2 drums, and printer.
    pub fn new() -> Self {
        Self {
            pc: Address::user(INITIAL_PC).expect("INITIAL_PC is valid user storage"),
            memory: ElectrostaticMemory::new(),
            r_quantities: RQuantities::default(),
            address_counter: AddressCounter::default(),
            switches: SenseSwitches::default(),
            tapes: [
                TapeUnit::new(TapeId::J),
                TapeUnit::new(TapeId::K),
                TapeUnit::new(TapeId::L),
                TapeUnit::new(TapeId::M),
            ],
            drums: [DrumUnit::new(DrumId::P), DrumUnit::new(DrumId::Q)],
            printer: LinePrinter::default(),
            checking: CheckingState::default(),
            listing: ListingBuffer::default(),
            pending_restart_addr: None,
        }
    }

    /// Returns the current program counter (`300..=1013`).
    pub fn pc(&self) -> Address {
        self.pc
    }

    /// Sets the program counter to a valid user storage cell (`300..=1013`).
    pub fn set_pc(&mut self, addr: Address) -> Result<(), SpeedError> {
        self.pc = Address::user(addr.raw())?;
        self.pending_restart_addr = None;
        Ok(())
    }

    /// Returns a shared reference to electrostatic memory.
    pub fn memory(&self) -> &ElectrostaticMemory {
        &self.memory
    }

    /// Returns a mutable reference to electrostatic memory.
    pub fn memory_mut(&mut self) -> &mut ElectrostaticMemory {
        &mut self.memory
    }

    /// Returns the current `R_A`, `R_B`, `R_C` quantities.
    pub fn r_quantities(&self) -> RQuantities {
        self.r_quantities
    }

    /// Returns a mutable reference to `R_A`, `R_B`, `R_C`.
    pub fn r_quantities_mut(&mut self) -> &mut RQuantities {
        &mut self.r_quantities
    }

    /// Returns the current address counter.
    pub fn address_counter(&self) -> AddressCounter {
        self.address_counter
    }

    /// Returns a mutable reference to the address counter.
    pub fn address_counter_mut(&mut self) -> &mut AddressCounter {
        &mut self.address_counter
    }

    /// Returns a mutable reference to operator panel sense switches.
    pub fn switches_mut(&mut self) -> &mut SenseSwitches {
        &mut self.switches
    }

    /// Returns a shared reference to tape unit `J`, `K`, `L`, or `M`.
    pub fn tape(&self, id: TapeId) -> &TapeUnit {
        &self.tapes[tape_index(id)]
    }

    /// Returns a mutable reference to tape unit `J`, `K`, `L`, or `M`.
    pub fn tape_mut(&mut self, id: TapeId) -> &mut TapeUnit {
        &mut self.tapes[tape_index(id)]
    }

    /// Returns a shared reference to magnetic drum `P` or `Q`.
    pub fn drum(&self, id: DrumId) -> &DrumUnit {
        &self.drums[drum_index(id)]
    }

    /// Returns a mutable reference to magnetic drum `P` or `Q`.
    pub fn drum_mut(&mut self, id: DrumId) -> &mut DrumUnit {
        &mut self.drums[drum_index(id)]
    }

    /// Returns a shared reference to the line printer output.
    pub fn printer(&self) -> &LinePrinter {
        &self.printer
    }

    /// Returns a shared reference to checking-loop state.
    pub fn checking(&self) -> &CheckingState {
        &self.checking
    }

    /// Returns a shared reference to the selective listing buffer.
    pub fn listing(&self) -> &ListingBuffer {
        &self.listing
    }

    fn block_word_count(start: Address, end: Address) -> Result<usize, SpeedError> {
        if start.raw() > end.raw() {
            return Err(SpeedError::InvalidCardFormat(format!(
                "invalid memory block range {}..={}",
                start.raw(),
                end.raw()
            )));
        }
        Address::user(start.raw())?;
        Address::user(end.raw())?;
        Ok(usize::from(end.raw() - start.raw()) + 1)
    }

    fn read_memory_block(&self, start: Address, end: Address) -> Result<Vec<Word72>, SpeedError> {
        let count = Self::block_word_count(start, end)?;
        let mut out = Vec::with_capacity(count);
        for raw in start.raw()..=end.raw() {
            out.push(self.memory.read(Address::user(raw)?)?);
        }
        Ok(out)
    }

    fn write_memory_block(&mut self, start: Address, words: &[Word72]) -> Result<(), SpeedError> {
        for (offset, &word) in words.iter().enumerate() {
            let off = u16::try_from(offset)
                .map_err(|_| SpeedError::AddressOutOfBounds(u16::MAX))?;
            let raw = start
                .raw()
                .checked_add(off)
                .ok_or(SpeedError::AddressOutOfBounds(u16::MAX))?;
            self.memory.write(Address::user(raw)?, word)?;
        }
        Ok(())
    }

    fn load_tape_record(&mut self, id: TapeId, block: &[Word72]) -> Result<(), SpeedError> {
        if let Some(stop) = self.tape_mut(id).write_record(block)? {
            return Err(SpeedError::InvalidCardFormat(format!(
                "loading control tape write stopped: {stop:?}"
            )));
        }
        Ok(())
    }

    /// Rewinds all 4 tapes, processes instruction/data/loading-control cards, and sets `PC = 300`.
    pub fn load_deck(&mut self, cards: &[PunchedCard]) -> Result<(), SpeedError> {
        for tape in &mut self.tapes {
            tape.rewind();
        }
        for card in cards {
            match card {
                PunchedCard::Instruction(instr_card) => {
                    let instr = instr_card.to_machine_instruction()?;
                    self.memory.write_instruction(instr_card.location, instr)?;
                }
                PunchedCard::Data(data_card) => {
                    for (offset, dec) in data_card.values.iter().enumerate() {
                        let off = u16::try_from(offset)
                            .map_err(|_| SpeedError::AddressOutOfBounds(u16::MAX))?;
                        let raw = data_card
                            .first_location
                            .raw()
                            .checked_add(off)
                            .ok_or(SpeedError::AddressOutOfBounds(u16::MAX))?;
                        let target = Address::user(raw)?;
                        self.memory.write_float(target, dec.to_speed_float()?)?;
                    }
                }
                PunchedCard::LoadingControl(ctrl) => {
                    let block = self.read_memory_block(ctrl.first_addr, ctrl.last_addr)?;
                    match ctrl.unit {
                        StorageUnit::TapeJ => self.load_tape_record(TapeId::J, &block)?,
                        StorageUnit::TapeK => self.load_tape_record(TapeId::K, &block)?,
                        StorageUnit::TapeL => self.load_tape_record(TapeId::L, &block)?,
                        StorageUnit::TapeM => self.load_tape_record(TapeId::M, &block)?,
                        StorageUnit::DrumP => {
                            let _ = self
                                .drum_mut(DrumId::P)
                                .write_block(ctrl.drum_addr, &block)?;
                        }
                        StorageUnit::DrumQ => {
                            let _ = self
                                .drum_mut(DrumId::Q)
                                .write_block(ctrl.drum_addr, &block)?;
                        }
                    }
                }
            }
        }
        self.pc = Address::user(INITIAL_PC).expect("INITIAL_PC is valid user storage");
        self.pending_restart_addr = None;
        Ok(())
    }

    fn advance_pc(&self, delta: u16) -> Result<Address, SpeedError> {
        Address::user(self.pc.raw().saturating_add(delta))
    }

    fn read_addr_field(&self, target: Address, field: AddrField) -> Result<Address, SpeedError> {
        if target.raw() == SPECIAL_R_CONTROL_ADDR {
            return match field {
                AddrField::A => Address::new(self.r_quantities.r_a),
                AddrField::B => Address::new(self.r_quantities.r_b),
                AddrField::C => Address::new(self.r_quantities.r_c),
                AddrField::D => Err(SpeedError::ControlRegionAccess(target.raw())),
            };
        }
        let instr = self.memory.read_instruction(target)?;
        Ok(match field {
            AddrField::A => instr.a,
            AddrField::B => instr.b,
            AddrField::C => instr.c,
            AddrField::D => instr.d,
        })
    }

    fn write_addr_field(
        &mut self,
        target: Address,
        field: AddrField,
        val: Address,
    ) -> Result<(), SpeedError> {
        if target.raw() == SPECIAL_R_CONTROL_ADDR {
            match field {
                AddrField::A => self.r_quantities.r_a = val.raw(),
                AddrField::B => self.r_quantities.r_b = val.raw(),
                AddrField::C => self.r_quantities.r_c = val.raw(),
                AddrField::D => return Err(SpeedError::ControlRegionAccess(target.raw())),
            }
            return Ok(());
        }
        let mut instr = self.memory.read_instruction(target)?;
        match field {
            AddrField::A => instr.a = val,
            AddrField::B => instr.b = val,
            AddrField::C => instr.c = val,
            AddrField::D => instr.d = val,
        }
        self.memory.write_instruction(target, instr)
    }

    fn write_tape_op(
        &mut self,
        id: TapeId,
        eff_a: Address,
        eff_b: Address,
    ) -> Result<Op1Outcome, SpeedError> {
        let block = self.read_memory_block(eff_a, eff_b)?;
        match self.tape_mut(id).write_record(&block)? {
            Some(stop) => Ok(Op1Outcome::Stop(stop)),
            None => Ok(Op1Outcome::Normal),
        }
    }

    fn read_tape_op(
        &mut self,
        id: TapeId,
        backward: bool,
        eff_a: Address,
        eff_b: Address,
    ) -> Result<Op1Outcome, SpeedError> {
        let count = Self::block_word_count(eff_a, eff_b)?;
        let outcome = if backward {
            self.tape_mut(id).read_backward(count)?
        } else {
            self.tape_mut(id).read_forward(count)?
        };
        match outcome {
            TapeReadOutcome::Ok(words) => {
                self.write_memory_block(eff_a, &words)?;
                Ok(Op1Outcome::Normal)
            }
            TapeReadOutcome::EndOfFileSkip => Ok(Op1Outcome::SkipInstructions(1)),
            TapeReadOutcome::ChecksumMismatchSkip => Ok(Op1Outcome::SkipInstructions(2)),
            TapeReadOutcome::Stop(stop) => Ok(Op1Outcome::Stop(stop)),
        }
    }

    fn skip_tape_op(&mut self, id: TapeId, backward: bool) -> Result<Op1Outcome, SpeedError> {
        let outcome = if backward {
            self.tape_mut(id).skip_backward()?
        } else {
            self.tape_mut(id).skip_forward()?
        };
        Ok(match outcome {
            TapeSkipOutcome::Ok => Op1Outcome::Normal,
            TapeSkipOutcome::EndOfFileSkip => Op1Outcome::SkipInstructions(1),
            TapeSkipOutcome::Stop(stop) => Op1Outcome::Stop(stop),
        })
    }

    fn write_drum_op(
        &mut self,
        id: DrumId,
        eff_a: Address,
        eff_b: Address,
        eff_c: Address,
    ) -> Result<Op1Outcome, SpeedError> {
        let block = self.read_memory_block(eff_a, eff_b)?;
        Ok(match self.drum_mut(id).write_block(eff_c, &block)? {
            DrumWriteOutcome::Ok => Op1Outcome::Normal,
            DrumWriteOutcome::VerifyDiscrepancySkip => Op1Outcome::SkipInstructions(1),
        })
    }

    fn read_drum_op(
        &mut self,
        id: DrumId,
        eff_a: Address,
        eff_b: Address,
        eff_c: Address,
    ) -> Result<Op1Outcome, SpeedError> {
        let count = Self::block_word_count(eff_a, eff_b)?;
        let words = self.drum(id).read_block(eff_c, count)?;
        self.write_memory_block(eff_a, &words)?;
        Ok(Op1Outcome::Normal)
    }

    fn execute_op1(
        &mut self,
        op1: Op1,
        eff_a: Address,
        eff_b: Address,
        eff_c: Address,
    ) -> Result<Op1Outcome, SpeedError> {
        match op1 {
            Op1::Add
            | Op1::Sub
            | Op1::AddAb
            | Op1::AbAdd
            | Op1::SubAb
            | Op1::AbSub
            | Op1::Mpy
            | Op1::NgMpy
            | Op1::Div
            | Op1::NgDiv => {
                let qa = self.memory.read_float(eff_a)?;
                let qb = self.memory.read_float(eff_b)?;
                match execute_op1_arithmetic(op1, qa, qb)? {
                    ArithmeticOutcome::Value(val) => {
                        self.memory.write_float(eff_c, val)?;
                        Ok(Op1Outcome::Normal)
                    }
                    ArithmeticOutcome::Stop(stop) => Ok(Op1Outcome::Stop(stop)),
                }
            }
            Op1::Sqrt | Op1::Sine | Op1::Artan | Op1::Exp | Op1::Ln => {
                let qa = self.memory.read_float(eff_a)?;
                let res = execute_op1_math(op1, qa)?;
                self.memory.write_float(eff_c, res)?;
                Ok(Op1Outcome::Normal)
            }
            Op1::WrTpJ | Op1::WrTpK | Op1::WrTpL | Op1::WrTpM => {
                self.write_tape_op(tape_id_for_op1(op1), eff_a, eff_b)
            }
            Op1::RfTpJ | Op1::RfTpK | Op1::RfTpL | Op1::RfTpM => {
                self.read_tape_op(tape_id_for_op1(op1), false, eff_a, eff_b)
            }
            Op1::RbTpJ | Op1::RbTpK | Op1::RbTpL | Op1::RbTpM => {
                self.read_tape_op(tape_id_for_op1(op1), true, eff_a, eff_b)
            }
            Op1::SfTpJ | Op1::SfTpK | Op1::SfTpL | Op1::SfTpM => {
                self.skip_tape_op(tape_id_for_op1(op1), false)
            }
            Op1::SbTpJ | Op1::SbTpK | Op1::SbTpL | Op1::SbTpM => {
                self.skip_tape_op(tape_id_for_op1(op1), true)
            }
            Op1::RwTpJ | Op1::RwTpK | Op1::RwTpL | Op1::RwTpM => {
                self.tape_mut(tape_id_for_op1(op1)).rewind();
                Ok(Op1Outcome::Normal)
            }
            Op1::EfTpJ | Op1::EfTpK | Op1::EfTpL | Op1::EfTpM => {
                self.tape_mut(tape_id_for_op1(op1)).end_file()?;
                Ok(Op1Outcome::Normal)
            }
            Op1::WrDrP => self.write_drum_op(DrumId::P, eff_a, eff_b, eff_c),
            Op1::WrDrQ => self.write_drum_op(DrumId::Q, eff_a, eff_b, eff_c),
            Op1::RfDrP => self.read_drum_op(DrumId::P, eff_a, eff_b, eff_c),
            Op1::RfDrQ => self.read_drum_op(DrumId::Q, eff_a, eff_b, eff_c),
            Op1::Print => {
                let count = Self::block_word_count(eff_a, eff_b)?;
                let mut vals = Vec::with_capacity(count);
                for raw in eff_a.raw()..=eff_b.raw() {
                    vals.push(self.memory.read_float(Address::user(raw)?)?);
                }
                let _ = self.printer.print_block(&vals, eff_c.raw())?;
                Ok(Op1Outcome::Normal)
            }
            Op1::NoOp => Ok(Op1Outcome::Normal),
        }
    }

    /// Fetches, interprets (`R`-code), executes `OP1` then `OP2`, logs listing if enabled, and advances `PC`.
    pub fn step(&mut self) -> Result<StepOutcome, SpeedError> {
        let instr = self.memory.read_instruction(self.pc)?;
        let (eff_a, eff_b, eff_c) =
            self.r_quantities
                .effective_addresses(instr.r_code, instr.a, instr.b, instr.c)?;

        match self.execute_op1(instr.op1, eff_a, eff_b, eff_c)? {
            Op1Outcome::Normal => {}
            Op1Outcome::SkipInstructions(skip) => {
                let next_pc = self.advance_pc(1 + skip)?;
                self.pc = next_pc;
                return Ok(StepOutcome::Continued { next_pc });
            }
            Op1Outcome::Stop(stop) => return Ok(StepOutcome::Stopped(stop)),
        }

        if let Some((ia, ib, ic)) = ti_flags(instr.op2) {
            self.r_quantities.apply_ti(ia, ib, ic)?;
            let next_pc = Address::user(instr.d.raw())?;
            self.pc = next_pc;
            return Ok(StepOutcome::Continued { next_pc });
        }

        if let Some((da, db, dc)) = td_flags(instr.op2) {
            if !self.r_quantities.apply_td(da, db, dc) {
                return Ok(StepOutcome::Stopped(MachineStop::RQuantityUnderflow));
            }
            let next_pc = Address::user(instr.d.raw())?;
            self.pc = next_pc;
            return Ok(StepOutcome::Continued { next_pc });
        }

        let next_pc = match instr.op2 {
            Op2::NoOp => self.advance_pc(1)?,
            Op2::Tr => Address::user(instr.d.raw())?,
            Op2::TrPl => {
                let qc = self.memory.read_float(eff_c)?;
                if !qc.is_zero_fraction() && !qc.negative {
                    Address::user(instr.d.raw())?
                } else {
                    self.advance_pc(1)?
                }
            }
            Op2::TrMn => {
                let qc = self.memory.read_float(eff_c)?;
                if !qc.is_zero_fraction() && qc.negative {
                    Address::user(instr.d.raw())?
                } else {
                    self.advance_pc(1)?
                }
            }
            Op2::TrZ => {
                let qc = self.memory.read_float(eff_c)?;
                if qc.is_zero_fraction() {
                    Address::user(instr.d.raw())?
                } else {
                    self.advance_pc(1)?
                }
            }
            Op2::SnTrP => {
                if self.switches.p_switch_4 {
                    Address::user(instr.d.raw())?
                } else {
                    self.advance_pc(1)?
                }
            }
            Op2::SnTrQ => {
                if self.switches.q_switch_5 {
                    Address::user(instr.d.raw())?
                } else {
                    self.advance_pc(1)?
                }
            }
            Op2::SetRA => {
                self.r_quantities.r_a = instr.d.raw();
                self.advance_pc(1)?
            }
            Op2::SetRB => {
                self.r_quantities.r_b = instr.d.raw();
                self.advance_pc(1)?
            }
            Op2::SetRC => {
                self.r_quantities.r_c = instr.d.raw();
                self.advance_pc(1)?
            }
            Op2::SkRA => {
                let delta = if self.r_quantities.r_a == instr.d.raw() {
                    2
                } else {
                    1
                };
                self.advance_pc(delta)?
            }
            Op2::SkRB => {
                let delta = if self.r_quantities.r_b == instr.d.raw() {
                    2
                } else {
                    1
                };
                self.advance_pc(delta)?
            }
            Op2::SkRC => {
                let delta = if self.r_quantities.r_c == instr.d.raw() {
                    2
                } else {
                    1
                };
                self.advance_pc(delta)?
            }
            Op2::RAddA | Op2::RAddB | Op2::RAddC | Op2::RAddD => {
                let field = match instr.op2 {
                    Op2::RAddA => AddrField::A,
                    Op2::RAddB => AddrField::B,
                    Op2::RAddC => AddrField::C,
                    _ => AddrField::D,
                };
                let val = self.read_addr_field(instr.d, field)?;
                self.address_counter.reset_and_add(val);
                self.advance_pc(1)?
            }
            Op2::AddA | Op2::AddB | Op2::AddC | Op2::AddD => {
                let field = match instr.op2 {
                    Op2::AddA => AddrField::A,
                    Op2::AddB => AddrField::B,
                    Op2::AddC => AddrField::C,
                    _ => AddrField::D,
                };
                let val = self.read_addr_field(instr.d, field)?;
                self.address_counter.add(val)?;
                self.advance_pc(1)?
            }
            Op2::SubA | Op2::SubB | Op2::SubC | Op2::SubD => {
                let field = match instr.op2 {
                    Op2::SubA => AddrField::A,
                    Op2::SubB => AddrField::B,
                    Op2::SubC => AddrField::C,
                    _ => AddrField::D,
                };
                let val = self.read_addr_field(instr.d, field)?;
                self.address_counter.sub(val)?;
                self.advance_pc(1)?
            }
            Op2::StA | Op2::StB | Op2::StC | Op2::StD => {
                let field = match instr.op2 {
                    Op2::StA => AddrField::A,
                    Op2::StB => AddrField::B,
                    Op2::StC => AddrField::C,
                    _ => AddrField::D,
                };
                let val = self.address_counter.stored_address();
                self.write_addr_field(instr.d, field, val)?;
                self.advance_pc(1)?
            }
            Op2::Skip => {
                let delta =
                    if self.address_counter.value().unsigned_abs() == u32::from(instr.d.raw()) {
                        2
                    } else {
                        1
                    };
                self.advance_pc(delta)?
            }
            Op2::Stop => {
                self.pending_restart_addr = Some(instr.d);
                return Ok(StepOutcome::Stopped(MachineStop::ProgramStop {
                    restart_addr: instr.d,
                }));
            }
            Op2::PrCh | Op2::StCh | Op2::EchTr => {
                todo!("Checking OP2 operations implemented in Sprint 6")
            }
            other => unreachable!("TI/TD handled above: {other:?}"),
        };

        self.pc = next_pc;
        Ok(StepOutcome::Continued { next_pc })
    }

    /// Restarts execution after a `STOP` (`OP2 = 123`) by transferring control to its `D` address.
    pub fn restart_after_stop(&mut self) -> Result<Address, SpeedError> {
        let target = self.pending_restart_addr.take().ok_or_else(|| {
            SpeedError::CheckingProtocolViolation("no pending STOP restart address".to_string())
        })?;
        let next = Address::user(target.raw())?;
        self.pc = next;
        Ok(next)
    }

    /// Runs instructions continuously until a `MachineStop` occurs or `max_steps` is exceeded.
    pub fn run_until_stop(&mut self, max_steps: usize) -> Result<MachineStop, SpeedError> {
        for _ in 0..max_steps {
            if let StepOutcome::Stopped(stop) = self.step()? {
                return Ok(stop);
            }
        }
        Err(SpeedError::CheckingProtocolViolation(format!(
            "machine did not stop within {max_steps} steps"
        )))
    }
}
