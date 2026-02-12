use zerodmg_utils::little_endian::{u8_get_bit, u8s_to_u16, u16_to_u8s};

use zerodmg_codes::instruction::{
    FlagCondition, Instruction, U8Register, U8SecondaryRegister, U16Register,
};

use super::GameBoy;
use super::memory::MemoryController;
use super::video::VideoController;

#[derive(Debug, Clone, Copy)]
pub struct CPUData {
    /// Clock ticks
    t: u64,
    /// A/Accumulator register
    a: u8,
    /// F/Flags register
    f: u8,
    /// BC register/B and C registers
    b: u8,
    c: u8,
    /// DE register/D and E registers
    d: u8,
    e: u8,
    /// HL register/H and L registers
    h: u8,
    l: u8,
    /// SP/Stack Pointer register
    sp: u16,
    /// PC/Program Counter register
    pc: u16,
    /// Interrupt Master Enable register
    ime: bool,
    /// Interrupt Enable register 0xFFFF
    ie: u8,
    /// Interrupt Flag/trigger register 0xFF0F
    ift: u8,
    /// Disable interrupts after next instruction
    #[expect(dead_code)]
    di_pending: bool,
    /// Enable interrupt after next instruction
    #[expect(dead_code)]
    ei_pending: bool,
}

pub struct InstructionExecution {
    pub t_0: u64,
    pub t_1: u64,
    pub instruction: Instruction,
    /// Formats some additional debug information about the execution.
    pub tracer: Option<Box<dyn Fn() -> String>>,
    pub source: InstructionSource,
}

pub trait CPUController:
    GetSetRegisters<U8Register, u8>
    + GetSetRegisters<U16Register, u16>
    + GetSetRegisters<U8SecondaryRegister, u8>
{
    fn tick(&mut self) -> InstructionExecution;
    fn relative_jump(&mut self, n: i8);
    fn stack_push(&mut self, value: u16);
    fn stack_pop(&mut self) -> u16;
    #[expect(dead_code)]
    fn af(&self) -> u16;
    #[expect(dead_code)]
    fn set_af(&mut self, value: u16);
    fn c_flag(&self) -> bool;
    #[expect(dead_code)]
    fn set_c_flag(&mut self, value: bool);
    fn h_flag(&self) -> bool;
    fn set_h_flag(&mut self, value: bool);
    fn n_flag(&self) -> bool;
    fn set_n_flag(&mut self, value: bool);
    fn z_flag(&self) -> bool;
    fn set_z_flag(&mut self, value: bool);
    fn set_znhc_flags(&mut self, z: bool, n: bool, h: bool, c: bool);
    fn iter_bytes_at_pc<'gb>(&'gb mut self) -> PCMemoryIterator<'gb>;
    fn instruction_from_pc(&mut self) -> Instruction;
    fn condition(&self, condition: FlagCondition) -> bool;
    fn pop_interrupt(&mut self) -> Option<InterruptType>;
    fn ie(&self) -> u8;
    fn set_ie(&mut self, value: u8);
    fn ift(&self) -> u8;
    fn set_ift(&mut self, value: u8);
}

impl CPUData {
    pub fn new() -> Self {
        Self {
            t: 0x0000000000000000,
            a: rand::random(),
            f: rand::random(),
            b: rand::random(),
            c: rand::random(),
            d: rand::random(),
            e: rand::random(),
            h: rand::random(),
            l: rand::random(),
            sp: 0x0000,
            pc: 0x0000,
            ime: true,
            ie: 0xFF,
            ift: 0x00,
            di_pending: false,
            ei_pending: false,
        }
    }

    /// DMG post-boot register state (after boot ROM completes).
    pub fn post_boot() -> Self {
        Self {
            t: 0,
            a: 0x01,   // DMG
            f: 0xB0,   // Z=1 N=0 H=1 C=1
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            ime: false, // Interrupts disabled after boot
            ie: 0x00,
            ift: 0x00,
            di_pending: false,
            ei_pending: false,
        }
    }

    /// Returns the current program counter value.
    pub fn pc(&self) -> u16 {
        self.pc
    }
}

/// Iterates over bytes at PC, while incrementing it, in a borrowed [GameBoy].
pub struct PCMemoryIterator<'gb> {
    gb: &'gb mut GameBoy,
}

impl<'gb> Iterator for PCMemoryIterator<'gb> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let pc_0 = self.gb.cpu.pc;
        let byte = self.gb.mem(pc_0);
        let pc_1 = pc_0.wrapping_add(0x001);
        self.gb.cpu.pc = pc_1;
        Some(byte)
    }
}

#[derive(Clone, Copy)]
pub enum InterruptType {
    VBlank,
    LcdStatus,
    TimerOverflow,
    SerialTransfer,
    ButtonAction,
}

impl std::fmt::Display for InterruptType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::InterruptType::*;
        write!(
            f,
            "{}",
            match self {
                VBlank => "VBLANK",
                LcdStatus => "LCDSTA",
                TimerOverflow => "TIMERO",
                SerialTransfer => "SERIAL",
                ButtonAction => "BUTTON",
            }
        )
    }
}

impl InterruptType {
    fn handler_address(self) -> u16 {
        use self::InterruptType::*;
        match self {
            VBlank => 0x40,
            LcdStatus => 0x48,
            TimerOverflow => 0x50,
            SerialTransfer => 0x58,
            ButtonAction => 0x60,
        }
    }
}

#[derive(Clone, Copy)]
pub enum InstructionSource {
    ProgramCounter(u16),
    Interrupt(InterruptType),
}

impl std::fmt::Display for InstructionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::InstructionSource::*;
        match self {
            ProgramCounter(address) => write!(f, "0x{:04X}", address),
            Interrupt(interrupt_type) => write!(f, "{}", interrupt_type),
        }
    }
}

impl CPUController for GameBoy {
    fn tick(&mut self) -> InstructionExecution {
        use zerodmg_codes::instruction::prelude::*;

        let has_interrupt = self.pop_interrupt();

        let source;
        let instruction;

        // if self.cpu.pc == 0x0007 {
        //     // temporarily prevent blanking of video memory
        //     instruction = Instruction::DEC_16(U16Register::HL);
        //     source = InstructionSource::ProgramCounter(self.cpu.pc);
        //     self.cpu.pc = 0x0008;
        // } else
        // if true {
        //     instruction = NOP;
        //     source = InstructionSource::ProgramCounter(0xF0BA);
        // } else
        if let Some(interrupt) = has_interrupt {
            // disable interrupts
            self.cpu.ime = false;
            source = InstructionSource::Interrupt(interrupt);
            instruction = Instruction::CALL(interrupt.handler_address());
        } else {
            source = InstructionSource::ProgramCounter(self.cpu.pc);
            instruction = self.instruction_from_pc();
        };

        // Tracing disabled

        let t_0 = self.cpu.t;
        let cycles;
        let tracer: Option<Box<dyn Fn() -> String>>;
        macro_rules! trace {
            ($($x:expr),*) => {
                tracer = Some(Box::new(move || { format!($($x),*) }))
            }
        }

        match instruction {
            // Control
            NOP => {
                cycles = 1;
                tracer = None;
            }
            HALT => {
                // HALT: CPU stops advancing PC until an interrupt occurs.
                // If no interrupt pending (IE & IF == 0), hold PC at HALT so it
                // re-executes next tick. The caller advances video/timer between
                // ticks, which will eventually fire an interrupt.
                // If interrupt pending, resume (PC already past HALT is correct).
                if self.cpu.ie & self.cpu.ift == 0 {
                    // No interrupt pending — stay at HALT instruction
                    self.cpu.pc -= 1; // back up PC to re-execute HALT
                }
                // Either way, consume 1 cycle
                cycles = 1;
                tracer = None;
            }
            STOP(_unused) => {
                // STOP halts CPU and LCD until a button is pressed.
                // For now, treat as NOP.
                cycles = 1;
                tracer = None;
            }
            EI => {
                // Enable interrupts (takes effect after the next instruction)
                self.cpu.ime = true;
                cycles = 1;
                tracer = None;
            }
            DI => {
                // Disable interrupts
                self.cpu.ime = false;
                cycles = 1;
                tracer = None;
            }
            HCF(_variant) => unimplemented!("CPU instruction: HCF (halt and catch fire)"),
            // 8-Bit Arithmatic and Logic
            INC(target) => {
                let (old_value, extra_read_cycles) = self.read_register(target);
                let new_value = old_value.wrapping_add(1);
                let extra_write_cycles = self.set_register(target, new_value);
                self.set_z_flag(new_value == 0);
                self.set_n_flag(false);
                // Half-carry: carry from bit 3 to bit 4
                self.set_h_flag((old_value & 0x0F) + 1 > 0x0F);
                cycles = 1 + extra_read_cycles + extra_write_cycles;
                trace!(
                    "{}₀ = 0x{:02X}, {}₁ = 0x{:02X}",
                    target, old_value, target, new_value
                );
            }
            DEC(target) => {
                let (old_value, extra_read_cycles) = self.read_register(target);
                let new_value = old_value.wrapping_sub(1);
                let extra_write_cycles = self.set_register(target, new_value);
                self.set_z_flag(new_value == 0);
                self.set_n_flag(true);
                // Half-carry (borrow from bit 4): set if lower nibble was 0
                self.set_h_flag((old_value & 0x0F) == 0);
                cycles = 1 + extra_read_cycles + extra_write_cycles;
                trace!(
                    "{}₀ = 0x{:02X}, {}₁ = 0x{:02X}",
                    target, old_value, target, new_value
                );
            }
            ADD(source) => {
                let a_0 = self.cpu.a;
                let (value, extra_read_cycles) = self.read_register(source);
                let a_1 = a_0.wrapping_add(value);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) + (value & 0x0F) > 0x0F;
                let c = (a_0 as u16) + (value as u16) > 0xFF;
                self.set_znhc_flags(a_1 == 0, false, h, c);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, A₁ = 0x{:02X}",
                    a_0, source, value, a_1
                );
            }
            ADC(source) => {
                let a_0 = self.cpu.a;
                let (value, extra_read_cycles) = self.read_register(source);
                let carry = if self.c_flag() { 1u8 } else { 0u8 };
                let a_1 = a_0.wrapping_add(value).wrapping_add(carry);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) + (value & 0x0F) + carry > 0x0F;
                let c = (a_0 as u16) + (value as u16) + (carry as u16) > 0xFF;
                self.set_znhc_flags(a_1 == 0, false, h, c);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, carry = {}, A₁ = 0x{:02X}",
                    a_0, source, value, carry, a_1
                );
            }
            SUB(source) => {
                let (value, extra_read_cycles) = self.read_register(source);
                let a_0 = self.cpu.a;
                let a_1 = a_0.wrapping_sub(value);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) < (value & 0x0F);
                let c = a_0 < value;
                self.set_znhc_flags(a_1 == 0, true, h, c);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, A₁ = 0x{:02X}",
                    a_0, source, value, a_1
                );
            }
            SBC(source) => {
                let a_0 = self.cpu.a;
                let (value, extra_read_cycles) = self.read_register(source);
                let carry = if self.c_flag() { 1u8 } else { 0u8 };
                let a_1 = a_0.wrapping_sub(value).wrapping_sub(carry);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) < (value & 0x0F) + carry;
                let c = (a_0 as u16) < (value as u16) + (carry as u16);
                self.set_znhc_flags(a_1 == 0, true, h, c);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, carry = {}, A₁ = 0x{:02X}",
                    a_0, source, value, carry, a_1
                );
            }
            AND(source) => {
                let (value, extra_read_cycles) = self.read_register(source);
                let a_0 = self.cpu.a;
                let a_1 = a_0 & value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, true, false);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, A₁ = 0x{:02X}",
                    a_0, source, value, a_1
                );
            }
            XOR(source) => {
                let a_0 = self.cpu.a;
                let (value, extra_read_cycles) = self.read_register(source);
                let a_1 = a_0 ^ value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, false, false);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, A₁ = 0x{:02X}",
                    a_0, source, value, a_1
                );
            }
            OR(source) => {
                let (value, extra_read_cycles) = self.read_register(source);
                let a_0 = self.cpu.a;
                let a_1 = a_0 | value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, false, false);
                cycles = 1 + extra_read_cycles;
                trace!(
                    "A₀ = 0x{:02X}, {} = 0x{:02X}, A₁ = 0x{:02X}",
                    a_0, source, value, a_1
                );
            }
            CP(source) => {
                let (value, extra_read_cycles) = self.read_register(source);
                let a = self.cpu.a;
                let result = a.wrapping_sub(value);
                let h = (a & 0x0F) < (value & 0x0F);
                let c = a < value;
                self.set_znhc_flags(result == 0, true, h, c);
                cycles = 1 + extra_read_cycles;
                trace!("A = 0x{:02X}, {} = 0x{:02X}", a, source, value);
            }
            ADD_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let a_1 = a_0.wrapping_add(value);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) + (value & 0x0F) > 0x0F;
                let c = (a_0 as u16) + (value as u16) > 0xFF;
                self.set_znhc_flags(a_1 == 0, false, h, c);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            ADC_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let carry = if self.c_flag() { 1u8 } else { 0u8 };
                let a_1 = a_0.wrapping_add(value).wrapping_add(carry);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) + (value & 0x0F) + carry > 0x0F;
                let c = (a_0 as u16) + (value as u16) + (carry as u16) > 0xFF;
                self.set_znhc_flags(a_1 == 0, false, h, c);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, carry = {}, A₁ = 0x{:02X}", a_0, carry, a_1);
            }
            SUB_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let a_1 = a_0.wrapping_sub(value);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) < (value & 0x0F);
                let c = a_0 < value;
                self.set_znhc_flags(a_1 == 0, true, h, c);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            SBC_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let carry = if self.c_flag() { 1u8 } else { 0u8 };
                let a_1 = a_0.wrapping_sub(value).wrapping_sub(carry);
                self.cpu.a = a_1;
                let h = (a_0 & 0x0F) < (value & 0x0F) + carry;
                let c = (a_0 as u16) < (value as u16) + (carry as u16);
                self.set_znhc_flags(a_1 == 0, true, h, c);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, carry = {}, A₁ = 0x{:02X}", a_0, carry, a_1);
            }
            AND_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let a_1 = a_0 & value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, true, false);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            XOR_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let a_1 = a_0 ^ value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, false, false);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            OR_IMMEDIATE(value) => {
                let a_0 = self.cpu.a;
                let a_1 = a_0 | value;
                self.cpu.a = a_1;
                self.set_znhc_flags(a_1 == 0, false, false, false);
                cycles = 2;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            CP_IMMEDIATE(value) => {
                let a = self.cpu.a;
                let result = a.wrapping_sub(value);
                let half_carry = (a & 0xF) < (value & 0xF);
                self.set_znhc_flags(result == 0, true, half_carry, a < value);
                let z_flag = self.z_flag();
                let c_flag = self.c_flag();
                cycles = 2;
                trace!("A = 0x{:02X}, F_Z = {}, F_C = {}", a, z_flag, c_flag);
            }
            CPL => {
                // Complement A (flip all bits)
                let a_0 = self.cpu.a;
                self.cpu.a = !a_0;
                self.set_n_flag(true);
                self.set_h_flag(true);
                cycles = 1;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, !a_0);
            }
            CCF => {
                // Complement carry flag
                let c_0 = self.c_flag();
                self.set_znhc_flags(self.z_flag(), false, false, !c_0);
                cycles = 1;
                trace!("C₀ = {}, C₁ = {}", c_0, !c_0);
            }
            SCF => {
                // Set carry flag
                self.set_znhc_flags(self.z_flag(), false, false, true);
                cycles = 1;
                tracer = None;
            }
            DAA => {
                // Decimal Adjust Accumulator (BCD correction)
                let mut a = self.cpu.a;
                let n = self.n_flag();
                let h = self.h_flag();
                let c = self.c_flag();
                let mut new_c = false;
                if !n {
                    // After addition
                    if c || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        new_c = true;
                    }
                    if h || (a & 0x0F) > 0x09 {
                        a = a.wrapping_add(0x06);
                    }
                } else {
                    // After subtraction
                    if c {
                        a = a.wrapping_sub(0x60);
                        new_c = true;
                    }
                    if h {
                        a = a.wrapping_sub(0x06);
                    }
                }
                let a_0 = self.cpu.a;
                self.cpu.a = a;
                self.set_znhc_flags(a == 0, n, false, new_c);
                cycles = 1;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a);
            }
            // 16-Bit Arithmatic and Logic
            INC_16(target) => {
                let old_value = self.get_register(target);
                let new_value = old_value.wrapping_add(1);
                self.set_register(target, new_value);
                cycles = 2;
                trace!(
                    "{:?}₀ = 0x{:02X}, {:?}₁ = 0x{:02X}",
                    target, old_value, target, new_value
                );
            }
            DEC_16(target) => {
                let old_value = self.get_register(target);
                let new_value = old_value.wrapping_sub(1);
                self.set_register(target, new_value);
                cycles = 2;
                trace!(
                    "{:?}₀ = 0x{:02X}, {:?}₁ = 0x{:02X}",
                    target, old_value, target, new_value
                );
            }
            ADD_TO_HL(source) => {
                let hl_0 = self.get_register(U16Register::HL);
                let value = self.get_register(source);
                let hl_1 = hl_0.wrapping_add(value);
                self.set_register(U16Register::HL, hl_1);
                let h = (hl_0 & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
                let c = (hl_0 as u32) + (value as u32) > 0xFFFF;
                self.set_n_flag(false);
                self.set_h_flag(h);
                self.set_znhc_flags(self.z_flag(), false, h, c);
                cycles = 2;
                trace!(
                    "HL₀ = 0x{:04X}, {:?} = 0x{:04X}, HL₁ = 0x{:04X}",
                    hl_0, source, value, hl_1
                );
            }
            ADD_SP(offset) => {
                let sp_0 = self.cpu.sp;
                let sp_1 = (sp_0 as i32 + offset as i32) as u16;
                // Flags are based on the low byte addition
                let h = (sp_0 & 0x000F) + ((offset as u8) as u16 & 0x000F) > 0x000F;
                let c = (sp_0 & 0x00FF) + ((offset as u8) as u16) > 0x00FF;
                self.cpu.sp = sp_1;
                self.set_znhc_flags(false, false, h, c);
                cycles = 4;
                trace!("SP₀ = 0x{:04X}, SP₁ = 0x{:04X}", sp_0, sp_1);
            }
            // 8-Bit Bitwise Operations
            RL(register) => {
                let f_c_0 = self.c_flag();
                let value_0 = self.get_register(register);
                let value_1 = (value_0 << 1) + if f_c_0 { 1 } else { 0 };
                let f_c_1 = value_0 & 0b1000_0000 > 0;
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, f_c_1);
                cycles = 2;
                trace!(
                    "Fc₀ = {}, {}₀ = 0x{:02X}, Fc₁ = {}, {}₁ = 0x{:02X}",
                    f_c_0, register, value_0, f_c_1, register, value_1
                );
            }
            RLA => {
                let f_c_0 = self.c_flag();
                let a_0 = self.cpu.a;
                let a_1 = (a_0 << 1) + if f_c_0 { 1 } else { 0 };
                let f_c_1 = a_0 & 0b1000_0000 > 0;
                self.cpu.a = a_1;
                // RLA always clears Z (unlike RL)
                self.set_znhc_flags(false, false, false, f_c_1);
                cycles = 1;
                trace!(
                    "Fc₀ = {}, A₀ = 0x{:02X}, Fc₁ = {}, A₁ = 0x{:02X}",
                    f_c_0, a_0, f_c_1, a_1
                );
            }
            RLC(register) => {
                let value_0 = self.get_register(register);
                let high_bit = value_0 >> 7;
                let value_1 = (value_0 << 1) | high_bit;
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, high_bit != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            RLCA => {
                let a_0 = self.cpu.a;
                let high_bit = a_0 >> 7;
                let a_1 = (a_0 << 1) | high_bit;
                self.cpu.a = a_1;
                // RLCA always clears Z (unlike RLC)
                self.set_znhc_flags(false, false, false, high_bit != 0);
                cycles = 1;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            RR(register) => {
                let value_0 = self.get_register(register);
                let old_carry = if self.c_flag() { 1u8 } else { 0u8 };
                let new_carry = value_0 & 1;
                let value_1 = (value_0 >> 1) | (old_carry << 7);
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, new_carry != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            RRA => {
                let a_0 = self.cpu.a;
                let old_carry = if self.c_flag() { 1u8 } else { 0u8 };
                let new_carry = a_0 & 1;
                let a_1 = (a_0 >> 1) | (old_carry << 7);
                self.cpu.a = a_1;
                self.set_znhc_flags(false, false, false, new_carry != 0);
                cycles = 1;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            RRC(register) => {
                let value_0 = self.get_register(register);
                let low_bit = value_0 & 1;
                let value_1 = (value_0 >> 1) | (low_bit << 7);
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, low_bit != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            RRCA => {
                let a_0 = self.cpu.a;
                let low_bit = a_0 & 1;
                let a_1 = (a_0 >> 1) | (low_bit << 7);
                self.cpu.a = a_1;
                self.set_znhc_flags(false, false, false, low_bit != 0);
                cycles = 1;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            SRL(register) => {
                let value_0 = self.get_register(register);
                let low_bit = value_0 & 1;
                let value_1 = value_0 >> 1;
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, low_bit != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            SRA(register) => {
                let value_0 = self.get_register(register);
                let low_bit = value_0 & 1;
                // Arithmetic shift: preserve bit 7
                let value_1 = (value_0 >> 1) | (value_0 & 0x80);
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, low_bit != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            SLA(register) => {
                let value_0 = self.get_register(register);
                let high_bit = value_0 >> 7;
                let value_1 = value_0 << 1;
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, high_bit != 0);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            SWAP(register) => {
                let value_0 = self.get_register(register);
                let value_1 = (value_0 >> 4) | (value_0 << 4);
                self.set_register(register, value_1);
                self.set_znhc_flags(value_1 == 0, false, false, false);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            BIT(bit, register) => {
                let value = self.get_register(register);
                let result = !u8_get_bit(value, bit.index());
                self.set_z_flag(result);
                self.set_n_flag(false);
                self.set_h_flag(true);
                cycles = 2;
                trace!("Z₁ = {}", result);
            }
            SET(bit, register) => {
                let value_0 = self.get_register(register);
                let value_1 = value_0 | (1 << bit.index());
                self.set_register(register, value_1);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            RES(bit, register) => {
                let value_0 = self.get_register(register);
                let value_1 = value_0 & !(1 << bit.index());
                self.set_register(register, value_1);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, {}₁ = 0x{:02X}", register, value_0, register, value_1);
            }
            // 8-Bit Loads
            LD_8_INTERNAL(dest, source) => {
                let dest_value_0 = self.get_register(dest);
                let (source_value, extra_read_cycles) = self.read_register(source);
                let extra_write_cycles = self.set_register(dest, source_value);
                cycles = 1 + extra_read_cycles + extra_write_cycles;
                trace!(
                    "{} = {}, {}₀ = {}",
                    source, source_value, dest, dest_value_0
                );
            }
            LD_8_IMMEDIATE(dest, value) => {
                let dest_value_0 = self.get_register(dest);
                let extra_write_cycles = self.set_register(dest, value);
                cycles = 2 + extra_write_cycles;
                trace!("{}₀ = 0x{:02X}", dest, dest_value_0);
            }
            LD_8_TO_SECONDARY(dest) => {
                let dest_value_0 = self.get_register(dest);
                let a = self.cpu.a;
                self.set_register(dest, a);
                cycles = 2;
                trace!("{}₀ = 0x{:02X}, A = 0x{:02X}", dest, dest_value_0, a)
            }
            LD_8_FROM_SECONDARY(source) => {
                let a_0 = self.cpu.a;
                // Must use read_register for HLI/HLD side effects
                let (a_1, extra_read_cycles) = self.read_register(source);
                self.cpu.a = a_1;
                cycles = 2 + extra_read_cycles;
                trace!("A₀ = 0x{:02X}, {} = 0x{:02X}", a_0, source, a_1)
            }
            LD_8_TO_FF_IMMEDIATE(offset) => {
                let a = self.cpu.a;
                let address = 0xFF00 + u16::from(offset);
                self.set_mem(address, a);
                cycles = 4;
                trace!("A = 0x{:02X}", a);
            }
            LD_8_FROM_FF_IMMEDIATE(offset) => {
                let a_0 = self.cpu.a;
                let a_1 = self.mem(0xFF00 + u16::from(offset));
                self.cpu.a = a_1;
                cycles = 3;
                trace!("A₀ = 0x{:02X}, A₁ = 0x{:02X}", a_0, a_1);
            }
            LD_8_TO_FF_C => {
                let a = self.cpu.a;
                let c = self.cpu.c;
                let address = 0xFF00 + u16::from(c);
                let old_value = self.mem(address);
                self.set_mem(address, a);
                cycles = 2;
                trace!(
                    "C = 0x{:02X}, A = 0x{:02X}, (0xFF00 + C)₀ = 0x{:02X}",
                    c, a, old_value
                );
            }
            LD_8_FROM_FF_C => {
                let c = self.cpu.c;
                let address = 0xFF00 + u16::from(c);
                let a_0 = self.cpu.a;
                let a_1 = self.mem(address);
                self.cpu.a = a_1;
                cycles = 2;
                trace!(
                    "C = 0x{:02X}, A₀ = 0x{:02X}, A₁ = 0x{:02X}",
                    c, a_0, a_1
                );
            }
            LD_8_TO_MEMORY_IMMEDIATE(address) => {
                let a = self.cpu.a;
                let old_value = self.mem(address);
                self.set_mem(address, a);
                cycles = 4;
                trace!("A = {:02X}, (0x{:04X})₀ = 0x{:02X}", address, a, old_value);
            }
            LD_8_FROM_MEMORY_IMMEDIATE(address) => {
                let a_0 = self.cpu.a;
                let a_1 = self.mem(address);
                self.cpu.a = a_1;
                cycles = 4;
                trace!("(0x{:04X}) = 0x{:02X}, A₀ = 0x{:02X}", address, a_1, a_0);
            }
            // 16-Bit Loads
            LD_16_IMMEDIATE(dest, value) => {
                let old_value = self.get_register(dest);
                self.set_register(dest, value);
                cycles = 3;
                trace!("{:?}₀ = 0x{:04X}", dest, old_value);
            }
            LD_HL_FROM_SP => {
                let sp = self.cpu.sp;
                self.set_register(U16Register::HL, sp);
                cycles = 2;
                trace!("SP = 0x{:04X}", sp);
            }
            LD_HL_FROM_SP_PLUS(offset) => {
                let sp = self.cpu.sp;
                let result = (sp as i32 + offset as i32) as u16;
                self.set_register(U16Register::HL, result);
                let h = (sp & 0x000F) + ((offset as u8) as u16 & 0x000F) > 0x000F;
                let c = (sp & 0x00FF) + ((offset as u8) as u16) > 0x00FF;
                self.set_znhc_flags(false, false, h, c);
                cycles = 3;
                trace!("SP = 0x{:04X}, HL₁ = 0x{:04X}", sp, result);
            }
            LD_SP_TO_IMMEDIATE_ADDRESS(address) => {
                let sp = self.cpu.sp;
                let (lo, hi) = u16_to_u8s(sp);
                self.set_mem(address, lo);
                self.set_mem(address.wrapping_add(1), hi);
                cycles = 5;
                trace!("SP = 0x{:04X}, (0x{:04X}) = SP", sp, address);
            }
            PUSH(register) => {
                let value = self.get_register(register);
                self.stack_push(value);
                let sp_1 = self.cpu.sp;
                cycles = 4;
                trace!("{:?} = 0x{:02X}, SP₁ = 0x{:04X}", register, value, sp_1);
            }
            POP(register) => {
                let value = self.stack_pop();
                let sp_1 = self.cpu.sp;
                self.set_register(register, value);
                cycles = 3;
                trace!("{:?}₁ = 0x{:02X}, SP₁ = 0x{:04X}", register, value, sp_1);
            }
            PUSH_AF => {
                let a = self.cpu.a;
                let f = self.cpu.f;
                let af = u16::from(a) << 8 | u16::from(f);
                self.stack_push(af);
                let sp_1 = self.cpu.sp;
                cycles = 4;
                trace!("AF = 0x{:04X}, SP₁ = 0x{:04X}", af, sp_1);
            }
            POP_AF => {
                let af = self.stack_pop();
                let sp_1 = self.cpu.sp;
                self.cpu.a = (af >> 8) as u8;
                // Lower nibble of F is always 0 on GB
                self.cpu.f = (af & 0xF0) as u8;
                cycles = 3;
                trace!("AF₁ = 0x{:04X}, SP₁ = 0x{:04X}", af, sp_1);
            }
            // Jumps and Calls
            JP_IF(condition, address) => {
                if self.condition(condition) {
                    self.cpu.pc = address;
                    trace!("jumped - condition true");
                    cycles = 4;
                } else {
                    trace!("skipped - condition false");
                    cycles = 3;
                }
            }
            JP(address) => {
                self.cpu.pc = address;
                cycles = 4;
                tracer = None;
            }
            JP_HL => {
                let hl = self.get_register(U16Register::HL);
                self.cpu.pc = hl;
                cycles = 1;
                trace!("HL = 0x{:04X}", hl);
            }
            JR_IF(condition, offset) => {
                if self.condition(condition) {
                    self.relative_jump(offset);
                    trace!("jumped - condition true");
                    cycles = 3;
                } else {
                    trace!("skipped - condition false");
                    cycles = 2;
                }
            }
            JR(offset) => {
                self.relative_jump(offset);
                cycles = 3;
                tracer = None;
            }
            CALL_IF(condition, address) => {
                if self.condition(condition) {
                    let pc_0 = self.cpu.pc;
                    self.stack_push(pc_0);
                    self.cpu.pc = address;
                    let sp_1 = self.cpu.sp;
                    cycles = 6;
                    trace!("SP₁ = {:04X}", sp_1);
                } else {
                    cycles = 3;
                    trace!("skipped - condition false");
                }
            }
            CALL(address) => {
                let pc_0 = self.cpu.pc;
                self.stack_push(pc_0);
                self.cpu.pc = address;
                let sp_1 = self.cpu.sp;
                cycles = 6;
                trace!("SP₁ = {:04X}", sp_1);
            }
            RST(address) => {
                let pc_0 = self.cpu.pc;
                let pc_1 = address.address().into();
                self.stack_push(pc_0);
                self.cpu.pc = pc_1;
                cycles = 4;
                tracer = None;
            }
            RET => {
                let pc_1 = self.stack_pop();
                let sp_1 = self.cpu.sp;
                self.cpu.pc = pc_1;
                cycles = 2;
                trace!("SP₁ = {:04X}", sp_1);
            }
            RET_IF(condition) => {
                if self.condition(condition) {
                    let pc_1 = self.stack_pop();
                    let sp_1 = self.cpu.sp;
                    self.cpu.pc = pc_1;
                    cycles = 5;
                    trace!("returned - PC₁ = 0x{:04X}, SP₁ = 0x{:04X}", pc_1, sp_1);
                } else {
                    cycles = 2;
                    trace!("skipped - condition false");
                }
            }
            RETI => {
                // Return from interrupt handler and re-enable interrupts
                let pc_1 = self.stack_pop();
                let sp_1 = self.cpu.sp;
                self.cpu.pc = pc_1;
                self.cpu.ime = true;
                cycles = 4;
                trace!("PC₁ = 0x{:04X}, SP₁ = 0x{:04X}", pc_1, sp_1);
            }
        }

        let t_1 = t_0 + cycles;
        self.cpu.t = t_1;

        InstructionExecution {
            instruction,
            t_0,
            t_1,
            source,
            tracer,
        }
    }

    /// Returns the next InterruptType currently set in the interrupt register,
    /// and unsets it there.
    fn pop_interrupt(&mut self) -> Option<InterruptType> {
        // Only dispatch interrupts when IME (Interrupt Master Enable) is set
        if !self.cpu.ime {
            return None;
        }
        let enabled_and_triggered = self.cpu.ie & self.cpu.ift;
        if enabled_and_triggered & 0b00001 != 0 {
            self.cpu.ift &= !0b00001;
            Some(InterruptType::VBlank)
        } else if enabled_and_triggered & 0b00010 != 0 {
            self.cpu.ift &= !0b00010;
            Some(InterruptType::LcdStatus)
        } else if enabled_and_triggered & 0b00100 != 0 {
            self.cpu.ift &= !0b00100;
            Some(InterruptType::TimerOverflow)
        } else if enabled_and_triggered & 0b01000 != 0 {
            self.cpu.ift &= !0b01000;
            Some(InterruptType::SerialTransfer)
        } else if enabled_and_triggered & 0b10000 != 0 {
            self.cpu.ift &= !0b10000;
            Some(InterruptType::ButtonAction)
        } else {
            None
        }
    }

    fn ie(&self) -> u8 {
        self.cpu.ie
    }

    fn set_ie(&mut self, ie: u8) {
        self.cpu.ie = ie;
    }

    fn ift(&self) -> u8 {
        self.cpu.ift
    }

    fn set_ift(&mut self, ift: u8) {
        self.cpu.ift = ift;
    }

    // Returns the instruction in memory at PC, and advances PC past it.
    fn instruction_from_pc(&mut self) -> Instruction {
        Instruction::from_byte_iter(&mut self.iter_bytes_at_pc())
            .expect("failed to decode instruction at PC")
    }

    // Returns an Iterator that yields bytes from memory at PC++.
    fn iter_bytes_at_pc<'gb>(&'gb mut self) -> PCMemoryIterator<'gb> {
        PCMemoryIterator { gb: self }
    }

    fn relative_jump(&mut self, n: i8) {
        self.cpu.pc = (i32::from(self.cpu.pc) + i32::from(n)) as u16;
    }

    fn stack_push(&mut self, value: u16) {
        let sp0 = self.cpu.sp;
        let sp1 = sp0 - 2;
        let (value_low, value_high) = u16_to_u8s(value);
        // GB is little-endian: low byte at lower address
        self.set_mem(sp1, value_low);
        self.set_mem(sp1 + 1, value_high);
        self.cpu.sp = sp1;
    }

    fn stack_pop(&mut self) -> u16 {
        let sp0 = self.cpu.sp;
        let sp1 = sp0 + 2;
        // GB is little-endian: low byte at lower address
        let value_low = self.mem(sp0);
        let value_high = self.mem(sp0 + 1);
        let value = u8s_to_u16(value_low, value_high);
        self.cpu.sp = sp1;
        value
    }

    fn af(&self) -> u16 {
        u8s_to_u16(self.cpu.f, self.cpu.a)
    }

    fn set_af(&mut self, value: u16) {
        let (f, a) = u16_to_u8s(value);
        self.cpu.a = a;
        self.cpu.f = f;
    }

    fn c_flag(&self) -> bool {
        (self.cpu.f & 0x10) == 0x10
    }

    fn set_c_flag(&mut self, value: bool) {
        if value {
            self.cpu.f |= 0x10;
        } else {
            self.cpu.f &= !0x10;
        }
    }

    fn h_flag(&self) -> bool {
        (self.cpu.f & 0x20) == 0x20
    }

    fn set_h_flag(&mut self, value: bool) {
        if value {
            self.cpu.f |= 0x20;
        } else {
            self.cpu.f &= !0x20;
        }
    }

    fn n_flag(&self) -> bool {
        (self.cpu.f & 0x40) == 0x40
    }

    fn set_n_flag(&mut self, value: bool) {
        if value {
            self.cpu.f |= 0x40;
        } else {
            self.cpu.f &= !0x40;
        }
    }

    fn z_flag(&self) -> bool {
        (self.cpu.f & 0x80) == 0x80
    }

    fn set_z_flag(&mut self, value: bool) {
        if value {
            self.cpu.f |= 0x80;
        } else {
            self.cpu.f &= !0x80;
        }
    }

    fn set_znhc_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.cpu.f = if z { 0x80 } else { 0x00 }
            | if n { 0x40 } else { 0x00 }
            | if h { 0x20 } else { 0x00 }
            | if c { 0x10 } else { 0x00 };
    }

    fn condition(&self, condition: FlagCondition) -> bool {
        use zerodmg_codes::instruction::prelude::*;
        match condition {
            if_Z => self.z_flag(),
            if_NZ => !self.z_flag(),
            if_C => self.c_flag(),
            if_NC => !self.c_flag(),
        }
    }
}

pub trait GetSetRegisters<Register, RegisterValue> {
    /// Reads the value in the given register.
    ///
    /// If this is a pseudo-register like (HL+), this may have side effects.
    fn read_register(&mut self, register: Register) -> (RegisterValue, u64) {
        (self.get_register(register), 0)
    }

    /// Reads the value in the given register, suppressing any side effects.
    fn get_register(&self, register: Register) -> RegisterValue;

    /// Updates the value in the given register.
    fn set_register(&mut self, register: Register, value: RegisterValue) -> u64;
}

impl GetSetRegisters<U8Register, u8> for GameBoy {
    fn read_register(&mut self, register: U8Register) -> (u8, u64) {
        use zerodmg_codes::instruction::prelude::*;
        (
            self.get_register(register),
            match register {
                AT_HL => 1,
                _ => 0,
            },
        )
    }

    fn get_register(&self, register: U8Register) -> u8 {
        use zerodmg_codes::instruction::prelude::*;
        match register {
            B => self.cpu.b,
            C => self.cpu.c,
            D => self.cpu.d,
            E => self.cpu.e,
            H => self.cpu.h,
            L => self.cpu.l,
            AT_HL => {
                let hl = self.get_register(HL);
                self.mem(hl)
            }
            A => self.cpu.a,
        }
    }

    fn set_register(&mut self, register: U8Register, value: u8) -> u64 {
        use zerodmg_codes::instruction::prelude::*;
        let mut extra_cycles = 0;
        match register {
            B => self.cpu.b = value,
            C => self.cpu.c = value,
            D => self.cpu.d = value,
            E => self.cpu.e = value,
            H => self.cpu.h = value,
            L => self.cpu.l = value,
            AT_HL => {
                extra_cycles = 1;
                let hl = self.get_register(HL);
                self.set_mem(hl, value);
            }
            A => {
                self.cpu.a = value;
            }
        }
        extra_cycles
    }
}

impl GetSetRegisters<U16Register, u16> for GameBoy {
    fn get_register(&self, register: U16Register) -> u16 {
        use zerodmg_codes::instruction::prelude::*;
        match register {
            BC => u8s_to_u16(self.cpu.c, self.cpu.b),
            DE => u8s_to_u16(self.cpu.e, self.cpu.d),
            HL => u8s_to_u16(self.cpu.l, self.cpu.h),
            SP => self.cpu.sp,
        }
    }

    fn set_register(&mut self, register: U16Register, value: u16) -> u64 {
        use zerodmg_codes::instruction::prelude::*;
        let (low, high) = u16_to_u8s(value);
        match register {
            BC => {
                self.cpu.b = high;
                self.cpu.c = low;
            }
            DE => {
                self.cpu.d = high;
                self.cpu.e = low;
            }
            HL => {
                self.cpu.h = high;
                self.cpu.l = low;
            }
            SP => {
                self.cpu.sp = value;
            }
        }
        0
    }
}

impl GetSetRegisters<U8SecondaryRegister, u8> for GameBoy {
    fn read_register(&mut self, register: U8SecondaryRegister) -> (u8, u64) {
        use zerodmg_codes::instruction::prelude::*;
        (
            match register {
                AT_HL_Plus => {
                    let hl_0 = self.get_register(HL);
                    let hl_1 = hl_0.wrapping_add(0x0001);
                    self.set_register(HL, hl_1);
                    self.mem(hl_0)
                }
                AT_HL_Minus => {
                    let hl_0 = self.get_register(HL);
                    let hl_1 = hl_0.wrapping_sub(0x0001);
                    self.set_register(HL, hl_1);
                    self.mem(hl_0)
                }
                _ => self.get_register(register),
            },
            0,
        )
    }

    fn get_register(&self, register: U8SecondaryRegister) -> u8 {
        use zerodmg_codes::instruction::prelude::*;
        let address = match register {
            AT_BC => self.get_register(BC),
            AT_DE => self.get_register(DE),
            AT_HL_Plus | AT_HL_Minus => self.get_register(HL),
        };
        self.mem(address)
    }

    fn set_register(&mut self, register: U8SecondaryRegister, value: u8) -> u64 {
        use zerodmg_codes::instruction::prelude::*;
        match register {
            AT_BC => {
                let bc = self.get_register(BC);
                self.set_mem(bc, value);
            }
            AT_DE => {
                let de = self.get_register(DE);
                self.set_mem(de, value);
            }
            AT_HL_Plus => {
                let hl_0 = self.get_register(HL);
                let hl_1 = hl_0.wrapping_add(0x0001);
                self.set_mem(hl_0, value);
                self.set_register(HL, hl_1);
            }
            AT_HL_Minus => {
                let hl_0 = self.get_register(HL);
                let hl_1 = hl_0.wrapping_sub(0x0001);
                self.set_mem(hl_0, value);
                self.set_register(HL, hl_1);
            }
        }
        0
    }
}
