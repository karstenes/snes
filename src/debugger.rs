use std::{borrow::BorrowMut, vec};

use super::*;
use crate::cpu::*;
use anyhow::Result;
use ahash::AHashMap;

#[derive(Debug, Clone)]
pub struct InstructionWrapper {
    location: u32,
    branchfrom: Vec<u32>,
    branchto: Option<u32>,
    instruction: InstructionContext
}

#[derive(Debug, Default)]
pub struct DebugState {
    x: bool,
    m: bool,
    e: bool,
}

fn sets_m(snes: &Console, stack: &Vec<DebugState>, instr: &InstructionContext) -> Result<bool> {
    Ok(match instr.opcode {
        OpCode::SEP => (memory::peek_byte(snes, instr.data_addr)? & 0x20) != 0,
        OpCode::PLP => {
            if !stack.is_empty() {
                if snes.cpu.P.e {
                    (memory::peek_byte(snes, (snes.cpu.S + 1) as u32)? & 0x20) != 0
                } else {
                    (memory::peek_byte(snes, u32::from_be_bytes([0x00, 0x00, 0x01,snes.cpu.S as u8])+1)? & 0x20) != 0
                }
            } else {
                stack.last().unwrap().m
            } 
        }
        _ => false
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
                    (memory::peek_byte(snes, u32::from_be_bytes([0x00, 0x00, 0x01,snes.cpu.S as u8])+1)? & 0x20) == 0
                }
            } else {
                !stack.last().unwrap().m
            } 
        }
        _ => false
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
                    (memory::peek_byte(snes, u32::from_be_bytes([0x00, 0x00, 0x01,snes.cpu.S as u8])+1)? & 0x10) != 0
                }
            } else {
                stack.last().unwrap().x
            } 
        }
        _ => false
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
                    (memory::peek_byte(snes, u32::from_be_bytes([0x00, 0x00, 0x01,snes.cpu.S as u8])+1)? & 0x10) == 0
                }
            } else {
                !stack.last().unwrap().x
            } 
        }
        _ => false
    })
}

fn clears_e(snes: &Console, instr: &InstructionContext) -> bool {
    match instr.opcode {
        OpCode::XCE => snes.cpu.P.c,
        _ => false
    }
}

pub fn debug_instructions(snes: &Console, start: u32) -> Result<Vec<InstructionWrapper>> {
    let mut cycle = 0;
    let mut state = DebugState::default();
    state.x = snes.cpu.P.x;
    state.m = snes.cpu.P.m;
    state.e = snes.cpu.P.e;
    let mut instructions = Vec::<InstructionWrapper>::default();
    let mut temppc = start;
    let mut tempstack = Vec::<DebugState>::new();
    let mut knowninstructions: AHashMap<u32, usize> = AHashMap::new();
    let mut branchinstructions = Vec::<usize>::default();
    loop {
        let opcode = memory::peek_byte(snes, temppc)?;
        let currinstr = decode_instruction(snes, opcode, temppc)?;
        let branchto = if currinstr.opcode.is_branch() && currinstr.data_addr > start {
            branchinstructions.push(cycle);
            Some(currinstr.data_addr)
        } else {
            None
        };
        instructions.push(InstructionWrapper{
            location: temppc,
            branchfrom: Vec::<u32>::default(),
            branchto,
            instruction: currinstr.clone()
        });
        knowninstructions.insert(temppc, cycle);
        if currinstr.opcode.is_jump() {break;}
        if currinstr.opcode.is_return() {break;}
        if currinstr.opcode == OpCode::BRK {break;}
        if cycle >= 30 {break;}
        temppc += currinstr.length(state.m, state.x) as u32;
        cycle += 1;
        if sets_m(snes, &tempstack, &currinstr)? {state.m = true}
        if sets_x(snes, &tempstack, &currinstr)? {state.x = true}
        if clears_m(snes, &tempstack, &currinstr)? {state.m = false}
        if clears_x(snes, &tempstack, &currinstr)? {state.x = false}
        if clears_e(snes, &currinstr) {state.e = false}
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

mod tests {
use std::path;
use anyhow::Result;
use crate::cartridge;
use crate::cpu;
use crate::MMIORegisters;
use crate::Console;

use super::debug_instructions;
    #[test]
    fn test_debugger() -> Result<()> {
        let cartridge = cartridge::load_rom(path::Path::new("./super_metroid.sfc"))?;
        let mut ram = vec![0; 0x200000];
        let mut snes = Console {
            cpu: cpu::CPU::new(),
            cartridge,
            ram,
            mmio: MMIORegisters::default()
        };
        let disassembled = debug_instructions(&snes, 0x808435)?;
        println!("{:#?}", disassembled);
        Ok(())
    }
}