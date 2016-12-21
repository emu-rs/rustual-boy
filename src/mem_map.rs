pub const VIP_START: u32 = 0x00000000;
pub const VIP_LENGTH: u32 = 0x01000000;
pub const VIP_END: u32 = VIP_START + VIP_LENGTH - 1;

pub const VSU_START: u32 = 0x01000000;
pub const VSU_LENGTH: u32 = 0x01000000;
pub const VSU_END: u32 = VSU_START + VSU_LENGTH - 1;

pub const LINK_CONTROL_REG: u32 = 0x02000000;
pub const AUX_LINK_REG: u32 = 0x02000004;
pub const LINK_TRANSMIT_DATA_REG: u32 = 0x02000008;
pub const LINK_RECEIVE_DATA_REG: u32 = 0x0200000c;
pub const GAME_PAD_INPUT_LOW_REG: u32 = 0x02000010;
pub const GAME_PAD_INPUT_HIGH_REG: u32 = 0x02000014;
pub const TIMER_COUNTER_RELOAD_LOW_REG: u32 = 0x02000018;
pub const TIMER_COUNTER_RELOAD_HIGH_REG: u32 = 0x0200001c;
pub const TIMER_CONTROL_REG: u32 = 0x02000020;
pub const WAIT_CONTROL_REG: u32 = 0x02000024;
pub const GAME_PAD_INPUT_CONTROL_REG: u32 = 0x02000028;

pub const CARTRIDGE_EXPANSION_START: u32 = 0x04000000;
pub const CARTRIDGE_EXPANSION_LENGTH: u32 = 0x01000000;
pub const CARTRIDGE_EXPANSION_END: u32 = CARTRIDGE_EXPANSION_START + CARTRIDGE_EXPANSION_LENGTH - 1;

pub const WRAM_START: u32 = 0x05000000;
pub const WRAM_LENGTH: u32 = 0x01000000;
pub const WRAM_END: u32 = WRAM_START + WRAM_LENGTH - 1;

pub const CARTRIDGE_ROM_START: u32 = 0x07000000;
pub const CARTRIDGE_ROM_LENGTH: u32 = 0x01000000;
pub const CARTRIDGE_ROM_END: u32 = CARTRIDGE_ROM_START + CARTRIDGE_ROM_LENGTH - 1;

pub enum MappedAddress {
    Vip(u32),

    Vsu(u32),

    LinkControlReg,
    AuxLinkReg,
    LinkTransmitDataReg,
    LinkReceiveDataReg,
    GamePadInputLowReg,
    GamePadInputHighReg,
    TimerCounterReloadLowReg,
    TimerCounterReloadHighReg,
    TimerControlReg,
    WaitControlReg,
    GamePadInputControlReg,

    CartridgeExpansion(u32),

    Wram(u32),

    CartridgeRom(u32)
}

pub fn map_address(addr: u32) -> MappedAddress {
    let addr = addr & 0x07ffffff;
    match addr {
        VIP_START ... VIP_END => MappedAddress::Vip(addr),

        VSU_START ... VSU_END => MappedAddress::Vsu(addr),

        LINK_CONTROL_REG => MappedAddress::LinkControlReg,
        AUX_LINK_REG => MappedAddress::AuxLinkReg,
        LINK_TRANSMIT_DATA_REG => MappedAddress::LinkTransmitDataReg,
        LINK_RECEIVE_DATA_REG => MappedAddress::LinkReceiveDataReg,
        GAME_PAD_INPUT_LOW_REG => MappedAddress::GamePadInputLowReg,
        GAME_PAD_INPUT_HIGH_REG => MappedAddress::GamePadInputHighReg,
        TIMER_COUNTER_RELOAD_LOW_REG => MappedAddress::TimerCounterReloadLowReg,
        TIMER_COUNTER_RELOAD_HIGH_REG => MappedAddress::TimerCounterReloadHighReg,
        TIMER_CONTROL_REG => MappedAddress::TimerControlReg,
        WAIT_CONTROL_REG => MappedAddress::WaitControlReg,
        GAME_PAD_INPUT_CONTROL_REG => MappedAddress::GamePadInputControlReg,

        CARTRIDGE_EXPANSION_START ... CARTRIDGE_EXPANSION_END => MappedAddress::CartridgeExpansion(addr - CARTRIDGE_EXPANSION_START),

        WRAM_START ... WRAM_END => MappedAddress::Wram(addr - WRAM_START),

        CARTRIDGE_ROM_START ... CARTRIDGE_ROM_END => MappedAddress::CartridgeRom(addr - CARTRIDGE_ROM_START),

        _ => panic!("Unrecognized addr: 0x{:08x}", addr)
    }
}