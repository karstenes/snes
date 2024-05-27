use super::Console;
use anyhow::{Result, ensure, bail};

#[derive(Debug)]
pub enum OpCode {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL,
    BRA, BRK, BRL, BVC, BVS, CLC, CLD, CLI, CLV, CMP,
    CPX, CPY, COP, DEC, DEX, DEY, EOR, INC, INX, INY,
    JMP, JML, JSR, JSL, LDA, LDX, LDY, LSR, MVN, MVP,
    NOP, ORA, PEA, PEI, PER, PHA, PHD, PHK, PHX, PHY,
    PLA, PLD, PLP, PLX, PLY, REP, ROL, ROR, RTI, RTS,
    RTL, SBC, SEC, SED, SEI, SEP, STA, STX, STY, STP,
    STZ, TAX, TAY, TCD, TCS, TDC, TSC, TSX, TXA, TXS,
    TXY, TYA, TYX, TRB, TSB, WAI, XCE
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
    StackIndexed
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
    b: bool
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
    pub PC: u32    
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
            b: false
        }
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
            PC: 0
        }
    }
}

pub struct InstructionContext {
    opcode: OpCode,
    mode: AddrMode,
    address: u32,
    cycles: u8
}

pub fn decode_addressing_mode(opcode: u8) -> Result<AddrMode> {
    let aaa = (opcode & 0b11100000) >> 5;
    let bbb = (opcode & 0b00011100) >> 2;
    let cc = opcode & 0b00000011;

    match opcode {
        0x00 | 0x08 | 0x0B | 0x10 | 0x18 | 0x1A | 0x1B | 0x28 | 0x2B | 0x30 | 0x38 | 0x3A | 0x3B | 0x40 | 0x42 | 0x48 | 0x4B | 0x50 |
        0x58 | 0x5A | 0x5B | 0x60 | 0x68 | 0x6B | 0x70 | 0x78 | 0x7A | 0x7B | 0x88 | 0x8A | 0x8B | 0x90 | 0x98 | 0x9A | 0x9B |
        0xA8 | 0xAA | 0xAB | 0xB0 | 0xB8 | 0xBA | 0xBB | 0xC8 | 0xCA | 0xCB | 0xD0 | 0xD8 | 0xDA | 0xDB | 0xE8 | 0xEA | 0xEB |
        0xF0 | 0xF8 | 0xFA | 0xFB => return Ok(AddrMode::Implied), // Single byte instructions
        0x14 | 0x64 | 0xD4 => return Ok(AddrMode::Direct), // TRB zp, STZ zp, PEI dir
        0x1C | 0x20 | 0x9C => return Ok(AddrMode::Absolute), // TRB abs, JSR abs, STZ abs
        0x22 | 0x5C => return Ok(AddrMode::Long), // JMP long,
        0x44 | 0x52 => return Ok(AddrMode::SourceDestination), // MVN src,dest, MVP src,dest
        0xDC => return Ok(AddrMode::AbsoluteSWord),
        0x74 => return Ok(AddrMode::DirectX), // STZ zp,X
        0x7C | 0xFC => return Ok(AddrMode::AbsoluteIndexedIndirect), // JMP (abs,X), JSR (abs,X)
        0x80 => return Ok(AddrMode::RelativeByte), // BRA rel8
        0x82 => return Ok(AddrMode::RelativeWord), // BRL rel16
        0x02 | 0x62 | 0x89 | 0xC2 | 0xE2 | 0xF4 => return Ok(AddrMode::Immediate), // COP immed, PER immed, BIT immed, REP immed, SEP immed, PEA immed
        0x9E => return Ok(AddrMode::AbsoluteX), // STZ abs,X
        _ => ()
    }

    let mode: AddrMode = match cc {
        0b00 => {
            match bbb {
                0b000 => AddrMode::Immediate,
                0b001 => AddrMode::Direct,
                0b010 => bail!("Unknown opcode {:02X}", opcode),
                0b011 => AddrMode::Absolute,
                0b100 => bail!("Unknown opcode {:02X}", opcode),
                0b101 => AddrMode::DirectX,
                0b110 => bail!("Unknown opcode {:02X}", opcode),
                0b111 => AddrMode::AbsoluteX,
                _ => unreachable!()
            }
        },
        0b01 => {
            match bbb {
                0b000 => AddrMode::IndexedDirectWord,
                0b001 => AddrMode::Direct,
                0b010 => AddrMode::Immediate,
                0b011 => AddrMode::Absolute,
                0b100 => AddrMode::DirectIndexedWord,
                0b101 => AddrMode::DirectX,
                0b110 => AddrMode::AbsoluteY,
                0b111 => AddrMode::AbsoluteX,
                _ => unreachable!()
            }
        },
        0b10 => {
            match bbb {
                0b000 => AddrMode::Immediate,
                0b001 => AddrMode::Direct,
                0b010 => AddrMode::Accumulator,
                0b011 => AddrMode::Absolute,
                0b100 => AddrMode::DirectWord,
                0b101 => AddrMode::DirectX,
                0b110 => bail!("Unknown opcode {:02X}", opcode),
                0b111 => AddrMode::AbsoluteX,
                _ => unreachable!()
            }
        }
        0b11 => {
            match bbb {
                0b000 => AddrMode::Stack,
                0b001 => AddrMode::DirectSWord,
                0b010 => bail!("Unknown opcode {:02X}", opcode),
                0b011 => AddrMode::Long,
                0b100 => AddrMode::StackIndexed,
                0b101 => AddrMode::DirectIndexedSWord,
                0b110 => bail!("Unknown opcode {:02X}", opcode),
                0b111 => AddrMode::LongX,
                _ => unreachable!()
            }
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
    todo!("Implement Address Calculations")
}

fn execute_opcode(snes: &Console, instruction: OpCode, mode: AddrMode) -> Result<()> {

    Ok(())
}

pub fn interpret_opcode(snes: &mut Console, instruction: u8) -> Result<InstructionContext> {
    let (opcode, mode) = match instruction {
        0x00 => (OpCode::BRK, AddrMode::Implied),
        0x02 => (OpCode::COP, AddrMode::Immediate),
        0x21 => (OpCode::AND, AddrMode::IndexedDirectWord),
        0x23 => (OpCode::AND, AddrMode::Stack),
        0x25 => (OpCode::AND, AddrMode::Direct),
        0x27 => (OpCode::AND, AddrMode::DirectSWord),
        0x29 => (OpCode::AND, AddrMode::Immediate),
        0x2D => (OpCode::AND, AddrMode::Absolute),
        0x2F => (OpCode::AND, AddrMode::Long),
        0x31 => (OpCode::AND, AddrMode::DirectIndexedWord),
        0x32 => (OpCode::AND, AddrMode::DirectWord),
        0x33 => (OpCode::AND, AddrMode::StackIndexed),
        0x35 => (OpCode::AND, AddrMode::DirectX),
        0x37 => (OpCode::AND, AddrMode::DirectIndexedSWord),
        0x39 => (OpCode::AND, AddrMode::AbsoluteY),
        0x3D => (OpCode::AND, AddrMode::AbsoluteX),
        0x3F => (OpCode::AND, AddrMode::LongX),
        0x61 => (OpCode::ADC, AddrMode::IndexedDirectWord),
        0x63 => (OpCode::ADC, AddrMode::Stack),
        0x65 => (OpCode::ADC, AddrMode::Direct),
        0x67 => (OpCode::ADC, AddrMode::DirectSWord),
        0x69 => (OpCode::ADC, AddrMode::Immediate),
        0x6D => (OpCode::ADC, AddrMode::Absolute),
        0x6F => (OpCode::ADC, AddrMode::Long),
        0x71 => (OpCode::ADC, AddrMode::DirectIndexedWord),
        0x72 => (OpCode::ADC, AddrMode::DirectWord),
        0x73 => (OpCode::ADC, AddrMode::StackIndexed),
        0x75 => (OpCode::ADC, AddrMode::DirectX),
        0x77 => (OpCode::ADC, AddrMode::DirectIndexedSWord),
        0x79 => (OpCode::ADC, AddrMode::AbsoluteY),
        0x7D => (OpCode::ADC, AddrMode::AbsoluteX),
        0x7F => (OpCode::ADC, AddrMode::LongX),
        _ => {
            unimplemented!()
        }
    };
    let address = calculate_address(&snes, &mode)?;
    let cycles = calculate_cycles(&snes, &opcode)?;

    Ok(InstructionContext{
        opcode,
        mode, 
        address, 
        cycles})
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