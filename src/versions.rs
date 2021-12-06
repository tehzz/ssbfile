#[derive(Debug, Copy, Clone)]
pub(crate) struct SSBInfo {
    pub(crate) version: &'static str,
    pub(crate) crc: (u32, u32),
    pub(crate) table_start: usize,
    pub(crate) table_end: usize,
}

impl SSBInfo {
    /// each entry is 12 (0xC) bytes long,
    /// and there is a dummy entry at the end of the table
    pub const fn total_entries(&self) -> usize {
        ((self.table_end - self.table_start) / 12) - 1
    }
}

const SSB_ROMS_INFO: &[SSBInfo] = &[SSBInfo {
    version: "NALE", // US NTSC
    crc: (0x916B8B5B, 0x780B85A4),
    table_start: 0x1AC870,
    table_end: 0x1B2C6C,
}];

pub(crate) fn find_version(rom: &[u8]) -> Option<&'static SSBInfo> {
    let crc1_bytes: [u8; 4] = rom[0x10..0x14].try_into().expect("valid rom");
    let crc2_bytes: [u8; 4] = rom[0x14..0x18].try_into().expect("valid rom");
    let crc = (
        u32::from_be_bytes(crc1_bytes),
        u32::from_be_bytes(crc2_bytes),
    );

    for info in SSB_ROMS_INFO {
        if info.crc == crc {
            return Some(info);
        }
    }

    None
}
