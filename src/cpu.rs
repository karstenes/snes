use std::ops::Add;

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

pub enum AddrMode {
    Absolute,
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

fn calculate_cycles(snes: &Console, opcode: &OpCode) -> Result<u8> {

    Ok(0)
}

fn execute_opcode(snes: &Console, instruction: OpCode, mode: AddrMode) -> Result<()> {

    Ok(())
}

pub fn interpret_opcode(snes: &mut Console, instruction: u8) -> Result<InstructionContext> {
    let (opcode, mode, address) = match instruction {
        0x00 => (OpCode::BRK, AddrMode::Implied, 0),
        0x02 => (OpCode::COP, AddrMode::Immediate, (snes.cpu.PC + (snes.cpu.K << 16) + 1)),
        _ => {
            unimplemented!()
        }
    };
    let cycles = calculate_cycles(&snes, &opcode)?;

    Ok(InstructionContext{
        opcode,
        mode, 
        address, 
        cycles})
}