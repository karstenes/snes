mod cpu;
mod cartridge;

use cartridge::*;
use cpu::*;
use std::process::ExitCode;

#[allow(non_snake_case)]

pub struct Console{
    cpu: CPU
}

fn main() -> ExitCode {
    let mut snes = Console { cpu: CPU::new() };
    interpret_opcode(&mut snes);
    ExitCode::SUCCESS
}
