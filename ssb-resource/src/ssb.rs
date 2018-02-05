use byteorder::{BE, ByteOrder};
use rom_info::{N64Header, N64ParseError, extract_asm_immediate};
use std::collections::BTreeMap;

#[derive(Fail, Debug)]
pub enum Ssb64Error {
    #[fail(display = "Unable to read basic N64 ROM information")]
    N64ParseError( #[cause] N64ParseError),
    #[fail(display = "Gamecode <{}> is not a known version of SSB64", _0)]
    UnknownVersion(String),
    #[fail(display = "Requested file id <{}> was higher than total files <{}>", _0, _1)]
    IllegalFile(u32, u32),
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

/// This struct holds extracted information from a rom byte slice
#[derive(Debug)]
pub struct Ssb64 {
    version: Ssb64Version,
    resource_table: ResourceTbl, 
}

impl Ssb64 {
    pub fn from_rom(rom: & [u8]) -> Result<Self, Ssb64Error> {
        let version = Ssb64Version::from_rom(rom)?;
        let resource_table = ResourceTbl::from_rom(rom, version);

        Ok(Ssb64{version, resource_table})
    }
    pub fn get_res_tbl_entry(&self, rom: &[u8], entry: u32) 
        -> Result<(ResTblEntry, usize), Ssb64Error>
    {
        self.resource_table.get_entry(rom, entry)
    }
}

/// Struct to hold a pointer to the resource file table data
#[derive(Debug)]
pub struct ResourceTbl {
    entries_count: u32,
    start: u32,
    eof: [u8; 12],
    ptr_to_next_tbl: u32,
    end: u32,
}

impl ResourceTbl {
    fn from_rom(rom: &[u8], version: Ssb64Version) -> Self {
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
        let entries_end = (start + 12 * entries_count) as usize;

        // there is one final entry at the end of the table that points to the start
        // of the next table (for images and sprites) 
        let eof = rom[entries_end..entries_end+12]
            .iter()
            .enumerate()
            .fold([0u8; 12], | mut arr, (i, b) | { arr[i] = *b; arr });
        let ptr_to_next_tbl = BE::read_u32(&eof[0..4]);
        let end = (entries_end + 12) as u32;

        ResourceTbl{entries_count, start, eof, ptr_to_next_tbl, end}
    }

    /// Return a tupple containing a ResTblEntry struct and a pointer to that entry's data
    /// in the rom_data byte slice
    fn get_entry(&self, rom_data: &[u8], id: u32) -> Result<(ResTblEntry, usize), Ssb64Error> 
    {
        let &ResourceTbl{start, end, entries_count, ..} = self;
        let start = start as usize; let end = end as usize;

        if id > entries_count { 
            return Err(Ssb64Error::IllegalFile(id, entries_count))
        }

        let id = id as usize;
        let table_data = &rom_data[start..(end-12)];
        let entry_data = unsafe {
            &*(table_data[id*12..(id+1)*12].as_ptr() as *const [u8; 12])
        };
        let entry = ResTblEntry::from(entry_data);
        
        Ok( (entry, entry.calc_ptr(start)) )
    }
}

/// The resource file table information about data
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ResTblEntry {
    compressed: bool,
    offset: u32,
    internal_ptr_list: Option<u32>,
    external_ptr_list: Option<u32>,
    compressed_size: u32,
    decompressed_size: u32,
}

impl ResTblEntry {
    /// Calculate the ROM pointer for a resource table entry, based on the table start
    /// offset `table_start`
    #[inline]
    fn calc_ptr(&self, table_start: usize) -> usize {
        let &Self{offset, ..} = self;
        let offset = offset as usize;
        table_start + offset
    }
    /// Check if the associated data for this entry is vpk0 compressed
    #[inline]
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }
    /// Get a tupple containing the compressed size and decompressed size
    /// of the associated data for this table entry
    pub fn get_size(&self) -> (u32, u32) {
        (self.compressed_size, self.decompressed_size)
    }
}

impl<'a> From<&'a [u8; 12]> for ResTblEntry {
    fn from(arr: &[u8; 12]) -> Self {
        let (offset, compressed) = {
            let word = BE::read_u32(&arr[0..4]);
            ((word & 0x7FFFFFFF), (word & 0x80000000) != 0)
        };
        let internal = BE::read_u16(&arr[4..6]);
        let internal_ptr_list = if internal == 0xFFFF 
            { None } else {Some((internal as u32) << 2)};

        let external = BE::read_u16(&arr[8..10]);
        let external_ptr_list = if external == 0xFFFF 
            { None } else {Some((external as u32) << 2)};
        
        let compressed_size   = (BE::read_u16(&arr[6..8]) as u32) << 2;
        let decompressed_size = (BE::read_u16(&arr[10..12]) as u32) << 2;

        ResTblEntry {
            compressed, offset, 
            internal_ptr_list, external_ptr_list, 
            compressed_size, decompressed_size
        }
    }
}

/// An output struct that holds all of the important, extra information about a resource file
/// If `compress` is `true`, the file will be compressed.
/// The two collections of pointers--`internal_ptrs` and `external_ptrs` are 
/// lists of either offsets (internal) or an offset and file pair (external)
/// to "resource file pointer-ify" 
pub struct ResFileInfo {
    compress: bool,
    internal_ptrs: Option<Vec<u32>>,
    external_ptrs: Option<BTreeMap<u32, u16>>,
}

impl ResFileInfo {
    fn from_ResTblEntry(entry: ResTblEntry, file: &[u8], externals: Option<&[u16]>) 
    -> Self
    {
        let compress      = entry.compressed;
        let internal_ptrs = entry.internal_ptr_list.map(|v| collect_ptr_list(v as usize, file));
        let external_ptrs = entry.external_ptr_list
            .map(|v| {
                let ptrs  = collect_ptr_list(v as usize, file);
                // check for externals without unwrapping
                let f_idx = externals.unwrap(); 
                // check that the ptrs.len() == f_idx.len()
                let iter = ptrs.iter().zip(f_idx.iter());
                let mut map = BTreeMap::new();
                for (ptr, file) in iter {
                    map.insert(*ptr, *file);
                }
                
                map
            });

        ResFileInfo{ compress, internal_ptrs, external_ptrs }
    }
}
/// Create a `Vec` containing offsets to the pointers in the `file` buffer
/// So, collect pointers to "resource file" pointer, without touching the 
fn collect_ptr_list(start: usize, file: &[u8]) -> Vec<u32> {
    let length = file.len();    //TODO: use for error checking... 
    let mut cur = start;
    let mut output = vec![start as u32];
    loop {
        let next = BE::read_u16(&file[cur..cur+2]);
        if next == 0xFFFF { 
            break
        }

        cur = (next as usize) << 2;
        output.push(cur as u32);
    }

    output
}