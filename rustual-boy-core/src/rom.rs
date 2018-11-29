use encoding::DecoderTrap;
use encoding::all::WINDOWS_31J;
use encoding::types::EncodingRef;

use std::io;
use std::fs::read;
use std::path::Path;
use std::borrow::Cow;
use std::string::FromUtf8Error;

pub const MIN_ROM_SIZE: usize = 1024;
pub const MAX_ROM_SIZE: usize = 16 * 1024 * 1024;

// TODO: Proper display impl
#[derive(Debug)]
pub enum RomError {
    InvalidSize,
    Io(io::Error),
}

#[derive(Clone)]
pub struct Rom {
    pub bytes: Box<[u8]>,
    pub size: usize,
}

impl Rom {
    pub fn load<P: AsRef<Path>>(file_name: P) -> Result<Rom, RomError> {
        let file = read(file_name).map_err(|e| RomError::Io(e))?;

        Rom::from_slice(&file)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Rom, RomError> {
        let size = slice.len();
        if size < MIN_ROM_SIZE || size > MAX_ROM_SIZE || !size.is_power_of_two() {
            return Err(RomError::InvalidSize);
        }

        let mut bytes = vec![0xff; MAX_ROM_SIZE].into_boxed_slice();
        bytes[..size].copy_from_slice(&slice);

        Ok(Rom {
            bytes,
            size,
        })
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let addr = self.mask_addr(addr);
        self.bytes[addr as usize]
    }

    pub fn read_halfword(&self, addr: u32) -> u16 {
        let addr = addr & 0xfffffffe;
        let addr = self.mask_addr(addr);
        (self.bytes[addr as usize] as u16) |
        ((self.bytes[(addr + 1) as usize] as u16) << 8)
    }

    fn mask_addr(&self, addr: u32) -> u32 {
        let mask = (self.size - 1) as u32;
        addr & mask
    }

    pub fn name(&self) -> Result<String, Cow<'static, str>> {
        let header_offset = self.header_offset();
        let name_offset = header_offset;
        let name_bytes = &self.bytes[name_offset..name_offset + 0x14];
        // Windows-31J is a superset of Shift JIS, which technically makes this
        //  code a bit too permissive, but saves us from writing our own decoder
        //  just to read ROM names. Even if we did try to write our own,
        //  I haven't seen any documentation that mentions which specific Shift JIS
        //  version we should use in the first place, especially since the more
        //  widely-used ones were standardized in 1997, after the Virtual Boy was
        //  in production.
        let shift_jis_encoding = WINDOWS_31J as EncodingRef;
        shift_jis_encoding.decode(name_bytes, DecoderTrap::Strict)
    }

    pub fn maker_code(&self) -> Result<String, FromUtf8Error> {
        let header_offset = self.header_offset();
        let maker_code_offset = header_offset + 0x19;
        let maker_code_bytes = &self.bytes[maker_code_offset..maker_code_offset + 2];
        let mut maker_code_vec = Vec::new();
        maker_code_vec.extend_from_slice(maker_code_bytes);
        String::from_utf8(maker_code_vec)
    }

    pub fn game_code(&self) -> Result<String, FromUtf8Error> {
        let header_offset = self.header_offset();
        let game_code_offset = header_offset + 0x1b;
        let game_code_bytes = &self.bytes[game_code_offset..game_code_offset + 2];
        let mut game_code_vec = Vec::new();
        game_code_vec.extend_from_slice(game_code_bytes);
        String::from_utf8(game_code_vec)
    }

    pub fn game_version_byte(&self) -> u8 {
        let header_offset = self.header_offset();
        self.bytes[header_offset + 0x1f]
    }

    fn header_offset(&self) -> usize {
        self.bytes.len() - 544
    }
}
