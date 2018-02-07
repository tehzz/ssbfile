extern crate clap;
extern crate path_abs;
#[macro_use] extern crate failure;
extern crate ssb_resource;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::{Error};
use ssb_resource::{export};
use path_abs::{PathAbs};
use std::io::{Read};

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let err_msg = "Error writing to stderr";

        writeln!(stderr, "Error: {}", e).expect(err_msg);

        for e in e.causes().skip(1) {
            writeln!(stderr, "Caused by: {}", e).expect(err_msg);
        }

        // The backtrace is not always generated. Run with `RUST_BACKTRACE=1`
        let backtrace = e.backtrace();
        writeln!(stderr, "backtrace: {:?}", backtrace).expect(err_msg);

        ::std::process::exit(1);
    }
}
enum Mode {
    Export,
    Complete,
    Import,
    Add,
}

// Command Strings for clap
const CMD_EXPORT: &'static str   = "export";
const CMD_COMPLETE: &'static str = "complete";
const CMD_IMPORT: &'static str   = "import";
const CMD_ADD: &'static str      = "add";
// Positional Args for clap
const POS_ROM: &'static str   = "rom";
const POS_ENTRY: &'static str = "file-index";
// Flag Args for clap
const F_RAW: &'static str      = "raw";
const F_CVRTPTR: &'static str  = "convert-ptrs";
const F_RENAME: &'static str   = "rename";
const F_HEXNAME: &'static str  = "hex-name";
const F_FILEONLY: &'static str = "file-only";
const F_INFOONLY: &'static str = "info-only";

fn run() -> Result<(), Error> {
    let matches = cli().get_matches();

    println!("Hello, world!");
    println!("{:?}", matches);

    let mode = match matches.subcommand_name() {
        Some(CMD_EXPORT)   => Mode::Export,
        Some(CMD_COMPLETE) => Mode::Complete,
        Some(CMD_IMPORT)   => Mode::Import,
        Some(CMD_ADD)      => Mode::Add,
        Some(unk)          => bail!("Unknown subcommand <{}>", unk),
        None               => bail!("No subcommand passed to application"),
    };

    match mode {
        Mode::Export   => export(matches.subcommand_matches(CMD_EXPORT).unwrap())?,
        Mode::Complete => unimplemented!(),
        Mode::Import   => unimplemented!(),
        Mode::Add      => unimplemented!(),
    }

    Ok(())
}

fn export(matches: &ArgMatches) -> Result<(), Error> {
    let rom_path = matches.value_of(POS_ROM).unwrap();
    let file_idx: u32 = if let Some(idx) = matches.value_of(POS_ENTRY) {
        if idx.len() > 2 && &idx[0..2] == "0x" {
            u32::from_str_radix(&idx[2..], 16)?
        } else { 
            u32::from_str_radix(idx, 10)?
        }
    } else { bail!("No file index provided") };

    let mut rom_file = PathAbs::new(&rom_path)?.into_file()?.read()?;
    let mut rom = Vec::new();

    rom_file.read_to_end(&mut rom)?;

    println!("{} - {}", rom_path, file_idx);
    println!("rom size: {}", rom.len());

    Ok(())
}

fn cli<'a, 'b>() -> App<'a, 'b> {
    let rom_arg = Arg::with_name(POS_ROM)
        .help("Path to SSB64 rom")
        .takes_value(true)
        .required(true)
        .index(1);
    
    let entry_arg = Arg::with_name(POS_ENTRY)
        .help("The index of the file to extract (zero-indexed)")
        .long_help("The index of the file to extract from the ROM. Both decimal and hex (0x) values are accepted")
        .takes_value(true)
        .required(true)
        .index(2);
    
    let raw_flag = Arg::with_name(F_RAW)
        .help("Do not decode vpk0 files when exporting")
        .takes_value(false)
        .conflicts_with(F_CVRTPTR)
        .short("r")
        .long("raw");
    
    let convert_pointers_flag = Arg::with_name(F_CVRTPTR)
        .help("Convert from the internally used pointer nodes to file offsets")
        .takes_value(false)
        .conflicts_with(F_RAW)
        .short("c")
        .long("convertptrs");

    let rename_flag = Arg::with_name(F_RENAME)
        .help("Rename the extracted file from the default name to [rename]")
        .long_help("Rename any files extracted from a resource table entry to [rename].bin|.yaml|.vpk0.\n\
        By default, files are named based on their index value (either in decimal or hex)")
        .takes_value(true)
        .short("n")
        .long("rename");
    
    let hex_name_flag = Arg::with_name(F_HEXNAME)
        .help("Name file with its index in hex, rather than in decimal")
        .takes_value(false)
        .conflicts_with(F_RENAME)
        .short("x")
        .long("hex-name");
    
    let file_only_flag = Arg::with_name(F_FILEONLY)
        .help("Extract only the binary data associated with a resource file")
        .takes_value(false)
        .conflicts_with(F_INFOONLY)
        .conflicts_with(F_CVRTPTR)
        .short("f")
        .long("file");
    
    let info_only_flag = Arg::with_name(F_INFOONLY)
        .help("Extract the extra information associated with a resource file as a .yaml file")
        .takes_value(false)
        .conflicts_with(F_FILEONLY)
        .conflicts_with(F_CVRTPTR)
        .conflicts_with(F_RAW)
        .short("i")
        .long("info");
        
    let export_complete = SubCommand::with_name(CMD_COMPLETE)
        .about("Fully export and process all data associated with a resource file")
        .long_about("This command will export a resource file by \n\
            recursively including all files specified by the file's \n\
            included files list and processing all pointers. \n\
            The exported binary will be similar to what happens when \n\
            this file is brought into RAM by SSB64")
        .args(&[ 
            rom_arg.clone(), 
            entry_arg.clone(), 
            rename_flag.clone(),
            hex_name_flag.clone(),
        ]);
    
    let export = SubCommand::with_name(CMD_EXPORT)
        .about("Export a resource file from an SSB64 ROM.")
        .long_about("Export a resource file from an SSB64 ROM.\n\
            Configurable to export just the file, just the included files list, \n\
            the file and included file list, or the combined file and included file list.")
        .args(&[
            rom_arg.clone(),
            entry_arg.clone(),
            file_only_flag.clone(),
            info_only_flag.clone(),
            raw_flag.clone(),
            hex_name_flag.clone(),
            rename_flag.clone(),
        ]);

    let import = SubCommand::with_name(CMD_IMPORT);

    App::new(env!("CARGO_PKG_NAME"))
        .about("A command-line utility to export, import, and add files to SSB64's resource file table")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(export)
        .subcommand(export_complete)
        .subcommand(import)
}
