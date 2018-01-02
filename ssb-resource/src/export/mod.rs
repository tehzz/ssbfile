/* This file will have the functions need to export a file or an included files list from an u32 index
 * into the ssb64 resource table
 * Basic idea:
 *  export_file(index: u32) -> Result<Vec<u8>, ExportError>
 *  export_includes(index: u32) -> Result<Vec<u8>, ExportError>
 * maybe there should be only one underlying function...? 
*/

use ssb::{Ssb64, Ssb64Error};

#[derive(Fail, Debug)]
pub enum ExportError {
    #[fail( display = "Problem parsing ROM as a version of SSB64")]
    Ssb64Error(#[cause] Ssb64Error)
}

impl From<Ssb64Error> for ExportError {
    fn from(e: Ssb64Error) -> Self {
        ExportError::Ssb64Error(e)
    }
}


pub fn export_file(rom: &[u8], index: u32) -> Result<Vec<u8>, ExportError> 
{
    // process the rom into a Ssb64 struct
    let ssb = Ssb64::from_rom(rom)?;

    unimplemented!()
}