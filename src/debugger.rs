use super::*;
use crate::cpu::*;
use ahash::AHashMap;
use color_eyre::Result;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct InstructionWrapper {
    location: u32,
    status: DebugState,
    branchfrom: Vec<u32>,
    branchto: Option<u32>,
    data: u16,
    instruction: InstructionContext,
}

#[derive(Debug, Clone)]
pub enum Flag {
    BranchStart(u32),
    BranchCont(u32),
    BranchEnd(u32),
}

#[derive(Debug, Clone)]
pub struct DisassemblerLine {
    pub location: u32,
    pub flags: Vec<Flag>,
    pub disassembled: InstructionWrapper,
}

#[derive(Debug, Clone)]
pub struct DisassemblerContext {
    pub lines: Vec<DisassemblerLine>,
    pub branchtable: Vec<usize>,
    pub branchdepth: usize,
    pub startloc: u32,
}

impl Default for DisassemblerContext {
    fn default() -> Self {
        DisassemblerContext {
            lines: Vec::default(),
            branchtable: Vec::default(),
            branchdepth: 0,
            startloc: 0,
        }
    }
}

impl std::fmt::Display for InstructionWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.instruction.mode {
            AddrMode::SourceDestination => {
                write!(
                    f,
                    "${:06X}: {} {:06X}, {:06X} ({})",
                    self.instruction.inst_addr,
                    self.instruction.opcode,
                    self.instruction.data_addr,
                    self.instruction.dest_addr.unwrap(),
                    self.instruction.mode
                )
            }
            AddrMode::Accumulator | AddrMode::Implied => {
                write!(
                    f,
                    "${:06X}: {} ({})",
                    self.instruction.inst_addr, self.instruction.opcode, self.instruction.mode
                )
            }
            AddrMode::Immediate => match &self.instruction.opcode {
                OpCode::REP | OpCode::SEP | OpCode::WDM => {
                    write!(
                        f,
                        "${:06X}: {} #{:02X} ({})",
                        self.instruction.inst_addr,
                        self.instruction.opcode,
                        self.data & 0xFF,
                        self.instruction.mode
                    )
                }
                OpCode::PEA | OpCode::PER => {
                    write!(
                        f,
                        "${:06X}: {} #{:04X} ({})",
                        self.instruction.inst_addr,
                        self.instruction.opcode,
                        self.data,
                        self.instruction.mode
                    )
                }
                _ => {
                    if self.status.m {
                        write!(
                            f,
                            "${:06X}: {} #{:02X} ({})",
                            self.instruction.inst_addr,
                            self.instruction.opcode,
                            self.data & 0xFF,
                            self.instruction.mode
                        )
                    } else {
                        write!(
                            f,
                            "${:06X}: {} #{:04X} ({})",
                            self.instruction.inst_addr,
                            self.instruction.opcode,
                            self.data,
                            self.instruction.mode
                        )
                    }
                }
            },
            _ => {
                write!(
                    f,
                    "${:06X}: {} ${:06X} ({})",
                    self.instruction.inst_addr,
                    self.instruction.opcode,
                    self.instruction.data_addr,
                    self.instruction.mode
                )
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugState {
    x: bool,
    m: bool,
    e: bool,
}

#[derive(Debug, Error)]
pub(crate) enum DisassemblerError {
    DisassemblyError(DisassemblyError),
    #[error(transparent)]
    Other(#[from] color_eyre::eyre::Report),
}

#[derive(Debug)]
pub(crate) struct DisassemblyError {
    pub(crate) instructions: Vec<InstructionWrapper>,
    pub(crate) status: Console,
    pub(crate) source: color_eyre::eyre::Error,
}

impl std::fmt::Display for DisassemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = &self.instructions.len();
        for (i, instr) in self.instructions.iter().enumerate() {
            if i == len - 1 {
                write!(f, "=> {:} <=\n", instr)?;
            } else {
                write!(f, "{:}\n", instr)?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for DisassemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisassemblerError::DisassemblyError(e) => e.fmt(f),
            DisassemblerError::Other(e) => e.fmt(f),
        }
    }
}

fn sets_m(snes: &Console, stack: &Vec<DebugState>, instr: &InstructionContext) -> Result<bool> {
    Ok(match instr.opcode {
        OpCode::SEP => (memory::peek_byte(snes, instr.data_addr)? & 0x20) != 0,
        OpCode::PLP => {
            if !stack.is_empty() {
                if snes.cpu.P.e {
                    (memory::peek_byte(snes, (snes.cpu.S + 1) as u32)? & 0x20) != 0
                } else {
                    (memory::peek_byte(
                        snes,
                        u32::from_be_bytes([0x00, 0x00, 0x01, snes.cpu.S as u8]) + 1,
                    )? & 0x20)
                        != 0
                }
            } else {
                stack.last().unwrap().m
            }
        }
        _ => false,
    })
}

fn clears_m(snes: &Console, stack: &Vec<DebugState>, instr: &InstructionContext) -> Result<bool> {
    Ok(match instr.opcode {
        OpCode::REP => (memory::peek_byte(snes, instr.data_addr)? & 0x20) != 0,
        OpCode::PLP => {
            if !stack.is_empty() {
                if snes.cpu.P.e {
                    (memory::peek_byte(snes, (snes.cpu.S + 1) as u32)? & 0x20) == 0
                } else {
                    (memory::peek_byte(
                        snes,
                        u32::from_be_bytes([0x00, 0x00, 0x01, snes.cpu.S as u8]) + 1,
                    )? & 0x20)
                        == 0
                }
            } else {
                !stack.last().unwrap().m
            }
        }
        _ => false,
    })
}

fn sets_x(snes: &Console, stack: &Vec<DebugState>, instr: &InstructionContext) -> Result<bool> {
    Ok(match instr.opcode {
        OpCode::SEP => (memory::peek_byte(snes, instr.data_addr)? & 0x10) != 0,
        OpCode::PLP => {
            if !stack.is_empty() {
                if snes.cpu.P.e {
                    (memory::peek_byte(snes, (snes.cpu.S + 1) as u32)? & 0x10) != 0
                } else {
                    (memory::peek_byte(
                        snes,
                        u32::from_be_bytes([0x00, 0x00, 0x01, snes.cpu.S as u8]) + 1,
                    )? & 0x10)
                        != 0
                }
            } else {
                stack.last().unwrap().x
            }
        }
        _ => false,
    })
}

fn clears_x(snes: &Console, stack: &Vec<DebugState>, instr: &InstructionContext) -> Result<bool> {
    Ok(match instr.opcode {
        OpCode::SEP => (memory::peek_byte(snes, instr.data_addr)? & 0x10) == 0,
        OpCode::PLP => {
            if !stack.is_empty() {
                if snes.cpu.P.e {
                    (memory::peek_byte(snes, (snes.cpu.S + 1) as u32)? & 0x10) == 0
                } else {
                    (memory::peek_byte(
                        snes,
                        u32::from_be_bytes([0x00, 0x00, 0x01, snes.cpu.S as u8]) + 1,
                    )? & 0x10)
                        == 0
                }
            } else {
                !stack.last().unwrap().x
            }
        }
        _ => false,
    })
}

fn clears_e(snes: &Console, instr: &InstructionContext) -> bool {
    match instr.opcode {
        OpCode::XCE => snes.cpu.P.c,
        _ => false,
    }
}

pub fn debug_simulation(
    snes: &Console,
    maxlines: usize,
) -> Result<DisassemblerContext, DisassemblerError> {
    let mut snes_sim = snes.clone();
    let start = snes.cpu.get_pc();
    let mut cycle = 0;
    let mut instructions: Vec<InstructionWrapper> = Vec::with_capacity(30);
    let mut knowninstructions: AHashMap<u32, usize> = AHashMap::new();
    let mut branchinstructions = Vec::<usize>::default();
    loop {
        let op = memory::read_byte(&snes_sim, snes_sim.cpu.get_pc())?;
        let instr = cpu::decode_instruction(&snes_sim, op, snes_sim.cpu.get_pc())?;
        let branchto = if instr.opcode.is_branch() && instr.data_addr > start {
            branchinstructions.push(cycle);
            Some(instr.data_addr)
        } else {
            None
        };
        instructions.push(InstructionWrapper {
            location: snes_sim.cpu.get_pc(),
            status: DebugState {
                x: snes_sim.cpu.P.x,
                m: snes_sim.cpu.P.m,
                e: snes_sim.cpu.P.e,
            },
            branchfrom: Vec::<u32>::default(),
            branchto: branchto,
            data: match instr.mode {
                AddrMode::Immediate => memory::read_word(&snes_sim, instr.data_addr)?,
                _ => 0x00,
            },
            instruction: instr.clone(),
        });
        knowninstructions.insert(snes_sim.cpu.get_pc(), cycle);
        if instr.opcode.is_branch()
            || instr.opcode.is_jump()
            || instr.opcode.is_return()
            || instr.opcode.is_subroutine()
        {
            snes_sim.cpu.PC += instr.length(snes.cpu.P.m, snes.cpu.P.x) as u16;
        } else {
            let res = match cpu::execute_instruction(&mut snes_sim, &instr) {
                Ok(x) => x,
                Err(e) => {
                    return Err(DisassemblerError::DisassemblyError(DisassemblyError {
                        instructions: instructions.clone(),
                        status: snes_sim.clone(),
                        source: e,
                    }))
                }
            };
        }
        if cycle > maxlines {
            break;
        };
        cycle += 1;
        if instr.opcode == OpCode::BRK {
            break;
        }
        if instr.opcode.is_subroutine() {
            break;
        }
        if instr.opcode.is_return() {
            break;
        }
    }
    'calcbranch: for instr in branchinstructions.iter() {
        let currbranchdest = instructions[*instr].branchto.unwrap();
        let currbranchloc = instructions[*instr].location;
        if currbranchdest > instructions.last().unwrap().location {
            instructions[*instr].branchto = None;
            continue 'calcbranch;
        }
        if knowninstructions.contains_key(&currbranchdest) {
            let branchtarget = knowninstructions[&currbranchdest];
            instructions[branchtarget].branchfrom.push(currbranchloc);
        }
    }
    let disassembler_lines = instructions
        .iter()
        .map(|x| DisassemblerLine {
            location: x.location,
            flags: Vec::default(),
            disassembled: x.clone(),
        })
        .collect();
    return Ok(DisassemblerContext {
        lines: disassembler_lines,
        branchdepth: 0,
        branchtable: branchinstructions,
        startloc: start,
    });
}

pub fn debug_instructions(snes: &Console, start: u32) -> Result<Vec<InstructionWrapper>> {
    let mut cycle = 0;
    let mut state = DebugState::default();
    state.x = snes.cpu.P.x;
    state.m = snes.cpu.P.m;
    state.e = snes.cpu.P.e;
    let mut instructions = Vec::<InstructionWrapper>::default();
    let mut temppc = start;
    let mut dbr = snes.cpu.DBR;
    let mut tempstack = Vec::<DebugState>::new();
    let mut knowninstructions: AHashMap<u32, usize> = AHashMap::new();
    let mut branchinstructions = Vec::<usize>::default();
    loop {
        let opcode = memory::peek_byte(snes, temppc)?;
        let mut tempsnes = snes.clone();
        tempsnes.cpu.set_pc(temppc);
        tempsnes.cpu.DBR = dbr;
        tempsnes.cpu.P.x = state.x;
        tempsnes.cpu.P.m = state.m;
        tempsnes.cpu.P.e = state.e;
        let currinstr = decode_instruction(&tempsnes, opcode, temppc)?;
        let branchto = if currinstr.opcode.is_branch() && currinstr.data_addr > start {
            branchinstructions.push(cycle);
            Some(currinstr.data_addr)
        } else {
            None
        };

        instructions.push(InstructionWrapper {
            location: temppc,
            status: state.clone(),
            branchfrom: Vec::<u32>::default(),
            branchto,
            data: match currinstr.mode {
                AddrMode::Immediate => memory::read_word(snes, currinstr.data_addr)?,
                _ => 0,
            },
            instruction: currinstr.clone(),
        });
        knowninstructions.insert(temppc, cycle);
        if currinstr.opcode.is_jump() {
            break;
        }
        if currinstr.opcode.is_return() {
            break;
        }
        if currinstr.opcode == OpCode::BRK {
            break;
        }
        if currinstr.opcode == OpCode::PLB {
            dbr = memory::read_byte(&tempsnes, currinstr.data_addr)?
        }
        if cycle >= 30 {
            break;
        }
        temppc += currinstr.length(state.m, state.x) as u32;
        cycle += 1;
        if sets_m(&tempsnes, &tempstack, &currinstr)? {
            state.m = true
        }
        if sets_x(&tempsnes, &tempstack, &currinstr)? {
            state.x = true
        }
        if clears_m(&tempsnes, &tempstack, &currinstr)? {
            state.m = false
        }
        if clears_x(&tempsnes, &tempstack, &currinstr)? {
            state.x = false
        }
        if clears_e(&tempsnes, &currinstr) {
            state.e = false
        }
    }
    for instr in branchinstructions.iter() {
        let currbranchdest = instructions[*instr].branchto.unwrap();
        let currbranchloc = instructions[*instr].location;
        if knowninstructions.contains_key(&currbranchdest) {
            let branchtarget = knowninstructions[&currbranchdest];
            instructions[branchtarget].branchfrom.push(currbranchloc);
        }
    }
    Ok(instructions)
}

fn loop_to_target(
    mut context: DisassemblerContext,
    start: usize,
    target: u32,
) -> DisassemblerContext {
    let mut i;
    let up = if context.lines[start].disassembled.location > target {
        i = start - 1;
        let newflag = Flag::BranchEnd(context.lines[start].disassembled.location);
        context.lines[start].flags.push(newflag);
        true
    } else {
        i = start + 1;
        let newflag = Flag::BranchStart(context.lines[start].disassembled.location);
        context.lines[start].flags.push(newflag);
        false
    };
    loop {
        if context.lines[i].disassembled.location == target {
            match up {
                true => {
                    let newflag = Flag::BranchStart(context.lines[i].disassembled.location);
                    context.lines[i].flags.push(newflag);
                }
                false => {
                    let newflag = Flag::BranchEnd(context.lines[i].disassembled.location);
                    context.lines[i].flags.push(newflag);
                }
            };
            break;
        } else {
            let newflag = Flag::BranchCont(context.lines[i].disassembled.location);
            context.lines[i].flags.push(newflag);
        }
        match up {
            true => i -= 1,
            false => i += 1,
        };
    }
    return context;
}

pub fn render_wrapped_instructions(mut context: DisassemblerContext) -> DisassemblerContext {
    for branch in context.branchtable.clone() {
        if let None = context.lines[branch].disassembled.branchto {
            continue;
        };
        let target = context.lines[branch].disassembled.branchto.unwrap();
        context = loop_to_target(context, branch, target);
    }
    context.branchdepth = 0;
    for line in context.lines.iter() {
        if line.flags.len() > context.branchdepth {
            context.branchdepth = line.flags.len();
        }
    }
    return context;
}

// ╔║╚

mod tests {
    #![allow(unused_imports)]
    use crate::cartridge;
    use crate::cpu;
    use crate::debugger::Flag;
    use crate::Console;
    use crate::MMIORegisters;
    use color_eyre::Result;
    use std::path;

    use super::debug_instructions;
    use super::debug_simulation;
    use super::render_wrapped_instructions;
    #[test]
    fn test_debugger() -> Result<()> {
        let cartridge = cartridge::load_rom(path::Path::new("./super_metroid.sfc"))?;
        let mut ram = vec![0; 0x200000];
        let mut snes = Console {
            cpu: cpu::CPU::new(),
            cartridge,
            ram,
            mmio: MMIORegisters::default(),
        };
        snes.cpu.P.e = false;
        snes.cpu.set_pc(0x808423);
        let disassembled = debug_simulation(&snes, 100)?;
        let output = render_wrapped_instructions(disassembled);
        for line in output.lines {
            for _ in 0..output.branchdepth - line.flags.len() {
                print!(" ");
            }
            for flag in line.flags {
                match flag {
                    Flag::BranchStart(_) => print!("╔"),
                    Flag::BranchCont(_) => print!("║"),
                    Flag::BranchEnd(_) => print!("╚"),
                }
            }
            println!("{:}", line.disassembled);
        }
        Ok(())
    }
}
