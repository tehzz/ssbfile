#![allow(dead_code)]

use byteorder::{ByteOrder, BE};
use std::str;

/// The errors that can be caused from attempting to parse the N64 rom for its header
#[derive(Debug, Fail)]
pub enum N64ParseError {
    #[fail(display = "ROM Image was only {:#x} bytes", _0)]
    ImageTooSmall(usize),
    #[fail(display = "Unable to parse name of ROM image into a string")]
    Name(#[cause] str::Utf8Error),
    #[fail(display = "Unable to parse game code of ROM image into a string")]
    GameCode(#[cause] str::Utf8Error),
    #[fail(display = "Unknown Media Format <{}>", _0)]
    UnknownMediaFormat(char),
    #[fail(display = "Unknown Country <{}>", _0)]
    UnknownCountry(char),
}

/// A `struct` that contains "indentifying information" about an N64 ROM. 
pub struct N64Header<'rom> {
    crc1: u32,
    crc2: u32,
    name: &'rom str,
    game_code: &'rom str,
    format: N64MediaFormat,
    country_code: N64CountryCode,
    version: u8,
}
impl<'rom> N64Header<'rom> {
    /// Parse a byte slice of a big-endian ROM image into an N64Header struct. Note that this function
    /// assumes that the slice starts at the beginning of the ROM.
    pub fn from_rom(rom: &'rom [u8]) -> Result<Self, N64ParseError> {
        if rom.len() < 0x40 { return Err( N64ParseError::ImageTooSmall( rom.len() ) ) }

        let crc1 = BE::read_u32(&rom[0x10..0x14]);
        let crc2 = BE::read_u32(&rom[0x14..0x18]);
        let name = str::from_utf8(&rom[0x20..0x34])
            .map_err(|e| N64ParseError::Name(e))?;
        let game_code = str::from_utf8(&rom[0x3b..0x3f])
            .map_err(|e| N64ParseError::GameCode(e))?;
        let version = rom[0x3f];
        let format = N64MediaFormat::from_game_code(game_code)?;
        let country_code = N64CountryCode::from_game_code(game_code)?;

        Ok(N64Header {
            crc1, crc2, name, game_code, format, country_code, version
        })
    }
    pub fn get_game_code(&self) -> &str {
        self.game_code
    }
}

/// The types of N64 media as recognized by Nintendo for their "game code" naming schema.
enum N64MediaFormat {
    Cart,
    Disk,
    ExpandableCart,
}

impl N64MediaFormat {
    fn from_game_code(code: &str) -> Result<Self, N64ParseError>{
        use self::N64MediaFormat::*;

        let format = code.chars().next();

        match format {
            Some('N') => Ok(Cart),
            Some('D') => Ok(Disk),
            Some('E') => Ok(ExpandableCart),
            Some(unk) => Err(N64ParseError::UnknownMediaFormat(unk)),
            None => Err(N64ParseError::UnknownMediaFormat('?')),
        }  
    }
}

/// The regions for a game as recognized by Nintendo for their "game code" naming schema.
enum N64CountryCode {
    GenericNTSC,
    Brazilian,
    Chinese,
    German,
    NorthAmerica,
    French,
    Gateway64NTSC,
    Dutch,
    Italian,
    Japanese,
    Korean,
    Gateway64PAL,
    Canadian,
    European,
    Spanish,
    Australian,
    Scandinavian,
    Others,
}

impl N64CountryCode {

    fn from_game_code(code: &str) -> Result<Self, N64ParseError> {
        use self::N64CountryCode::*;

        let country = code.get(3..4);

        match country {
            Some("A") => Ok(GenericNTSC),
            Some("B") => Ok(Brazilian),
            Some("C") => Ok(Chinese),
            Some("D") => Ok(German),
            Some("E") => Ok(NorthAmerica),
            Some("F") => Ok(French),
            Some("G") => Ok(Gateway64NTSC),
            Some("H") => Ok(Dutch),
            Some("I") => Ok(Italian),
            Some("J") => Ok(Japanese),
            Some("K") => Ok(Korean),
            Some("L") => Ok(Gateway64PAL),
            Some("N") => Ok(Canadian),
            Some("P") => Ok(European),
            Some("S") => Ok(Spanish),
            Some("U") => Ok(Australian),
            Some("W") => Ok(Scandinavian),
            Some("X") | Some("Y") | Some("Z") => Ok(Others),
            Some(unk) => Err(N64ParseError::UnknownCountry(unk.as_bytes()[0] as char)),
            None => Err(N64ParseError::UnknownCountry('?'))
        }
    }
}
// Information: http://www.emutalk.net/threads/54892-File-spec-for-N64-ROM-formats?p=451757&viewfull=1#post451757

/// Helper function to extract an 32-bit value from the immediate fields
/// of two MIPS instructions
pub fn extract_asm_immediate(upper: u32, lower: u32) -> i32 {
    let u = (upper as i16 as i32) << 16;
    
    // check for ori (0x20..), as that's the only relevant unsigned opcode
    let l = if (lower >> 24) == 0x20 {
        (lower & 0xFFFF) as i32
    } else {
        (lower & 0xFFFF) as i16 as i32
    };
    
    u + l
}