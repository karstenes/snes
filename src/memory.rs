use crate::cartridge;

use super::Console;
use anyhow::{bail, ensure, Result};
use cartridge::*;
use log::trace;

pub fn read_word(snes: &Console, addr: u32) -> Result<u16> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                _ => bail!("Write to unknown/writeonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            read_rom_word(&snes.cartridge, addr)
        },
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            read_ram_word(&snes.ram, addr)
        },
        _ => {
            bail!("Memory access error! Tried to access {:06X}", addr)
        }
    }
}

pub fn peek_word(snes: &Console, addr: u32) -> Result<u16> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                _ => bail!("Read from unknown/writeonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            peek_rom_word(&snes.cartridge, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            peek_ram_word(&snes.ram, addr)
        }
        _ => {
            bail!("Memory access error! Tried to access {:06X}", addr)
        }
    }
}

pub fn read_byte(snes: &Console, addr: u32) -> Result<u8> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if (bank % 0x80) < 0x40
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                0x4212 => {
                    trace!("Unimplemented HBVJOY");
                    if snes.cpu.P.n {
                        Ok(0x00)
                    } else {
                        Ok(0xFF)
                    }
                }
                _ => bail!("Read from unknown/writeonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            read_rom_byte(&snes.cartridge, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank % 0x80) < 0x40 && addr_word < 0x2000) =>
        {
            read_ram_byte(&snes.ram, addr)
        }
        _ => {
            bail!("Memory access error! Tried to access {:06X}", addr)
        }
    }
}

pub fn peek_byte(snes: &Console, addr: u32) -> Result<u8> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                _ => bail!("Read from unknown/writeonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            peek_rom_byte(&snes.cartridge, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            peek_ram_byte(&snes.ram, addr)
        }
        _ => {
            bail!("Memory access error! Tried to access {:06X}", addr)
        }
    }
}

fn peek_ram_word(ram: &Vec<u8>, addr: u32) -> Result<u16> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        concat!(
            "Attemped to read RAM at address ${:06X},",
            " which is out of bounds."
        ),
        addr
    );
    let read_data = ram[(addr & 0x1FFFF) as usize] as u16 + ((ram[(addr & 0x1FFFF) as usize + 1] as u16) << 8);
    Ok(read_data)
}

fn peek_ram_byte(ram: &Vec<u8>, addr: u32) -> Result<u8> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        concat!(
            "Attemped to read RAM at address ${:06X},",
            " which is out of bounds."
        ),
        addr
    );
    let read_data = ram[(addr & 0x1FFFF) as usize];
    Ok(read_data)
}

fn read_ram_word(ram: &Vec<u8>, addr: u32) -> Result<u16> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        concat!(
            "Attemped to read RAM at address ${:06X},",
            " which is out of bounds."
        ),
        addr
    );
    let read_data = ram[(addr & 0x1FFFF) as usize] as u16 + ((ram[(addr & 0x1FFFF) as usize + 1] as u16) << 8);
    trace!("Read #{:04X} from RAM at address ${:06x}", read_data, addr);
    Ok(read_data)
}

fn read_ram_byte(ram: &Vec<u8>, addr: u32) -> Result<u8> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        concat!(
            "Attemped to read RAM at address ${:06X},",
            " which is out of bounds."
        ),
        addr
    );
    let read_data = ram[(addr & 0x1FFFF) as usize];
    trace!("Read #{:02X} from RAM at address ${:06X}", read_data, addr);
    Ok(read_data)
}

fn peek_rom_word(rom: &cartridge::Cartridge, addr: u32) -> Result<u16> {
    match rom.header.map_mode {
        MapMode::LoROM => {
            let mut page = (addr & 0xFF0000) >> 16;
            if page >= 0x80 {
                page -= 0x80;
            }
            let mut tempaddr = addr;
            if tempaddr >= 0x800000 {
                tempaddr -= 0x800000
            }
            let rom_addr = (tempaddr - ((page + 1) * 0x8000)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            Ok(read_data)
        }
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            Ok(read_data)
        }
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            Ok(read_data)
        }
    }
}

fn read_rom_word(rom: &cartridge::Cartridge, addr: u32) -> Result<u16> {
    match rom.header.map_mode {
        MapMode::LoROM => {
            let mut page = (addr & 0xFF0000) >> 16;
            if page >= 0x80 {
                page -= 0x80;
            }
            let mut tempaddr = addr;
            if tempaddr >= 0x800000 {
                tempaddr -= 0x800000
            }
            let rom_addr = (tempaddr - ((page + 1) * 0x8000)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data =
                (rom.rom_data[rom_addr] as u16) | (rom.rom_data[rom_addr + 1] as u16) << 8;
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
    }
}

fn peek_rom_byte(rom: &cartridge::Cartridge, addr: u32) -> Result<u8> {
    match rom.header.map_mode {
        MapMode::LoROM => {
            let mut page = (addr & 0xFF0000) >> 16;
            if page >= 0x80 {
                page -= 0x80;
            }
            let mut tempaddr = addr;
            if tempaddr >= 0x800000 {
                tempaddr -= 0x800000
            }
            let rom_addr = (tempaddr - ((page + 1) * 0x8000)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X} at {:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            Ok(read_data)
        }
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            Ok(read_data)
        }
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            Ok(read_data)
        }
    }
}

fn read_rom_byte(rom: &cartridge::Cartridge, addr: u32) -> Result<u8> {
    match rom.header.map_mode {
        MapMode::LoROM => {
            let mut page = (addr & 0xFF0000) >> 16;
            if page >= 0x80 {
                page -= 0x80;
            }
            let mut tempaddr = addr;
            if tempaddr >= 0x800000 {
                tempaddr -= 0x800000
            }
            let rom_addr = (tempaddr - ((page + 1) * 0x8000)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X} at {:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
        MapMode::HiROM => {
            let rom_addr = (addr & 0x3FFFFF) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
        MapMode::ExHiROM => {
            let rom_addr = ((addr & 0x3FFFFF) + (((addr & 0x800000) ^ 0x800000) >> 1)) as usize;
            ensure!(
                rom_addr < rom.header.rom_size,
                concat!(
                    "Attempted to access ROM address ${:06X}, ",
                    "which is outside the bounds of this rom with size {:}kB"
                ),
                rom_addr,
                rom.header.rom_size
            );
            let read_data = rom.rom_data[rom_addr];
            trace!("Read #{:04X} from ROM at address ${:06X}", read_data, addr);
            Ok(read_data)
        }
    }
}

pub fn write_word(snes: &mut Console, addr: u32, data: u16) -> Result<()> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                _ => bail!("Write byte to unknown/readonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            bail!("Attemped to write {:04X} to ROM at {:06X}", data, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            write_ram_word(&mut snes.ram, addr, data)
        }
        _ => {
            bail!("Memory access error! Tried to access {:06X}", addr)
        }
    }
}

pub fn write_byte(snes: &mut Console, addr: u32, data: u8) -> Result<()> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) => 
        {
            match addr_word {
                0x420D => {
                    trace!("Writing #{:02X} to MEMSEL", data);
                    snes.mmio.MEMSEL = data;
                    Ok(())
                }
                _ => bail!("Write byte to unknown/readonly MMIO Register")
            }
        },
        addr if (addr_word > 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            bail!("Attemped to write {:02X} to ROM at {:06X}", data, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            write_ram_byte(&mut snes.ram, addr, data)
        }
        addr if (bank % 0x80) < 0x40
            && addr_word >= 0x2000
            && addr_word < 0x8000 => {
            write_register_byte(snes, addr, data)
        }
        _ => {
            bail!("Memory access error! Tried to write to address {:06X}", addr)
        }
    }
}


fn write_ram_word(ram: &mut Vec<u8>, addr: u32, val: u16) -> Result<()> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        "Attemped to write RAM at address ${:04X}, which is out of bounds.",
        addr
    );
    trace!("Writing #{:04X} to RAM at address ${:06X}", val, addr);
    ram[(addr & 0x1FFFF) as usize] = (val & 0xFF) as u8;
    ram[((addr & 0x1FFFF) + 1) as usize] = ((val & 0xFF00) >> 8) as u8;
    Ok(())
}

fn write_ram_byte(ram: &mut Vec<u8>, addr: u32, val: u8) -> Result<()> {
    let ram_addr: usize = (addr & 0x200000) as usize;
    ensure!(
        ram_addr <= 0x200000,
        "Attemped to write RAM at address ${:04X}, which is out of bounds.",
        addr
    );
    trace!("Writing #{:02X} to RAM at address ${:06X}", val, addr);
    ram[(addr & 0x1FFFF) as usize] = val;
    Ok(())
}

fn write_register_byte(snes: &mut Console, addr: u32, val: u8) -> Result<()> {
    let addr_demirror = addr % 0x800000;
    let addr_word: u16 = (addr_demirror & 0xFFFF) as u16;
    ensure!(
        addr_demirror.to_be_bytes()[1] < 0x40 && 
        addr_word >= 0x2000 &&
        addr_word < 0x8000,
        "Attempted to write to register at ${:06X}, which is out of bounds",
        addr
    );
    match addr {
        0x2100 => {
            trace!("Wrote to INIDISP at {:06X}", addr);
        }
        _ => {}
    }
    Ok(())
}