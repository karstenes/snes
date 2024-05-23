mod cpu;
mod cartridge;

use anyhow::Ok;
use cartridge::*;
use cpu::*;
use std::process::ExitCode;
use std::env;
use std::path;
use anyhow::Result;
use env_logger;

#[allow(non_snake_case)]

pub struct Console{
    cpu: CPU
}

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let file_path = path::Path::new(&args[1]);

    let cart = load_rom(file_path)?;

    println!("{:?}", cart);

    return Ok(());
}
