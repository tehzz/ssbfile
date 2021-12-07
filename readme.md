# ssbfile
Utility to export data files from the SSB64 rom. It can export the raw data, the decompressed data, or the decompressed and relocated data.

## Usage
```
ssbfile 0.1.0
A quick utility to export the relocatable data from SSB64

USAGE:
    ssbfile [FLAGS] [OPTIONS] <id> --rom <rom>

FLAGS:
    -e, --emit-relocs    
            emit the location and values of the internal and external relocations
    -h, --help           
            Prints help information
    -V, --version        
            Prints version information

OPTIONS:
    -m, --mode <mode>        
            three ways to export a file: raw, decompress, or reloc
            raw          export the raw data
            decompress   decompress the data, if necessary
            reloc        calculate the relocations (based on a base address of 0) 
            [default: reloc]
    -o, --output <output>    
            output for exported file, or file-id if not present
    -r, --rom <rom>          
            path to SSB64 rom

ARGS:
    <id>    
            file id to export
```
