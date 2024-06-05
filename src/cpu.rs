use super::Console;
use anyhow::{bail, ensure, Result};

#[derive(Debug)]
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
            OpCode::XCE => "XCE"
        };
        write!(f, "{}", instr)
    }
}

#[derive(Debug)]
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

impl AddrMode {
    fn length(&self, m: bool, x: bool) -> usize {
        match self {
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
            AddrMode::Immediate => 3 - ((m as usize) | (x as usize)),
            AddrMode::Implied => 1,
            AddrMode::Long => 4,
            AddrMode::LongX => 4,
            AddrMode::RelativeByte => 2,
            AddrMode::RelativeWord => 3,
            AddrMode::SourceDestination => 3,
            AddrMode::Stack => 2,
            AddrMode::StackIndexed => 2
        }
    }
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
            AddrMode::StackIndexed => "(Stack, S), Y"
        };
        write!(f, "{}", mode)
    }
}

#[derive(Debug)]
pub struct Flags {
    /// Negative
    n: bool,
    /// Overflow
    v: bool,
    /// Memory width
    m: bool,
    /// Index register width
    x: bool,
    /// Decimal mode
    d: bool,
    /// Interrupt disable
    i: bool,
    /// Zero
    z: bool,
    /// Carry
    c: bool,
    /// Emulation mode
    e: bool,
    /// Break
    b: bool,
}

#[derive(Debug)]
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
    pub DBR: u16,
    /// Direct Addressing Register (16 bit)
    pub D: u16,
    /// Program Bank Register (8 bit, but stored as 32 bits to speed up emulation)
    pub K: u32,
    /// Flags Register
    pub P: Flags,
    /// Program Counter (16 bit, but stored as 32 bits to speed up emulation)
    pub PC: u32,
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
            e: false,
            b: false,
        }
    }
}

impl std::fmt::Display for Flags {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flagstring = String::default();
        flagstring.push(if self.n {'N'} else {'n'});
        flagstring.push(if self.v {'V'} else {'v'});
        flagstring.push(if self.m {'M'} else {'m'});
        flagstring.push(if self.x {'X'} else {'x'});
        flagstring.push(if self.d {'D'} else {'d'});
        flagstring.push(if self.i {'I'} else {'i'});
        flagstring.push(if self.z {'Z'} else {'z'});
        flagstring.push(if self.c {'C'} else {'c'});
        flagstring.push(if self.e {'E'} else {'e'});
        flagstring.push(if self.b {'B'} else {'b'});
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
}

impl std::fmt::Display for CPU {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A:   {:04X}\nX:   {:04X}\nY:   {:04X}\nS:   {:04X}\nDBR: {:04X}\nD:   {:04X}\nK:   {:04X}\nP:   {}\nPC:  ${:06X}", 
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

#[derive(Debug)]
pub struct InstructionContext {
    opcode: OpCode,
    mode: AddrMode,
    inst_addr: u32,
    data_addr: u32,
    cycles: u8,
}

impl std::fmt::Display for InstructionContext {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:06X}: {}, {}", self.inst_addr, self.opcode, self.mode)
    }
}

pub fn decode_addressing_mode(opcode: u8) -> Result<AddrMode> {
    let aaa = (opcode & 0b11100000) >> 5;
    let bbb = (opcode & 0b00011100) >> 2;
    let cc = opcode & 0b00000011;

    match opcode {
        0x00 | 0x08 | 0x0B | 0x10 | 0x18 | 0x1A | 0x1B | 0x28 | 0x2B | 0x30 | 0x38 | 0x3A
        | 0x3B | 0x40 | 0x48 | 0x4B | 0x50 | 0x58 | 0x5A | 0x5B | 0x60 | 0x68 | 0x6B
        | 0x70 | 0x78 | 0x7A | 0x7B | 0x88 | 0x8A | 0x8B | 0x90 | 0x98 | 0x9A | 0x9B | 0xA8
        | 0xAA | 0xAB | 0xB0 | 0xB8 | 0xBA | 0xBB | 0xC8 | 0xCA | 0xCB | 0xD0 | 0xD8 | 0xDA
        | 0xDB | 0xE8 | 0xEA | 0xEB | 0xF0 | 0xF8 | 0xFA | 0xFB => return Ok(AddrMode::Implied), // Single byte instructions
        0x14 | 0x64 | 0xD4 => return Ok(AddrMode::Direct), // TRB zp, STZ zp, PEI dir
        0x1C | 0x20 | 0x9C => return Ok(AddrMode::Absolute), // TRB abs, JSR abs, STZ abs
        0x22 | 0x5C => return Ok(AddrMode::Long),          // JMP long,
        0x44 | 0x52 => return Ok(AddrMode::SourceDestination), // MVN src,dest, MVP src,dest
        0xDC => return Ok(AddrMode::AbsoluteSWord),
        0x74 => return Ok(AddrMode::DirectX), // STZ zp,X
        0x7C | 0xFC => return Ok(AddrMode::AbsoluteIndexedIndirect), // JMP (abs,X), JSR (abs,X)
        0x80 => return Ok(AddrMode::RelativeByte), // BRA rel8
        0x82 => return Ok(AddrMode::RelativeWord), // BRL rel16
        0x02 | 0x42 | 0x62 | 0x89 | 0xC2 | 0xE2 | 0xF4 => return Ok(AddrMode::Immediate), // COP immed, PER immed, BIT immed, REP immed, SEP immed, PEA immed
        0x9E => return Ok(AddrMode::AbsoluteX),                                    // STZ abs,X
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

fn calculate_cycles(snes: &Console, opcode: &OpCode) -> Result<u8> {
    Ok(0)
}

fn calculate_address(snes: &Console, mode: &AddrMode) -> Result<u32> {
    // TODO Implement Address Calculations
    Ok(0)
}

fn execute_opcode(snes: &Console, instruction: OpCode, mode: AddrMode) -> Result<()> {
    Ok(())
}

pub fn decode_instruction(snes: &mut Console, instruction: u8) -> Result<InstructionContext> {
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
    let address = calculate_address(&snes, &mode)?;
    let cycles = calculate_cycles(&snes, &opcode)?;

    Ok(InstructionContext {
        opcode,
        mode,
        inst_addr: snes.cpu.PC,
        data_addr: address,
        cycles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addrmode_decode() {
        for x in 0..=0xFF {
            let addrmode = decode_addressing_mode(x);
            match addrmode {
                Ok(_) => assert!(true),
                Err(_) => {
                    assert!(false, "{:02X}: {:?}", x, addrmode);
                }
            }
        }
    }
}
