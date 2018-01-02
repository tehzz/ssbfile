extern crate byteorder;
#[macro_use] extern crate failure;

mod rom_info;
mod ssb;
mod export;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
