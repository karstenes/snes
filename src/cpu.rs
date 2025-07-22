use crate::memory::{self, read_byte, read_word, write_byte, write_word};

use super::Console;
use color_eyre::{eyre::bail, eyre::Ok, Result};
use log::trace;

const OLD_OPCODES: [OpCode; 65] = [
    OpCode::LDA,
    OpCode::LDX,
    OpCode::LDY,
    OpCode::STA,
    OpCode::STX,
    OpCode::STY,
    OpCode::STZ,
    OpCode::PHA,
    OpCode::PHX,
    OpCode::PHY,
    OpCode::PHP,
    OpCode::PLA,
    OpCode::PLX,
    OpCode::PLY,
    OpCode::PLP,
    OpCode::TSX,
    OpCode::TXS,
    OpCode::INX,
    OpCode::INY,
    OpCode::DEX,
    OpCode::DEY,
    OpCode::INC,
    OpCode::DEC,
    OpCode::ASL,
    OpCode::LSR,
    OpCode::ROL,
    OpCode::ROR,
    OpCode::AND,
    OpCode::ORA,
    OpCode::EOR,
    OpCode::BIT,
    OpCode::CMP,
    OpCode::CPX,
    OpCode::CPY,
    OpCode::TRB,
    OpCode::TSB,
    OpCode::ADC,
    OpCode::SBC,
    OpCode::JMP,
    OpCode::JSR,
    OpCode::RTS,
    OpCode::RTI,
    OpCode::BRA,
    OpCode::BEQ,
    OpCode::BNE,
    OpCode::BCC,
    OpCode::BCS,
    OpCode::BVC,
    OpCode::BVS,
    OpCode::BMI,
    OpCode::BPL,
    OpCode::CLC,
    OpCode::CLD,
    OpCode::CLI,
    OpCode::CLV,
    OpCode::SEC,
    OpCode::SED,
    OpCode::SEI,
    OpCode::TAX,
    OpCode::TAY,
    OpCode::TXA,
    OpCode::TYA,
    OpCode::NOP,
    OpCode::BRK,
    OpCode::PEI,
];

#[derive(Debug, PartialEq, Default, Clone)]
pub enum OpCode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRA,
    BRK,
    BRL,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    COP,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JML,
    JSR,
    JSL,
    LDA,
    LDX,
    LDY,
    LSR,
    MVN,
    MVP,
    #[default]
    NOP,
    ORA,
    PEA,
    PEI,
    PER,
    PHA,
    PHB,
    PHD,
    PHK,
    PHP,
    PHX,
    PHY,
    PLA,
    PLB,
    PLD,
    PLP,
    PLX,
    PLY,
    REP,
    ROL,
    ROR,
    RTI,
    RTS,
    RTL,
    SBC,
    SEC,
    SED,
    SEI,
    SEP,
    STA,
    STX,
    STY,
    STP,
    STZ,
    TAX,
    TAY,
    TCD,
    TCS,
    TDC,
    TSC,
    TSX,
    TXA,
    TXS,
    TXY,
    TYA,
    TYX,
    TRB,
    TSB,
    WAI,
    WDM,
    XBA,
    XCE,
}

impl OpCode {
    pub fn is_branch(&self) -> bool {
        matches!(
            self,
            OpCode::BRA
                | OpCode::BCC
                | OpCode::BCS
                | OpCode::BEQ
                | OpCode::BMI
                | OpCode::BNE
                | OpCode::BPL
                | OpCode::BVC
                | OpCode::BVS
        )
    }
    pub fn is_jump(&self) -> bool {
        matches!(self, OpCode::JMP | OpCode::JSR | OpCode::JML | OpCode::JSL)
    }
    pub fn is_interrupt(&self) -> bool {
        matches!(self, OpCode::BRK | OpCode::COP)
    }
    pub fn is_subroutine(&self) -> bool {
        matches!(self, OpCode::JML | OpCode::JSR)
    }
    pub fn is_return(&self) -> bool {
        matches!(self, OpCode::RTS | OpCode::RTI | OpCode::RTL)
    }
    pub fn is_old(&self) -> bool {
        OLD_OPCODES.contains(self)
    }
    pub fn changes_x(&self) -> bool {
        matches!(self, OpCode::PLP | OpCode::RTI | OpCode::REP | OpCode::SEP)
    }
    pub fn changes_m(&self) -> bool {
        matches!(self, OpCode::PLP | OpCode::RTI | OpCode::REP | OpCode::SEP)
    }
    fn uses_x(&self) -> bool {
        matches!(
            self,
            OpCode::CPX
                | OpCode::CPY
                | OpCode::DEX
                | OpCode::DEY
                | OpCode::INX
                | OpCode::INY
                | OpCode::LDX
                | OpCode::LDY
                | OpCode::STX
                | OpCode::STY
                | OpCode::PHX
                | OpCode::PHY
                | OpCode::PLX
                | OpCode::PLY
                | OpCode::TAX
                | OpCode::TAY
                | OpCode::TSX
                | OpCode::TXS
                | OpCode::TXY
                | OpCode::TYX
        )
    }
    fn uses_m(&self) -> bool {
        matches!(
            self,
            OpCode::ADC
                | OpCode::SBC
                | OpCode::AND
                | OpCode::EOR
                | OpCode::ORA
                | OpCode::ASL
                | OpCode::LSR
                | OpCode::ROL
                | OpCode::ROR
                | OpCode::BIT
                | OpCode::CMP
                | OpCode::DEC
                | OpCode::INC
                | OpCode::LDA
                | OpCode::STA
                | OpCode::STZ
                | OpCode::PHA
                | OpCode::PLA
                | OpCode::TRB
                | OpCode::TSB
                | OpCode::TXA
                | OpCode::TYA
        )
    }
}

impl std::fmt::Display for OpCode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let instr = match self {
            OpCode::ADC => "ADC",
            OpCode::AND => "AND",
            OpCode::ASL => "ASL",
            OpCode::BCC => "BCC",
            OpCode::BCS => "BCS",
            OpCode::BEQ => "BEQ",
            OpCode::BIT => "BIT",
            OpCode::BMI => "BMI",
            OpCode::BNE => "BNE",
            OpCode::BPL => "BPL",
            OpCode::BRA => "BRA",
            OpCode::BRK => "BRK",
            OpCode::BRL => "BRL",
            OpCode::BVC => "BVC",
            OpCode::BVS => "BVS",
            OpCode::CLC => "CLC",
            OpCode::CLD => "CLD",
            OpCode::CLI => "CLI",
            OpCode::CLV => "CLV",
            OpCode::CMP => "CMP",
            OpCode::CPX => "CPX",
            OpCode::CPY => "CPY",
            OpCode::COP => "COP",
            OpCode::DEC => "DEC",
            OpCode::DEX => "DEX",
            OpCode::DEY => "DEY",
            OpCode::EOR => "EOR",
            OpCode::INC => "INC",
            OpCode::INX => "INX",
            OpCode::INY => "INY",
            OpCode::JMP => "JMP",
            OpCode::JML => "JML",
            OpCode::JSR => "JSR",
            OpCode::JSL => "JSL",
            OpCode::LDA => "LDA",
            OpCode::LDX => "LDX",
            OpCode::LDY => "LDY",
            OpCode::LSR => "LSR",
            OpCode::MVN => "MVN",
            OpCode::MVP => "MVP",
            OpCode::NOP => "NOP",
            OpCode::ORA => "ORA",
            OpCode::PEA => "PEA",
            OpCode::PEI => "PEI",
            OpCode::PER => "PER",
            OpCode::PHA => "PHA",
            OpCode::PHB => "PHB",
            OpCode::PHD => "PHD",
            OpCode::PHK => "PHK",
            OpCode::PHP => "PHP",
            OpCode::PHX => "PHX",
            OpCode::PHY => "PHY",
            OpCode::PLA => "PLA",
            OpCode::PLB => "PLB",
            OpCode::PLD => "PLD",
            OpCode::PLP => "PLP",
            OpCode::PLX => "PLX",
            OpCode::PLY => "PLY",
            OpCode::REP => "REP",
            OpCode::ROL => "ROL",
            OpCode::ROR => "ROR",
            OpCode::RTI => "RTI",
            OpCode::RTS => "RTS",
            OpCode::RTL => "RTL",
            OpCode::SBC => "SBC",
            OpCode::SEC => "SEC",
            OpCode::SED => "SED",
            OpCode::SEI => "SEI",
            OpCode::SEP => "SEP",
            OpCode::STA => "STA",
            OpCode::STX => "STX",
            OpCode::STY => "STY",
            OpCode::STP => "STP",
            OpCode::STZ => "STZ",
            OpCode::TAX => "TAX",
            OpCode::TAY => "TAY",
            OpCode::TCD => "TCD",
            OpCode::TCS => "TCS",
            OpCode::TDC => "TDC",
            OpCode::TSC => "TSC",
            OpCode::TSX => "TSX",
            OpCode::TXA => "TXA",
            OpCode::TXS => "TXS",
            OpCode::TXY => "TXY",
            OpCode::TYA => "TYA",
            OpCode::TYX => "TYX",
            OpCode::TRB => "TRB",
            OpCode::TSB => "TSB",
            OpCode::WAI => "WAI",
            OpCode::WDM => "WDM",
            OpCode::XBA => "XBA",
            OpCode::XCE => "XCE",
        };
        write!(f, "{}", instr)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum AddrMode {
    Absolute,
    AbsoluteWord,
    AbsoluteSWord,
    /// Absolute,X
    AbsoluteX,
    /// Absolute,Y
    AbsoluteY,
    /// (Absolute)
    AbsoluteIndirectWord,
    /// [Absolute]
    AbsoluteIndirectSWord,
    /// (Absolute,X)
    AbsoluteIndexedIndirect,
    Accumulator,
    Direct,
    /// Direct,X
    DirectX,
    /// Direct,Y
    DirectY,
    /// (Direct)
    DirectWord,
    /// [Direct]
    DirectSWord,
    /// (Direct,X)
    IndexedDirectWord,
    /// (Direct), Y
    DirectIndexedWord,
    /// [Direct], Y
    DirectIndexedSWord,
    Immediate,
    #[default]
    Implied,
    Long,
    /// Long,X
    LongX,
    RelativeByte,
    RelativeWord,
    SourceDestination,
    // (Stack,S)
    Stack,
    /// (Stack,S),Y
    StackIndexed,
}

impl std::fmt::Display for AddrMode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            AddrMode::Absolute => "Absolute",
            AddrMode::AbsoluteWord => "(Absolute)",
            AddrMode::AbsoluteSWord => "[Absolute]",
            AddrMode::AbsoluteX => "Absolute, X",
            AddrMode::AbsoluteY => "Absolute, Y",
            AddrMode::AbsoluteIndirectWord => "(Absolute, X)",
            AddrMode::AbsoluteIndirectSWord => todo!(),
            AddrMode::AbsoluteIndexedIndirect => todo!(),
            AddrMode::Accumulator => "Accumulator",
            AddrMode::Direct => "Direct",
            AddrMode::DirectX => "Direct, X",
            AddrMode::DirectY => "Direct, Y",
            AddrMode::DirectWord => "(Direct)",
            AddrMode::DirectSWord => "[Direct]",
            AddrMode::IndexedDirectWord => "(Direct, X)",
            AddrMode::DirectIndexedWord => "(Direct), Y",
            AddrMode::DirectIndexedSWord => "[Direct], Y",
            AddrMode::Immediate => "Immediate",
            AddrMode::Implied => "Implied",
            AddrMode::Long => "Long",
            AddrMode::LongX => "Long, X",
            AddrMode::RelativeByte => "Relative8",
            AddrMode::RelativeWord => "Relative16",
            AddrMode::SourceDestination => "src, dst",
            AddrMode::Stack => "(Stack, S)",
            AddrMode::StackIndexed => "(Stack, S), Y",
        };
        write!(f, "{}", mode)
    }
}

#[derive(Debug, Clone)]
pub struct Flags {
    /// Negative
    pub n: bool,
    /// Overflow
    pub v: bool,
    /// Memory width
    pub m: bool,
    /// Index register width
    pub x: bool,
    /// Decimal mode
    pub d: bool,
    /// Interrupt disable
    pub i: bool,
    /// Zero
    pub z: bool,
    /// Carry
    pub c: bool,
    /// Emulation mode
    pub e: bool,
    /// Break
    pub b: bool,
}

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
/// The 65C816 CPU
pub struct CPU {
    /// Accumulator (16 bit)
    pub A: u16,
    /// X Register (16 bit)
    pub X: u16,
    /// Y Register (16 bit)
    pub Y: u16,
    /// Stack Pointer (16 bit)
    pub S: u16,
    /// Databank Register (16 bit)
    pub DBR: u8,
    /// Direct Addressing Register (16 bit)
    pub D: u16,
    /// Program Bank Register (8 bit)
    pub K: u8,
    /// Flags Register
    pub P: Flags,
    /// Program Counter (16 bit)
    pub PC: u16,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            n: false,
            v: false,
            m: false,
            x: false,
            d: false,
            i: false,
            z: false,
            c: false,
            e: true,
            b: false,
        }
    }
}

impl std::fmt::Display for Flags {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flagstring = String::default();
        flagstring.push(if self.n { 'N' } else { 'n' });
        flagstring.push(if self.v { 'V' } else { 'v' });
        flagstring.push(if self.m { 'M' } else { 'm' });
        flagstring.push(if self.x { 'X' } else { 'x' });
        flagstring.push(if self.d { 'D' } else { 'd' });
        flagstring.push(if self.i { 'I' } else { 'i' });
        flagstring.push(if self.z { 'Z' } else { 'z' });
        flagstring.push(if self.c { 'C' } else { 'c' });
        flagstring.push(if self.e { 'E' } else { 'e' });
        flagstring.push(if self.b { 'B' } else { 'b' });
        write!(f, "{}", flagstring)
    }
}

impl CPU {
    /// Init CPU to 0
    pub fn new() -> CPU {
        CPU {
            A: 0,
            X: 0,
            Y: 0,
            S: 0,
            DBR: 0,
            D: 0,
            K: 0,
            P: Flags::new(),
            PC: 0,
        }
    }
    pub fn p_byte(&self) -> u8 {
        let mut p = 0;
        p |= (self.P.n as u8) << 7;
        p |= (self.P.v as u8) << 6;
        p |= (self.P.m as u8) << 5;
        p |= ((self.P.x && !self.P.e) as u8) << 4;
        p |= (self.P.d as u8) << 3;
        p |= (self.P.i as u8) << 2;
        p |= (self.P.z as u8) << 1;
        p |= self.P.c as u8;
        p
    }
    pub fn set_p(&mut self, p: u8) {
        self.P.c = (p & 0b00000001) != 0;
        self.P.z = (p & 0b00000010) != 0;
        self.P.i = (p & 0b00000100) != 0;
        self.P.d = (p & 0b00001000) != 0;
        self.P.x = (p & 0b00010000) != 0;
        self.P.m = (p & 0b00100000) != 0;
        self.P.v = (p & 0b01000000) != 0;
        self.P.n = (p & 0b10000000) != 0;
        if self.P.e {
            self.P.x = true;
        }
    }
    pub fn get_pc(&self) -> u32 {
        self.PC as u32 | (self.K as u32) << 16
    }
    pub fn set_pc(&mut self, addr: u32) {
        self.PC = (addr & 0xFFFF) as u16;
        self.K = addr.to_be_bytes()[1];
    }
}

impl std::fmt::Display for CPU {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A:   {:04X}\nX:   {:04X}\nY:   {:04X}\nS:   {:04X}\nDBR: {:04X}\nD:   {:04X}\nK:   {:02X}\nP:   {}\nPC:  ${:04X}",
            self.A,
            self.X,
            self.Y,
            self.S,
            self.DBR,
            self.D,
            self.K,
            self.P,
            self.PC
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct InstructionContext {
    pub opcode: OpCode,
    pub mode: AddrMode,
    pub inst_addr: u32,
    pub data_addr: u32,
    pub dest_addr: Option<u32>,
}

impl InstructionContext {
    pub fn with_source<'a>(&'a self, snes: &'a Console) -> InstructionContextWrapper<'a> {
        InstructionContextWrapper {
            context: self,
            snes: snes,
        }
    }
    pub fn length(&self, m: bool, x: bool) -> usize {
        match self.opcode {
            OpCode::REP | OpCode::SEP => return 2,
            OpCode::PEA | OpCode::PER => return 3,
            _ => {}
        }
        match self.mode {
            AddrMode::Absolute => 3,
            AddrMode::AbsoluteWord => 3,
            AddrMode::AbsoluteSWord => 3,
            AddrMode::AbsoluteX | AddrMode::AbsoluteY => 3,
            AddrMode::AbsoluteIndirectWord => 3,
            AddrMode::AbsoluteIndirectSWord => todo!(),
            AddrMode::AbsoluteIndexedIndirect => todo!(),
            AddrMode::Accumulator => 1,
            AddrMode::Direct => 2,
            AddrMode::DirectX | AddrMode::DirectY => 2,
            AddrMode::DirectWord => 3,
            AddrMode::DirectSWord => 2,
            AddrMode::IndexedDirectWord => 2,
            AddrMode::DirectIndexedWord => 2,
            AddrMode::DirectIndexedSWord => 2,
            AddrMode::Immediate => {
                if self.opcode.uses_m() {
                    3 - (m as usize)
                } else if self.opcode.uses_x() {
                    3 - (x as usize)
                } else {
                    3
                }
            }
            AddrMode::Implied => 1,
            AddrMode::Long => 4,
            AddrMode::LongX => 4,
            AddrMode::RelativeByte => 2,
            AddrMode::RelativeWord => 3,
            AddrMode::SourceDestination => 3,
            AddrMode::Stack => 2,
            AddrMode::StackIndexed => 2,
        }
    }
}

pub struct InstructionContextWrapper<'a> {
    pub context: &'a InstructionContext,
    pub snes: &'a Console,
}

impl<'a> std::fmt::Display for InstructionContextWrapper<'a> {
    // #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.context.mode {
            AddrMode::SourceDestination => {
                write!(
                    f,
                    "${:06X}: {} {:06X}, {:06X}",
                    self.context.inst_addr,
                    self.context.opcode,
                    self.context.data_addr,
                    self.context.dest_addr.unwrap()
                )
            }
            AddrMode::Accumulator | AddrMode::Implied => {
                write!(
                    f,
                    "${:06X}: {}",
                    self.context.inst_addr, self.context.opcode
                )
            }
            AddrMode::Immediate => match &self.context.opcode {
                OpCode::REP | OpCode::SEP | OpCode::WDM => {
                    let data = memory::peek_byte(self.snes, self.context.data_addr).unwrap();
                    write!(
                        f,
                        "${:06X}: {} #{:02X}",
                        self.context.inst_addr, self.context.opcode, data
                    )
                }
                OpCode::PEA | OpCode::PER => {
                    let data = memory::peek_word(self.snes, self.context.data_addr).unwrap();
                    write!(
                        f,
                        "${:06X}: {} #{:04X}",
                        self.context.inst_addr, self.context.opcode, data
                    )
                }
                _ => {
                    if self.snes.cpu.P.m {
                        let data = memory::peek_byte(self.snes, self.context.data_addr).unwrap();
                        write!(
                            f,
                            "${:06X}: {} #{:02X}",
                            self.context.inst_addr, self.context.opcode, data
                        )
                    } else {
                        let data = memory::peek_word(self.snes, self.context.data_addr).unwrap();
                        write!(
                            f,
                            "${:06X}: {} #{:04X}",
                            self.context.inst_addr, self.context.opcode, data
                        )
                    }
                }
            },
            _ => {
                write!(
                    f,
                    "${:06X}: {} ${:06X}",
                    self.context.inst_addr, self.context.opcode, self.context.data_addr,
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CPUExecutionResult {
    Normal,
    BranchTaken,
    Jump,
    Subroutine(u32),
    Return,
    Interrupt,
}

pub fn decode_addressing_mode(opcode: u8) -> Result<AddrMode> {
    let bbb = (opcode & 0b00011100) >> 2;
    let cc = opcode & 0b00000011;

    match opcode {
        0x00 | 0x08 | 0x0B | 0x18 | 0x1B | 0x28 | 0x2B | 0x38 | 0x3B | 0x40 | 0x48 | 0x4B
        | 0x58 | 0x5A | 0x5B | 0x60 | 0x68 | 0x6B | 0x78 | 0x7A | 0x7B | 0x88 | 0x8A | 0x8B
        | 0x98 | 0x9A | 0x9B | 0xA8 | 0xAA | 0xAB | 0xB8 | 0xBA | 0xBB | 0xC8 | 0xCA | 0xCB
        | 0xD8 | 0xDA | 0xDB | 0xE8 | 0xEA | 0xEB | 0xF8 | 0xFA | 0xFB => {
            return Ok(AddrMode::Implied)
        } // Single byte instructions
        0x0A | 0x1A | 0x2A | 0x3A | 0x4A | 0x6A => return Ok(AddrMode::Accumulator), // ASL A, INC A, ROL A, DEC A, LSR A, ROR A
        0x14 | 0x64 | 0xD4 => return Ok(AddrMode::Direct), // TRB zp, STZ zp, PEI dir
        0x1C | 0x20 | 0x9C => return Ok(AddrMode::Absolute), // TRB abs, JSR abs, STZ abs
        0x22 | 0x5C => return Ok(AddrMode::Long),          // JMP long,
        0x44 | 0x52 => return Ok(AddrMode::SourceDestination), // MVN src,dest, MVP src,dest
        0xDC => return Ok(AddrMode::AbsoluteSWord),
        0x6C => return Ok(AddrMode::AbsoluteIndirectWord), // JMP (abs)
        0x74 => return Ok(AddrMode::DirectX),              // STZ zp,X
        0x7C | 0xFC => return Ok(AddrMode::AbsoluteIndexedIndirect), // JMP (abs,X), JSR (abs,X)
        0x10 | 0x30 | 0x50 | 0x70 | 0x80 | 0x90 | 0xB0 | 0xD0 | 0xF0 => {
            return Ok(AddrMode::RelativeByte)
        } // BRA rel8
        0x82 => return Ok(AddrMode::RelativeWord),         // BRL rel16
        0x02 | 0x42 | 0x62 | 0x89 | 0xC2 | 0xE2 | 0xF4 => return Ok(AddrMode::Immediate), // COP immed, PER immed, BIT immed, REP immed, SEP immed, PEA immed
        0x9E => return Ok(AddrMode::AbsoluteX), // STZ abs,X
        _ => (),
    }

    let mode: AddrMode = match cc {
        0b00 => match bbb {
            0b000 => AddrMode::Immediate,
            0b001 => AddrMode::Direct,
            0b010 => bail!("Unknown opcode {:02X}", opcode),
            0b011 => AddrMode::Absolute,
            0b100 => bail!("Unknown opcode {:02X}", opcode),
            0b101 => AddrMode::DirectX,
            0b110 => bail!("Unknown opcode {:02X}", opcode),
            0b111 => AddrMode::AbsoluteX,
            _ => unreachable!(),
        },
        0b01 => match bbb {
            0b000 => AddrMode::IndexedDirectWord,
            0b001 => AddrMode::Direct,
            0b010 => AddrMode::Immediate,
            0b011 => AddrMode::Absolute,
            0b100 => AddrMode::DirectIndexedWord,
            0b101 => AddrMode::DirectX,
            0b110 => AddrMode::AbsoluteY,
            0b111 => AddrMode::AbsoluteX,
            _ => unreachable!(),
        },
        0b10 => match bbb {
            0b000 => AddrMode::Immediate,
            0b001 => AddrMode::Direct,
            0b010 => AddrMode::Accumulator,
            0b011 => AddrMode::Absolute,
            0b100 => AddrMode::DirectWord,
            0b101 => AddrMode::DirectX,
            0b110 => bail!("Unknown opcode {:02X}", opcode),
            0b111 => AddrMode::AbsoluteX,
            _ => unreachable!(),
        },
        0b11 => match bbb {
            0b000 => AddrMode::Stack,
            0b001 => AddrMode::DirectSWord,
            0b010 => bail!("Unknown opcode {:02X}", opcode),
            0b011 => AddrMode::Long,
            0b100 => AddrMode::StackIndexed,
            0b101 => AddrMode::DirectIndexedSWord,
            0b110 => bail!("Unknown opcode {:02X}", opcode),
            0b111 => AddrMode::LongX,
            _ => unreachable!(),
        },
        _ => {
            unreachable!()
        }
    };
    Ok(mode)
}

fn calculate_cycles(snes: &Console, instruction: InstructionContext) -> Result<u8> {
    let w = ((snes.cpu.D & 0xFF) > 1) as u8;
    let p = ((instruction.data_addr & 0xFF0000) >> 16) as u8 != snes.cpu.K;
    let m = snes.cpu.P.m as u8;
    let cycles = match instruction.opcode {
        OpCode::JMP | OpCode::JML => match instruction.mode {
            AddrMode::Absolute => 3,
            AddrMode::Long => 4,
            AddrMode::AbsoluteIndirectWord => 5,
            AddrMode::AbsoluteIndexedIndirect | AddrMode::AbsoluteIndirectSWord => 6,
            _ => unreachable!(),
        },
        OpCode::JSL | OpCode::JSR => match instruction.mode {
            AddrMode::Long => 8,
            AddrMode::Absolute => 6,
            AddrMode::AbsoluteIndexedIndirect => 8,
            _ => unreachable!(),
        },
        OpCode::RTL | OpCode::RTS => 6,
        OpCode::RTI => 7 - snes.cpu.P.e as u8,
        OpCode::MVN | OpCode::MVP => 7,
        OpCode::NOP | OpCode::WDM => 2,
        OpCode::PEA => 5,
        OpCode::PER => 6,
        OpCode::PEI => 6 + w,
        OpCode::ADC | OpCode::SBC => match instruction.mode {
            AddrMode::IndexedDirectWord => 7 - m + w,
            AddrMode::Stack => 5 - m,
            AddrMode::Direct => 4 - m + w,
            AddrMode::DirectSWord => 7 - m + w,
            AddrMode::Immediate => 3 - m,
            AddrMode::Absolute => 5 - m,
            AddrMode::Long => 6 - m,
            AddrMode::DirectIndexedWord => {
                7 - m + w - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::DirectWord => 6 - m + w,
            AddrMode::StackIndexed => 8 - m,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::DirectIndexedSWord => 7 - m + w,
            AddrMode::AbsoluteY | AddrMode::AbsoluteX => {
                6 - m - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            _ => unreachable!(),
        },
        OpCode::CMP => match instruction.mode {
            AddrMode::IndexedDirectWord => 7 - m + w,
            AddrMode::Stack => 5 - m,
            AddrMode::Direct => 4 - m + w,
            AddrMode::DirectSWord => 7 - m + w,
            AddrMode::Immediate => 3 - m,
            AddrMode::Absolute => 5 - m,
            AddrMode::DirectIndexedWord => {
                7 - m + w - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::DirectWord => 6 - m + w,
            AddrMode::StackIndexed => 8 - m,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::DirectIndexedSWord => 7 - m + w,
            AddrMode::AbsoluteY | AddrMode::AbsoluteX => {
                6 - m - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::LongX => 6 - m,
            _ => unreachable!(),
        },
        OpCode::CPX | OpCode::CPY => match instruction.mode {
            AddrMode::Immediate => 3 - snes.cpu.P.x as u8,
            AddrMode::Direct => 4 - snes.cpu.P.x as u8 + w,
            AddrMode::Absolute => 5 - snes.cpu.P.x as u8,
            _ => unreachable!(),
        },
        OpCode::DEC | OpCode::DEX | OpCode::DEY | OpCode::INC | OpCode::INX | OpCode::INY => {
            match instruction.mode {
                AddrMode::Accumulator => 2,
                AddrMode::Direct => 7 - (2 * m) + w,
                AddrMode::Absolute => 8 - (2 * m),
                AddrMode::DirectX => 8 - (2 * m) + w,
                AddrMode::AbsoluteX => 9 - (2 * m),
                AddrMode::Implied => 2,
                _ => unreachable!(),
            }
        }
        OpCode::AND | OpCode::EOR | OpCode::ORA => match instruction.mode {
            AddrMode::IndexedDirectWord => 7 - m + w,
            AddrMode::Stack => 5 - m,
            AddrMode::Direct => 4 - m + w,
            AddrMode::DirectSWord => 7 - m + w,
            AddrMode::Immediate => 3 - m,
            AddrMode::Absolute => 5 - m,
            AddrMode::Long => 6 - m,
            AddrMode::DirectIndexedWord => {
                7 - m + w - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::DirectWord => 6 - m + w,
            AddrMode::StackIndexed => 8 - m,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::DirectIndexedSWord => 7 - m + w,
            AddrMode::AbsoluteY | AddrMode::AbsoluteX => {
                6 - m - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::LongX => 6 - m,
            _ => unreachable!(),
        },
        OpCode::BIT => match instruction.mode {
            AddrMode::Direct => 4 - m + w,
            AddrMode::Absolute => 5 - m,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::AbsoluteX => 6 - m - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8,
            AddrMode::Immediate => 3 - m,
            _ => unreachable!(),
        },
        OpCode::TRB | OpCode::TSB => match instruction.mode {
            AddrMode::Direct => 7 - (2 * m) + w,
            AddrMode::Absolute => 8 - (2 * m),
            _ => unreachable!(),
        },
        OpCode::ASL | OpCode::LSR | OpCode::ROL | OpCode::ROR => match instruction.mode {
            AddrMode::Direct => 7 - (2 * m) + w,
            AddrMode::Accumulator => 2,
            AddrMode::Absolute => 8 - (2 * m),
            AddrMode::DirectX => 8 - (2 * m) + w,
            AddrMode::AbsoluteX => 9 - (2 * m),
            _ => unreachable!(),
        },
        OpCode::BRA => 3 + (snes.cpu.P.e || p) as u8,
        OpCode::BCC => 2 + (!snes.cpu.P.c as u8) + (!snes.cpu.P.c || snes.cpu.P.e || p) as u8,
        OpCode::BCS => 2 + (snes.cpu.P.c as u8) + (snes.cpu.P.c || snes.cpu.P.e || p) as u8,
        OpCode::BEQ => 2 + (snes.cpu.P.z as u8) + (snes.cpu.P.z || snes.cpu.P.e || p) as u8,
        OpCode::BMI => 2 + (snes.cpu.P.n as u8) + (snes.cpu.P.n || snes.cpu.P.e || p) as u8,
        OpCode::BNE => 2 + (!snes.cpu.P.n as u8) + (!snes.cpu.P.n || snes.cpu.P.e || p) as u8,
        OpCode::BPL => 2 + (!snes.cpu.P.z as u8) + (!snes.cpu.P.z || snes.cpu.P.e || p) as u8,
        OpCode::BVC => 2 + (snes.cpu.P.v as u8) + (snes.cpu.P.v || snes.cpu.P.e || p) as u8,
        OpCode::BVS => 2 + (!snes.cpu.P.v as u8) + (!snes.cpu.P.v || snes.cpu.P.e || p) as u8,
        OpCode::BRL => 4,
        OpCode::BRK | OpCode::COP => 8 - snes.cpu.P.e as u8,
        OpCode::CLC
        | OpCode::CLD
        | OpCode::CLI
        | OpCode::CLV
        | OpCode::SEC
        | OpCode::SED
        | OpCode::SEI => 2,
        OpCode::REP | OpCode::SEP => 3,
        OpCode::LDA | OpCode::STA => match instruction.mode {
            AddrMode::IndexedDirectWord => 7 - m + w,
            AddrMode::Stack => 5 - m,
            AddrMode::Direct => 4 - m + w,
            AddrMode::DirectSWord => 7 - m + w,
            AddrMode::Immediate => 3 - m,
            AddrMode::Absolute => 5 - m,
            AddrMode::Long => 6 - m,
            AddrMode::DirectIndexedWord => {
                7 - m + w - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            AddrMode::DirectWord => 6 - m + w,
            AddrMode::StackIndexed => 8 - m,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::DirectIndexedSWord => 7 - m + w,
            AddrMode::AbsoluteY | AddrMode::AbsoluteX => {
                6 - m - snes.cpu.P.x as u8 + (snes.cpu.P.x || p) as u8
            }
            _ => unreachable!(),
        },
        OpCode::LDX | OpCode::LDY | OpCode::STX | OpCode::STY => match instruction.mode {
            AddrMode::Immediate => 3 - snes.cpu.P.x as u8,
            AddrMode::Direct => 4 - snes.cpu.P.x as u8 + w,
            AddrMode::Absolute => 5 - snes.cpu.P.x as u8,
            AddrMode::DirectY | AddrMode::DirectX => 5 - snes.cpu.P.x as u8 + w,
            AddrMode::AbsoluteY | AddrMode::AbsoluteX => {
                6 - (2 * snes.cpu.P.x as u8) + (snes.cpu.P.x || p) as u8
            }
            _ => unreachable!(),
        },
        OpCode::STZ => match instruction.mode {
            AddrMode::Direct => 4 - m + w,
            AddrMode::DirectX => 5 - m + w,
            AddrMode::Absolute => 5 - m,
            AddrMode::AbsoluteX => 6 - m,
            _ => unreachable!(),
        },
        OpCode::PHA => 4 - m,
        OpCode::PHX | OpCode::PHY => 4 - snes.cpu.P.x as u8,
        OpCode::PLA => 5 - m,
        OpCode::PLX | OpCode::PLY => 5 - snes.cpu.P.x as u8,
        OpCode::PHB | OpCode::PHK | OpCode::PHP => 3,
        OpCode::PHD | OpCode::PLB | OpCode::PLP => 4,
        OpCode::PLD => 5,
        OpCode::STP | OpCode::WAI => 3,
        OpCode::TAX
        | OpCode::TAY
        | OpCode::TSX
        | OpCode::TXA
        | OpCode::TXS
        | OpCode::TXY
        | OpCode::TYA
        | OpCode::TYX
        | OpCode::TCD
        | OpCode::TCS
        | OpCode::TDC
        | OpCode::TSC => 2,
        OpCode::XBA => 3,
        OpCode::XCE => 2,
    };
    Ok(cycles)
}

fn calculate_address(
    snes: &Console,
    op: &OpCode,
    mode: &AddrMode,
    loc: u32,
) -> Result<(u32, Option<u32>)> {
    let l = memory::peek_byte(snes, loc + 1)? as u32;
    let h = memory::peek_byte(snes, loc + 2)? as u32;
    let mut dest = None;
    let addr = match mode {
        AddrMode::Absolute => {
            if op.is_jump() {
                ((snes.cpu.K as u32) << 16) | h << 8 | l
            } else {
                ((snes.cpu.DBR as u32) << 16) | h << 8 | l
            }
        }
        AddrMode::AbsoluteWord => h << 8 | l,
        AddrMode::AbsoluteSWord => h << 8 | l,
        AddrMode::AbsoluteX => {
            let refaddr = ((snes.cpu.DBR as u32) << 16) | h << 8 | l;
            refaddr + snes.cpu.X as u32
        }
        AddrMode::AbsoluteY => {
            let refaddr = ((snes.cpu.DBR as u32) << 16) | h << 8 | l;
            refaddr + snes.cpu.Y as u32
        }
        AddrMode::AbsoluteIndirectWord => {
            let refaddr = h << 8 | l;
            (snes.cpu.K as u32) << 16 | memory::read_word(snes, refaddr)? as u32
        }
        AddrMode::AbsoluteIndirectSWord => {
            let refaddr = h << 8 | l;
            let low = memory::read_word(snes, refaddr)? as u32;
            let high = memory::read_byte(snes, refaddr + 2)? as u32;
            high << 16 | low
        }
        AddrMode::AbsoluteIndexedIndirect => {
            let refaddr = ((snes.cpu.K as u32) << 16) | h << 8 | l;
            refaddr + snes.cpu.X as u32
        }
        AddrMode::Accumulator => snes.cpu.get_pc(),
        AddrMode::Direct => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 && op.is_old() {
                ((snes.cpu.D & 0xFF00) as u32) >> 8 | l
            } else {
                (snes.cpu.D as u32) + l
            }
        }
        AddrMode::DirectX => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 {
                let templ = (snes.cpu.X as u8).wrapping_add(l as u8);
                ((snes.cpu.D & 0xFF00) as u32) >> 8 | templ as u32
            } else {
                (snes.cpu.D as u32) + l + snes.cpu.X as u32
            }
        }
        AddrMode::DirectY => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 {
                let templ = (snes.cpu.Y as u8).wrapping_add(l as u8);
                ((snes.cpu.D & 0xFF00) as u32) >> 8 | templ as u32
            } else {
                (snes.cpu.D as u32) + l + snes.cpu.Y as u32
            }
        }
        AddrMode::DirectWord => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 {
                let temp_addr = ((snes.cpu.D & 0xFF00) as u32) >> 8 | l;
                let pointer = memory::read_word(snes, temp_addr)?;
                (snes.cpu.DBR as u32) << 16 | pointer as u32
            } else {
                let temp_addr = (snes.cpu.D as u32) + l;
                let pointer = memory::read_word(snes, temp_addr)?;
                (snes.cpu.DBR as u32) << 16 | pointer as u32
            }
        }
        AddrMode::DirectSWord => {
            let temp_addr = (snes.cpu.D as u32) + l;
            let pointer = memory::read_word(snes, temp_addr)?;
            (snes.cpu.DBR as u32) << 16 | pointer as u32
        }
        AddrMode::IndexedDirectWord => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 {
                let temp_addr = ((snes.cpu.D & 0xFF00) as u32) >> 8 | l;
                let pointer = memory::read_word(snes, temp_addr.wrapping_add(snes.cpu.X as u32))?;
                (snes.cpu.DBR as u32) << 16 | pointer as u32
            } else {
                let temp_addr = (snes.cpu.D as u32) + l;
                let pointer = memory::read_word(snes, temp_addr.wrapping_add(snes.cpu.X as u32))?;
                (snes.cpu.DBR as u32) << 16 | pointer as u32
            }
        }
        AddrMode::DirectIndexedWord => {
            if snes.cpu.P.e && (snes.cpu.D & 0xFF) == 0x00 {
                let temp_addr = ((snes.cpu.D & 0xFF00) as u32) >> 8 | l;
                let pointer = memory::read_word(snes, temp_addr)?;
                let temp_data_addr = (snes.cpu.DBR as u32) << 16 | pointer as u32;
                temp_data_addr.wrapping_add(snes.cpu.Y as u32)
            } else {
                let temp_addr = (snes.cpu.D as u32) + l;
                let pointer = memory::read_word(snes, temp_addr)?;
                let temp_data_addr = (snes.cpu.DBR as u32) << 16 | pointer as u32;
                temp_data_addr.wrapping_add(snes.cpu.Y as u32)
            }
        }
        AddrMode::DirectIndexedSWord => {
            let temp_addr = (snes.cpu.D as u32) + l;
            let pointer_lo = memory::read_byte(snes, temp_addr)?;
            let pointer_mid = memory::read_byte(snes, temp_addr + 1)?;
            let pointer_hi = memory::read_byte(snes, temp_addr + 2)?;
            let temp_data_addr = u32::from_be_bytes([0x00, pointer_hi, pointer_mid, pointer_lo]);
            temp_data_addr.wrapping_add(snes.cpu.Y as u32)
        }
        AddrMode::Immediate => snes.cpu.get_pc() + 1,
        AddrMode::Implied => snes.cpu.get_pc(),
        AddrMode::Long => {
            let hh = memory::read_byte(snes, snes.cpu.get_pc() + 3)?;
            u32::from_be_bytes([0x00, hh, h as u8, l as u8])
        }
        AddrMode::LongX => {
            let hh = memory::read_byte(snes, snes.cpu.get_pc() + 3)?;
            let temp = u32::from_be_bytes([0x00, hh, h as u8, l as u8]);
            temp + snes.cpu.X as u32
        }
        AddrMode::RelativeByte => {
            let temp = if l <= 0x7F {
                snes.cpu.PC + 2 + l as u16
            } else {
                snes.cpu.PC - 254 + l as u16
            }
            .to_be_bytes();
            u32::from_be_bytes([0x00, snes.cpu.K, temp[0], temp[1]])
        }
        AddrMode::RelativeWord => {
            let temp = snes
                .cpu
                .PC
                .wrapping_add(3)
                .wrapping_add(u16::from_be_bytes([h as u8, l as u8]))
                .to_be_bytes();
            u32::from_be_bytes([0x00, snes.cpu.K, temp[0], temp[1]])
        }
        AddrMode::SourceDestination => {
            dest = Some(l << 16 | snes.cpu.X as u32);
            h << 16 | snes.cpu.Y as u32
        }
        AddrMode::Stack => (l as u16 + snes.cpu.S) as u32,
        AddrMode::StackIndexed => {
            let low = memory::read_byte(snes, (l as u16 + snes.cpu.S) as u32)?;
            let high = memory::read_byte(snes, (l as u16 + snes.cpu.S + 1) as u32)?;
            let temp = u32::from_be_bytes([0, snes.cpu.DBR, high, low]);
            temp + snes.cpu.Y as u32
        }
    };
    Ok((addr, dest))
}

pub fn decode_instruction(snes: &Console, instruction: u8, loc: u32) -> Result<InstructionContext> {
    let opcode = match instruction {
        0x00 => OpCode::BRK,
        0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0D | 0x0F | 0x11 | 0x12 | 0x13 | 0x15 | 0x17
        | 0x19 | 0x1D | 0x1F => OpCode::ORA,
        0x02 => OpCode::COP,
        0x04 | 0x0C => OpCode::TSB,
        0x06 | 0x0A | 0x0E | 0x16 | 0x1E => OpCode::ASL,
        0x08 => OpCode::PHP,
        0x0B => OpCode::PHD,
        0x10 => OpCode::BPL,
        0x14 | 0x1C => OpCode::TRB,
        0x18 => OpCode::CLC,
        0x1A | 0xE6 | 0xEE | 0xF6 | 0xFE => OpCode::INC,
        0x1B => OpCode::TCS,
        0x20 | 0xFC => OpCode::JSR,
        0x21 | 0x23 | 0x25 | 0x27 | 0x29 | 0x2D | 0x2F | 0x31 | 0x32 | 0x33 | 0x35 | 0x37
        | 0x39 | 0x3D | 0x3F => OpCode::AND,
        0x22 => OpCode::JSL,
        0x24 | 0x2C | 0x34 | 0x3C | 0x89 => OpCode::BIT,
        0x26 | 0x2A | 0x2E | 0x36 | 0x3E => OpCode::ROL,
        0x28 => OpCode::PLP,
        0x2B => OpCode::PLD,
        0x30 => OpCode::BMI,
        0x38 => OpCode::SEC,
        0x3A | 0xC6 | 0xCE | 0xD6 | 0xDE => OpCode::DEC,
        0x3B => OpCode::TSC,
        0x40 => OpCode::RTI,
        0x41 | 0x43 | 0x45 | 0x47 | 0x49 | 0x4D | 0x4F | 0x51 | 0x52 | 0x53 | 0x55 | 0x57
        | 0x59 | 0x5D | 0x5F => OpCode::EOR,
        0x42 => OpCode::WDM,
        0x44 => OpCode::MVP,
        0x46 | 0x4A | 0x4E | 0x56 | 0x5E => OpCode::LSR,
        0x48 => OpCode::PHA,
        0x4B => OpCode::PHK,
        0x4C | 0x5C | 0x6C | 0x7C => OpCode::JMP,
        0x50 => OpCode::BVC,
        0x54 => OpCode::MVN,
        0x58 => OpCode::CLI,
        0x5A => OpCode::PHY,
        0x5B => OpCode::TCD,
        0x60 => OpCode::RTS,
        0x61 | 0x63 | 0x65 | 0x67 | 0x69 | 0x6D | 0x6F | 0x71 | 0x72 | 0x73 | 0x75 | 0x77
        | 0x79 | 0x7D | 0x7F => OpCode::ADC,
        0x62 => OpCode::PER,
        0x64 | 0x74 | 0x9C | 0x9E => OpCode::STZ,
        0x66 | 0x6A | 0x6E | 0x76 | 0x7E => OpCode::ROR,
        0x68 => OpCode::PLA,
        0x6B => OpCode::RTL,
        0x70 => OpCode::BVS,
        0x78 => OpCode::SEI,
        0x7A => OpCode::PLY,
        0x7B => OpCode::TDC,
        0x80 => OpCode::BRA,
        0x81 | 0x83 | 0x85 | 0x87 | 0x8D | 0x8F | 0x91 | 0x92 | 0x93 | 0x95 | 0x97 | 0x99
        | 0x9D | 0x9F => OpCode::STA,
        0x82 => OpCode::BRL,
        0x84 | 0x8C | 0x94 => OpCode::STY,
        0x86 | 0x8E | 0x96 => OpCode::STX,
        0x88 => OpCode::DEY,
        0x8A => OpCode::TXA,
        0x8B => OpCode::PHB,
        0x90 => OpCode::BCC,
        0x98 => OpCode::TYA,
        0x9A => OpCode::TXS,
        0x9B => OpCode::TXY,
        0xA0 | 0xA4 | 0xAC | 0xB4 | 0xBC => OpCode::LDY,
        0xA1 | 0xA3 | 0xA5 | 0xA7 | 0xA9 | 0xAD | 0xAF | 0xB1 | 0xB2 | 0xB3 | 0xB5 | 0xB7
        | 0xB9 | 0xBD | 0xBF => OpCode::LDA,
        0xA2 | 0xA6 | 0xAE | 0xB6 | 0xBE => OpCode::LDX,
        0xA8 => OpCode::TAY,
        0xAA => OpCode::TAX,
        0xAB => OpCode::PLB,
        0xB0 => OpCode::BCS,
        0xB8 => OpCode::CLV,
        0xBA => OpCode::TSX,
        0xBB => OpCode::TYX,
        0xC0 | 0xC4 | 0xCC => OpCode::CPY,
        0xC1 | 0xC3 | 0xC5 | 0xC7 | 0xC9 | 0xCD | 0xCF | 0xD1 | 0xD2 | 0xD3 | 0xD5 | 0xD7
        | 0xD9 | 0xDD | 0xDF => OpCode::CMP,
        0xC2 => OpCode::REP,
        0xC8 => OpCode::INY,
        0xCA => OpCode::DEX,
        0xCB => OpCode::WAI,
        0xD0 => OpCode::BNE,
        0xD4 => OpCode::PEI,
        0xD8 => OpCode::CLD,
        0xDA => OpCode::PHX,
        0xDB => OpCode::STP,
        0xDC => OpCode::JML,
        0xE0 | 0xE4 | 0xEC => OpCode::CPX,
        0xE1 | 0xE3 | 0xE5 | 0xE7 | 0xE9 | 0xED | 0xEF | 0xF1 | 0xF2 | 0xF3 | 0xF5 | 0xF7
        | 0xF9 | 0xFD | 0xFF => OpCode::SBC,
        0xE2 => OpCode::SEP,
        0xE8 => OpCode::INX,
        0xEA => OpCode::NOP,
        0xEB => OpCode::XBA,
        0xF0 => OpCode::BEQ,
        0xF4 => OpCode::PEA,
        0xF8 => OpCode::SED,
        0xFA => OpCode::PLX,
        0xFB => OpCode::XCE,
    };
    let mode = decode_addressing_mode(instruction)?;
    let address = calculate_address(&snes, &opcode, &mode, loc)?;

    Ok(InstructionContext {
        opcode,
        mode,
        inst_addr: loc,
        data_addr: address.0,
        dest_addr: address.1,
    })
}

fn push_byte(snes: &mut Console, data: u8) -> Result<()> {
    match snes.cpu.P.e {
        true => {
            let mut sl = (snes.cpu.S & 0xFF) as u8;
            memory::write_byte(snes, 0x000100 | sl as u32, data)?;
            sl = sl.wrapping_sub(1);
            snes.cpu.S &= 0xFF00;
            snes.cpu.S |= sl as u16;
        }
        false => {
            memory::write_byte(snes, snes.cpu.S as u32, data)?;
            snes.cpu.S = snes.cpu.S.wrapping_sub(1);
        }
    }
    Ok(())
}

fn pull_byte(snes: &mut Console) -> Result<u8> {
    match snes.cpu.P.e {
        true => {
            let mut sl = (snes.cpu.S & 0xFF) as u8;
            sl = sl.wrapping_add(1);
            snes.cpu.S &= 0xFF00;
            snes.cpu.S |= sl as u16;
            memory::read_byte(snes, 0x000100 | sl as u32)
        }
        false => {
            snes.cpu.S = snes.cpu.S.wrapping_add(1);
            memory::read_byte(snes, snes.cpu.S as u32)
        }
    }
}

fn push_word(snes: &mut Console, data: u16) -> Result<()> {
    let datal = (data & 0xFF) as u8;
    let datah = ((data & 0xFF00) >> 8) as u8;
    match snes.cpu.P.e {
        true => {
            let mut sl = (snes.cpu.S & 0xFF) as u8;
            memory::write_byte(snes, 0x000100 | sl as u32, datah)?;
            sl = sl.wrapping_sub(1);
            memory::write_byte(snes, 0x000100 | sl as u32, datal)?;
            sl = sl.wrapping_sub(1);
            snes.cpu.S &= 0xFF00;
            snes.cpu.S |= sl as u16;
        }
        false => {
            memory::write_byte(snes, snes.cpu.S as u32, datah)?;
            snes.cpu.S = snes.cpu.S.wrapping_sub(1);
            memory::write_byte(snes, snes.cpu.S as u32, datal)?;
            snes.cpu.S = snes.cpu.S.wrapping_sub(1);
        }
    }
    Ok(())
}

fn pull_word(snes: &mut Console) -> Result<u16> {
    match snes.cpu.P.e {
        true => {
            let mut sl = (snes.cpu.S & 0xFF) as u8;
            sl = sl.wrapping_add(1);
            let datal = memory::read_byte(snes, 0x000100 | sl as u32)?;
            sl = sl.wrapping_add(1);
            let datah = memory::read_byte(snes, 0x000100 | sl as u32)?;
            snes.cpu.S &= 0xFF00;
            snes.cpu.S |= sl as u16;
            Ok(u16::from_be_bytes([datah, datal]))
        }
        false => {
            snes.cpu.S = snes.cpu.S.wrapping_add(1);
            let datal = memory::read_byte(snes, snes.cpu.S as u32)?;
            snes.cpu.S = snes.cpu.S.wrapping_add(1);
            let datah = memory::read_byte(snes, snes.cpu.S as u32)?;
            Ok(u16::from_be_bytes([datah, datal]))
        }
    }
}

fn push_sword(snes: &mut Console, data: u32) -> Result<()> {
    let databytes = data.to_le_bytes();
    memory::write_byte(snes, snes.cpu.S as u32, databytes[1])?;
    snes.cpu.S = snes.cpu.S.wrapping_sub(1);
    memory::write_byte(snes, snes.cpu.S as u32, databytes[2])?;
    snes.cpu.S = snes.cpu.S.wrapping_sub(1);
    memory::write_byte(snes, snes.cpu.S as u32, databytes[3])?;
    snes.cpu.S = snes.cpu.S.wrapping_sub(1);
    Ok(())
}

fn pull_sword(snes: &mut Console) -> Result<u32> {
    snes.cpu.S = snes.cpu.S.wrapping_add(1);
    let datal = memory::read_byte(snes, snes.cpu.S as u32)?;
    snes.cpu.S = snes.cpu.S.wrapping_add(1);
    let datam = memory::read_byte(snes, snes.cpu.S as u32)?;
    snes.cpu.S = snes.cpu.S.wrapping_add(1);
    let datah = memory::read_byte(snes, snes.cpu.S as u32)?;
    Ok(u32::from_be_bytes([0x00, datah, datam, datal]))
}

pub fn execute_nmi(snes: &mut Console) -> Result<()> {
    push_byte(snes, snes.cpu.K)?;
    push_word(snes, snes.cpu.PC)?;
    push_byte(snes, snes.cpu.p_byte())?;
    snes.cpu.PC = snes.cartridge.header.interrupt_vectors.nmi;
    snes.cpu.K = 0;
    Ok(())
}

fn execute_instruction_emu(
    snes: &mut Console,
    instruction: &InstructionContext,
) -> Result<CPUExecutionResult> {
    trace!("Executing {} in emu mode", instruction.opcode);
    match instruction.opcode {
        OpCode::BRK => {
            push_word(snes, (instruction.inst_addr & 0xFF) as u16 + 2)?;
            let p = snes.cpu.p_byte() | 0b00010000;
            push_byte(snes, p)?;
            snes.cpu.K = 0;
            snes.cpu.PC = snes.cartridge.header.interrupt_vectors.brk_emu;
            snes.cpu.P.i = true;
            snes.cpu.P.d = false;
            return Ok(CPUExecutionResult::Interrupt);
        }
        OpCode::JMP => {
            snes.cpu.set_pc(instruction.data_addr);
            return Ok(CPUExecutionResult::Jump);
        }
        OpCode::SEI => {
            snes.cpu.P.i = true;
        }
        OpCode::CLC => {
            snes.cpu.P.c = false;
        }
        OpCode::XCE => {
            let temp = snes.cpu.P.c;
            snes.cpu.P.c = snes.cpu.P.e;
            snes.cpu.P.e = temp;
        }
        _ => todo!(
            "Implement remaining instructions, tried to execute: {} in emu mode",
            instruction.opcode
        ),
    }
    snes.cpu.PC += instruction.length(snes.cpu.P.m, snes.cpu.P.x) as u16;
    Ok(CPUExecutionResult::Normal)
}

pub fn execute_instruction(
    snes: &mut Console,
    instruction: &InstructionContext,
) -> Result<CPUExecutionResult> {
    if snes.cpu.P.e {
        return execute_instruction_emu(snes, &instruction);
    }
    trace!("Executing {}", instruction.with_source(snes));
    match instruction.opcode {
        OpCode::JMP => {
            snes.cpu.set_pc(instruction.data_addr);
            return Ok(CPUExecutionResult::Jump);
        }
        OpCode::JSL => {
            push_byte(snes, snes.cpu.K)?;
            push_word(snes, (instruction.inst_addr as u16).wrapping_add(3))?;
            snes.cpu.set_pc(instruction.data_addr);
            return Ok(CPUExecutionResult::Subroutine(instruction.data_addr));
        }
        OpCode::JSR => {
            push_word(snes, (instruction.inst_addr as u16).wrapping_add(2))?;
            snes.cpu.set_pc(instruction.data_addr);
            return Ok(CPUExecutionResult::Subroutine(instruction.data_addr));
        }
        OpCode::RTI => {
            let p = pull_byte(snes)?;
            snes.cpu.set_p(p);
            snes.cpu.PC = pull_word(snes)?;
            snes.cpu.K = pull_byte(snes)?;
            return Ok(CPUExecutionResult::Return);
        }
        OpCode::ADC => {
            if snes.cpu.P.d {
                unimplemented!("Decimal mode");
                // let mut temp_A: [u8; 4] = [0;4];
                // temp_A[0] = (snes.cpu.A & 0x000F) as u8;
                // temp_A[1] = ((snes.cpu.A & 0x00F0) >> 4) as u8;
                // temp_A[2] = ((snes.cpu.A & 0x0F00) >> 8) as u8;
                // temp_A[3] = ((snes.cpu.A & 0xF000) >> 12) as u8;
                // let mut temp_val: [u8; 4] = [0;4];
                // let temp = memory::read_word(snes, instruction.data_addr)?;
                // temp_val[0] = (temp & 0x000F) as u8;
                // temp_val[1] = ((temp & 0x00F0) >> 4) as u8;
                // temp_val[2] = ((temp & 0x0F00) >> 8) as u8;
                // temp_val[3] = ((temp & 0xF000) >> 12) as u8;
            } else {
                let overflow: bool;
                let overflow2: bool;
                (snes.cpu.A, overflow) = snes
                    .cpu
                    .A
                    .overflowing_add(memory::read_word(snes, instruction.data_addr)?);
                (snes.cpu.A, overflow2) = snes.cpu.A.overflowing_add(snes.cpu.P.c as u16);
                snes.cpu.P.z = snes.cpu.A == 0;
                snes.cpu.P.n = snes.cpu.A >= 0b10000000;
                snes.cpu.P.v = overflow || overflow2;
                snes.cpu.P.c = overflow || overflow2;
            }
        }
        OpCode::AND => {
            if snes.cpu.P.m {
                let temp =
                    (snes.cpu.A & 0xFF) as u8 & memory::read_byte(snes, instruction.data_addr)?;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.n = temp & 0x80 != 0;
                snes.cpu.A &= 0xFF00;
                snes.cpu.A |= temp as u16;
            } else {
                let temp = snes.cpu.A & memory::read_word(snes, instruction.data_addr)?;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.n = temp & 0x8000 != 0;
                snes.cpu.A = temp;
            }
        }
        OpCode::ASL => {
            if snes.cpu.P.m {
                if instruction.mode == AddrMode::Accumulator {
                    let temp = snes.cpu.A << 1;
                    snes.cpu.P.c = (snes.cpu.A & 0x80) > 0;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = temp >= 0b10000000;
                    snes.cpu.A = temp;
                } else {
                    let data = memory::read_byte(snes, instruction.data_addr)?;
                    let temp = data << 1;
                    snes.cpu.P.c = (data & 0x80) > 0;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = temp >= 0b10000000;
                    memory::write_byte(snes, instruction.data_addr, temp)?;
                }
            } else {
                if instruction.mode == AddrMode::Accumulator {
                    let temp = snes.cpu.A << 1;
                    snes.cpu.P.c = (snes.cpu.A & 0x8000) > 0;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = temp >= 0b10000000;
                    snes.cpu.A = temp;
                } else {
                    let data = memory::read_word(snes, instruction.data_addr)?;
                    let temp = data << 1;
                    snes.cpu.P.c = (data & 0x8000) > 0;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = temp >= 0b10000000;
                    memory::write_word(snes, instruction.data_addr, temp)?;
                }
            }
        }
        OpCode::BCC => {
            if !snes.cpu.P.c {
                snes.cpu.PC = instruction.data_addr as u16;
                return Ok(CPUExecutionResult::BranchTaken);
            }
        }
        OpCode::BEQ => {
            if snes.cpu.P.z {
                snes.cpu.PC = instruction.data_addr as u16;
                return Ok(CPUExecutionResult::BranchTaken);
            }
        }
        OpCode::BRA => {
            snes.cpu.PC = instruction.data_addr as u16;
            return Ok(CPUExecutionResult::BranchTaken);
        }
        OpCode::BIT => {
            if instruction.mode != AddrMode::Immediate {
                snes.cpu.P.z = snes.cpu.A.to_be_bytes()[1]
                    & memory::read_byte(snes, instruction.data_addr)?
                    == 0;
            } else {
                if snes.cpu.P.m {
                    let temp = memory::read_byte(snes, instruction.data_addr)?;
                    snes.cpu.P.z = temp & snes.cpu.A.to_be_bytes()[1] == 0;
                    snes.cpu.P.n = temp & 0x80 > 0;
                    snes.cpu.P.v = temp & 0x40 > 0;
                } else {
                    let temp = memory::read_word(snes, instruction.data_addr)?;
                    snes.cpu.P.z = temp & snes.cpu.A == 0;
                    snes.cpu.P.n = temp & 0x8000 > 0;
                    snes.cpu.P.v = temp & 0x4000 > 0;
                }
            }
        }
        OpCode::BPL => {
            if !snes.cpu.P.n {
                snes.cpu.PC = instruction.data_addr as u16;
                return Ok(CPUExecutionResult::BranchTaken);
            }
        }
        OpCode::BMI => {
            if snes.cpu.P.n {
                snes.cpu.PC = instruction.data_addr as u16;
                return Ok(CPUExecutionResult::BranchTaken);
            }
        }
        OpCode::BNE => {
            if !snes.cpu.P.z {
                snes.cpu.PC = instruction.data_addr as u16;
                return Ok(CPUExecutionResult::BranchTaken);
            }
        }
        OpCode::BRK => {
            push_byte(snes, snes.cpu.K)?;
            push_word(snes, (instruction.inst_addr & 0xFF) as u16 + 2)?;
            push_byte(snes, snes.cpu.p_byte())?;
            snes.cpu.K = 0;
            snes.cpu.PC = snes.cartridge.header.interrupt_vectors.brk;
            return Ok(CPUExecutionResult::Interrupt);
        }
        OpCode::CLC => {
            snes.cpu.P.c = false;
        }
        OpCode::CLI => {
            snes.cpu.P.i = false;
        }
        OpCode::SEC => {
            snes.cpu.P.c = true;
        }
        OpCode::CMP => {
            if snes.cpu.P.m {
                let data = memory::read_byte(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.A as u16, data as u16);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = (snes.cpu.A as u16) >= (data as u16);
            } else {
                let data = memory::read_word(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.A, data);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = snes.cpu.A >= data;
            }
        }
        OpCode::CPX => {
            if snes.cpu.P.x {
                let data = memory::read_byte(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.X as u16, data as u16);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = (snes.cpu.X as u16) >= (data as u16);
            } else {
                let data = memory::read_word(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.X, data);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = snes.cpu.X >= data;
            }
        }
        OpCode::CPY => {
            if snes.cpu.P.x {
                let data = memory::read_byte(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.Y as u16, data as u16);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = (snes.cpu.Y as u16) >= (data as u16);
            } else {
                let data = memory::read_word(snes, instruction.data_addr)?;
                let temp = u16::wrapping_sub(snes.cpu.Y, data);
                snes.cpu.P.n = temp >= 0b10000000;
                snes.cpu.P.z = temp == 0;
                snes.cpu.P.c = snes.cpu.Y >= data;
            }
        }
        OpCode::DEC => {
            if instruction.mode == AddrMode::Accumulator {
                if snes.cpu.P.m {
                    let mut temp = (snes.cpu.A & 0xFF) as u8;
                    temp = temp.wrapping_sub(1);
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp as u16;
                    snes.cpu.P.n = (temp & 0x80) != 0;
                    snes.cpu.P.z = temp == 0;
                } else {
                    snes.cpu.A = snes.cpu.A.wrapping_sub(1);
                    snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
                    snes.cpu.P.z = snes.cpu.A == 0;
                }
            } else {
                if snes.cpu.P.m {
                    let mut data = memory::read_byte(snes, instruction.data_addr)?;
                    data = data.wrapping_sub(1);
                    memory::write_byte(snes, instruction.data_addr, data)?;
                    snes.cpu.P.n = (data & 0x80) != 0;
                    snes.cpu.P.z = data == 0;
                } else {
                    let mut data = memory::read_word(snes, instruction.data_addr)?;
                    data = data.wrapping_sub(1);
                    memory::write_word(snes, instruction.data_addr, data)?;
                    snes.cpu.P.n = (data & 0x8000) != 0;
                    snes.cpu.P.z = data == 0;
                }
            }
        }
        OpCode::DEX => {
            if snes.cpu.P.x {
                let mut temp = (snes.cpu.X & 0xFF) as u8;
                temp = temp.wrapping_sub(1);
                snes.cpu.X &= 0xFF00;
                snes.cpu.X |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.X = snes.cpu.X.wrapping_sub(1);
                snes.cpu.P.n = (snes.cpu.X & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.X == 0;
            }
        }
        OpCode::DEY => {
            if snes.cpu.P.x {
                let mut temp = (snes.cpu.Y & 0xFF) as u8;
                temp = temp.wrapping_sub(1);
                snes.cpu.Y &= 0xFF00;
                snes.cpu.Y |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.Y = snes.cpu.Y.wrapping_sub(1);
                snes.cpu.P.n = (snes.cpu.Y & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.Y == 0;
            }
        }
        OpCode::INC => {
            if instruction.mode == AddrMode::Accumulator {
                if snes.cpu.P.m {
                    let mut temp = (snes.cpu.A & 0xFF) as u8;
                    temp = temp.wrapping_add(1);
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp as u16;
                    snes.cpu.P.n = (temp & 0x80) != 0;
                    snes.cpu.P.z = temp == 0;
                } else {
                    snes.cpu.A = snes.cpu.A.wrapping_add(1);
                    snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
                    snes.cpu.P.z = snes.cpu.A == 0;
                }
            } else {
                if snes.cpu.P.m {
                    let mut data = memory::read_byte(snes, instruction.data_addr)?;
                    data = data.wrapping_add(1);
                    memory::write_byte(snes, instruction.data_addr, data)?;
                    snes.cpu.P.n = (data & 0x80) != 0;
                    snes.cpu.P.z = data == 0;
                } else {
                    let mut data = memory::read_word(snes, instruction.data_addr)?;
                    data = data.wrapping_add(1);
                    memory::write_word(snes, instruction.data_addr, data)?;
                    snes.cpu.P.n = (data & 0x8000) != 0;
                    snes.cpu.P.z = data == 0;
                }
            }
        }
        OpCode::INX => {
            if snes.cpu.P.x {
                let mut temp = (snes.cpu.X & 0xFF) as u8;
                temp = temp.wrapping_add(1);
                snes.cpu.X &= 0xFF00;
                snes.cpu.X |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.X = snes.cpu.X.wrapping_add(1);
                snes.cpu.P.n = (snes.cpu.X & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.X == 0;
            }
        }
        OpCode::INY => {
            if snes.cpu.P.x {
                let mut temp = (snes.cpu.Y & 0xFF) as u8;
                temp = temp.wrapping_add(1);
                snes.cpu.Y &= 0xFF00;
                snes.cpu.Y |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.Y = snes.cpu.Y.wrapping_add(1);
                snes.cpu.P.n = (snes.cpu.Y & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.Y == 0;
            }
        }
        OpCode::LSR => {
            if snes.cpu.P.m {
                if instruction.mode == AddrMode::Accumulator {
                    snes.cpu.P.c = (snes.cpu.A & 0x01) != 0;
                    let mut temp_a = (snes.cpu.A & 0xFE) as u8;
                    temp_a >>= 1;
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp_a as u16;
                    snes.cpu.P.z = temp_a == 0;
                    snes.cpu.P.n = false;
                } else {
                    let mut temp = read_byte(snes, instruction.data_addr)?;
                    snes.cpu.P.c = (temp & 0x01) != 0;
                    let temp = (temp & 0xFE) >> 1;
                    write_byte(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = false;
                }
            } else {
                if instruction.mode == AddrMode::Accumulator {
                    snes.cpu.P.c = (snes.cpu.A & 0x0001) != 0;
                    let temp_a = (snes.cpu.A & 0xFFFE) >> 1;
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp_a;
                    snes.cpu.P.z = temp_a == 0;
                    snes.cpu.P.n = false;
                } else {
                    let mut temp = read_word(snes, instruction.data_addr)?;
                    snes.cpu.P.c = (temp & 0x0001) != 0;
                    let temp = (temp & 0xFFFE) >> 1;
                    write_word(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = false;
                }
            }
        }
        OpCode::ROL => {
            if snes.cpu.P.m {
                let temp_c;
                if instruction.mode == AddrMode::Accumulator {
                    let mut temp_a = (snes.cpu.A & 0xFF) as u8;
                    temp_c = (snes.cpu.A & 0x00F0) as u8;
                    temp_a = (temp_a & 0x7F) << 1;
                    temp_a += snes.cpu.P.c as u8;
                    snes.cpu.P.c = temp_c != 0;
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp_a as u16;
                    snes.cpu.P.z = temp_a == 0;
                    snes.cpu.P.n = (temp_a & 0x80) != 0;
                } else {
                    let mut temp = read_byte(snes, instruction.data_addr)?;
                    temp_c = (temp & 0xF0) as u8;
                    temp = (temp & 0x7F) << 1;
                    temp += snes.cpu.P.c as u8;
                    snes.cpu.P.c = temp_c != 0;
                    write_byte(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = (temp & 0x80) != 0;
                }
            } else {
                let temp_c;
                if instruction.mode == AddrMode::Accumulator {
                    let mut temp_a = snes.cpu.A;
                    temp_c = temp_a & 0xF000;
                    temp_a = (temp_a & 0x7FFF) << 1;
                    temp_a += snes.cpu.P.c as u16;
                    snes.cpu.P.c = temp_c != 0;
                    snes.cpu.A = temp_a;
                    snes.cpu.P.z = temp_a != 0;
                    snes.cpu.P.n = (temp_a & 0x8000) != 0;
                } else {
                    let mut temp = read_word(snes, instruction.data_addr)?;
                    temp_c = temp & 0xF000;
                    temp = (temp & 0x7FFF) << 1;
                    temp += snes.cpu.P.c as u16;
                    snes.cpu.P.c = temp_c != 0;
                    write_word(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp != 0;
                    snes.cpu.P.n = (temp & 0x8000) != 0;
                }
            }
        }
        OpCode::ROR => {
            if snes.cpu.P.m {
                if instruction.mode == AddrMode::Accumulator {
                    snes.cpu.P.c = (snes.cpu.A & 0x01) != 0;
                    let temp_a = (snes.cpu.A & 0xFE) >> 1;
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp_a as u16;
                    snes.cpu.P.z = temp_a == 0;
                    snes.cpu.P.n = (temp_a & 0x80) != 0;
                } else {
                    let temp = memory::read_byte(snes, instruction.data_addr)?;
                    snes.cpu.P.c = (temp & 0x01) != 0;
                    let temp = (temp & 0xFE) >> 1;
                    memory::write_byte(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = (temp & 0x80) != 0;
                }
            } else {
                if instruction.mode == AddrMode::Accumulator {
                    snes.cpu.P.c = (snes.cpu.A & 0x0001) != 0;
                    let temp_a = (snes.cpu.A & 0xFFFE) >> 1;
                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= temp_a;
                    snes.cpu.P.z = temp_a == 0;
                    snes.cpu.P.n = (temp_a & 0x8000) != 0;
                } else {
                    let temp = memory::read_word(snes, instruction.data_addr)?;
                    snes.cpu.P.c = (temp & 0x0001) != 0;
                    let temp = (temp & 0xFFFE) >> 1;
                    memory::write_word(snes, instruction.data_addr, temp)?;
                    snes.cpu.P.z = temp == 0;
                    snes.cpu.P.n = (temp & 0x8000) != 0;
                }
            }
        }
        OpCode::STA => {
            if snes.cpu.P.m {
                memory::write_byte(snes, instruction.data_addr, snes.cpu.A.to_le_bytes()[0])?;
            } else {
                memory::write_word(snes, instruction.data_addr, snes.cpu.A)?;
            }
        }
        OpCode::STX => {
            if snes.cpu.P.x {
                memory::write_byte(snes, instruction.data_addr, snes.cpu.X.to_le_bytes()[0])?;
            } else {
                memory::write_word(snes, instruction.data_addr, snes.cpu.X)?;
            }
        }
        OpCode::STY => {
            if snes.cpu.P.x {
                memory::write_byte(snes, instruction.data_addr, snes.cpu.Y.to_le_bytes()[0])?;
            } else {
                memory::write_word(snes, instruction.data_addr, snes.cpu.Y)?;
            }
        }
        OpCode::STZ => {
            if snes.cpu.P.m {
                memory::write_byte(snes, instruction.data_addr, 0)?;
            } else {
                memory::write_word(snes, instruction.data_addr, 0)?;
            }
        }
        OpCode::REP => {
            let flags = memory::read_byte(snes, instruction.data_addr)?;
            if flags & 0b1 != 0 {
                snes.cpu.P.c = false;
            }
            if flags & 0b10 != 0 {
                snes.cpu.P.z = false;
            }
            if flags & 0b100 != 0 {
                snes.cpu.P.i = false;
            }
            if flags & 0b1000 != 0 {
                snes.cpu.P.d = false;
            }
            if flags & 0b10000 != 0 {
                snes.cpu.P.x = false;
            }
            if flags & 0b100000 != 0 {
                snes.cpu.P.m = false;
            }
            if flags & 0b1000000 != 0 {
                snes.cpu.P.v = false;
            }
            if flags & 0b10000000 != 0 {
                snes.cpu.P.n = false;
            }
        }
        OpCode::SEP => {
            let flags = memory::read_byte(snes, instruction.data_addr)?;
            if flags & 0b1 != 0 {
                snes.cpu.P.c = true;
            }
            if flags & 0b10 != 0 {
                snes.cpu.P.z = true;
            }
            if flags & 0b100 != 0 {
                snes.cpu.P.i = true;
            }
            if flags & 0b1000 != 0 {
                snes.cpu.P.d = true;
            }
            if flags & 0b10000 != 0 {
                snes.cpu.P.x = true;
            }
            if flags & 0b100000 != 0 {
                snes.cpu.P.m = true;
            }
            if flags & 0b1000000 != 0 {
                snes.cpu.P.v = true;
            }
            if flags & 0b10000000 != 0 {
                snes.cpu.P.n = true;
            }
        }
        OpCode::ORA => {
            snes.cpu.A |= memory::read_word(snes, instruction.data_addr)?;
        }
        OpCode::LDA => {
            if snes.cpu.P.m {
                snes.cpu.A &= 0xFF00;
                let temp = memory::read_byte(snes, instruction.data_addr)?;
                snes.cpu.A |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.A = memory::read_word(snes, instruction.data_addr)?;
                snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.A == 0;
            }
        }
        OpCode::LDX => {
            if snes.cpu.P.x {
                snes.cpu.X &= 0xFF00;
                let temp = memory::read_byte(snes, instruction.data_addr)?;
                snes.cpu.X |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.X = memory::read_word(snes, instruction.data_addr)?;
                snes.cpu.P.n = (snes.cpu.X & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.X == 0;
            }
        }
        OpCode::LDY => {
            if snes.cpu.P.x {
                snes.cpu.Y &= 0xFF00;
                let temp = memory::read_byte(snes, instruction.data_addr)?;
                snes.cpu.Y |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.Y = memory::read_word(snes, instruction.data_addr)?;
                snes.cpu.P.n = (snes.cpu.Y & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.Y == 0;
            }
        }
        OpCode::PHA => {
            if snes.cpu.P.m {
                push_byte(snes, snes.cpu.A.to_le_bytes()[0])?;
            } else {
                push_word(snes, snes.cpu.A)?;
            }
        }
        OpCode::PHK => {
            push_byte(snes, snes.cpu.K)?;
        }
        OpCode::PHB => {
            push_byte(snes, snes.cpu.DBR)?;
        }
        OpCode::PHD => {
            push_word(snes, snes.cpu.D)?;
        }
        OpCode::PHP => {
            push_byte(snes, snes.cpu.p_byte())?;
        }
        OpCode::PHX => {
            if snes.cpu.P.x {
                push_byte(snes, snes.cpu.X.to_le_bytes()[0])?;
            } else {
                push_word(snes, snes.cpu.X)?;
            }
        }
        OpCode::PHY => {
            if snes.cpu.P.x {
                push_byte(snes, snes.cpu.Y.to_le_bytes()[0])?;
            } else {
                push_word(snes, snes.cpu.Y)?;
            }
        }
        OpCode::PLA => {
            if snes.cpu.P.m {
                let temp = pull_byte(snes)?;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.A &= 0xFF00;
                snes.cpu.A |= temp as u16;
            } else {
                let temp = pull_word(snes)?;
                snes.cpu.P.n = (temp & 0x8000) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.A = temp;
            }
        }
        OpCode::PLP => {
            let p = pull_byte(snes)?;
            snes.cpu.set_p(p);
        }
        OpCode::PLB => {
            let temp = pull_byte(snes)?;
            snes.cpu.P.n = (temp & 0x80) != 0;
            snes.cpu.P.z = temp == 0;
            snes.cpu.DBR = temp;
        }
        OpCode::PLD => {
            let temp = pull_word(snes)?;
            snes.cpu.P.n = (temp & 0x8000) != 0;
            snes.cpu.P.z = temp == 0;
            snes.cpu.D = temp;
        }
        OpCode::PLX => {
            if snes.cpu.P.x {
                let temp = pull_byte(snes)?;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.X &= 0xFF00;
                snes.cpu.X |= temp as u16;
            } else {
                let temp = pull_word(snes)?;
                snes.cpu.P.n = (temp & 0x8000) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.X = temp;
            }
        }
        OpCode::PLY => {
            if snes.cpu.P.x {
                let temp = pull_byte(snes)?;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.Y &= 0xFF00;
                snes.cpu.Y |= temp as u16;
            } else {
                let temp = pull_word(snes)?;
                snes.cpu.P.n = (temp & 0x8000) != 0;
                snes.cpu.P.z = temp == 0;
                snes.cpu.Y = temp;
            }
        }
        OpCode::RTS => {
            snes.cpu.PC = pull_word(snes)?;
            snes.cpu.PC += 1;
            return Ok(CPUExecutionResult::Return);
        }
        OpCode::RTL => {
            let pc_l = pull_byte(snes)?;
            let pc_h = pull_byte(snes)?;
            snes.cpu.PC = u16::from_be_bytes([pc_h, pc_l]);
            snes.cpu.K = pull_byte(snes)?;
        }
        OpCode::TAX => {
            if snes.cpu.P.x {
                let temp = (snes.cpu.A & 0xFF) as u8;
                snes.cpu.X &= 0xFF00;
                snes.cpu.X |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.X = snes.cpu.A;
                snes.cpu.P.n = (snes.cpu.X & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.X == 0;
            }
        }
        OpCode::TAY => {
            if snes.cpu.P.x {
                let temp = (snes.cpu.A & 0xFF) as u8;
                snes.cpu.Y &= 0xFF00;
                snes.cpu.Y |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.Y = snes.cpu.A;
                snes.cpu.P.n = (snes.cpu.Y & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.Y == 0;
            }
        }
        OpCode::TSX => {
            if snes.cpu.P.x {
                let temp = (snes.cpu.S & 0xFF) as u8;
                snes.cpu.X &= 0xFF00;
                snes.cpu.X |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.X = snes.cpu.S;
                snes.cpu.P.n = (snes.cpu.X & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.X == 0;
            }
        }
        OpCode::TXS => {
            snes.cpu.S = snes.cpu.X;
            snes.cpu.P.n = (snes.cpu.S & 0x8000) != 0;
            snes.cpu.P.z = snes.cpu.S == 0;
        }
        OpCode::TXA => {
            if snes.cpu.P.m {
                let temp = (snes.cpu.X & 0xFF) as u8;
                snes.cpu.A &= 0xFF00;
                snes.cpu.A |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.A = snes.cpu.X;
                snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.A == 0;
            }
        }
        OpCode::TYA => {
            if snes.cpu.P.m {
                let temp = (snes.cpu.Y & 0xFF) as u8;
                snes.cpu.A &= 0xFF00;
                snes.cpu.A |= temp as u16;
                snes.cpu.P.n = (temp & 0x80) != 0;
                snes.cpu.P.z = temp == 0;
            } else {
                snes.cpu.A = snes.cpu.Y;
                snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
                snes.cpu.P.z = snes.cpu.A == 0;
            }
        }
        OpCode::TCD => {
            snes.cpu.D = snes.cpu.A;
            snes.cpu.P.n = (snes.cpu.D & 0x8000) != 0;
            snes.cpu.P.z = snes.cpu.D == 0;
        }
        OpCode::TDC => {
            snes.cpu.A = snes.cpu.D;
            snes.cpu.P.n = (snes.cpu.A & 0x8000) != 0;
            snes.cpu.P.z = snes.cpu.A == 0;
        }
        OpCode::TRB => {
            if snes.cpu.P.m {
                let val = memory::read_byte(snes, instruction.data_addr)?;
                let result = val & ((snes.cpu.A & 0xFF) ^ 0xFF) as u8;
                memory::write_byte(snes, instruction.data_addr, result)?;
                snes.cpu.P.z = result == 0;
            } else {
                let val = memory::read_word(snes, instruction.data_addr)?;
                let result = val & (snes.cpu.A ^ 0xFFFF);
                memory::write_word(snes, instruction.data_addr, result)?;
                snes.cpu.P.z = result == 0;
            }
        }
        OpCode::TSB => {
            if snes.cpu.P.m {
                let val = memory::read_byte(snes, instruction.data_addr)?;
                let result = val | (snes.cpu.A & 0xFF) as u8;
                memory::write_byte(snes, instruction.data_addr, result)?;
                snes.cpu.P.z = result == 0;
            } else {
                let val = memory::read_word(snes, instruction.data_addr)?;
                let result = val | snes.cpu.A;
                memory::write_word(snes, instruction.data_addr, result)?;
                snes.cpu.P.z = result == 0;
            }
        }
        OpCode::SBC => {
            if snes.cpu.P.d {
                unimplemented!("Decimal mode");
            } else {
                if snes.cpu.P.m {
                    let data = memory::read_byte(snes, instruction.data_addr)?;
                    let a_val = (snes.cpu.A & 0xFF) as u8;
                    let carry = if snes.cpu.P.c { 1 } else { 0 };
                    let result = a_val.wrapping_sub(data).wrapping_sub(1 - carry);

                    snes.cpu.P.z = result == 0;
                    snes.cpu.P.n = (result & 0x80) != 0;
                    snes.cpu.P.c = (a_val as u16 + carry as u16) >= (data as u16 + 1);
                    snes.cpu.P.v = ((a_val ^ data) & 0x80 != 0) && ((a_val ^ result) & 0x80 != 0);

                    snes.cpu.A &= 0xFF00;
                    snes.cpu.A |= result as u16;
                } else {
                    let data = memory::read_word(snes, instruction.data_addr)?;
                    let carry = if snes.cpu.P.c { 1 } else { 0 };
                    let result = snes.cpu.A.wrapping_sub(data).wrapping_sub(1 - carry);

                    snes.cpu.P.z = result == 0;
                    snes.cpu.P.n = (result & 0x8000) != 0;
                    snes.cpu.P.c = (snes.cpu.A as u32 + carry as u32) >= (data as u32 + 1);
                    snes.cpu.P.v = ((snes.cpu.A ^ data) & 0x8000 != 0)
                        && ((snes.cpu.A ^ result) & 0x8000 != 0);

                    snes.cpu.A = result;
                }
            }
        }
        OpCode::XBA => {
            let l = snes.cpu.A & 0xFF;
            let h = (snes.cpu.A & 0xFF00) >> 8;
            snes.cpu.A = (l << 8) | h;
        }
        OpCode::XCE => {
            let temp = snes.cpu.P.c;
            snes.cpu.P.c = snes.cpu.P.e;
            snes.cpu.P.e = temp;

            // When switching to emulation mode, force M=1, X=1, and SH=01
            if snes.cpu.P.e {
                snes.cpu.P.m = true;
                snes.cpu.P.x = true;
                snes.cpu.S = (snes.cpu.S & 0x00FF) | 0x0100;
            }
        }
        _ => todo!("{} Not implemented yet", instruction.opcode),
    }
    snes.cpu.PC += instruction.length(snes.cpu.P.m, snes.cpu.P.x) as u16;
    Ok(CPUExecutionResult::Normal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::{
        CartHardware, Cartridge, ExtraHardware, InterruptVectorTable, MapMode, Region, RomHeader,
        RomSpeed,
    };
    use crate::registers::{DMARegisters, MMIORegisters};
    use crate::Console;

    #[test]
    fn test_addrmode_decode() {
        for x in 0..=0xFF {
            let addrmode = decode_addressing_mode(x);
            match addrmode {
                Result::Ok(_) => assert!(true),
                Err(_) => {
                    assert!(false, "{:02X}: {:?}", x, addrmode);
                }
            }
        }
    }

    #[test]
    fn test_accumulator_mode_opcodes() {
        // Test that shift/rotate and increment/decrement instructions on accumulator
        // use the correct Accumulator addressing mode
        let test_cases = [
            (0x0A, "ASL A"),
            (0x2A, "ROL A"),
            (0x4A, "LSR A"),
            (0x6A, "ROR A"),
            (0x1A, "INC A"),
            (0x3A, "DEC A"),
        ];

        for (opcode, name) in test_cases {
            let result = decode_addressing_mode(opcode).unwrap();
            assert_eq!(
                result,
                AddrMode::Accumulator,
                "{} (0x{:02X}) should be Accumulator mode, got {:?}",
                name,
                opcode,
                result
            );
        }
    }

    fn setup_test_console() -> Console {
        // A minimal console for testing
        let cartridge = Cartridge {
            header: RomHeader {
                title: "Test ROM".to_string(),
                map_mode: MapMode::LoROM,
                rom_speed: RomSpeed::Slow,
                extra_hardware: CartHardware::new(ExtraHardware::RomOnly, None),
                rom_size: 32 * 1024,
                ram_size: 8 * 1024,
                country: Region::NTSC,
                developer_id: 0,
                rom_version: 0,
                checksum_complement: 0,
                checksum: 0,
                interrupt_vectors: InterruptVectorTable {
                    cop: 0,
                    brk: 0xFFE6,
                    abort: 0,
                    nmi: 0,
                    irq: 0xFFEE,
                    cop_emu: 0,
                    brk_emu: 0xFFFE,
                    abort_emu: 0,
                    nmi_emu: 0,
                    reset: 0x8000,
                    irq_emu: 0xFFFE,
                },
                expanded_header: None,
            },
            rom_data: vec![0; 32 * 1024],
        };

        let mut snes = Console {
            cpu: CPU::new(),
            cartridge,
            ram: vec![0; 0x200000],
            mmio: MMIORegisters::default(),
            dma: DMARegisters::default(),
        };
        snes.cpu.P.e = false; // native mode
        snes.cpu.P.m = true; // 8-bit accumulator by default for tests
        snes.cpu.P.x = true; // 8-bit index by default for tests
        snes.cpu.P.i = true; // interrupts disabled by default for tests
        snes.cpu.set_pc(0x7E0000); // Point to RAM instead of ROM
        snes
    }

    // Helper to run a single instruction from a byte slice
    fn run_test_instruction(snes: &mut Console, opcode: &[u8]) -> Result<CPUExecutionResult> {
        // Write opcode to memory at current PC
        for (i, byte) in opcode.iter().enumerate() {
            memory::write_byte(snes, snes.cpu.get_pc() + i as u32, *byte).unwrap();
        }

        let op = memory::read_byte(snes, snes.cpu.get_pc())?;
        let instruction = decode_instruction(snes, op, snes.cpu.get_pc())?;
        execute_instruction(snes, &instruction)
    }

    mod load_store_tests {
        use super::*;

        #[test]
        fn lda_immediate_8bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = true; // 8-bit accumulator
            run_test_instruction(&mut snes, &[0xA9, 0x42]).unwrap();
            assert_eq!(snes.cpu.A & 0xFF, 0x42);
            assert_eq!(snes.cpu.P.z, false);
            assert_eq!(snes.cpu.P.n, false);
        }

        #[test]
        fn lda_immediate_16bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = false; // 16-bit accumulator
            run_test_instruction(&mut snes, &[0xA9, 0x34, 0x12]).unwrap();
            assert_eq!(snes.cpu.A, 0x1234);
            assert_eq!(snes.cpu.P.z, false);
            assert_eq!(snes.cpu.P.n, false);
        }

        #[test]
        fn ldx_immediate_8bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.x = true; // 8-bit index
            run_test_instruction(&mut snes, &[0xA2, 0x42]).unwrap();
            assert_eq!(snes.cpu.X & 0xFF, 0x42);
        }

        #[test]
        fn ldy_absolute_16bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.x = false; // 16-bit index
            snes.cpu.DBR = 0;
            memory::write_word(&mut snes, 0x1234, 0xABCD).unwrap();
            run_test_instruction(&mut snes, &[0xAC, 0x34, 0x12]).unwrap();
            assert_eq!(snes.cpu.Y, 0xABCD);
        }

        #[test]
        fn sta_absolute_16bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = false; // 16-bit accumulator
            snes.cpu.A = 0xABCD;
            snes.cpu.DBR = 0;
            run_test_instruction(&mut snes, &[0x8D, 0x34, 0x12]).unwrap();
            assert_eq!(memory::read_word(&snes, 0x1234).unwrap(), 0xABCD);
        }

        #[test]
        fn stz_direct() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = false; // 16-bit
            snes.cpu.D = 0x1000;
            memory::write_word(&mut snes, 0x1020, 0xFFFF).unwrap();
            run_test_instruction(&mut snes, &[0x64, 0x20]).unwrap();
            assert_eq!(memory::read_word(&snes, 0x1020).unwrap(), 0x0000);
        }
    }

    mod arithmetic_tests {
        use super::*;

        #[test]
        fn adc_immediate_8bit_no_carry() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = true;
            snes.cpu.A = 0x10;
            snes.cpu.P.c = false;
            run_test_instruction(&mut snes, &[0x69, 0x10]).unwrap();
            assert_eq!(snes.cpu.A & 0xFF, 0x20);
            assert!(!snes.cpu.P.c);
            assert!(!snes.cpu.P.v);
        }

        #[test]
        fn sbc_immediate_8bit_borrow() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = true;
            snes.cpu.A = 0x10;
            snes.cpu.P.c = true; // No borrow
            run_test_instruction(&mut snes, &[0xE9, 0x12]).unwrap();
            assert_eq!(snes.cpu.A & 0xFF, 0xFE); // 16 - 18 = -2
            assert!(!snes.cpu.P.c); // Borrow occurred
        }

        #[test]
        fn inx_16bit_wrap() {
            let mut snes = setup_test_console();
            snes.cpu.P.x = false; // 16-bit index
            snes.cpu.X = 0xFFFF;
            run_test_instruction(&mut snes, &[0xE8]).unwrap();
            assert_eq!(snes.cpu.X, 0x0000);
            assert!(snes.cpu.P.z);
        }

        #[test]
        fn dey_8bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.x = true; // 8-bit index
            snes.cpu.Y = 0x01;
            run_test_instruction(&mut snes, &[0x88]).unwrap();
            assert_eq!(snes.cpu.Y & 0xFF, 0x00);
            assert!(snes.cpu.P.z);
        }

        #[test]
        fn dec_absolute() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = true; // 8-bit mode
            snes.cpu.DBR = 0;
            memory::write_byte(&mut snes, 0x1234, 0x80).unwrap();
            run_test_instruction(&mut snes, &[0xCE, 0x34, 0x12]).unwrap();
            assert_eq!(memory::read_byte(&snes, 0x1234).unwrap(), 0x7F);
        }
    }

    mod branch_jump_tests {
        use super::*;

        #[test]
        fn beq_not_taken() {
            let mut snes = setup_test_console();
            snes.cpu.P.z = false;
            let start_pc = snes.cpu.get_pc();
            run_test_instruction(&mut snes, &[0xF0, 0x10]).unwrap();
            assert_eq!(snes.cpu.get_pc(), start_pc + 2);
        }

        #[test]
        fn beq_taken() {
            let mut snes = setup_test_console();
            snes.cpu.P.z = true;
            let start_pc = snes.cpu.get_pc();
            run_test_instruction(&mut snes, &[0xF0, 0x10]).unwrap();
            assert_eq!(snes.cpu.get_pc(), start_pc + 2 + 0x10);
        }

        #[test]
        fn jsr_and_rts() {
            let mut snes = setup_test_console();
            snes.cpu.S = 0x1FF;
            let start_pc = snes.cpu.get_pc();

            // JSR $9034
            run_test_instruction(&mut snes, &[0x20, 0x34, 0x90]).unwrap();
            assert_eq!(snes.cpu.get_pc() & 0xFFFF, 0x9034);
            assert_eq!(snes.cpu.S, 0x1FD);

            // RTS
            run_test_instruction(&mut snes, &[0x60]).unwrap();
            assert_eq!(snes.cpu.get_pc(), start_pc + 3);
            assert_eq!(snes.cpu.S, 0x1FF);
        }
    }

    mod stack_tests {
        use super::*;

        #[test]
        fn pha_pla_16bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = false; // 16-bit
            snes.cpu.A = 0xABCD;
            snes.cpu.S = 0x1FF;

            run_test_instruction(&mut snes, &[0x48]).unwrap(); // PHA
            assert_eq!(snes.cpu.S, 0x1FD);
            snes.cpu.A = 0; // Clear accumulator

            run_test_instruction(&mut snes, &[0x68]).unwrap(); // PLA
            assert_eq!(snes.cpu.S, 0x1FF);
            assert_eq!(snes.cpu.A, 0xABCD);
            assert!(snes.cpu.P.n);
        }

        #[test]
        fn php_plp() {
            let mut snes = setup_test_console();
            snes.cpu.S = 0x1FF;
            snes.cpu.P.c = true;
            snes.cpu.P.n = true;

            run_test_instruction(&mut snes, &[0x08]).unwrap(); // PHP

            snes.cpu.P.c = false;
            snes.cpu.P.n = false;

            run_test_instruction(&mut snes, &[0x28]).unwrap(); // PLP
            assert_eq!(snes.cpu.p_byte(), 0b10110101); // N, M, X, I, C should be set as per initial state
        }
    }

    mod transfer_tests {
        use super::*;

        #[test]
        fn tax_16bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.x = false; // 16-bit index
            snes.cpu.A = 0xABCD;
            run_test_instruction(&mut snes, &[0xAA]).unwrap();
            assert_eq!(snes.cpu.X, 0xABCD);
            assert!(snes.cpu.P.n);
        }

        #[test]
        fn txa_8bit() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = true; // 8-bit accumulator
            snes.cpu.A = 0xFF00; // Will be ignored
            snes.cpu.X = 0x12AD;
            run_test_instruction(&mut snes, &[0x8A]).unwrap();
            assert_eq!(snes.cpu.A & 0xFF, 0xAD);
            assert!(snes.cpu.P.n);
        }

        #[test]
        fn tcd_and_tdc() {
            let mut snes = setup_test_console();
            snes.cpu.A = 0x1234;
            run_test_instruction(&mut snes, &[0x5B]).unwrap(); // TCD
            assert_eq!(snes.cpu.D, 0x1234);

            snes.cpu.A = 0;
            run_test_instruction(&mut snes, &[0x7B]).unwrap(); // TDC
            assert_eq!(snes.cpu.A, 0x1234);
        }
    }

    mod flag_tests {
        use super::*;

        #[test]
        fn clc_sec() {
            let mut snes = setup_test_console();
            snes.cpu.P.c = true;
            run_test_instruction(&mut snes, &[0x18]).unwrap(); // CLC
            assert!(!snes.cpu.P.c);
            run_test_instruction(&mut snes, &[0x38]).unwrap(); // SEC
            assert!(snes.cpu.P.c);
        }

        #[test]
        fn sep_rep_flags() {
            let mut snes = setup_test_console();
            snes.cpu.P.m = false;
            snes.cpu.P.x = false;

            // Set M and X flags
            run_test_instruction(&mut snes, &[0xE2, 0x30]).unwrap(); // SEP #$30
            assert!(snes.cpu.P.m);
            assert!(snes.cpu.P.x);

            // Reset M and X flags
            run_test_instruction(&mut snes, &[0xC2, 0x30]).unwrap(); // REP #$30
            assert!(!snes.cpu.P.m);
            assert!(!snes.cpu.P.x);
        }

        #[test]
        fn xce_native_to_emu() {
            let mut snes = setup_test_console();
            snes.cpu.P.e = false;
            snes.cpu.P.c = true; // C=1 to set E=1
            snes.cpu.P.m = false;
            snes.cpu.P.x = false;
            snes.cpu.S = 0xFFDD;

            run_test_instruction(&mut snes, &[0xFB]).unwrap(); // XCE

            assert!(snes.cpu.P.e);
            assert!(!snes.cpu.P.c); // C becomes old E
            assert!(snes.cpu.P.m); // forced
            assert!(snes.cpu.P.x); // forced
            assert_eq!(snes.cpu.S, 0x01DD); // SH forced to 01
        }
    }
}
