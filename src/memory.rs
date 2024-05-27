use std::fs::read;

use crate::cartridge;

use super::Console;
use cartridge::*;
use log::{info, debug, trace};
use anyhow::{Result, Context, bail, ensure};

pub fn read_word(snes: &mut Console, addr: u32) -> Result<u16> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80)) ||
                    bank >= 0xC0 || (bank >= 0x40 && bank < 0x7E) => {
            read_rom_word(&snes.cartridge, addr)
        },
        addr if (bank >= 0x7E && bank < 0x80) ||
                    ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) => {
            read_ram_word(&snes.ram, addr)
        }    
        _ => {bail!("Memory access error! Tried to access {:06X}", addr)}
    }
}

pub fn read_byte(snes: &mut Console, addr: u32) -> Result<u8> {
    Ok(1)
}

fn read_ram_word(ram: &Vec<u8>, addr: u32) -> Result<u16> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(ram_addr <= 0x200000, "Attemped to read RAM at address ${:04X}, which is out of bounds.", addr);
    let read_data = ram[ram_addr] as u16 + ((ram[ram_addr + 1] as u16) << 8);
    trace!("Read #{:04X} from RAM at address ${:06x}", read_data, addr);
    Ok(read_data)
}

fn read_rom_word(rom: &cartridge::Cartridge, addr: u32) -> Result<u16> {
    match rom.header.map_mode {
        MapMode::LoROM => {    
            let rom_addr = (addr - (((addr & 0xFF0000) >> 16) + 1 * 0x8000)) as usize;
            ensure!(rom_addr < rom.header.rom_size,
                concat!("Attempted to access ROM address ${:06X}, ",
                            "which is outside the bounds of this rom with size {:}kB"),
                rom_addr, rom.header.rom_size
            );
            let read_data = (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)      
        },
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            ensure!(rom_addr < rom.header.rom_size,
                concat!("Attempted to access ROM address ${:06X}, ",
                            "which is outside the bounds of this rom with size {:}kB"),
                rom_addr, rom.header.rom_size
            );
            let read_data = (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        },
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            ensure!(rom_addr < rom.header.rom_size,
                concat!("Attempted to access ROM address ${:06X}, ",
                            "which is outside the bounds of this rom with size {:}kB"),
                rom_addr, rom.header.rom_size
            );
            let read_data = (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr+1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
    }
}

fn write_ram_word(ram: &mut Vec<u8>, addr: u32, val: u16) -> Result<()> { 
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(ram_addr <= 0x200000, "Attemped to read RAM at address ${:04X}, which is out of bounds.", addr);
    trace!("Writing #{:04X} to RAM at address ${:06X}", val, addr);
    ram[(addr & 0x1FFFF) as usize] = (val & 0xFF) as u8;
    ram[((addr & 0x1FFFF) + 1) as usize] = ((val & 0xFF00) >> 8) as u8;
    Ok(())
}