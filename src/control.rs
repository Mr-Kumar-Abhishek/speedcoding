//! Phase 4: Program counter, `OP2` control transfers, R-quantities, address counter,
//! and the top-level `SpeedCoMachine` interpreter.

use crate::arithmetic::{ArithmeticOutcome, execute_op1_arithmetic, execute_op1_math};
use crate::core_types::{
    Address, ElectrostaticMemory, INITIAL_PC, MachineStop, Op1, Op2, RCode, SPECIAL_R_CONTROL_ADDR,
    SpeedError,
};
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

/// Complete simulated IBM 701 Speedcoding System (SpeedCo I) machine.
#[derive(Debug, Clone)]
pub struct SpeedCoMachine {
    pc: Address,
    memory: ElectrostaticMemory,
    r_quantities: RQuantities,
    address_counter: AddressCounter,
    switches: SenseSwitches,
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

    /// Rewinds all 4 tapes, processes instruction/data/loading-control cards, and sets `PC = 300`.
    pub fn load_deck(&mut self, _cards: &[PunchedCard]) -> Result<(), SpeedError> {
        todo!()
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

    fn execute_op1(
        &mut self,
        op1: Op1,
        eff_a: Address,
        eff_b: Address,
        eff_c: Address,
    ) -> Result<Option<MachineStop>, SpeedError> {
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
                    ArithmeticOutcome::Value(val) => self.memory.write_float(eff_c, val)?,
                    ArithmeticOutcome::Stop(stop) => return Ok(Some(stop)),
                }
            }
            Op1::Sqrt | Op1::Sine | Op1::Artan | Op1::Exp | Op1::Ln => {
                let qa = self.memory.read_float(eff_a)?;
                let res = execute_op1_math(op1, qa)?;
                self.memory.write_float(eff_c, res)?;
            }
            Op1::NoOp => {}
            _ => todo!("I/O OP1 operations implemented in Sprint 5"),
        }
        Ok(None)
    }

    /// Fetches, interprets (`R`-code), executes `OP1` then `OP2`, logs listing if enabled, and advances `PC`.
    pub fn step(&mut self) -> Result<StepOutcome, SpeedError> {
        let instr = self.memory.read_instruction(self.pc)?;
        let (eff_a, eff_b, eff_c) =
            self.r_quantities
                .effective_addresses(instr.r_code, instr.a, instr.b, instr.c)?;

        if let Some(stop) = self.execute_op1(instr.op1, eff_a, eff_b, eff_c)? {
            return Ok(StepOutcome::Stopped(stop));
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
