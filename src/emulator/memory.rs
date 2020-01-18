use super::cartridge::Cartridge;
use std::io;

use super::TIMER_ADDRESS;
use super::TIMER_CONTROLLER;
use super::TIMER_MODULATOR;

const RBM_ADDRESS: usize = 0x147;
const MEMORY_SIZE: usize = 0x10000;

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const MAX_RAMBANK: usize = 4;

pub enum RomBankMode {
    No,
    MBC1,
    MBC2
}

pub struct Memory {
    rom: Box<[u8; MEMORY_SIZE]>,
    cart: Cartridge,
    rom_bank_mode: RomBankMode,

    current_rom_bank: usize,
    ram_banks: Box<[u8; MAX_RAMBANK * RAM_BANK_SIZE]>,
    current_ram_bank: usize,

    enable_ram: bool,
    enable_rom: bool,
}

impl Memory {
    pub fn from_file(filename: &str) -> Result<Self, io::Error> {
        let cart = Cartridge::from_file(filename)?;

        let rbm_byte = cart.read(RBM_ADDRESS);
        let rbm = match rbm_byte {
            0 => RomBankMode::No,
            1 | 2 | 3 => RomBankMode::MBC1,
            5 | 6 => RomBankMode::MBC2,
            _ => panic!("Unknown ROM Bank Mode Byte {}!", rbm_byte),
        };

        let mut mem = Memory { 
            rom: Box::new([0; MEMORY_SIZE]),
            cart: cart,
            rom_bank_mode: rbm,
            current_rom_bank: 0,
            ram_banks: Box::new([0; MAX_RAMBANK * RAM_BANK_SIZE]),
            current_ram_bank: 0,
            enable_ram: false,
            enable_rom: false,
        };

        mem.init();
        Ok(mem)
    }

    fn init(&mut self) {
        self.rom[0xFF05] = 0x00;
        self.rom[0xFF06] = 0x00;
        self.rom[0xFF07] = 0x00;
        self.rom[0xFF10] = 0x80;
        self.rom[0xFF11] = 0xBF;
        self.rom[0xFF12] = 0xF3;
        self.rom[0xFF14] = 0xBF;
        self.rom[0xFF16] = 0x3F;
        self.rom[0xFF17] = 0x00;
        self.rom[0xFF19] = 0xBF;
        self.rom[0xFF1A] = 0x7F;
        self.rom[0xFF1B] = 0xFF;
        self.rom[0xFF1C] = 0x9F;
        self.rom[0xFF1E] = 0xBF;
        self.rom[0xFF20] = 0xFF;
        self.rom[0xFF21] = 0x00;
        self.rom[0xFF22] = 0x00;
        self.rom[0xFF23] = 0xBF;
        self.rom[0xFF24] = 0x77;
        self.rom[0xFF25] = 0xF3;
        self.rom[0xFF26] = 0xF1;
        self.rom[0xFF40] = 0x91;
        self.rom[0xFF42] = 0x00;
        self.rom[0xFF43] = 0x00;
        self.rom[0xFF45] = 0x00;
        self.rom[0xFF47] = 0xFC;
        self.rom[0xFF48] = 0xFF;
        self.rom[0xFF49] = 0xFF;
        self.rom[0xFF4A] = 0x00;
        self.rom[0xFF4B] = 0x00;
        self.rom[0xFFFF] = 0x00; 
    }

    pub fn write(&mut self, address: usize, data: u8) {
        match address {
            MEMORY_SIZE..=std::usize::MAX => {
                panic!("Attempting to write to address {} which is out of range!", address);
            },
            0x0000..=0x7FFF => { 
                self.handle_banking(address, data);
            },
            0xA000..=0xBFFF => {
                if self.enable_ram {
                    let translated = (address - 0xA000) + (self.current_ram_bank * RAM_BANK_SIZE);
                    self.ram_banks[translated] = data;
                }
            }
            0xE000..=0xFDFF => {
                // Echo memory; write to this address and 0x2000 addresses back
                self.rom[address] = data;
                self.write(address - 0x2000, data);
            },
            0xFEA0..=0xFEFE => { 
                warn!("Attempting to write to address {} which is restricted!", address);
            },
            0xFF04 => {
                // Divider register - if written, set to 0
                self.rom[address] = 0;
            },
            0xFF44 => {
                // Scanline counter - if written, set to 0
                self.rom[address] = 0;
            },
            _ => { 
                self.rom[address] = data; 
            }
        }
    }

    pub fn write_force(&mut self, address: usize, data: u8) {
        // Does not do any checks for addresses, just writes directly 
        self.rom[address] = data;
    }

    pub fn read_force(&self, address: usize) -> u8 {
        self.rom[address]
    }

    fn handle_banking(&mut self, address: usize, data: u8) {
        match address {
            0x0000..=0x1FFF => {
                match self.rom_bank_mode {
                    RomBankMode::MBC1 | RomBankMode::MBC2 => {
                        self.handle_ram_bank_enable(address, data);
                    },
                    _ => ()
                }
            },
            0x2000..=0x3FFF => {
                match self.rom_bank_mode {
                    RomBankMode::MBC1 | RomBankMode::MBC2 => {
                        self.handle_change_lo_rom_bank(data);
                    },
                    _ => ()
                }
            },
            0x4000..=0x5FFF => {
                match self.rom_bank_mode {
                    RomBankMode::MBC1 => {
                        if self.enable_rom {
                            self.handle_change_hi_rom_bank(data)
                        }
                        else {
                            self.handle_change_ram_bank(data);
                        }
                    },
                    _ => ()
                }
            },
            0x6000..=0x7FFF => {
                match self.rom_bank_mode {
                    RomBankMode::MBC1 => {
                        self.handle_change_rom_ram_mode(data);
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }

    fn handle_ram_bank_enable(&mut self, address: usize, data: u8) {
        match self.rom_bank_mode {
            RomBankMode::MBC2 => {
                if (address & 0b00010000) != 0 {
                    return;
                }
            },
            _ => ()
        }

        let lower_nibble = data & 0xF;
        match lower_nibble {
            0xA => {
                self.enable_ram = true;
            },
            0x0 => {
                self.enable_ram = false;
            },
            _ => ()
        }


    }

    fn handle_change_lo_rom_bank(&mut self, data: u8) {
        match self.rom_bank_mode {
            RomBankMode::MBC2 => {
                self.current_rom_bank = (data & 0xF) as usize;
                self.current_rom_bank += 1;
            },
            _ => ()
        }

        let lower_five = data & 0b00011111;

        self.current_rom_bank &= 0b11100000;
        self.current_rom_bank |= lower_five as usize;

        if self.current_rom_bank == 0 {
            self.current_rom_bank += 1;
        }
    }

    fn handle_change_hi_rom_bank(&mut self, data: u8) {
        self.current_rom_bank &= 0b00011111;

        let masked_data = data & 0b11100000;
        self.current_rom_bank |= masked_data as usize;

        if self.current_rom_bank == 0 {
            self.current_rom_bank += 1;
        }
    }

    fn handle_change_ram_bank(&mut self, data: u8) {
        self.current_ram_bank = (data & 0x3) as usize;
    }

    fn handle_change_rom_ram_mode(&mut self, data: u8) {
        let bit_one = data & 0b00000001;
        self.enable_rom = bit_one == 0;

        if self.enable_rom {
            self.current_ram_bank = 0;
        }
    }

    pub fn read(&self, address: usize) -> u8 {
        match address {
            MEMORY_SIZE..=std::usize::MAX => {
                panic!("Attempting to read address {} which is out of range!", address);
            },
            0x4000..=0x7FFF => {
                // Reading from ROM bank
                let translated = (address - 0x4000) + (self.current_rom_bank * ROM_BANK_SIZE);
                self.cart.read(translated)
            },
            0xA000..=0xBFFF => {
                // Reading from RAM bank
                let translated = (address - 0xA000) + (self.current_ram_bank * RAM_BANK_SIZE);
                self.ram_banks[translated]
            },
            _ => self.rom[address]
        }
    }
}
