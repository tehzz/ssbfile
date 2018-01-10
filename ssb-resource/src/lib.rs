extern crate byteorder;
#[macro_use] extern crate failure;
extern crate vpk0;

mod rom_info;
mod ssb;
mod export;

use export::{ExportError};
pub use export::{file};

#[derive(Fail, Debug)]
pub enum ResError {
    #[fail(display = "Problem exporting file")]
    Export(#[cause] ExportError)
}

impl From<ExportError> for ResError {
    fn from(e: ExportError) -> Self { ResError::Export(e) }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
