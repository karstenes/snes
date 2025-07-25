// use super::Console;

#[allow(non_snake_case)]
#[derive(Clone, Debug, Default)]
pub struct MMIORegisters {
    pub APUIO0: u8,
    pub APUIO1: u8,
    pub APUIO2: u8,
    pub APUIO3: u8,
    pub WMDATA: u8,
    pub WMADDL: u8,
    pub WMADDM: u8,
    pub WMADDH: u8,
    pub JOYOUT: u8,
    pub JOYSER0: u8,
    pub JOYSER1: u8,
    pub NMITIMEN: u8,
    pub WRIO: u8,
    pub WRMPYA: u8,
    pub WRMPYB: u8,
    pub WRDIVL: u8,
    pub WRDIVH: u8,
    pub WRDIVB: u8,
    pub HTIMEL: u8,
    pub HTIMEH: u8,
    pub VTIMEL: u8,
    pub VTIMEH: u8,
    pub MEMSEL: u8,
    pub RDNMI: u8,
    pub TIMEUP: u8,
    pub HBVJOY: u8,
    pub RDIO: u8,
    pub RDDIVL: u8,
    pub RDDIVH: u8,
    pub RDMPYL: u8,
    pub RDMPYH: u8,
}

#[allow(non_snake_case)]
#[derive(Debug, Default)]
pub struct PPURegisters {
    pub MPYL: u8,
    pub MPYM: u8,
    pub MPYH: u8,
    pub SLHV: u8,
    pub VMDATALREAD: u8,
    pub VMDATAHREAD: u8,
    pub CGDATAREAD: u16,
    pub OPHCT: u16,
    pub OPVCT: u16,
    pub STAT77: u8,
    pub STAT78: u8,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct DMARegisters {
    pub MDMAEN: u8,
    pub HDMAEN: u8,
    pub DMAPn: Vec<u8>,
    pub BBADn: Vec<u8>,
    pub A1TnL: Vec<u8>,
    pub A1TnH: Vec<u8>,
    pub A1nB: Vec<u8>,
    pub DASnL: Vec<u8>,
    pub DASnH: Vec<u8>,
    pub DASBn: Vec<u8>,
    pub A2TnL: Vec<u8>,
    pub A2TnH: Vec<u8>,
    pub NLTRn: Vec<u8>,
    pub UNUSEDn: Vec<u8>,
}

impl Default for DMARegisters {
    fn default() -> Self {
        Self {
            MDMAEN: 0,
            HDMAEN: 0,
            DMAPn: vec![0; 8],
            BBADn: vec![0; 8],
            A1TnL: vec![0; 8],
            A1TnH: vec![0; 8],
            A1nB: vec![0; 8],
            DASnL: vec![0; 8],
            DASnH: vec![0; 8],
            DASBn: vec![0; 8],
            A2TnL: vec![0; 8],
            A2TnH: vec![0; 8],
            NLTRn: vec![0; 8],
            UNUSEDn: vec![0; 8],
        }
    }
}
