bitflags! {
    pub struct Flags: u8 {
        const ZERO = 0b1000_0000;
        const SUBTRACT = 0b0100_0000;
        const HALF_CARRY = 0b0010_0000;
        const CARRY = 0b0001_0000;
    }
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: Flags,
    pub h: u8,
    pub l: u8,
}

impl std::convert::From<u8> for Flags {
    fn from(n: u8) -> Self { 
        Flags::from_bits_truncate(n)
    }
}

impl std::convert::From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        flags.bits
    }
}

impl Registers {
    pub fn new() -> Self {
        let mut registers = Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Flags::from(0),
            h: 0,
            l: 0,
        };
        registers.init();
        registers
    }

    fn init(&mut self) {
        self.set_af(0x01B0);
        self.set_bc(0x0013);
        self.set_de(0x00D8);
        self.set_hl(0x014D);
    }

    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 |
        u8::from(self.f) as u16
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 |
        self.c as u16
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 |
        self.e as u16
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 |
        self.l as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = Flags::from((value & 0x00FF) as u8);
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
}

#[cfg(test)]
mod test {

    use super::Flags;

    #[test]
    fn flags_to_u8_individual() {
        assert_eq!(Flags::ZERO.bits, u8::from(Flags::ZERO));
        assert_eq!(Flags::SUBTRACT.bits, u8::from(Flags::SUBTRACT));
        assert_eq!(Flags::HALF_CARRY.bits, u8::from(Flags::HALF_CARRY));
        assert_eq!(Flags::CARRY.bits, u8::from(Flags::CARRY));
    }

    #[test]
    fn flags_to_u8_all() {
        let flags = Flags::ZERO | Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY;
        assert_eq!(flags.bits, u8::from(flags));
    }

    #[test]
    fn u8_to_flags_individual() {
        assert!(Flags::from(Flags::ZERO).contains(Flags::ZERO));
        assert!(Flags::from(Flags::SUBTRACT).contains(Flags::SUBTRACT));
        assert!(Flags::from(Flags::HALF_CARRY).contains(Flags::HALF_CARRY));
        assert!(Flags::from(Flags::CARRY).contains(Flags::CARRY));
    }

    #[test]
    fn u8_to_flags_all() {
        let flags = Flags::ZERO | Flags::SUBTRACT | Flags::HALF_CARRY | Flags::CARRY;
        assert!(Flags::from(flags).contains(Flags::ZERO));
        assert!(Flags::from(flags).contains(Flags::SUBTRACT));
        assert!(Flags::from(flags).contains(Flags::HALF_CARRY));
        assert!(Flags::from(flags).contains(Flags::CARRY));
    }

}