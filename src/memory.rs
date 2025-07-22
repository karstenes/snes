use crate::cartridge;

use super::Console;
use cartridge::*;
use color_eyre::{
    eyre::{bail, ensure, eyre, Ok},
    Result,
};
use log::{error, trace};

pub fn read_word(snes: &Console, addr: u32) -> Result<u16> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4200 && addr_word < 0x4220) =>
        {
            match addr_word {
                _ => bail!("Write to unknown/writeonly MMIO Register"),
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            read_rom_word(&snes.cartridge, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            read_ram_word(&snes.ram, addr)
        }
        _ => {
            return Err(eyre!("Memory access error! Tried to access {:06X}", addr));
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
                _ => bail!("Read from unknown/writeonly MMIO Register"),
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
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
            return Err(eyre!("Memory access error! Tried to access {:06X}", addr));
        }
    }
}

pub fn read_byte(snes: &Console, addr: u32) -> Result<u8> {
    let bank = (addr & 0xFF0000) >> 16;
    let addr_word = addr & 0xFFFF;
    match addr {
        addr if (bank % 0x80) < 0x40 && (addr_word >= 0x4200 && addr_word < 0x4220) => {
            match addr_word {
                0x4210 => {
                    error!("Unimplemented RDNMI");
                    Ok(0x00)
                }
                0x4211 => {
                    error!("Unimplemented TIMEUP");
                    Ok(0x00)
                }
                0x4212 => {
                    trace!("Unimplemented HBVJOY");
                    if snes.cpu.P.n {
                        Ok(0x00)
                    } else {
                        Ok(0xFF)
                    }
                }
                0x4213 => {
                    error!("Unimplemented RDIO");
                    Ok(0x00)
                }
                0x4214 => {
                    error!("Unimplemented RDDIVL");
                    Ok(0x00)
                }
                0x4215 => {
                    error!("Unimplemented RDDIVH");
                    Ok(0x00)
                }
                0x4216 => {
                    error!("Unimplemented RDMPYL");
                    Ok(0x00)
                }
                0x4217 => {
                    error!("Unimplemented RDMPYH");
                    Ok(0x00)
                }
                0x4218 | 0x4219 | 0x421A | 0x421B | 0x421C | 0x421D | 0x421E | 0x421F => {
                    error!("Unimplemented joypad #{:04X}", addr_word);
                    Ok(0x00)
                }
                _ => bail!("Read from unknown/writeonly MMIO Register"),
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            read_rom_byte(&snes.cartridge, addr)
        }
        addr if (bank >= 0x7E && bank < 0x80) || ((bank % 0x80) < 0x40 && addr_word < 0x2000) => {
            read_ram_byte(&snes.ram, addr)
        }
        _ => return Err(eyre!("Memory access error! Tried to access {:06X}", addr)),
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
                _ => bail!("Read from unknown/writeonly MMIO Register"),
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
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
            return Err(eyre!("Memory access error! Tried to access {:06X}", addr));
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
    let read_data =
        ram[(addr & 0x1FFFF) as usize] as u16 + ((ram[(addr & 0x1FFFF) as usize + 1] as u16) << 8);
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
    let read_data =
        ram[(addr & 0x1FFFF) as usize] as u16 + ((ram[(addr & 0x1FFFF) as usize + 1] as u16) << 8);
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
            ensure!(
                rom_addr < rom.rom_data.len(),
                concat!(
                    "Attempted to access ROM address ${:06X} at {:06X}, ",
                    "which is outside the bounds of the rom vector with size {:06X}\n",
                    "rom has size {:}kB"
                ),
                rom_addr,
                addr,
                rom.rom_data.len(),
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
                _ => bail!("Write byte to unknown/readonly MMIO Register"),
            }
        }
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4300 && addr_word < 0x4380) =>
        {
            let dma_no = ((addr_word & 0x00F0) >> 4) as usize;
            ensure!(
                dma_no < 8,
                "DMA number {} out of bounds! Valid range is 0-7.",
                dma_no
            );
            let dma_reg = addr_word & 0x000F;
            match dma_reg {
                0x2 => {
                    trace!(
                        "Writing #{:04X} to DMA Source Address for DMA {}",
                        data,
                        dma_no
                    );
                    snes.dma.A1TnL[dma_no] = (data & 0x00FF) as u8;
                    snes.dma.A1TnH[dma_no] = ((data & 0xFF00) >> 8) as u8;
                    Ok(())
                }
                0x5 => {
                    trace!("Writing #{:04X} to DMA Length for DMA {}", data, dma_no);
                    snes.dma.DASnL[dma_no] = (data & 0x00FF) as u8;
                    snes.dma.DASnH[dma_no] = ((data & 0xFF00) >> 8) as u8;
                    Ok(())
                }
                0x8 => {
                    trace!(
                        "Writing #{:04X} to HDMA table address for DMA {}",
                        data,
                        dma_no
                    );
                    snes.dma.A2TnL[dma_no] = (data & 0x00FF) as u8;
                    snes.dma.A2TnH[dma_no] = ((data & 0xFF00) >> 8) as u8;
                    Ok(())
                }
                _ => {
                    return Err(eyre!(
                        "Word write to byte width DMA register {:04X} at {:06X}",
                        dma_reg,
                        addr
                    ))
                }
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
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
            return Err(eyre!("Memory access error! Tried to access {:06X}", addr));
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
                0x4200 => {
                    trace!("Writing #{:02X} to MEMSEL", data);
                    snes.mmio.NMITIMEN = data;
                    Ok(())
                }
                0x4201 => {
                    trace!("Writing #{:02X} to WRIO", data);
                    error!("Joypads umimplemented!");
                    Ok(())
                }
                0x4202 => {
                    trace!("Writing #{:02X} to WRMPYA", data);
                    snes.mmio.WRMPYA = data;
                    Ok(())
                }
                0x4203 => {
                    trace!("Writing #{:02X} to WRMPYB", data);
                    snes.mmio.WRMPYB = data;
                    Ok(())
                }
                0x4204 => {
                    trace!("Writing #{:02X} to WRDIVL", data);
                    snes.mmio.WRDIVL = data;
                    Ok(())
                }
                0x4205 => {
                    trace!("Writing #{:02X} to WRMPYB", data);
                    snes.mmio.WRDIVH = data;
                    Ok(())
                }
                0x4206 => {
                    trace!("Writing #{:02X} to WRDIVB", data);
                    snes.mmio.WRDIVB = data;
                    Ok(())
                }
                0x4207 => {
                    trace!("Writing #{:02X} to HTIMEL", data);
                    snes.mmio.HTIMEL = data;
                    Ok(())
                }
                0x4208 => {
                    trace!("Writing #{:02X} to HTIMEH", data);
                    snes.mmio.HTIMEH = data;
                    Ok(())
                }
                0x4209 => {
                    trace!("Writing #{:02X} to VTIMEL", data);
                    snes.mmio.VTIMEL = data;
                    Ok(())
                }
                0x420A => {
                    trace!("Writing #{:02X} to VTIMEH", data);
                    snes.mmio.VTIMEH = data;
                    Ok(())
                }
                0x420B => {
                    trace!("Writing #{:02X} to MDMAEN", data);
                    snes.dma.MDMAEN = data;
                    Ok(())
                }
                0x420C => {
                    trace!("Writing #{:02X} to HDMAEN", data);
                    snes.dma.HDMAEN = data;
                    Ok(())
                }
                0x420D => {
                    trace!("Writing #{:02X} to MEMSEL", data);
                    snes.mmio.MEMSEL = data;
                    Ok(())
                }
                _ => {
                    return Err(eyre!(
                        "Write byte to unknown/readonly MMIO Register #{:04X}",
                        addr_word
                    ))
                }
            }
        }
        addr if ((bank < 0x40) || (bank >= 0x80 && bank < 0xC0))
            && (addr_word >= 0x4300 && addr_word < 0x4380) =>
        {
            let dma_no = ((addr_word & 0x00F0) >> 4) as usize;
            ensure!(
                dma_no < 8,
                "DMA number {} out of bounds! Valid range is 0-7.",
                dma_no
            );
            let dma_reg = addr_word & 0x000F;
            match dma_reg {
                0x0 => {
                    trace!("Writing #{:02X} to DMAP {}", data, dma_no);
                    snes.dma.DMAPn[dma_no] = data;
                    Ok(())
                }
                0x1 => {
                    trace!("Writing #{:02X} to BBBAD {}", data, dma_no);
                    snes.dma.BBADn[dma_no] = data;
                    Ok(())
                }
                0x2 => {
                    trace!("Writing #{:02X} to A1TnL {}", data, dma_no);
                    snes.dma.A1TnL[dma_no] = data;
                    Ok(())
                }
                0x3 => {
                    trace!("Writing #{:02X} to A1TnH {}", data, dma_no);
                    snes.dma.A1TnH[dma_no] = data;
                    Ok(())
                }
                0x4 => {
                    trace!("Writing #{:02X} to A1B {}", data, dma_no);
                    snes.dma.A1nB[dma_no] = data;
                    Ok(())
                }
                0x5 => {
                    trace!("Writing #{:02X} to DASnL {}", data, dma_no);
                    snes.dma.DASnL[dma_no] = data;
                    Ok(())
                }
                0x6 => {
                    trace!("Writing #{:02X} to DASnH {}", data, dma_no);
                    snes.dma.DASnH[dma_no] = data;
                    Ok(())
                }
                0x7 => {
                    trace!("Writing #{:02X} to DASB {}", data, dma_no);
                    snes.dma.DASBn[dma_no] = data;
                    Ok(())
                }
                0x8 => {
                    trace!("Writing #{:02X} to A2TnL {}", data, dma_no);
                    snes.dma.A2TnL[dma_no] = data;
                    Ok(())
                }
                0x9 => {
                    trace!("Writing #{:02X} to A2TnH {}", data, dma_no);
                    snes.dma.A2TnH[dma_no] = data;
                    Ok(())
                }
                0xA => {
                    trace!("Writing #{:02x} to NLTR {}", data, dma_no);
                    snes.dma.NLTRn[dma_no] = data;
                    Ok(())
                }
                0xB | 0xF => {
                    trace!("Writing #{:02X} to unused {}", data, dma_no);
                    snes.dma.UNUSEDn[dma_no] = data;
                    Ok(())
                }
                _ => {
                    return Err(eyre!(
                        "Write byte to unknown/readonly DMA register {:04X} at {:06X}",
                        dma_reg,
                        addr
                    ));
                }
            }
        }
        addr if (addr_word >= 0x8000 && (bank < 0x40 || bank >= 0x80))
            || bank >= 0xC0
            || (bank >= 0x40 && bank < 0x7E) =>
        {
            return Err(eyre!(
                "Attemped to write {:02X} to ROM at {:06X}",
                data,
                addr
            ));
        }
        addr if (bank >= 0x7E && bank < 0x80)
            || ((bank < 0x40 || (bank >= 0x80 && bank < 0xC0)) && addr_word < 0x2000) =>
        {
            write_ram_byte(&mut snes.ram, addr, data)
        }
        addr if (bank % 0x80) < 0x40 && addr_word >= 0x2000 && addr_word < 0x8000 => {
            write_register_byte(snes, addr, data)
        }
        _ => {
            return Err(eyre!(
                "Memory access error! Tried to write to address {:06X}",
                addr
            ))
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
        addr_demirror.to_be_bytes()[1] < 0x40 && addr_word >= 0x2000 && addr_word < 0x8000,
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cpu::*;
    use crate::registers::*;

    fn create_test_console_lorom() -> Console {
        let rom_data = vec![0; 0x200000]; // 2MB ROM filled with zeros

        // Create a dummy cartridge with LoROM mapping
        let cartridge = Cartridge {
            header: RomHeader {
                title: "TEST ROM".to_string(),
                map_mode: MapMode::LoROM,
                rom_speed: RomSpeed::Slow,
                extra_hardware: CartHardware {
                    extra_hardware: ExtraHardware::RomOnly,
                    coprocessor: None,
                },
                rom_size: 2 * 1024 * 1024,
                ram_size: 0,
                country: Region::NTSC,
                developer_id: 0x01,
                rom_version: 0,
                checksum_complement: 0xFFFF,
                checksum: 0x0000,
                interrupt_vectors: InterruptVectorTable {
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
            },
            rom_data,
        };

        Console {
            cpu: CPU::new(),
            cartridge,
            ram: vec![0; 0x200000], // 2MB RAM
            mmio: MMIORegisters::default(),
            dma: DMARegisters::default(),
        }
    }

    fn create_test_console_hirom() -> Console {
        let mut console = create_test_console_lorom();
        console.cartridge.header.map_mode = MapMode::HiROM;
        console
    }

    fn create_test_console_exhirom() -> Console {
        let mut console = create_test_console_lorom();
        console.cartridge.header.map_mode = MapMode::ExHiROM;
        console.cartridge.rom_data = vec![0; 0x400000]; // 4MB ROM for ExHiROM
        console
    }

    #[test]
    fn test_ram_read_write_byte() {
        let mut console = create_test_console_lorom();

        // Test writing and reading from RAM in bank 0x7E
        let addr = 0x7E0100;
        let test_value = 0xAB;

        write_byte(&mut console, addr, test_value).unwrap();
        let read_value = read_byte(&console, addr).unwrap();

        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_ram_read_write_word() {
        let mut console = create_test_console_lorom();

        // Test writing and reading a word from RAM
        let addr = 0x7E0200;
        let test_value = 0x1234;

        write_word(&mut console, addr, test_value).unwrap();
        let read_value = read_word(&console, addr).unwrap();

        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_ram_mirroring() {
        let mut console = create_test_console_lorom();

        // Test that low RAM is mirrored in bank 0
        let test_value = 0xCD;
        let ram_addr = 0x7E0000; // Direct RAM access
        let mirror_addr = 0x000000; // Mirrored access

        write_byte(&mut console, ram_addr, test_value).unwrap();
        let mirrored_value = read_byte(&console, mirror_addr).unwrap();

        assert_eq!(mirrored_value, test_value);
    }

    #[test]
    fn test_rom_read_lorom() {
        let mut console = create_test_console_lorom();

        // Put test data in ROM
        console.cartridge.rom_data[0x100] = 0xEF;

        // Read from LoROM mapping (bank 0, address >= 0x8000)
        let rom_addr = 0x008100; // LoROM bank 0, maps to ROM offset 0x100
        let read_value = read_byte(&console, rom_addr).unwrap();

        assert_eq!(read_value, 0xEF);
    }

    #[test]
    fn test_rom_read_hirom() {
        let mut console = create_test_console_hirom();

        // Put test data in ROM
        console.cartridge.rom_data[0x100] = 0xFE;

        // Read from HiROM mapping
        let rom_addr = 0xC00100; // Direct HiROM access
        let read_value = read_byte(&console, rom_addr).unwrap();

        assert_eq!(read_value, 0xFE);
    }

    #[test]
    fn test_rom_write_fails() {
        let mut console = create_test_console_lorom();

        // Attempting to write to ROM should fail
        let rom_addr = 0x008100;
        let result = write_byte(&mut console, rom_addr, 0x42);

        assert!(result.is_err());
    }

    #[test]
    fn test_mmio_register_write() {
        let mut console = create_test_console_lorom();

        // Test writing to NMITIMEN register
        let nmitimen_addr = 0x004200;
        let test_value = 0x81;

        write_byte(&mut console, nmitimen_addr, test_value).unwrap();
        assert_eq!(console.mmio.NMITIMEN, test_value);
    }

    #[test]
    fn test_mmio_register_read() {
        let console = create_test_console_lorom();

        // Test reading from HBVJOY register (should return 0xFF when n flag is false)
        let hbvjoy_addr = 0x004212;
        let read_value = read_byte(&console, hbvjoy_addr).unwrap();

        // Should return 0xFF since cpu.P.n is false by default
        assert_eq!(read_value, 0xFF);
    }

    #[test]
    fn test_dma_register_write() {
        let mut console = create_test_console_lorom();

        // Test writing to DMA register
        let dma_addr = 0x004300; // DMAP0
        let test_value = 0x42;

        write_byte(&mut console, dma_addr, test_value).unwrap();
        assert_eq!(console.dma.DMAPn[0], test_value);
    }

    #[test]
    fn test_dma_register_word_write() {
        let mut console = create_test_console_lorom();

        // Test writing word to DMA source address
        let dma_addr = 0x004302; // A1T0L/A1T0H
        let test_value = 0x1234;

        write_word(&mut console, dma_addr, test_value).unwrap();
        assert_eq!(console.dma.A1TnL[0], 0x34); // Low byte
        assert_eq!(console.dma.A1TnH[0], 0x12); // High byte
    }

    #[test]
    fn test_peek_vs_read() {
        let console = create_test_console_lorom();

        // Both peek and read should return the same value for RAM
        let ram_addr = 0x7E0000;
        let peek_value = peek_byte(&console, ram_addr).unwrap();
        let read_value = read_byte(&console, ram_addr).unwrap();

        assert_eq!(peek_value, read_value);
    }

    #[test]
    fn test_invalid_address_read() {
        let console = create_test_console_lorom();

        // Try to read from an invalid address range
        let invalid_addr = 0x500000; // Should be invalid
        let result = read_byte(&console, invalid_addr);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_address_write() {
        let mut console = create_test_console_lorom();

        // Try to write to an invalid address range
        let invalid_addr = 0x500000;
        let result = write_byte(&mut console, invalid_addr, 0x42);

        assert!(result.is_err());
    }

    #[test]
    fn test_ram_boundary_wrapping() {
        let mut console = create_test_console_lorom();

        // Test that RAM addresses wrap properly
        let test_value = 0x55;
        let high_ram_addr = 0x7FFFFF; // End of RAM region

        write_byte(&mut console, high_ram_addr, test_value).unwrap();
        let wrapped_addr = 0x7E0000 | (high_ram_addr & 0x1FFFF);
        let read_value = read_byte(&console, wrapped_addr).unwrap();

        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_word_read_write_endianness() {
        let mut console = create_test_console_lorom();

        // Test that words are stored in little-endian format
        let addr = 0x7E1000;
        let test_word = 0xABCD;

        write_word(&mut console, addr, test_word).unwrap();

        // Check individual bytes
        let low_byte = read_byte(&console, addr).unwrap();
        let high_byte = read_byte(&console, addr + 1).unwrap();

        assert_eq!(low_byte, 0xCD); // Low byte first
        assert_eq!(high_byte, 0xAB); // High byte second

        // Check word read
        let read_word = read_word(&console, addr).unwrap();
        assert_eq!(read_word, test_word);
    }

    #[test]
    fn test_multiple_dma_channels() {
        let mut console = create_test_console_lorom();

        // Test writing to different DMA channels
        for channel in 0..8 {
            let dma_addr = 0x004300 + (channel << 4); // DMAP for each channel
            let test_value = 0x10 + channel as u8;

            write_byte(&mut console, dma_addr, test_value).unwrap();
            assert_eq!(console.dma.DMAPn[channel as usize], test_value);
        }
    }

    #[test]
    fn test_exhirom_mapping() {
        let mut console = create_test_console_exhirom();

        // Put test data in ROM
        console.cartridge.rom_data[0x100] = 0x77;

        // Test ExHiROM specific mapping
        let exhirom_addr = 0xC00100;
        let read_value = read_byte(&console, exhirom_addr).unwrap();

        assert_eq!(read_value, 0x77);
    }

    #[test]
    fn test_stack_region_access() {
        let mut console = create_test_console_lorom();

        // Test accessing the stack region (page 1 in bank 0)
        let stack_addr = 0x0001FF;
        let test_value = 0x88;

        write_byte(&mut console, stack_addr, test_value).unwrap();
        let read_value = read_byte(&console, stack_addr).unwrap();

        assert_eq!(read_value, test_value);
    }

    #[test]
    fn test_rom_size_boundary() {
        let console = create_test_console_lorom();

        // Try to read from an address that should be invalid
        let invalid_addr = 0x500000; // Definitely invalid address
        let result = read_byte(&console, invalid_addr);

        assert!(result.is_err());
    }

    #[test]
    fn test_ppu_register_write() {
        let mut console = create_test_console_lorom();

        // Test writing to PPU register region
        let ppu_addr = 0x002100; // INIDISP
        let result = write_byte(&mut console, ppu_addr, 0x0F);

        // Should succeed (registers are write-only in this implementation)
        assert!(result.is_ok());
    }
}
