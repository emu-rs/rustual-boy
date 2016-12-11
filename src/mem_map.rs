pub const LINK_CONTROL_REG: u32 = 0x02000000;
pub const AUX_LINK_REG: u32 = 0x02000004;
pub const WAIT_CONTROL_REG: u32 = 0x02000024;

pub const WRAM_START: u32 = 0x05000000;
pub const WRAM_LENGTH: u32 = 0x01000000;
pub const WRAM_END: u32 = WRAM_START + WRAM_LENGTH - 1;

pub const CARTRIDGE_ROM_START: u32 = 0x07000000;
pub const CARTRIDGE_ROM_LENGTH: u32 = 0x01000000;
pub const CARTRIDGE_ROM_END: u32 = CARTRIDGE_ROM_START + CARTRIDGE_ROM_LENGTH - 1;

pub enum MappedAddress {
    LinkControlReg,
    AuxLinkReg,
    WaitControlReg,
    Wram(u32),
    CartridgeRom(u32)
}

pub fn map_address(addr: u32) -> MappedAddress {
    let addr = addr & 0x07ffffff;
    match addr {
        LINK_CONTROL_REG => MappedAddress::LinkControlReg,
        AUX_LINK_REG => MappedAddress::AuxLinkReg,
        WAIT_CONTROL_REG => MappedAddress::WaitControlReg,
        WRAM_START ... WRAM_END => MappedAddress::Wram(addr - WRAM_START),
        CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => MappedAddress::CartridgeRom(addr - CARTRIDGE_ROM_START),
        _ => panic!("Unrecognized addr: 0x{:08x}", addr)
    }
}