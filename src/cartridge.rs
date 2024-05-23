use std::{fs, path::Path, str};
use anyhow::{Result, Context, bail};
use num_enum::TryFromPrimitive;

pub enum MapMode {
    LoROM = 0,
    HiROM = 1,
    ExHiROM = 5
}

pub enum RomSpeed {
    Slow,
    Fast
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum ExtraHardware {
    RomOnly,
    RomRam,
    RomRamBattery,
    RomCoprocessor,
    RomCoprocessorRam,
    RomCoprocessorRamBattery,
    RomCoprocessorBattery,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Coprocessor {
    DSP,
    /// GSU
    SuperFX,
    OBC1,
    SA1,
    SDD1,
    SRTC,
    Other = 0xE,
    Custom = 0xF
}

pub enum ChipsetSubtype {
    SPC7110,
    ST010,
    ST018,
    CX4
}

pub struct CartHardware {
    extra_hardware: ExtraHardware,
    coprocessor: Coprocessor
}
 
#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Region {
    NTSC,
    PAL
}

pub struct ExpandedHeader {
    maker_code: String,
    game_code: String,
    /// 1<<N Kilobytes, here stored as size in kB
    expansion_rom_size: usize,
    /// 1<<N Kilobytes, here stored as size in kB
    expansion_ram_size: usize,
    special_version: u8,
    chipset_subtype: ChipsetSubtype
}

pub struct RomHeader {
    title: String,
    map_mode: MapMode,
    rom_speed: RomSpeed,
    extra_hardware: CartHardware, 
    /// 1<<N Kilobytes, here stored as size in kB
    rom_size: usize,
    /// 1<<N Kilobytes, here stored as size in kB
    ram_size: usize,
    country: Region,
    developer_id: u8,
    rom_version: u8,
    checksum_complement: u16,
    checksum: u16,
    interrupt_vectors: [u16;16],
    expanded_header: Option<ExpandedHeader>
}

pub struct Cartridge {
    header: RomHeader,
    rom_data: Vec<u8>
}

fn load_rom_header(file: &Vec<u8>) -> Result<RomHeader> {
    let checksum: u16 = file
                        .iter()
                        .fold(0u16, |sum, i| sum.wrapping_add(*i as u16));

    let checksum_complement = checksum ^ 0xFFFF;

    let mapping = 
    if (file[0x75DC] as u16) << 8 & (file[0x75DD] as u16) == checksum_complement &&
        (file[0x75DE] as u16) << 8 & (file[0x75DF] as u16) == checksum {
            MapMode::LoROM
    } else if (file[0xFFDC] as u16) << 8 & (file[0xFFDD] as u16) == checksum_complement &&
        (file[0xFFDE] as u16) << 8 & (file[0xFFDF] as u16) == checksum {
            MapMode::HiROM
    } else if (file[0x40FFDC] as u16) << 8 & (file[0x40FFDD] as u16) == checksum_complement &&
        (file[0x40FFDE] as u16) << 8 & (file[0x40FFDF] as u16) == checksum {
            MapMode::ExHiROM
    } else {
        bail!("No checksum found in rom file. Is this a valid SNES rom?")
    };

    let header_slice = match mapping {
        MapMode::LoROM => &file[0x7FC0..=0x7FE0],
        MapMode::HiROM => &file[0xFFC0..=0xFFE0],
        MapMode::ExHiROM => &file[0x40FFC0..=0x40FFE0]
    };

    let title = str::from_utf8(&header_slice[0..=0x14])
        .context("Failed to convert cartridge title to a rust str")?
        .to_string();

    let rom_speed = match header_slice[0x15] & 0b00010000 {
        0 => RomSpeed::Slow,
        _ => RomSpeed::Fast
    };

    let hardware = ExtraHardware::try_from(header_slice[0x16] & 0xF)
        .with_context(|| format!("Unknown Hardware {:02X}", header_slice[0x16] & 0xF))?;

    let coprocessor = Coprocessor::try_from((header_slice[0x16] & 0xF0) >> 4)
        .with_context(|| format!("Unknown Coprocessor {:02X}",(header_slice[0x16] & 0xF0) >> 4))?;

    let extra_hardware = CartHardware {
        extra_hardware: hardware,
        coprocessor
    };

    let rom_size = 1usize << header_slice[0x17];

    let ram_size = 1usize << header_slice[0x18];

    let country = Region::try_from(header_slice[0x19])
        .with_context(|| format!("Unknown region {:04X}", header_slice[0x19]))?;

    let developer_id = header_slice[0x1A];

    let rom_version = header_slice[0x1B];

    let interrupt_vectors: [u16; 16] = header_slice[0x20..0x40]
        .chunks_exact(2)
        .map(|x| (x[0] as u16) & ((x[1] as u16) << 8))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let header = RomHeader {
        title,
        map_mode: mapping,
        rom_speed,
        extra_hardware,
        rom_size,
        ram_size,
        country,
        developer_id,
        rom_version,
        checksum_complement,
        checksum,
        interrupt_vectors,
        expanded_header: None
    };

    return Ok(header);
}

pub fn load_rom(rom_file: &Path) -> Result<Cartridge> {
    let file: Vec<u8> = fs::read(&rom_file)
        .with_context(|| format!("Failed to read rom file {}", rom_file.display()))?;

    let header = load_rom_header(&file)?;    

    let cart = Cartridge {
        header,
        rom_data: file.clone()
    };

    return Ok(cart)
}