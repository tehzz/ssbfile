/* This file will have the functions need to export a file or an included files list from an u32 index
 * into the ssb64 resource table
 * Basic idea:
 *  export_file(index: u32) -> Result<Vec<u8>, ExportError>
 *  export_includes(index: u32) -> Result<Vec<u8>, ExportError>
 * maybe there should be only one underlying function...? 
*/
use std::io::Cursor;
use ssb::{Ssb64, Ssb64Error};
use vpk0::{self, VpkError};
use ResError;

/// Errors arising from exporting resource file data from an SSB64 ROM
#[derive(Fail, Debug)]
pub enum ExportError {
    #[fail( display = "Problem parsing ROM as a version of SSB64")]
    Ssb64Error(#[cause] Ssb64Error),
    #[fail (display = "Problem decoding vpk0 data")]
    VpkDecode(#[cause] VpkError),
}

impl From<Ssb64Error> for ExportError {
    fn from(e: Ssb64Error) -> Self { ExportError::Ssb64Error(e) }
}

impl From<VpkError> for ExportError {
    fn from(e: VpkError) -> Self { ExportError::VpkDecode(e) }
}

/// The real function that exports the file data from a ROM byte slice
fn export_file(rom: &[u8], index: u32, decompress: bool) 
    -> Result<Vec<u8>, ExportError> 
{
    // process the rom into a Ssb64 struct
    let ssb = Ssb64::from_rom(rom)?;
    let (entry, ptr) = ssb.get_res_tbl_entry(rom, index)?;
    let compressed = entry.is_compressed();
    let (compressed_size, ..) = entry.get_size();

    // extract data, and decompress if necessary
    let raw_data = &rom[ptr..ptr + compressed_size as usize];
    let data = if compressed && decompress {
        let csr = Cursor::new(raw_data);
        vpk0::decode(csr)?
    } else {
        raw_data.to_vec()
    };

    Ok(data)
}

/// Export the external pointer nodes of file `index` from ROM slice `&[u8] rom`
//fn export_externals(rom: &[u8], index: u32)

/// Export file number `index` from a `&[u8] ROM` buffer.
/// If `decompress` is `true`, the exported file is decompressed from its raw VPK0 data;
/// otherwise, the raw data is returned (which could be either the actual binary file, or
/// a vpk0 compressed file.)
pub fn file(rom: &[u8], index: u32, decompress: bool)
    -> Result<Vec<u8>, ResError>
{
    export_file(rom, index, decompress).map_err(|e| e.into())
}