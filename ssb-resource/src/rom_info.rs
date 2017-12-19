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

// move this enum to a module that deals more closely with editing the ssb64 ROM
/*
pub enum SSB64Version {
    NtscU,
    NtscJ,
    PalE,
    PalA,
}
*/
/// A `struct` that contains "indentifying information" about an N64 ROM. 
struct N64Header<'rom> {
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
    fn from_rom(rom: &'rom [u8]) -> Result<Self, N64ParseError> {
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
/*     
    65 0x41 'A' (not documented, generic NTSC?)
    66 0x42 'B' "Brazilian"
    67 0x43 'C' "Chinese"
    68 0x44 'D' "German"
    69 0x45 'E' "North America"
    70 0x46 'F' "French"
    71 0x47 'G': Gateway 64 (NTSC)
    72 0x48 'H' "Dutch"
    73 0x49 'I' "Italian"
    74 0x4A 'J' "Japanese"
    75 0x4B 'K' "Korean"
    76 0x4C 'L': Gateway 64 (PAL)
    78 0x4E 'N' "Canadian"
    80 0x50 'P' "European (basic spec.)"
    83 0x53 'S' "Spanish"
    85 0x55 'U' "Australian"
    87 0x57 'W' "Scandinavian"
    88 0x58 'X' "Others"
    89 0x59 'Y' "Others"
    90 0x5A 'Z' "Others"
*/
// Information: http://www.emutalk.net/threads/54892-File-spec-for-N64-ROM-formats?p=451757&viewfull=1#post451757