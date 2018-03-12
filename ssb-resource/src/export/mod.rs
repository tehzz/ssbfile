/* This file will have the functions need to export a file or an included files list from an u32 index
 * into the ssb64 resource table
 * Change to
 *  both(file_index: u32, rom: &[u8]) <- convert interal pointers only, or not?
 *  file()
 *  includes()
 *  raw_file()
 *  raw_includes()
**/
use std::io::Cursor;
use ssb::{Ssb64, Ssb64Error, ResTblEntry, ResFileInfo};
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

/// Get an `Ssb64` struct with the file table, 
/// and file `index`'s data and its `ResTblEntry` from a buffer of the ROM data
fn get_triple(rom: &[u8], index: u32, decompress: bool) 
    -> Result<(Ssb64, Vec<u8>, ResTblEntry), ExportError> 
{
    // process the rom into a Ssb64 struct
    let ssb = Ssb64::from_rom(rom)?;
    println!("{:#?}", ssb);
    let (entry, ptr) = ssb.get_res_tbl_entry(rom, index)?;
    println!("file @ {:#X}:\n{:#?}", ptr, entry);
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

    Ok((ssb, data, entry))
}

/// Export id `entry`'s data and  information from a `&[u8]` SSB64 ROM buffer
fn get_file_and_info(rom: &[u8], entry: u32) 
    -> Result<(Vec<u8>, ResFileInfo), ExportError>
{
    let (ssb, file_data, tbl_entry) = get_triple(rom, entry, true)?;
    let req_files = ssb.get_res_tbl_includes(rom, entry)?;
    let file_info = ResFileInfo::from_tbl_entry(&tbl_entry, &file_data, req_files.as_ref().map(|v| &v[..]));

    Ok((file_data, file_info))
}

/// Export file number `index` from a `&[u8] ROM` buffer.
/// If `decompress` is `true`, the exported file is decompressed from its raw VPK0 data;
/// otherwise, the raw data is returned (which could be either the actual binary file, or
/// a vpk0 compressed file.)
pub fn file(rom: &[u8], index: u32, decompress: bool)
    -> Result<Vec<u8>, ResError>
{
    get_triple(rom, index, decompress)
        .map(|(_, d, _)| d)
        .map_err(|e| e.into())
}

/// Export information for file number `index` from a `&[u8] ROM` buffer.
/// In order to get file information, the file will have to be decompressed.
pub fn info(rom: &[u8], index: u32)
    -> Result<ResFileInfo, ResError>
{
    get_file_and_info(rom, index)
        .map( |(_, i)| i)
        .map_err(|e| e.into())
}

/// Export id `entry`'s data and  information from a `&[u8]` SSB64 ROM buffer
/// In order to get file information, the file will have to be decompressed.
pub fn file_and_info(rom: &[u8], index: u32)
    -> Result<(Vec<u8>, ResFileInfo), ResError>
{
    get_file_and_info(rom, index)
        .map_err(|e| e.into())
}