use color_eyre::{
    eyre::{bail, eyre, Context},
    Result,
};
use log::{debug, info};
use num_enum::TryFromPrimitive;
use std::{fs, path::Path, str};

#[derive(Clone, Debug)]
pub enum MapMode {
    LoROM = 0,
    HiROM = 1,
    ExHiROM = 5,
}

#[derive(Clone, Debug)]
pub enum RomSpeed {
    Slow,
    Fast,
}

#[derive(Clone, TryFromPrimitive, Debug)]
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

#[derive(Clone, TryFromPrimitive, Debug)]
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
    Custom = 0xF,
}

#[derive(Clone, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum ChipsetSubtype {
    SPC7110,
    ST010,
    ST018,
    CX4,
}

#[derive(Clone, Debug)]
pub struct CartHardware {
    pub extra_hardware: ExtraHardware,
    pub coprocessor: Option<Coprocessor>,
}

impl CartHardware {
    pub fn new(extra_hardware: ExtraHardware, coprocessor: Option<Coprocessor>) -> Self {
        Self {
            extra_hardware,
            coprocessor,
        }
    }
}
#[derive(Clone, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum Region {
    NTSC,
    PAL,
}

#[derive(Clone, Debug)]
pub struct InterruptVectorTable {
    pub cop: u16,
    pub brk: u16,
    pub abort: u16,
    pub nmi: u16,
    pub irq: u16,
    pub cop_emu: u16,
    pub brk_emu: u16,
    pub abort_emu: u16,
    pub nmi_emu: u16,
    pub reset: u16,
    pub irq_emu: u16,
}

#[derive(Clone, Debug)]
pub struct ExpandedHeader {
    maker_code: String,
    game_code: String,
    /// 1<<N Kilobytes, here stored as size in bytes
    expansion_rom_size: usize,
    /// 1<<N Kilobytes, here stored as size in bytes
    expansion_ram_size: usize,
    special_version: u8,
    chipset_subtype: ChipsetSubtype,
}

#[derive(Clone, Debug)]
pub struct RomHeader {
    pub title: String,
    pub map_mode: MapMode,
    pub rom_speed: RomSpeed,
    pub extra_hardware: CartHardware,
    /// 1<<N Kilobytes, here stored as size in bytes
    pub rom_size: usize,
    /// 1<<N Kilobytes, here stored as size in bytes
    pub ram_size: usize,
    pub country: Region,
    pub developer_id: u8,
    pub rom_version: u8,
    pub checksum_complement: u16,
    pub checksum: u16,
    pub interrupt_vectors: InterruptVectorTable,
    pub expanded_header: Option<ExpandedHeader>,
}

#[derive(Clone, Debug)]
pub struct Cartridge {
    pub header: RomHeader,
    pub rom_data: Vec<u8>,
}

fn load_rom_header(file: &Vec<u8>, bypass_checksum: bool) -> Result<RomHeader> {
    if file.len() % 1024 != 0 {
        debug!("Rom Dumper header found in file.");
    }

    let mut section_1_length: usize = 0x8000;
    while section_1_length * 2 <= file.len() {
        section_1_length *= 2;
    }
    let checksum: u16 = if section_1_length != file.len() {
        let mut section_2_length: usize = 0x8000;
        while section_2_length * 2 + section_1_length <= file.len() {
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
        file.iter().fold(0u16, |sum, i| sum.wrapping_add(*i as u16))
    };

    let checksum_complement = checksum ^ 0xFFFF;

    debug!(
        "Checksum {:04X} and Complement {:04X}",
        checksum, checksum_complement
    );
    let mapping = if (file[0x7FDC] as u16) | (file[0x7FDD] as u16) << 8 == checksum_complement
        && (file[0x7FDE] as u16) | (file[0x7FDF] as u16) << 8 == checksum
    {
        MapMode::LoROM
    } else if (file[0xFFDC] as u16) | (file[0xFFDD] as u16) << 8 == checksum_complement
        && (file[0xFFDE] as u16) | (file[0xFFDF] as u16) << 8 == checksum
    {
        MapMode::HiROM
    } else {
        if file.len() < 0x40FFDF && !bypass_checksum {
            return Err(eyre!(
                "No checksum found in rom file. Is this a valid SNES rom?"
            ));
        }
        MapMode::ExHiROM
    };

    debug!("Found {:?} mode ROM", mapping);

    let header_slice = match mapping {
        MapMode::LoROM => &file[0x7FC0..=0x7FFF],
        MapMode::HiROM => &file[0xFFC0..=0xFFFF],
        MapMode::ExHiROM => &file[0x40FFC0..=0x40FFFF],
    };

    let title = String::from_utf8(header_slice[0..=0x14].to_vec()).with_context(|| {
        format!(
            "Failed to convert cartridge title to a rust str: {:?}",
            &header_slice[0..=0x14]
        )
    })?;

    info!("ROM is \"{:}\"", title.trim_end());

    let rom_speed = match header_slice[0x15] & 0b00010000 {
        0 => RomSpeed::Slow,
        _ => RomSpeed::Fast,
    };

    debug!("Rom speed is {:?}", rom_speed);

    let hardware = ExtraHardware::try_from(header_slice[0x16] & 0xF)
        .with_context(|| format!("Unknown Hardware {:02X}", header_slice[0x16] & 0xF))?;

    let coprocessor = match header_slice[0x16] & 0x0F {
        3 | 4 | 5 | 6 => Some(
            Coprocessor::try_from((header_slice[0x16] & 0xF0) >> 4).with_context(|| {
                format!(
                    "Unknown Coprocessor {:02X}",
                    (header_slice[0x16] & 0xF0) >> 4
                )
            })?,
        ),
        _ => None,
    };

    let extra_hardware = CartHardware {
        extra_hardware: hardware,
        coprocessor,
    };

    debug!("Cartridge hardware: {:?}", extra_hardware);

    let rom_size = (1usize << header_slice[0x17]) * 1024;

    let ram_size = (1usize << header_slice[0x18]) * 1024;

    debug!(
        "ROM size: {:}kB, RAM size: {:}kB",
        rom_size / 1024,
        ram_size / 1024
    );

    let country = Region::try_from(header_slice[0x19])
        .with_context(|| format!("Unknown region {:04X}", header_slice[0x19]))?;

    debug!("Region: {:?}", country);

    let developer_id = header_slice[0x1A];

    debug!("Developer ID: {:02X}", developer_id);

    let rom_version = header_slice[0x1B];

    debug!("Rom Version: {:}", rom_version);

    let interrupt_vector_slice = header_slice[0x24..0x40]
        .chunks_exact(2)
        .map(|x| (x[0] as u16) | ((x[1] as u16) << 8))
        .collect::<Vec<u16>>();

    let interrupt_vectors = InterruptVectorTable {
        cop: interrupt_vector_slice[0],
        brk: interrupt_vector_slice[1],
        abort: interrupt_vector_slice[2],
        nmi: interrupt_vector_slice[3],
        irq: interrupt_vector_slice[5],
        cop_emu: interrupt_vector_slice[8],
        brk_emu: interrupt_vector_slice[9],
        abort_emu: interrupt_vector_slice[10],
        nmi_emu: interrupt_vector_slice[11],
        reset: interrupt_vector_slice[12],
        irq_emu: interrupt_vector_slice[13],
    };

    let expanded_header = if developer_id == 0x33 || header_slice[0x14] == 0x0 {
        debug!("Expanded header detected.");
        let expanded_header_slice = match mapping {
            MapMode::LoROM => &file[0x7FB0..=0x7FBF],
            MapMode::HiROM => &file[0xFFB0..=0xFFBF],
            MapMode::ExHiROM => &file[0x40FFB0..=0x40FFBF],
        };
        let maker_code = str::from_utf8(&expanded_header_slice[0..=0x1])
            .context("Failed to maker code to a rust str")?
            .to_string();
        debug!("Maker code {:}", maker_code);
        let game_code = str::from_utf8(&expanded_header_slice[0x2..=0x3])
            .context("Failed to game code to a rust str")?
            .to_string();
        debug!("Game code {:}", game_code);
        let expansion_rom_size = (1usize << expanded_header_slice[0xC]) * 1024;
        debug!("Expansion ROM size {:}kB", expansion_rom_size / 1024);
        let expansion_ram_size = (1usize << expanded_header_slice[0xD]) * 1024;
        debug!("Expansion RAM size {:}kB", expansion_ram_size / 1024);
        let special_version = expanded_header_slice[0xE];
        debug!("Special version {:02X}", special_version);
        let chipset_subtype = ChipsetSubtype::try_from((expanded_header_slice[0xF]) >> 4)
            .with_context(|| {
                format!(
                    "Unknown Chipset Subtype {:02X}",
                    (expanded_header_slice[0xF]) >> 4
                )
            })?;
        debug!("Chipset subtype {:?}", chipset_subtype);
        Some(ExpandedHeader {
            maker_code,
            game_code,
            expansion_rom_size,
            expansion_ram_size,
            special_version,
            chipset_subtype,
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
        expanded_header,
    };

    return Ok(header);
}

pub fn load_rom(rom_file: &Path, bypass_checksum: bool) -> Result<Cartridge> {
    let file: Vec<u8> = fs::read(&rom_file)
        .wrap_err_with(|| format!("Failed to read rom file {}", rom_file.display()))?;

    let header = load_rom_header(&file, bypass_checksum)?;

    let cart = Cartridge {
        header,
        rom_data: file.clone(),
    };

    return Ok(cart);
}

#[cfg(test)]
mod tests {

    // Helper function to create a minimal valid LoROM header
    fn create_lorom_header() -> Vec<u8> {
        let mut rom = vec![0; 0x8000]; // 32KB minimal LoROM

        // Set up header at 0x7FC0
        let header_start = 0x7FC0;

        // Title (21 bytes)
        let title = b"TEST ROM             ";
        rom[header_start..header_start + 21].copy_from_slice(title);

        // Map mode and ROM speed (0x7FD5)
        rom[0x7FD5] = 0x20; // LoROM, slow ROM

        // Hardware type (0x7FD6)
        rom[0x7FD6] = 0x00; // ROM only

        // ROM size (0x7FD7) - 32KB = 2^15, so we need 15-10 = 5
        rom[0x7FD7] = 0x08; // 256KB for easier testing

        // RAM size (0x7FD8)
        rom[0x7FD8] = 0x00; // No RAM

        // Country (0x7FD9)
        rom[0x7FD9] = 0x01; // NTSC

        // Developer ID (0x7FDA)
        rom[0x7FDA] = 0x01;

        // ROM version (0x7FDB)
        rom[0x7FDB] = 0x00;

        // Calculate and set checksum
        let checksum: u16 = rom
            .iter()
            .fold(0u16, |sum, &byte| sum.wrapping_add(byte as u16));
        let checksum_complement = checksum ^ 0xFFFF;

        // Set checksum complement (0x7FDC-0x7FDD)
        rom[0x7FDC] = (checksum_complement & 0xFF) as u8;
        rom[0x7FDD] = ((checksum_complement & 0xFF00) >> 8) as u8;

        // Set checksum (0x7FDE-0x7FDF)
        rom[0x7FDE] = (checksum & 0xFF) as u8;
        rom[0x7FDF] = ((checksum & 0xFF00) >> 8) as u8;

        // Set interrupt vectors
        let vector_start = 0x7FE4;
        // COP vector
        rom[vector_start] = 0x00;
        rom[vector_start + 1] = 0x80;
        // BRK vector
        rom[vector_start + 2] = 0x00;
        rom[vector_start + 3] = 0x80;
        // ABORT vector
        rom[vector_start + 4] = 0x00;
        rom[vector_start + 5] = 0x80;
        // NMI vector
        rom[vector_start + 6] = 0x00;
        rom[vector_start + 7] = 0x80;
        // RESET vector
        rom[vector_start + 8] = 0x00;
        rom[vector_start + 9] = 0x80;
        // IRQ vector
        rom[vector_start + 10] = 0x00;
        rom[vector_start + 11] = 0x80;

        // Emulation mode vectors
        let emu_vector_start = 0x7FF4;
        // COP vector (emu)
        rom[emu_vector_start] = 0x00;
        rom[emu_vector_start + 1] = 0x80;
        // Unknown vector
        rom[emu_vector_start + 2] = 0x00;
        rom[emu_vector_start + 3] = 0x80;
        // ABORT vector (emu)
        rom[emu_vector_start + 4] = 0x00;
        rom[emu_vector_start + 5] = 0x80;
        // NMI vector (emu)
        rom[emu_vector_start + 6] = 0x00;
        rom[emu_vector_start + 7] = 0x80;
        // RESET vector (emu)
        rom[emu_vector_start + 8] = 0x00;
        rom[emu_vector_start + 9] = 0x80;
        // IRQ/BRK vector (emu)
        rom[emu_vector_start + 10] = 0x00;
        rom[emu_vector_start + 11] = 0x80;

        rom
    }

    #[test]
    fn test_create_lorom_header() {
        let header = create_lorom_header();
        assert_eq!(header.len(), 0x8000);

        // Verify title
        let title_start = 0x7FC0;
        let title = &header[title_start..title_start + 21];
        assert_eq!(&title[0..8], b"TEST ROM");

        // Verify map mode
        assert_eq!(header[0x7FD5], 0x20);

        // Verify hardware type
        assert_eq!(header[0x7FD6], 0x00);
    }
}
