use std::fmt;

use super::memory::{InterruptAddress, MemoryBus};

/// A Sharp LR35902 CPU.
///
/// This one is similar to the Intel 8080 and Zilog Z80.
pub struct CPU {
    registers: Registers,
    sp: u16, //< stack pointer
    pc: u16, //< program counter
    ime: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self{
            registers: Registers::new(),
            sp: 0xFFFE,
            pc: 0,
            ime: false,
        }
    }

    pub fn step(&mut self, memory: &mut MemoryBus) {
        let instruction = {
            let mut instruction_byte = memory.read8(self.pc);
            let prefixed = instruction_byte == 0xCB;
            if prefixed {
                instruction_byte = memory.read8(self.pc + 1);
            }
            match Instruction::from_byte(instruction_byte, prefixed) {
                Some(instruction) => instruction,
                None => {
                    let instr = if prefixed {
                        format!("CB{:0>2X}", instruction_byte)
                    } else {
                        format!("{:0>2X}", instruction_byte)
                    };
                    panic!("Could not decode Instruction {} at {:0>4X}.",
                           instr, self.pc)
                }
            }
        };
        let mut instruction_bytes: u64 = 0;
        for i in self.pc..self.pc+instruction.len() {
            instruction_bytes <<= 8;
            instruction_bytes += memory.read8(i) as u64;
        }
        // eprintln!("{}", self.registers);
        // self.print_stack(memory);
        // eprintln!("{:0>4X}: {1:0>2$X} {3:?}", self.pc, instruction_bytes,
        //           2*instruction.len() as usize,
        //           instruction);
        self.execute(memory, instruction)
    }

    fn execute(&mut self, memory: &mut MemoryBus, instruction: Instruction) {
        use Instruction::*;
        match instruction {
            NOP => {
                self.pc += 1;
            }
            ADD(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                let (new_a, carry) = self.registers.a.overflowing_add(operand);
                self.registers.a = new_a;
                let mut f = 0;
                if new_a == 0 {
                    f |= Flag::Zero as u8;
                }
                if carry {
                    f |= Flag::Carry as u8;
                }
                // TODO: How is the HalfCarry flag set?
                self.registers.f = f;
            }
            SUB(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                let (new_a, carry) = self.registers.a.overflowing_sub(operand);
                self.registers.a = new_a;
                let mut f = Flag::Subtract as u8;
                if new_a == 0 {
                    f |= Flag::Zero as u8;
                }
                if carry {
                    f |= Flag::Carry as u8;
                }
                // TODO: How is the HalfCarry flag set?
                self.registers.f = f;
            }
            INC(inc_type) => {
                self.pc += 1;
                match inc_type {
                    IncDecType::IncDec8(operand) => {
                        let value = self.load_non_direct_arithmetic_operand(
                            memory, operand);
                        let (new, _carry) = value.overflowing_add(1);
                        self.write_non_direct_arithmetic_operand(memory,
                                                                 operand, new);
                        let mut f = 0;
                        if new == 0 {
                            f |= Flag::Zero as u8;
                        }
                        // TODO: How is the HalfCarry flag set?
                        let mask: u8 = Flag::Zero as u8
                                     | Flag::Subtract as u8
                                     | Flag::HalfCarry as u8;
                        self.registers.f = (self.registers.f & !mask) | f;
                    }
                    IncDecType::IncDec16(operand) => {
                        let value = self.load_inc_dec_16_operand(operand);
                        let (new, _carry) = value.overflowing_add(1);
                        self.write_inc_dec_16_operand(operand, new);
                    }
                }
            }
            DEC(dec_type) => {
                self.pc += 1;
                match dec_type {
                    IncDecType::IncDec8(operand) => {
                        let value = self.load_non_direct_arithmetic_operand(
                            memory, operand);
                        let (new, _carry) = value.overflowing_sub(1);
                        self.write_non_direct_arithmetic_operand(memory,
                                                                 operand, new);
                        let mut f = Flag::Subtract as u8;
                        if new == 0 {
                            f |= Flag::Zero as u8;
                        }
                        // TODO: How is the HalfCarry flag set?
                        let mask: u8 = Flag::Zero as u8
                                     | Flag::Subtract as u8
                                     | Flag::HalfCarry as u8;
                        self.registers.f = (self.registers.f & !mask) | f;
                    }
                    IncDecType::IncDec16(operand) => {
                        let value = self.load_inc_dec_16_operand(operand);
                        let (new, _carry) = value.overflowing_sub(1);
                        self.write_inc_dec_16_operand(operand, new);
                    }
                }
            }
            AND(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                self.registers.a &= operand;
                self.registers.f = if self.registers.a == 0 {
                    Flag::Zero as u8
                } else {
                    0
                } | Flag::HalfCarry as u8;
            }
            XOR(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                self.registers.a ^= operand;
                self.registers.f = if self.registers.a == 0 {
                    Flag::Zero as u8
                } else {
                    0
                };
            }
            OR(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                self.registers.a |= operand;
                self.registers.f = if self.registers.a == 0 {
                    Flag::Zero as u8
                } else {
                    0
                };
            }
            CP(operand) => {
                self.pc += 1;
                let operand = self.load_arithmetic_operand(memory, operand);
                let (cp, carry) = self.registers.a.overflowing_sub(operand);
                let mut f = Flag::Subtract as u8;
                if cp == 0 {
                    f |= Flag::Zero as u8;
                }
                // TODO: How is the HalfCarry flag set?
                if carry {
                    f |= Flag::Carry as u8;
                }
                self.registers.f = f;
            }
            LD(load_type) => match load_type {
                LoadType::Byte(to, from) => {
                    self.pc += 1;
                    let from = match from {
                        LoadByteSource::Register(reg) => {
                            self.registers.read8(reg)
                        }
                        LoadByteSource::D8 => {
                            let d8 = memory.read8(self.pc);
                            self.pc += 1;
                            d8
                        }
                        LoadByteSource::HLI => {
                            let hl = self.registers.read16(U16Register::HL);
                            memory.read8(hl)
                        }
                    };
                    match to {
                        LoadByteTarget::Register(reg) => {
                            self.registers.write8(reg, from);
                        }
                        LoadByteTarget::HLI => {
                            let hl = self.registers.read16(U16Register::HL);
                            memory.write8(hl, from);
                        }
                    }
                }
                LoadType::Word(to, from) => {
                    self.pc += 1;
                    let from = match from {
                        LoadWordSource::D16 => {
                            let d16 = memory.read16(self.pc);
                            self.pc += 2;
                            d16
                        }
                        LoadWordSource::SP => {
                            self.sp
                        }
                        LoadWordSource::HL => {
                            self.registers.read16(U16Register::HL)
                        }
                    };
                    match to {
                        LoadWordTarget::Register(reg) => {
                            self.registers.write16(reg, from);
                        }
                        LoadWordTarget::SP => {
                            self.sp = from;
                        }
                    }
                }
                LoadType::IndirectByteFromA(to) => {
                    self.pc += 1;
                    let from = self.registers.a;
                    use LoadIndirectByteOperand::*;
                    let address = match to {
                        Register(rr) => self.registers.read16(rr),
                        HLI_incrementing => {
                            let hl = self.registers.read16(U16Register::HL);
                            self.registers.write16(U16Register::HL, hl + 1);
                            hl
                        }
                        HLI_decrementing => {
                            let hl = self.registers.read16(U16Register::HL);
                            self.registers.write16(U16Register::HL, hl - 1);
                            hl
                        }
                        Address => {
                            let address = memory.read16(self.pc);
                            self.pc += 2;
                            address
                        }
                    };
                    memory.write8(address, from);
                }
                LoadType::IndirectByteToA(from) => {
                    self.pc += 1;
                    use LoadIndirectByteOperand::*;
                    let address = match from {
                        Register(rr) => self.registers.read16(rr),
                        HLI_incrementing => {
                            let hl = self.registers.read16(U16Register::HL);
                            self.registers.write16(U16Register::HL, hl + 1);
                            hl
                        }
                        HLI_decrementing => {
                            let hl = self.registers.read16(U16Register::HL);
                            self.registers.write16(U16Register::HL, hl - 1);
                            hl
                        }
                        Address => {
                            let address = memory.read16(self.pc);
                            self.pc += 2;
                            address
                        }
                    };
                    self.registers.a = memory.read8(address);
                }
            }
            LDH(load_type, load_direction) => {
                self.pc += 1;
                let address = match load_type {
                    LdhOperand::I8 => {
                        let d8 = memory.read8(self.pc);
                        self.pc += 1;
                        d8
                    }
                    LdhOperand::Ci => {
                        self.registers.c
                    }
                } as u16 + 0xFF00;
                match load_direction {
                    LdhDirection::FromA => {
                        memory.write8(address, self.registers.a);
                    }
                    LdhDirection::ToA => {
                        self.registers.a = memory.read8(address);
                    }
                }
            }
            RLA => {
                self.pc += 1;
                let carry = self.registers.a & 0x80 != 0;
                self.registers.a <<= 1;
                if self.registers.f & Flag::Carry as u8 != 0 {
                    self.registers.a += 1;
                };
                self.registers.f = if carry {
                    Flag::Carry as u8
                } else {
                    0
                };
            }
            RL(r) => {
                self.pc += 2;
                let mut v = self.load_non_direct_arithmetic_operand(memory, r);
                let carry = v & 0x80 != 0;
                v <<= 1;
                if self.registers.f & Flag::Carry as u8 != 0 {
                    v += 1;
                };
                self.write_non_direct_arithmetic_operand(memory, r, v);
                let mut f = 0;
                if v == 0 {
                    f |= Flag::Zero as u8;
                }
                if carry {
                    f |= Flag::Carry as u8;
                }
                self.registers.f = f;
            }
            RRA => {
                self.pc += 1;
                let carry = self.registers.a & 0x01 != 0;
                self.registers.a >>= 1;
                if self.registers.f & Flag::Carry as u8 != 0 {
                    self.registers.a += 0x80;
                };
                self.registers.f = if carry {
                    Flag::Carry as u8
                } else {
                    0
                };
            }
            RR(r) => {
                self.pc += 2;
                let mut v = self.load_non_direct_arithmetic_operand(memory, r);
                let carry = v & 0x01 != 0;
                v >>= 1;
                if self.registers.f & Flag::Carry as u8 != 0 {
                    v += 0x80;
                };
                self.write_non_direct_arithmetic_operand(memory, r, v);
                let mut f = 0;
                if v == 0 {
                    f |= Flag::Zero as u8;
                }
                if carry {
                    f |= Flag::Carry as u8;
                }
                self.registers.f = f;
            }
            BIT(bit, r) => {
                self.pc += 2;
                let v = self.load_non_direct_arithmetic_operand(memory, r);
                let set = v & (bit as u8) != 0;
                let mut f = self.registers.f;
                if set {
                    f &= !(Flag::Zero as u8);
                } else {
                    f |= Flag::Zero as u8;
                }
                let mask: u8 = Flag::Subtract as u8 | Flag::HalfCarry as u8;
                f = (f & !mask) | Flag::HalfCarry as u8;
                self.registers.f = f;
            }
            JP(condition) => {
                let nn = memory.read16(self.pc + 1);
                self.pc += 3;
                if self.test_jump_condition(condition) {
                    self.pc = nn;
                }
            }
            JR(condition) => {
                let e = memory.read8(self.pc + 1);
                // TODO: is e to be interpreted as 2s complement?
                let e = e as i8;
                self.pc += 2;
                if self.test_jump_condition(condition) {
                    self.pc = (self.pc as i16 + e as i16) as u16;
                }
            }
            CALL(condition) => {
                let nn = memory.read16(self.pc + 1);
                self.pc += 3;
                if self.test_jump_condition(condition) {
                    self.push(memory, self.pc);
                    self.pc = nn;
                }
            }
            RET(condition) => {
                if self.test_jump_condition(condition) {
                    let address = self.pop(memory);
                    self.pc = address;
                } else {
                    self.pc += 1;
                }
            }
            PUSH(register) => {
                self.pc += 1;
                self.push(memory, self.registers.read16(register));
            }
            POP(register) => {
                self.pc += 1;
                let value = self.pop(memory);
                // TODO: If we pop to AF, the lowest 4 bits should be set to 0
                self.registers.write16(register, value);
            }
            DI => {
                self.pc += 1;
                self.ime = false;
            }
        }
    }

    fn load_arithmetic_operand(&mut self, memory: &MemoryBus,
                               operand: ArithmeticOperand) -> u8 {
        match operand {
            ArithmeticOperand::Register(r) => self.registers.read8(r),
            ArithmeticOperand::HLI => {
                let hl = self.registers.read16(U16Register::HL);
                memory.read8(hl)
            }
            ArithmeticOperand::D8 => {
                let d8 = memory.read8(self.pc);
                self.pc += 1;
                d8
            }
        }
    }

    fn load_non_direct_arithmetic_operand(
            &self,
            memory: &MemoryBus,
            operand: NonDirectArithmeticOperand) -> u8 {
        match operand {
            NonDirectArithmeticOperand::Register(r) => self.registers.read8(r),
            NonDirectArithmeticOperand::HLI => {
                let hl = self.registers.read16(U16Register::HL);
                memory.read8(hl)
            }
        }
    }

    fn write_non_direct_arithmetic_operand(
            &mut self,
            memory: &mut MemoryBus,
            operand: NonDirectArithmeticOperand,
            value: u8) {
        match operand {
            NonDirectArithmeticOperand::Register(r) => {
                self.registers.write8(r, value);
            }
            NonDirectArithmeticOperand::HLI => {
                let hl = self.registers.read16(U16Register::HL);
                memory.write8(hl, value);
            }
        }
    }

    fn load_inc_dec_16_operand(&self, operand: IncDec16Operand) -> u16 {
        match operand {
            IncDec16Operand::Register(rr) => self.registers.read16(rr),
            IncDec16Operand::SP => self.sp,
        }
    }

    fn write_inc_dec_16_operand(&mut self, operand: IncDec16Operand,
                                value: u16) {
        match operand {
            IncDec16Operand::Register(rr) => self.registers.write16(rr, value),
            IncDec16Operand::SP => self.sp = value,
        }
    }

    fn test_jump_condition(&self, condition: JumpCondition) -> bool {
        use JumpCondition::*;
        match condition {
            Unconditional => true,
            NZ => self.registers.f & (Flag::Zero as u8) == 0,
            Z  => self.registers.f & (Flag::Zero as u8) != 0,
            NC => self.registers.f & (Flag::Carry as u8) == 0,
            C  => self.registers.f & (Flag::Carry as u8) != 0,
        }
    }

    fn push(&mut self, memory: &mut MemoryBus, value: u16) {
        self.sp -= 2;
        memory.write16(self.sp, value);
    }

    fn pop(&mut self, memory: &MemoryBus) -> u16 {
        let value = memory.read16(self.sp);
        self.sp += 2;
        value
    }

    fn print_stack(&self, memory: &MemoryBus) {
        eprintln!("Stack: SP = {:0>4X}", self.sp);
        let sp = self.sp;
        if sp == 0xFFFE {
            eprintln!("<empty>");
        } else {
            let mut p = 0xFFFE;
            while sp < p {
                p -= 2;
                eprintln!("{:0>4X}: {:0>4X}", p, memory.read16(p));
            }
        }
    }

    pub fn interrupts_are_enabled(&self) -> bool {
        self.ime
    }

    pub fn call_interrupt(&mut self, memory: &mut MemoryBus,
                          interrupt: InterruptAddress) {
        self.push(memory, self.pc);
        self.pc = interrupt as u16;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum U8Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum U16Register {
    AF,
    BC,
    DE,
    HL,
}

/// The CPU's registers
///
/// http://bgb.bircd.org/pandocs.htm#cpuregistersandflags
///
/// 16bit Hi   Lo   Name/Function
/// AF    A    -    Accumulator & Flags
/// BC    B    C    BC
/// DE    D    E    DE
/// HL    H    L    HL
pub struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl Registers {
    fn new() -> Self {
        Self{
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    pub fn read8(&self, register: U8Register) -> u8 {
        use U8Register::*;
        match register {
            A => self.a,
            F => self.f,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub fn write8(&mut self, register: U8Register, value: u8) {
        use U8Register::*;
        let register = match register {
            A => &mut self.a,
            F => &mut self.f,
            B => &mut self.b,
            C => &mut self.c,
            D => &mut self.d,
            E => &mut self.e,
            H => &mut self.h,
            L => &mut self.l,
        };
        *register = value;
    }

    pub fn read16(&self, register: U16Register) -> u16 {
        use U16Register::*;
        match register {
            AF => (self.a as u16) << 8 | self.f as u16,
            BC => (self.b as u16) << 8 | self.c as u16,
            DE => (self.d as u16) << 8 | self.e as u16,
            HL => (self.h as u16) << 8 | self.l as u16,
        }
    }

    pub fn write16(&mut self, register: U16Register, value: u16) {
        let high = ((value & 0xFF00) >> 8) as u8;
        let low = (value & 0xFF) as u8;
        use U16Register::*;
        match register {
            AF => {
                self.a = high;
                self.f = low;
            }
            BC => {
                self.b = high;
                self.c = low;
            }
            DE => {
                self.d = high;
                self.e = low;
            }
            HL => {
                self.h = high;
                self.l = low;
            }
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " A    F  B  C  D  E  H  L\n")?;
        write!(f, "{:0>2X} ", self.a)?;
        let zero = if self.f & Flag::Zero as u8 != 0 {'Z'} else {'z'};
        let sub = if self.f & Flag::Subtract as u8 != 0 {'N'} else {'n'};
        let half = if self.f & Flag::HalfCarry as u8 != 0 {'H'} else {'h'};
        let carry = if self.f & Flag::Carry as u8 != 0 {'C'} else {'c'};
        write!(f, "{}{}{}{} ", zero, sub, half, carry)?;
        write!(f, "{:0>2X} {:0>2X} {:0>2X} {:0>2X} {:0>2X} {:0>2X}",
               self.b,
               self.c,
               self.d,
               self.e,
               self.h,
               self.l,
               )?;
        Ok(())
    }
}

#[repr(u8)]
enum Flag {
    Zero = 1 << 7,
    Subtract = 1 << 6,
    HalfCarry = 1 << 5,
    Carry = 1 << 4,
}

#[derive(Copy, Clone, Debug)]
enum ArithmeticOperand {
    Register(U8Register),
    HLI,
    D8,
}

impl From<u8> for ArithmeticOperand {
    fn from(v: u8) -> Self {
        use ArithmeticOperand::*;
        use U8Register::*;
        match v {
            0b000 => Register(B),
            0b001 => Register(C),
            0b010 => Register(D),
            0b011 => Register(E),
            0b100 => Register(H),
            0b101 => Register(L),
            0b110 => HLI,
            0b111 => Register(A),
            _ => panic!("{:X} is not a valid ArithmeticOperand.", v),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum NonDirectArithmeticOperand {
    Register(U8Register),
    HLI,
}

impl From<u8> for NonDirectArithmeticOperand {
    fn from(v: u8) -> Self {
        use NonDirectArithmeticOperand::*;
        use U8Register::*;
        match v {
            0b000 => Register(B),
            0b001 => Register(C),
            0b010 => Register(D),
            0b011 => Register(E),
            0b100 => Register(H),
            0b101 => Register(L),
            0b110 => HLI,
            0b111 => Register(A),
            _ => panic!("{:X} is not a valid NonDirectArithmeticOperand.", v),
        }
    }
}

#[derive(Debug)]
enum LoadByteTarget {
    Register(U8Register),
    HLI,
}

impl From<u8> for LoadByteTarget {
    fn from(v: u8) -> Self {
        use LoadByteTarget::*;
        use U8Register::*;
        match v {
            0b000 => Register(B),
            0b001 => Register(C),
            0b010 => Register(D),
            0b011 => Register(E),
            0b100 => Register(H),
            0b101 => Register(L),
            0b110 => HLI,
            0b111 => Register(A),
            _ => panic!("{:X} is not a valid LoadByteTarget.", v),
        }
    }
}

#[derive(Debug)]
enum LoadWordTarget {
    Register(U16Register),
    SP,
}

impl From<u8> for LoadWordTarget {
    fn from(v: u8) -> Self {
        use LoadWordTarget::*;
        use U16Register::*;
        match v {
            0b00 => Register(BC),
            0b01 => Register(DE),
            0b10 => Register(HL),
            0b11 => SP,
            _ => panic!("{:X} is not a valid LoadWordTarget.", v),
        }
    }
}

#[derive(Debug)]
enum LoadByteSource {
    Register(U8Register),
    D8,
    HLI,
}

impl From<u8> for LoadByteSource {
    fn from(v: u8) -> Self {
        use LoadByteSource::*;
        use U8Register::*;
        match v {
            0b000 => Register(B),
            0b001 => Register(C),
            0b010 => Register(D),
            0b011 => Register(E),
            0b100 => Register(H),
            0b101 => Register(L),
            0b110 => HLI,
            0b111 => Register(A),
            _ => panic!("{:X} is not a valid LoadByteSource.", v),
        }
    }
}

#[derive(Debug)]
enum LoadWordSource {
    D16,
    SP,
    HL,
}

#[derive(Debug)]
enum LoadIndirectByteOperand {
    Register(U16Register),
    HLI_incrementing,
    HLI_decrementing,
    Address,
}

impl From<u8> for LoadIndirectByteOperand {
    fn from(v: u8) -> Self {
        use LoadIndirectByteOperand::*;
        use U16Register::*;
        match v {
            0b00 => Register(BC),
            0b01 => Register(DE),
            0b10 => HLI_incrementing,
            0b11 => HLI_decrementing,
            _ => panic!("{:X} is not a valid LoadIndirectByteOperand.", v),
        }
    }
}

#[derive(Debug)]
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
    IndirectByteFromA(LoadIndirectByteOperand),
    IndirectByteToA(LoadIndirectByteOperand),
}

#[derive(Debug)]
enum LdhOperand {
    I8,
    Ci,
}

#[derive(Debug)]
enum LdhDirection {
    ToA,
    FromA,
}

#[derive(Debug)]
enum IncDecType {
    IncDec8(NonDirectArithmeticOperand),
    IncDec16(IncDec16Operand),
}

#[derive(Copy, Clone, Debug)]
enum IncDec16Operand {
    Register(U16Register),
    SP,
}

impl From<u8> for IncDec16Operand {
    fn from(v: u8) -> Self {
        use IncDec16Operand::*;
        match v {
            0 => Register(U16Register::BC),
            1 => Register(U16Register::DE),
            2 => Register(U16Register::HL),
            3 => SP,
            _ => panic!("{:X} is not a valid IncDec16Operand.", v),
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
enum Bit {
    B0 = 1,
    B1 = 2,
    B2 = 4,
    B3 = 8,
    B4 = 16,
    B5 = 32,
    B6 = 64,
    B7 = 128,
}

impl From<u8> for Bit {
    fn from(v: u8) -> Self {
        use Bit::*;
        match v {
            0 => B0,
            1 => B1,
            2 => B2,
            3 => B3,
            4 => B4,
            5 => B5,
            6 => B6,
            7 => B7,
            _ => panic!("{:X} is not a valid Bit.", v),
        }
    }
}

#[derive(Debug)]
enum JumpCondition {
    Unconditional,
    NZ,
    Z,
    NC,
    C,
}

#[derive(Debug)]
enum Instruction {
    NOP,
    ADD(ArithmeticOperand),
    SUB(ArithmeticOperand),
    AND(ArithmeticOperand),
    XOR(ArithmeticOperand),
    OR(ArithmeticOperand),
    CP(ArithmeticOperand),
    INC(IncDecType),
    DEC(IncDecType),
    LD(LoadType),
    LDH(LdhOperand, LdhDirection),
    BIT(Bit, NonDirectArithmeticOperand),
    RLA,
    RL(NonDirectArithmeticOperand),
    RRA,
    RR(NonDirectArithmeticOperand),
    JP(JumpCondition),
    JR(JumpCondition),
    CALL(JumpCondition),
    RET(JumpCondition),
    PUSH(U16Register),
    POP(U16Register),
    DI,
}

impl Instruction {
    fn from_byte(instruction_byte: u8, prefixed: bool) -> Option<Self> {
        if prefixed {
            Self::from_byte_prefixed(instruction_byte)
        } else {
            Self::from_byte_nonprefixed(instruction_byte)
        }
    }

    fn from_byte_prefixed(instruction_byte: u8) -> Option<Self> {
        match instruction_byte {
            0x10..=0x17 => {
                let r = instruction_byte & 0b111;
                Some(Instruction::RL(r.into()))
            }
            0x18..=0x1F => {
                let r = instruction_byte & 0b111;
                Some(Instruction::RR(r.into()))
            }
            0x40..=0x7F => {
                let bit = (instruction_byte & 0b0011_1000) >> 3;
                let r = instruction_byte & 0b111;
                Some(Instruction::BIT(bit.into(), r.into()))
            }
            _ => None,
        }
    }

    fn from_byte_nonprefixed(instruction_byte: u8) -> Option<Self> {
        match instruction_byte {
            0x00 => Some(Instruction::NOP),
            0b00_000_001..=0b00_111_111
                    if instruction_byte & 0b111 == 0b110 => {
                let to = (instruction_byte & 0b111_000) >> 3;
                Some(Instruction::LD(LoadType::Byte(
                            to.into(), LoadByteSource::D8)))
            }
            0b00_000_001..=0b00_111_111
                    if instruction_byte & 0b1111 == 0b0001 => {
                let to = (instruction_byte & 0b11_0000) >> 4;
                Some(Instruction::LD(LoadType::Word(
                            to.into(), LoadWordSource::D16)))
            }
            0b0000_0011..=0b0011_0011
                    if instruction_byte & 0b1111 == 0b0011 => {
                let r = (instruction_byte & 0b11_0000) >> 4;
                Some(Instruction::INC(IncDecType::IncDec16(r.into())))
            }
            0b0000_1011..=0b0011_1011
                    if instruction_byte & 0b1111 == 0b1011 => {
                let r = (instruction_byte & 0b11_0000) >> 4;
                Some(Instruction::DEC(IncDecType::IncDec16(r.into())))
            }
            0b00_000_100..=0b00_111_100
                    if instruction_byte & 0b111 == 0b100 => {
                let r = (instruction_byte & 0b111_000) >> 3;
                Some(Instruction::INC(IncDecType::IncDec8(r.into())))
            }
            0b00_000_101..=0b00_111_101
                    if instruction_byte & 0b111 == 0b101 => {
                let r = (instruction_byte & 0b111_000) >> 3;
                Some(Instruction::DEC(IncDecType::IncDec8(r.into())))
            }
            0b01_000_001..=0b01_111_111 => {
                let from = instruction_byte & 0b111;
                let to = (instruction_byte & 0b111_000) >> 3;
                Some(Instruction::LD(LoadType::Byte(to.into(), from.into())))
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                let to = (instruction_byte & 0b110_000) >> 4;
                Some(Instruction::LD(LoadType::IndirectByteFromA(to.into())))
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                let to = (instruction_byte & 0b110_000) >> 4;
                Some(Instruction::LD(LoadType::IndirectByteToA(to.into())))
            }
            0x17 => {
                Some(Instruction::RLA)
            }
            0x1F => {
                Some(Instruction::RRA)
            }
            0x18 => {
                Some(Instruction::JR(JumpCondition::Unconditional))
            }
            0x20 => {
                Some(Instruction::JR(JumpCondition::NZ))
            }
            0x28 => {
                Some(Instruction::JR(JumpCondition::Z))
            }
            0x30 => {
                Some(Instruction::JR(JumpCondition::NC))
            }
            0x38 => {
                Some(Instruction::JR(JumpCondition::C))
            }
            0x80..=0x87 => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::ADD(operand.into()))
            }
            0x90..=0x97 => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::SUB(operand.into()))
            }
            0xA0..=0xA7 => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::AND(operand.into()))
            }
            0xA8..=0xAF => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::XOR(operand.into()))
            }
            0xB0..=0xB7 => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::OR(operand.into()))
            }
            0xB8..=0xBF => {
                let operand = instruction_byte & 0b111;
                Some(Instruction::CP(operand.into()))
            }
            0xC1 => {
                Some(Instruction::POP(U16Register::BC))
            }
            0xD1 => {
                Some(Instruction::POP(U16Register::DE))
            }
            0xE1 => {
                Some(Instruction::POP(U16Register::HL))
            }
            0xF1 => {
                Some(Instruction::POP(U16Register::AF))
            }
            0xC5 => {
                Some(Instruction::PUSH(U16Register::BC))
            }
            0xD5 => {
                Some(Instruction::PUSH(U16Register::DE))
            }
            0xE5 => {
                Some(Instruction::PUSH(U16Register::HL))
            }
            0xF5 => {
                Some(Instruction::PUSH(U16Register::AF))
            }
            0xC2 => {
                Some(Instruction::JP(JumpCondition::NZ))
            }
            0xC3 => {
                Some(Instruction::JP(JumpCondition::Unconditional))
            }
            0xCA => {
                Some(Instruction::JP(JumpCondition::Z))
            }
            0xD2 => {
                Some(Instruction::JP(JumpCondition::NC))
            }
            0xDA => {
                Some(Instruction::JP(JumpCondition::C))
            }
            0xCD => {
                Some(Instruction::CALL(JumpCondition::Unconditional))
            }
            0xC4 => {
                Some(Instruction::CALL(JumpCondition::NZ))
            }
            0xCC => {
                Some(Instruction::CALL(JumpCondition::Z))
            }
            0xD4 => {
                Some(Instruction::CALL(JumpCondition::NC))
            }
            0xDC => {
                Some(Instruction::CALL(JumpCondition::C))
            }
            0xC9 => {
                Some(Instruction::RET(JumpCondition::Unconditional))
            }
            0xC0 => {
                Some(Instruction::RET(JumpCondition::NZ))
            }
            0xC8 => {
                Some(Instruction::RET(JumpCondition::Z))
            }
            0xD0 => {
                Some(Instruction::RET(JumpCondition::NC))
            }
            0xD8 => {
                Some(Instruction::RET(JumpCondition::C))
            }
            0xE0 => {
                Some(Instruction::LDH(LdhOperand::I8, LdhDirection::FromA))
            }
            0xF0 => {
                Some(Instruction::LDH(LdhOperand::I8, LdhDirection::ToA))
            }
            0xE2 => {
                Some(Instruction::LDH(LdhOperand::Ci, LdhDirection::FromA))
            }
            0xF2 => {
                Some(Instruction::LDH(LdhOperand::Ci, LdhDirection::ToA))
            }
            0xF3 => {
                Some(Instruction::DI)
            }
            0xC6 => {
                Some(Instruction::ADD(ArithmeticOperand::D8))
            }
            0xCE => unimplemented!("ADC n"),
            0xD6 => {
                Some(Instruction::SUB(ArithmeticOperand::D8))
            }
            0xDE => unimplemented!("SBC n"),
            0xE6 => {
                Some(Instruction::AND(ArithmeticOperand::D8))
            }
            0xEE => {
                Some(Instruction::XOR(ArithmeticOperand::D8))
            }
            0xF6 => {
                Some(Instruction::OR(ArithmeticOperand::D8))
            }
            0xFE => {
                Some(Instruction::CP(ArithmeticOperand::D8))
            }
            0xEA => {
                Some(Instruction::LD(LoadType::IndirectByteFromA(
                            LoadIndirectByteOperand::Address)))
            }
            0xFA => {
                Some(Instruction::LD(LoadType::IndirectByteToA(
                            LoadIndirectByteOperand::Address)))
            }
            _ => None,
        }
    }

    fn len(&self) -> u16 {
        use Instruction::*;
        match self {
            NOP => 1,
            ADD(arithmetic_operand)
                | SUB(arithmetic_operand)
                | AND(arithmetic_operand)
                | XOR(arithmetic_operand)
                | OR (arithmetic_operand)
                | CP(arithmetic_operand) => match arithmetic_operand {
                ArithmeticOperand::D8 => 2,
                _ => 1,
            }
            INC(_inc_type) => 1,
            DEC(_dec_type) => 1,
            LD(load_type) => {
                match load_type {
                    LoadType::Byte(_target, source) => match source {
                        LoadByteSource::D8 => 2,
                        _ => 1,
                    }
                    LoadType::Word(_target, source) => match source {
                        LoadWordSource::D16 => 3,
                        _ => 1,
                    }
                    LoadType::IndirectByteFromA(target) => match target {
                        LoadIndirectByteOperand::Address => 3,
                        _ => 1,
                    }
                    LoadType::IndirectByteToA(source) => match source {
                        LoadIndirectByteOperand::Address => 3,
                        _ => 1,
                    }
                }
            }
            LDH(operand, _direction) => {
                match operand {
                    LdhOperand::I8 => 2,
                    LdhOperand::Ci => 1,
                }
            }
            BIT(_bit, _operand) => 2,
            RLA => 1,
            RL(_operand) => 2,
            RRA => 1,
            RR(_operand) => 2,
            JP(_condition) => 3,
            JR(_condition) => 2,
            CALL(_condition) => 3,
            RET(_condition) => 1,
            PUSH(_u16_register) => 1,
            POP(_u16_register) => 1,
            DI => 1,
        }
    }
}
