use crate::cartridge;

use super::Console;
use cartridge::*;
use log::{info, debug};
use anyhow::{Result, Context, bail};

pub fn read_word(snes: &mut Console, addr: u32) -> Result<u16> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80)) ||
            bank >= 0xC0 || (bank >= 0x40 && bank < 0x7E) => {
                read_rom_word(&snes.cartridge, addr)
        },        
        _ => {bail!("Memory access error! Tried to access {:06X}", addr)}
    }
}

pub fn read_byte(snes: &mut Console, addr: u32) -> Result<u8> {
    Ok(1)
}

fn read_rom_word(rom: &cartridge::Cartridge, addr: u32) -> Result<u16> {
    match rom.header.map_mode {
        MapMode::LoROM => {    
            let rom_addr = (addr - (((addr & 0xFF0000) >> 16) + 1 * 0x8000)) as usize;
            if rom_addr > rom.header.rom_size {
                bail!(concat!("Attempted to access ROM address ${:06X}, ",
                            "which is outside the bounds of this rom with size {:}kB"),
                            rom_addr, rom.header.rom_size)
            }
            debug!("Reading from rom address {:06X}", rom_addr);
            Ok((rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8)      
        },
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            debug!("Reading from rom address {:06X}", rom_addr);
            Ok((rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8)
        },
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            Ok((rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8)
        }
    }
}
