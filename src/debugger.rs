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
    pub endloc: u32,
}

impl Default for DisassemblerContext {
    fn default() -> Self {
        DisassemblerContext {
            lines: Vec::default(),
            branchtable: Vec::default(),
            branchdepth: 0,
            startloc: 0,
            endloc: 0,
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
                            "${:06X}: {} #{:04X}",
                            self.instruction.inst_addr, self.instruction.opcode, self.data,
                        )
                    }
                }
            },
            _ => {
                write!(
                    f,
                    "${:06X}: {} ${:06X} ({}) ",
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
        if instr.opcode == OpCode::BRK
            || instr.opcode.is_return()
            || instr.opcode.is_subroutine()
            || instr.opcode == OpCode::BRA
        {
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
        endloc: snes_sim.cpu.get_pc(),
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
        if currinstr.opcode == OpCode::BRA {
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
    use crate::DMARegisters;
    use crate::MMIORegisters;
    use color_eyre::Result;
    use std::path;

    use super::debug_instructions;
    use super::debug_simulation;
    use super::render_wrapped_instructions;
    use super::*;

    fn create_test_console() -> Console {
        // Create a minimal ROM with some test instructions
        let mut rom_data = vec![0u8; 0x20000]; // 128KB ROM
        
        // Add some simple 65C816 instructions for testing
        // LDA #$1234 (A9 34 12 in 16-bit mode)
        rom_data[0x0000] = 0xA9; // LDA immediate
        rom_data[0x0001] = 0x34; // Low byte
        rom_data[0x0002] = 0x12; // High byte
        
        // NOP (EA)
        rom_data[0x0003] = 0xEA; // NOP
        
        // BRA +2 (80 02)
        rom_data[0x0004] = 0x80; // BRA
        rom_data[0x0005] = 0x02; // Relative offset +2
        
        // Target of branch
        rom_data[0x0008] = 0xEA; // NOP
        
        // BRK (00)
        rom_data[0x0009] = 0x00; // BRK
        
        // Create fake ROM header for LoROM
        let header_start = 0x7FC0;
        let title = b"TEST ROM             ";  // 21 bytes
        rom_data[header_start..header_start + 0x15].copy_from_slice(title);
        rom_data[header_start + 0x15] = 0x20; // LoROM, slow
        rom_data[header_start + 0x17] = 0x08; // 256KB ROM size
        
        // Simple checksum
        let checksum: u16 = rom_data.iter().fold(0u16, |sum, &byte| sum.wrapping_add(byte as u16));
        let checksum_complement = checksum ^ 0xFFFF;
        
        rom_data[header_start + 0x1C] = (checksum_complement & 0xFF) as u8;
        rom_data[header_start + 0x1D] = (checksum_complement >> 8) as u8;
        rom_data[header_start + 0x1E] = (checksum & 0xFF) as u8;
        rom_data[header_start + 0x1F] = (checksum >> 8) as u8;

        let header = cartridge::RomHeader {
            title: "TEST ROM".to_string(),
            map_mode: cartridge::MapMode::LoROM,
            rom_speed: cartridge::RomSpeed::Slow,
            extra_hardware: cartridge::CartHardware::new(cartridge::ExtraHardware::RomOnly, None),
            rom_size: 256 * 1024,
            ram_size: 0,
            country: cartridge::Region::NTSC,
            developer_id: 0,
            rom_version: 1,
            checksum_complement,
            checksum,
            interrupt_vectors: cartridge::InterruptVectorTable {
                cop: 0x8000,
                brk: 0x8000,
                abort: 0x8000,
                nmi: 0x8000,
                irq: 0x8000,
                cop_emu: 0x8000,
                brk_emu: 0x8000,
                abort_emu: 0x8000,
                nmi_emu: 0x8000,
                reset: 0x8000,
                irq_emu: 0x8000,
            },
            expanded_header: None,
        };

        let cartridge = cartridge::Cartridge {
            header,
            rom_data,
        };

        let ram = vec![0; 0x200000];
        
        Console {
            cpu: cpu::CPU::new(),
            cartridge,
            ram,
            dma: DMARegisters::default(),
            mmio: MMIORegisters::default(),
        }
    }

    #[test]
    fn test_debug_state_default() {
        let state = DebugState::default();
        assert_eq!(state.x, false);
        assert_eq!(state.m, false);
        assert_eq!(state.e, false);
    }

    #[test]
    fn test_debug_state_clone() {
        let mut state = DebugState::default();
        state.x = true;
        state.m = true;
        state.e = false;
        
        let cloned = state.clone();
        assert_eq!(cloned.x, true);
        assert_eq!(cloned.m, true);
        assert_eq!(cloned.e, false);
    }

    #[test]
    fn test_flag_enum() {
        let flag_start = Flag::BranchStart(0x8000);
        let flag_cont = Flag::BranchCont(0x8001);
        let flag_end = Flag::BranchEnd(0x8002);
        
        // Test that flags can be created and formatted
        assert_eq!(format!("{:?}", flag_start), "BranchStart(32768)");
        assert_eq!(format!("{:?}", flag_cont), "BranchCont(32769)");
        assert_eq!(format!("{:?}", flag_end), "BranchEnd(32770)");
    }

    #[test]
    fn test_disassembler_line() {
        let mut console = create_test_console();
        console.cpu.set_pc(0x808000);
        console.cpu.P.m = false; // 16-bit accumulator
        
        let line = DisassemblerLine {
            location: 0x808000,
            flags: vec![Flag::BranchStart(0x808000)],
            disassembled: InstructionWrapper {
                location: 0x808000,
                status: DebugState { x: false, m: false, e: false },
                branchfrom: vec![],
                branchto: None,
                data: 0x1234,
                instruction: cpu::InstructionContext {
                    inst_addr: 0x808000,
                    data_addr: 0x808001,
                    dest_addr: None,
                    opcode: cpu::OpCode::LDA,
                    mode: cpu::AddrMode::Immediate,
                },
            },
        };
        
        assert_eq!(line.location, 0x808000);
        assert_eq!(line.flags.len(), 1);
        assert_eq!(line.disassembled.data, 0x1234);
    }

    #[test]
    fn test_instruction_wrapper_display() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: false, e: false },
            branchfrom: vec![],
            branchto: None,
            data: 0x1234,
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808001,
                dest_addr: None,
                opcode: cpu::OpCode::LDA,
                mode: cpu::AddrMode::Immediate,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("$808000"));
        assert!(display_str.contains("LDA"));
        assert!(display_str.contains("#$1234"));
    }

    #[test]
    fn test_instruction_wrapper_accumulator_mode() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: false, e: false },
            branchfrom: vec![],
            branchto: None,
            data: 0,
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808000,
                dest_addr: None,
                opcode: cpu::OpCode::ASL,
                mode: cpu::AddrMode::Accumulator,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("$808000"));
        assert!(display_str.contains("ASL"));
        assert!(display_str.contains("Accumulator"));
    }

    #[test]
    fn test_instruction_wrapper_source_destination() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: false, e: false },
            branchfrom: vec![],
            branchto: None,
            data: 0x1234,
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808001,
                dest_addr: Some(0x808002),
                opcode: cpu::OpCode::MVN,
                mode: cpu::AddrMode::SourceDestination,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("$808000"));
        assert!(display_str.contains("MVN"));
        assert!(display_str.contains("$808001"));
        assert!(display_str.contains("$808002"));
    }

    #[test]
    fn test_disassembler_context_default() {
        let context = DisassemblerContext::default();
        assert_eq!(context.lines.len(), 0);
        assert_eq!(context.branchtable.len(), 0);
        assert_eq!(context.branchdepth, 0);
        assert_eq!(context.startloc, 0);
        assert_eq!(context.endloc, 0);
    }

    #[test]
    fn test_debug_instructions_basic() -> Result<()> {
        let mut console = create_test_console();
        console.cpu.set_pc(0x808000);
        console.cpu.P.m = false; // 16-bit accumulator mode
        console.cpu.P.x = false; // 16-bit index mode
        console.cpu.P.e = false; // Native mode
        
        let instructions = debug_instructions(&console, 0x808000)?;
        
        assert!(!instructions.is_empty());
        assert_eq!(instructions[0].location, 0x808000);
        assert_eq!(instructions[0].instruction.opcode, cpu::OpCode::LDA);
        
        Ok(())
    }

    #[test] 
    fn test_render_wrapped_instructions_empty() {
        let context = DisassemblerContext::default();
        let rendered = render_wrapped_instructions(context);
        
        assert_eq!(rendered.lines.len(), 0);
        assert_eq!(rendered.branchdepth, 0);
    }

    #[test]
    fn test_render_wrapped_instructions_with_branches() {
        let mut context = DisassemblerContext::default();
        
        // Create a line with branch flags
        let line = DisassemblerLine {
            location: 0x808000,
            flags: vec![Flag::BranchStart(0x808000), Flag::BranchCont(0x808001)],
            disassembled: InstructionWrapper {
                location: 0x808000,
                status: DebugState::default(),
                branchfrom: vec![],
                branchto: Some(0x808010),
                data: 0,
                instruction: cpu::InstructionContext {
                    inst_addr: 0x808000,
                    data_addr: 0x808001,
                    dest_addr: None,
                    opcode: cpu::OpCode::BRA,
                    mode: cpu::AddrMode::RelativeByte,
                },
            },
        };
        
        context.lines.push(line);
        context.branchtable.push(0);
        
        let rendered = render_wrapped_instructions(context);
        assert_eq!(rendered.lines.len(), 1);
        assert_eq!(rendered.branchdepth, 2); // Should be set to the max flag count
    }

    #[test]
    fn test_disassembly_error_display() {
        let error = DisassemblyError {
            instructions: vec![],
            status: create_test_console(),
            source: color_eyre::eyre::eyre!("Test error"),
        };
        
        let display_str = format!("{}", error);
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_disassembler_error_display() {
        let error = DisassemblerError::Other(color_eyre::eyre::eyre!("Test error"));
        let display_str = format!("{}", error);
        assert!(display_str.contains("Test error"));
    }

    #[test]
    fn test_debug_simulation_basic() -> Result<()> {
        let mut console = create_test_console();
        console.cpu.set_pc(0x808000);
        console.cpu.P.m = false; // 16-bit mode
        console.cpu.P.x = false; // 16-bit mode
        console.cpu.P.e = false; // Native mode
        
        let context = debug_simulation(&console, 10)?;
        
        assert!(!context.lines.is_empty());
        assert_eq!(context.startloc, 0x808000);
        assert!(context.endloc >= 0x808000);
        
        Ok(())
    }

    #[test]
    fn test_branch_detection() -> Result<()> {
        let mut console = create_test_console();
        console.cpu.set_pc(0x808004); // Start at BRA instruction
        console.cpu.P.m = false;
        console.cpu.P.x = false;
        console.cpu.P.e = false;
        
        let context = debug_simulation(&console, 5)?;
        
        // Should detect the branch instruction
        let has_branch = context.lines.iter().any(|line| {
            line.disassembled.instruction.opcode == cpu::OpCode::BRA
        });
        
        assert!(has_branch);
        Ok(())
    }

    // Conditional test that only runs if the super_metroid.sfc file exists
    #[test]
    #[ignore] // Ignore by default since it requires a specific ROM file
    fn test_debugger_with_real_rom() -> Result<()> {
        if std::path::Path::new("./super_metroid.sfc").exists() {
            let cartridge = cartridge::load_rom(std::path::Path::new("./super_metroid.sfc"), false)?;
            let ram = vec![0; 0x200000];
            let mut snes = Console {
                cpu: cpu::CPU::new(),
                cartridge,
                ram,
                dma: DMARegisters::default(),
                mmio: MMIORegisters::default(),
            };
            snes.cpu.P.e = false;
            snes.cpu.set_pc(0x808423);
            
            let disassembled = debug_simulation(&snes, 100)?;
            let output = render_wrapped_instructions(disassembled);
            
            assert!(!output.lines.is_empty());
            
            for line in output.lines.iter().take(5) { // Just check first 5 lines
                for _ in 0..output.branchdepth.saturating_sub(line.flags.len()) {
                    print!(" ");
                }
                for flag in &line.flags {
                    match flag {
                        Flag::BranchStart(_) => print!("╔"),
                        Flag::BranchCont(_) => print!("║"),
                        Flag::BranchEnd(_) => print!("╚"),
                    }
                }
                println!("{}", line.disassembled);
            }
        } else {
            println!("Skipping test - super_metroid.sfc not found");
        }
        Ok(())
    }

    #[test]
    fn test_instruction_wrapper_with_8bit_immediate() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: true, e: false }, // 8-bit accumulator
            branchfrom: vec![],
            branchto: None,
            data: 0x34,
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808001,
                dest_addr: None,
                opcode: cpu::OpCode::LDA,
                mode: cpu::AddrMode::Immediate,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("$808000"));
        assert!(display_str.contains("LDA"));
        // Should display as 8-bit immediate in 8-bit mode
        assert!(display_str.contains("#$34") || display_str.contains("#$0034"));
    }

    #[test] 
    fn test_instruction_wrapper_rep_instruction() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: false, e: false },
            branchfrom: vec![],
            branchto: None,
            data: 0x30, // REP #$30 (enable 16-bit A and X/Y)
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808001,
                dest_addr: None,
                opcode: cpu::OpCode::REP,
                mode: cpu::AddrMode::Immediate,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("REP"));
        assert!(display_str.contains("#$30"));
    }

    #[test]
    fn test_instruction_wrapper_pea_instruction() {
        let wrapper = InstructionWrapper {
            location: 0x808000,
            status: DebugState { x: false, m: false, e: false },
            branchfrom: vec![],
            branchto: None,
            data: 0x1234,
            instruction: cpu::InstructionContext {
                inst_addr: 0x808000,
                data_addr: 0x808001,
                dest_addr: None,
                opcode: cpu::OpCode::PEA,
                mode: cpu::AddrMode::Immediate,
            },
        };
        
        let display_str = format!("{}", wrapper);
        assert!(display_str.contains("PEA"));
        assert!(display_str.contains("#$1234")); // PEA always uses 16-bit immediate
    }
}
