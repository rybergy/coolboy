use std::fs::File;
use std::io::{self, Read};
use std::boxed::Box;

const CARTRIDGE_SIZE: usize = 0x200_000;

pub struct Cartridge {
    data: Box<[u8; CARTRIDGE_SIZE]>,
    // data: Vec<u8>,
}

impl Cartridge {
    pub fn from_file(filename: &str) -> Result<Self, io::Error> {
        let mut file = File::open(filename)?;
        // let mut buffer = Vec::with_capacity(0x200_000);
        // let mut buffer = Vec::new();
        let mut buffer = [0; CARTRIDGE_SIZE];

        file.read(&mut buffer)?;
        Ok(Cartridge { data: Box::new(buffer) })
    }

    pub fn read(&self, address: usize) -> u8 {
        match address {
            CARTRIDGE_SIZE..=std::usize::MAX => {
                panic!("Attempting to access cartridge memory {} which is out of bounds!", address);
            },
            _ => self.data[address] 
        }
    }
}
