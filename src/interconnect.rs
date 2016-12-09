use rom::*;

pub struct Interconnect {
    rom: Rom,
}

impl Interconnect {
    pub fn new(rom: Rom) -> Interconnect {
        Interconnect { rom: rom }
    }
}
