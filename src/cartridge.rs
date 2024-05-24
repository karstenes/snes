use std::{fs, path::Path, str};
use anyhow::{Result, Context, bail, };
use num_enum::TryFromPrimitive;
use log::{info, debug};

#[derive(Debug)]
pub enum MapMode {
    LoROM = 0,
    HiROM = 1,
    ExHiROM = 5
}

#[derive(Debug)]
pub enum RomSpeed {
    Slow,
    Fast
}

#[derive(TryFromPrimitive,Debug)]
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

#[derive(TryFromPrimitive,Debug)]
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

#[derive(TryFromPrimitive,Debug)]
#[repr(u8)]
pub enum ChipsetSubtype {
    SPC7110,
    ST010,
    ST018,
    CX4
}

#[derive(Debug)]
pub struct CartHardware {
    extra_hardware: ExtraHardware,
    coprocessor: Option<Coprocessor>
}
 
#[derive(TryFromPrimitive,Debug)]
#[repr(u8)]
pub enum Region {
    PAL,
    NTSC
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Cartridge {
    pub header: RomHeader,
    pub rom_data: Vec<u8>
}

fn load_rom_header(file: &Vec<u8>) -> Result<RomHeader> {
    if file.len() % 1024 != 0 {
        debug!("Rom Dumper header found in file.");
    }

    let mut section_1_length: usize = 0x8000;
    while section_1_length*2 <= file.len() {
        section_1_length *= 2;
    }
    let checksum: u16 = if section_1_length != file.len() {
        let mut section_2_length: usize = 0x8000;
        while section_2_length*2 + section_1_length <= file.len() {
            section_2_length *= 2;
        }
        if section_1_length + section_2_length != file.len() {
            bail!("Rom is not a power of 2 size.")
        }
        let section_1_sum = file[..section_1_length]
                                    .iter()
                                    .fold(0u16, |sum, i| sum.wrapping_add(*i as u16));
        let section_2_sum = file[section_1_length..]
                                    .iter()
                                    .fold(0u16, |sum, i| sum.wrapping_add(*i as u16));
        section_1_sum.wrapping_add(section_2_sum.wrapping_mul(2))
    } else {                      
        file
            .iter()
            .fold(0u16, |sum, i| sum.wrapping_add(*i as u16))
    };
    

    // let checksum: u16 = file
    //                     .iter()
    //                     .fold(0u16, |sum, i| sum.wrapping_add(*i as u16));

    let checksum_complement = checksum ^ 0xFFFF;

    debug!("Checksum {:04X} and Complement {:04X}", checksum, checksum_complement);
    let mapping = 
    if (file[0x7FDC] as u16) | (file[0x7FDD] as u16) << 8 == checksum_complement &&
        (file[0x7FDE] as u16) | (file[0x7FDF] as u16) << 8 == checksum {
            MapMode::LoROM
    } else if (file[0xFFDC] as u16) | (file[0xFFDD] as u16) << 8 == checksum_complement &&
        (file[0xFFDE] as u16) | (file[0xFFDF] as u16) << 8 == checksum {
            MapMode::HiROM
    } else {
        if file.len() < 0x40FFDF {
            bail!("No checksum found in rom file. Is this a valid SNES rom?")
        }
        MapMode::ExHiROM
    };

    debug!("Found {:?} mode ROM", mapping);

    let header_slice = match mapping {
        MapMode::LoROM => &file[0x7FC0..=0x7FFF],
        MapMode::HiROM => &file[0xFFC0..=0xFFFF],
        MapMode::ExHiROM => &file[0x40FFC0..=0x40FFFF]
    };

    let title = str::from_utf8(&header_slice[0..=0x14])
        .context("Failed to convert cartridge title to a rust str")?
        .to_string();

    info!("ROM is \"{:}\"", title.trim_end());

    let rom_speed = match header_slice[0x15] & 0b00010000 {
        0 => RomSpeed::Slow,
        _ => RomSpeed::Fast
    };

    debug!("Rom speed is {:?}", rom_speed);

    let hardware = ExtraHardware::try_from(header_slice[0x16] & 0xF)
        .with_context(|| format!("Unknown Hardware {:02X}", header_slice[0x16] & 0xF))?;

    
    let coprocessor = match header_slice[0x16] & 0x0F {
        3 | 4 | 5 | 6 => Some(
                            Coprocessor::try_from((header_slice[0x16] & 0xF0) >> 4)
                            .with_context(|| format!("Unknown Coprocessor {:02X}",(header_slice[0x16] & 0xF0) >> 4))?
                        ),
        _ => None
    };

    let extra_hardware = CartHardware {
        extra_hardware: hardware,
        coprocessor
    };

    debug!("Extra hardware: {:?}", extra_hardware);

    let rom_size = 1usize << header_slice[0x17];

    let ram_size = 1usize << header_slice[0x18];

    debug!("ROM size: {:}kB, RAM size: {:}kB", rom_size, ram_size);

    let country = Region::try_from(header_slice[0x19])
        .with_context(|| format!("Unknown region {:04X}", header_slice[0x19]))?;

    debug!("Region: {:?}", country);

    let developer_id = header_slice[0x1A];

    debug!("Developer ID: {:02X}", developer_id);

    let rom_version = header_slice[0x1B];

    debug!("Rom Version: {:}", rom_version);

    let interrupt_vectors: [u16; 16] = header_slice[0x20..0x40]
        .chunks_exact(2)
        .map(|x| (x[0] as u16) & ((x[1] as u16) << 8))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let expanded_header = if developer_id == 0x33 || header_slice[0x14] == 0x0 {
        debug!("Expanded header detected.");
        let expanded_header_slice = match mapping {
            MapMode::LoROM => &file[0x7FB0..=0x7FBF],
            MapMode::HiROM => &file[0xFFB0..=0xFFBF],
            MapMode::ExHiROM => &file[0x40FFB0..=0x40FFBF]
        };
        let maker_code = str::from_utf8(&expanded_header_slice[0..=0x1])
                                .context("Failed to maker code to a rust str")?
                                .to_string();
        let game_code = str::from_utf8(&expanded_header_slice[0x2..=0x3])
                                .context("Failed to game code to a rust str")?
                                .to_string();
        let expansion_rom_size = 1usize << expanded_header_slice[0xC];
        let expansion_ram_size = 1usize << expanded_header_slice[0xD];
        let special_version = expanded_header_slice[0xE];
        let chipset_subtype = ChipsetSubtype::try_from((expanded_header_slice[0xF]) >> 4)
            .with_context(|| format!("Unknown Chipset Subtype {:02X}",(expanded_header_slice[0xF]) >> 4))?;
        Some(ExpandedHeader {
            maker_code,
            game_code,
            expansion_rom_size,
            expansion_ram_size,
            special_version,
            chipset_subtype
        })
    } else {
        None
    };

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
        expanded_header
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