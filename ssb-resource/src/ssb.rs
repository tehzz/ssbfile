use byteorder::{BE, ByteOrder};
use rom_info::{N64Header, N64ParseError, extract_asm_immediate};

#[derive(Fail, Debug)]
pub enum Ssb64Error {
    #[fail(display = "Unable to read basic N64 ROM information")]
    N64ParseError( #[cause] N64ParseError),
    #[fail(display = "Gamecode <{}> is not a known version of SSB64", _0)]
    UnknownVersion(String)
}

impl From<N64ParseError> for Ssb64Error {
    fn from(e: N64ParseError) -> Self {
        Ssb64Error::N64ParseError(e)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ssb64Version {
    NtscU,
    NtscJ,
    Pal,
    PalA,
}

impl Ssb64Version {
    fn from_rom(rom: &[u8]) -> Result<Self, Ssb64Error> {
        let header = N64Header::from_rom(rom)?;
        
        Ssb64Version::check_version(&header)
    }
    fn check_version(header: &N64Header) -> Result<Self, Ssb64Error> {
        use self::Ssb64Version::*;

        match header.get_game_code() {
            "NALE" => Ok(NtscU),
            "NALJ" => Ok(NtscJ),
            "NALP" => Ok(Pal),
            "NALU" => Ok(PalA),
            unk => Err( Ssb64Error::UnknownVersion(unk.to_string()) ),
        }
    }

    /// Get offsets for (a) pointer to the start of resource file table, 
    /// and (b) the location of the two ASM instructions for number of entries, or size?
    fn get_table_offsets(&self) -> (u32, (u32, u32)) {
        use self::Ssb64Version::*;

        match *self {
            NtscU => (0x41F08, (0x527E8, 0x527F8)),
            NtscJ => unimplemented!(),
            Pal   => unimplemented!(),
            PalA  => unimplemented!(),
        }
    }
}

/// This struct holds a pointer to the ROM data slice, and extracted information from the rom
#[derive(Debug)]
pub struct Ssb64<'rom> {
    version: Ssb64Version,
    rom: &'rom [u8],
    resource_table: ResourceTbl<'rom>, 
}

impl<'rom> Ssb64<'rom> {
    pub fn from_rom(rom: &'rom [u8]) -> Result<Self, Ssb64Error> {
        let version = Ssb64Version::from_rom(rom)?;
        let resource_table = ResourceTbl::from_rom(rom, version);

        Ok(Ssb64{version, rom, resource_table})
    }
}

/// Struct to hold a pointer to the resource file table data
#[derive(Debug)]
struct ResourceTbl<'rom> {
    entries_count: u32,
    start: u32,
    raw: &'rom [u8], // should this hold a mutable slice? or should the table be parsed into objects?
    eof: &'rom [u8],
    ptr_to_next_tbl: u32,
}

impl<'rom> ResourceTbl<'rom> {
    fn from_rom(rom: &'rom [u8], version: Ssb64Version) -> Self {
        // calculate or use size to create new slice of just the resource table
        let (ptr_to_table_start, (size_upper_instruct, size_lower_instruct)) = version.get_table_offsets();
        let ptr_to_table_start  = ptr_to_table_start as usize;
        let size_upper_instruct = size_upper_instruct as usize;
        let size_lower_instruct = size_lower_instruct as usize;
        
        let start = BE::read_u32(&rom[ptr_to_table_start..ptr_to_table_start+4]);
        let entries_count = {
            let upper = BE::read_u32(&rom[size_upper_instruct..size_upper_instruct+4]);
            let lower = BE::read_u32(&rom[size_lower_instruct..size_lower_instruct+4]);

            extract_asm_immediate(upper, lower) as u32
        };
        let end = (start + 12 * entries_count) as usize;

        let raw = &rom[start as usize..end];
        // there is one final entry at the end of the table that points to the start
        // of the next table (for images and sprites) 
        let eof = &rom[end..end+12];
        let ptr_to_next_tbl = BE::read_u32(&eof[0..4]);

        ResourceTbl{entries_count, start, raw, eof, ptr_to_next_tbl}
    }

    /// Return some sort of entry object that has the table info, plus pointer to file data?
    fn get_entry(id: u32) {
        unimplemented!()
    }
}