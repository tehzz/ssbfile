/* This file will have the functions need to export a file or an included files list from an u32 index
 * into the ssb64 resource table
 * Basic idea:
 *  export_file(index: u32) -> Result<Vec<u8>, ExportError>
 *  export_includes(index: u32) -> Result<Vec<u8>, ExportError>
 * maybe there should be only one underlying function...? 
* /