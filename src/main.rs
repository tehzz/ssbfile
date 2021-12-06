use anyhow::Result;
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

mod extract;
mod versions;

/// A quick utility to export the relocatable data from SSB64
#[derive(Debug, StructOpt)]
struct Opt {
    /// path to SSB64 rom
    #[structopt(short, long, parse(from_os_str))]
    rom: PathBuf,
    /// output for exported file, or file-id if not present
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// emit the location and values of the internal and external relocations
    #[structopt(short, long)]
    emit_relocs: bool,
    /// three ways to export a file: raw, decompress, or reloc
    ///
    /// raw          export the raw data
    ///
    /// decompress   decompress the data, if necessary
    ///
    /// reloc        calculate the relocations (based on a base address of 0)
    #[structopt(default_value = "reloc", short, long, parse(try_from_str))]
    mode: Mode,
    /// file id to export
    id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    RawBytes,
    Decompressed,
    Relocated,
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" | "bytes" => Ok(Self::RawBytes),
            "decompress" => Ok(Self::Decompressed),
            "reloc" | "full" => Ok(Self::Relocated),
            _ => Err(anyhow::anyhow!("Unknown mode <{}>", s)),
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    extract::data(opt)
}
