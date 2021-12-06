use crate::{versions::SSBInfo, Mode};
use anyhow::{anyhow, bail, Context, Result};
use std::{
    borrow::Cow,
    fmt, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

pub(crate) fn data(opt: crate::Opt) -> Result<()> {
    let rom =
        fs::read(&opt.rom).with_context(|| format!("issue opening <{}>", opt.rom.display()))?;
    let version = crate::versions::find_version(&rom)
        .ok_or_else(|| anyhow!("could not determine version for <{}>", opt.rom.display()))?;

    let entry = TableFile::get(opt.id, &rom, version)
        .with_context(|| format!("issue getting table entry for file <{}>", opt.id))?;

    let output = generate_filename(&opt, &entry);
    match opt.mode {
        Mode::RawBytes => fs::write(&*output, entry.raw)
            .with_context(|| format!("writing raw data to <{}>", output.display()))?,
        Mode::Decompressed => {
            let data = if entry.compressed {
                Cow::from(decompress(entry.raw, entry.id)?)
            } else {
                Cow::from(entry.raw)
            };

            fs::write(&*output, &*data)
                .with_context(|| format!("writing data to <{}>", output.display()))?
        }
        Mode::Relocated => {
            let file = if entry.compressed {
                decompress(entry.raw, entry.id)?
            } else {
                entry.raw.to_vec()
            };

            let (file, relocations) = relocate(file, &entry)
                .with_context(|| format!("relocating pointers in file <{}>", entry.id))?;

            fs::write(&*output, &file)
                .with_context(|| format!("writing data to <{}>", output.display()))?;

            if opt.emit_relocs {
                let f = generate_reloc_filename(&*output);
                let relocs = format!("{}", relocations);

                fs::write(&f, relocs.as_bytes())
                    .with_context(|| format!("writing relocations to <{}>", f.display()))?;
            }
        }
    }

    Ok(())
}

/// The start of the runtime relocation list in a file.
/// If the relocations are for pointers into external files,
/// there is the processed list of external file ids.
#[derive(Debug, Clone)]
enum RelocInfo {
    Internal(usize),
    External(usize, Vec<u16>),
}

impl RelocInfo {
    fn get_starting_offset(&self) -> usize {
        match self {
            Self::Internal(o) => *o,
            Self::External(o, _) => *o,
        }
    }

    fn get_external_files(&self) -> Option<&[u16]> {
        match self {
            Self::Internal(..) => None,
            Self::External(_, ex) => Some(ex.as_slice()),
        }
    }
}

struct TableFile<'r> {
    id: usize,
    /// offset from the end of the table
    offset: usize,
    compressed: bool,
    raw: &'r [u8],
    inreloc: Option<RelocInfo>,
    exreloc: Option<RelocInfo>,
}

impl<'r> TableFile<'r> {
    const ENTRY_SIZE: usize = 12;
    const COMPRESS_BIT: u32 = 0x80000000;

    fn get(id: usize, rom: &'r [u8], info: &SSBInfo) -> Result<Self> {
        fn read_checked_u16(raw: &[u8]) -> Result<Option<u16>> {
            raw.try_into()
                .map(u16::from_be_bytes)
                .map(|val| if val == 0xFFFF { None } else { Some(val) })
                .map_err(Into::into)
        }

        if id >= info.total_entries() {
            bail!(
                "Requested file <{}> but table only has {} entries (file id 0 to {})",
                id,
                info.total_entries(),
                info.total_entries() - 1
            );
        }

        let start = info.table_start + id as usize * Self::ENTRY_SIZE;
        let end = start + Self::ENTRY_SIZE;

        let entry = &rom[start..end];
        let offset = u32::from_be_bytes(entry[0..4].try_into()?);
        let compressed = offset & Self::COMPRESS_BIT > 0;
        let offset = (offset & !Self::COMPRESS_BIT) as usize;
        let size = u16::from_be_bytes(entry[6..8].try_into()?) as usize * 4;

        let raw = {
            let fstart = info.table_end + offset;
            let fend = fstart + size as usize;
            &rom[fstart..fend]
        };
        let inreloc = read_checked_u16(&entry[4..6])?
            .map(|x| x as usize * 4)
            .map(RelocInfo::Internal);
        let exreloc = read_checked_u16(&entry[8..10])?
            .map(|x| x as usize * 4)
            .map(|start| {
                Self::get_next_entry_offset(id, rom, info)
                    .and_then(|next_start| {
                        let exoffstart = offset + size;
                        let exoffsize = next_start - exoffstart;
                        let exstart = exoffstart + info.table_end;
                        let exend = exstart + exoffsize;

                        Self::parse_externs(&rom[exstart..exend])
                    })
                    .map(|externs| RelocInfo::External(start, externs))
            })
            .transpose()?;

        Ok(Self {
            id,
            offset,
            compressed,
            raw,
            inreloc,
            exreloc,
        })
    }

    fn parse_externs(raw: &[u8]) -> Result<Vec<u16>> {
        if raw.len() % 2 != 0 {
            bail!("expected list of BE u16, got slice of size {}", raw.len());
        }

        Ok(raw
            .chunks(2)
            .map(|e| u16::from_be_bytes(e.try_into().unwrap()))
            .collect())
    }

    fn get_next_entry_offset(id: usize, rom: &'r [u8], info: &SSBInfo) -> Result<usize> {
        let next = id + 1;
        if next >= info.total_entries() {
            let start = info.table_start + (next * Self::ENTRY_SIZE);
            let table_data_end = u32::from_be_bytes(rom[start..start + 4].try_into()?);

            Ok(table_data_end as usize)
        } else {
            Self::get(next, rom, info).map(|e| e.offset)
        }
    }
}

fn generate_filename<'a>(opt: &'a crate::Opt, entry: &TableFile) -> Cow<'a, Path> {
    opt.output.as_deref().map(Cow::from).unwrap_or_else(|| {
        let s = match opt.mode {
            Mode::RawBytes => format!(
                "raw-{:04}.{}",
                opt.id,
                if entry.compressed { "vpk" } else { "bin" }
            ),
            Mode::Decompressed | Mode::Relocated => format!("file-{:04}.bin", opt.id),
        };

        Cow::from(PathBuf::from(s))
    })
}

fn generate_reloc_filename(datafile: &Path) -> PathBuf {
    let name = format!(
        "{}-relocs.txt",
        datafile
            .file_stem()
            .expect("named bin output file")
            .to_string_lossy()
    );

    datafile.with_file_name(name)
}

fn decompress(data: &[u8], id: usize) -> Result<Vec<u8>> {
    vpk0::decode(Cursor::new(data)).with_context(|| format!("decompressing file <{}>", id))
}

fn relocate(mut file: Vec<u8>, entry: &TableFile) -> Result<(Vec<u8>, FileReloc)> {
    let mut relocs = FileReloc {
        internal: None,
        external: None,
    };
    // relocation data stored as BE {u16 next; u16 ptrOffset}
    // next * 4 is the location of the next relocation
    // ptrOffset * 4 + baseAddr is the value of the pointer
    if let Some(reloc) = &entry.inreloc {
        relocs.internal = Some(write_relocations(&mut file, reloc)?);
    }

    if let Some(exreloc) = &entry.exreloc {
        relocs.external = Some(write_relocations(&mut file, exreloc)?);
    }

    Ok((file, relocs))
}

fn write_relocations(file: &mut [u8], reloc: &RelocInfo) -> Result<Relocations> {
    const END: usize = 0xFFFF * 4;
    let mut relocations = Relocations::with_capacity(64);

    let mut ex = reloc.get_external_files().map(|x| x.into_iter());
    let mut next = reloc.get_starting_offset();
    while next != END {
        let reloc = &mut file[next..next + 4];
        let raw_next = u16::from_be_bytes(reloc[0..2].try_into()?);
        let raw_ptr = u16::from_be_bytes(reloc[2..4].try_into()?);

        let ptr = raw_ptr as u32 * 4;
        reloc.copy_from_slice(&ptr.to_be_bytes());
        // lazy, but whatever; if external use the file id; else just put in 0
        let fid = ex.as_mut().and_then(|x| x.next()).copied().unwrap_or(0);
        relocations.push((fid, next, ptr));

        next = raw_next as usize * 4;
    }

    Ok(relocations)
}

/// (file, &ptr, ptr)
type Relocations = Vec<(u16, usize, u32)>;

#[derive(Debug)]
struct FileReloc {
    internal: Option<Relocations>,
    external: Option<Relocations>,
}

impl fmt::Display for FileReloc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# Relocations")?;
        if let Some(internal) = &self.internal {
            writeln!(f, "## Internal Relocations")?;
            for &(_, offset, ptr) in internal {
                writeln!(f, "* {:06X} -> {:08X}", offset, ptr)?;
            }
        }
        writeln!(f, "")?;
        if let Some(external) = &self.external {
            writeln!(f, "## External Relocations")?;
            for &(fid, offset, ptr) in external {
                writeln!(f, "* {:06X} -> {:08X} from {}", offset, ptr, fid)?;
            }
        }
        Ok(())
    }
}
