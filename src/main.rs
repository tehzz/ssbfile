extern crate clap;
#[macro_use] extern crate failure;
extern crate ssb_resource;

use clap::{App, Arg, SubCommand};
use failure::{Error};
use ssb_resource::{export};

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

fn run() -> Result<(), Error> {
    let matches = cli().get_matches();

    println!("Hello, world!");
    println!("{:?}", matches);

    Ok(())
}

fn cli<'a, 'b>() -> App<'a, 'b> {
    let export_complete = SubCommand::with_name("complete")
        .about("Fully export all data associated with a resource file")
        .long_about("This command will export a resource file by \n\
        recursively including all files specified by the file's \n\
        included files list and processing all pointers. \n\
        The exported binary will be similar to what happens when \n\
        this file is brought into RAM by SSB64")
        .args(&[
            Arg::with_name("rom")
                .help("Path to SSB64 rom")
                .takes_value(true)
                .required(true)
                .index(1),
            Arg::with_name("file-index")
                .help("The index of the file to extract (zero-indexed)")
                .long_help("The index of the file to extract from the ROM. Both decimal and hex (0x) values are accepted")
                .takes_value(true)
                .required(true)
                .index(2),
                //.multiple(true) // maybe allow for more than 1 index to extract multiple files at once?
            Arg::with_name("rename")
                .help("Rename the extracted file from the default name to [rename]")
                .long_help("Rename any files extracted from a resource table entry to [rename].bin|.idx|.vpk0.\n\
                By default, files are named based on their index value (either in decimal or hex: see --hex-name)")
                .takes_value(true)
                .short("n")
                .long("rename"),
            Arg::with_name("hex-name")
                .help("Name file with its index in hex, rather than in decimal")
                .takes_value(false)
                .conflicts_with("rename")
                .short("x")
                .long("hex-name"),
        ]);
    
    let export = SubCommand::with_name("export")
        .about("Export a resource file from an SSB64 ROM.")
        .long_about("Export a resource file from an SSB64 ROM.\n\
        Configurable to export just the file, just the included files list, \n\
        the file and included file list, or the combined file and included file list.")
        .args(&[
            Arg::with_name("rom")
                .help("Path to SSB64 rom")
                .takes_value(true)
                .required(true)
                .index(1),
            Arg::with_name("file-index")
                .help("The index of the file to extract (zero-indexed)")
                .long_help("The index of the file to extract from the ROM. Both decimal and hex (0x) values are accepted")
                .takes_value(true)
                .required(true)
                .index(2),
                //.multiple(true) // maybe allow for more than 1 index to extract multiple files at once?
            Arg::with_name("file")
                .help("Extract the binary file associated with <file-index>")
                .long_help("Extract the binary file associated with <file-index>.\n\
                This flag will only extract the file data, and will not extract or contain the included files list.\n\
                To extract both the file and the included files list at once, pass \"-fi\".\n\
                To extract the combined file and included files list, pass \"-c\".")
                .takes_value(false)
                .conflicts_with("combined")
                .short("f")
                .long("file"),
            Arg::with_name("includes")
                .help("Extract the included files list associated with <file-index>")
                .long_help("Extract the included files list associated with <file-index>.\n\
                This flag will only extract the included files list, and will not extract or contain the file data.\n\
                To extract both the file and the included files list at once, pass \"-fi\".\n\
                To extract the combined file and included files list, pass \"-c\".")
                .takes_value(false)
                .conflicts_with("combined")
                .short("i")
                .long("includes"),
            Arg::with_name("combined")
                .help("Extract both the file data and the included files list into one file")
                .takes_value(false)
                .conflicts_with("combined")
                .short("c")
                .long("combined"),
            Arg::with_name("rename")
                .help("Rename the extracted file from the default name to [rename]")
                .long_help("Rename any files extracted from a resource table entry to [rename].bin|.idx|.vpk0.\n\
                By default, files are named based on their index value (either in decimal or hex: see --hex-name)")
                .takes_value(true)
                .short("n")
                .long("rename"),
            Arg::with_name("hex-name")
                .help("Name file with its index in hex, rather than in decimal")
                .takes_value(false)
                .conflicts_with("rename")
                .short("x")
                .long("hex-name"),
            Arg::with_name("raw")
                .help("Do not decode vpk0 files when exporting a file")
                .takes_value(false)
                .short("r")
                .long("raw")
        ])
        .subcommand(export_complete);

    let import = SubCommand::with_name("import");

    App::new(env!("CARGO_PKG_NAME"))
        .about("A command-line utility to export, import, and add files to SSB64's resource file table")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(export)
        .subcommand(import)
}
