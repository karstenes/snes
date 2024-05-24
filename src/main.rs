#![allow(unused_variables, dead_code)]

mod cpu;
mod cartridge;
mod memory;

use anyhow::Ok;
use cartridge::*;
use cpu::*;
use std::env;
use std::path;
use anyhow::Result;
use env_logger;

#[allow(non_snake_case)]

#[derive(Debug)]
pub struct Console {
    cpu: CPU,
    cartridge: Cartridge
}

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let file_path = path::Path::new(&args[1]);

    let cartridge = load_rom(file_path)?;

    let mut snes = Console {
        cpu: CPU::new(),
        cartridge
    };

    println!("{:04X}", memory::read_word(&mut snes, 0x10FFC0)?);

    return Ok(());
}
