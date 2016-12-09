use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use std::io::Read;
use std::fs::File;
use std::path::Path;

pub const MIN_ROM_SIZE: usize = 512 * 1024;
pub const MAX_ROM_SIZE: usize = 16 * 1024 * 1024;

pub struct Rom {
    pub bytes: Box<[u8]>
}

impl Rom {
    pub fn load<P: AsRef<Path>>(rom_file_name: P) -> Rom {
        let mut rom_buf = Vec::new();
        let mut rom_file = File::open(&rom_file_name).unwrap();
        rom_file.read_to_end(&mut rom_buf).unwrap();
        let rom_size = rom_buf.len();

        if rom_size < MIN_ROM_SIZE || rom_size > MAX_ROM_SIZE || rom_size.count_ones() != 1 {
            panic!("Invalid ROM size");
        }

        Rom {
            bytes: rom_buf.into_boxed_slice()
        }
    }

    fn header_offset(&self) -> usize {
        self.size() - 544
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    pub fn name(&self) -> String {
        let header_offset = self.header_offset();
        let name_offset = header_offset;
        let name_bytes = &self.bytes[name_offset..name_offset + 0x14];
        let shift_jis_encoding = encoding_from_whatwg_label("shift-jis").unwrap();
        shift_jis_encoding.decode(name_bytes, DecoderTrap::Strict).unwrap()
    }

    pub fn maker_code(&self) -> String {
        let header_offset = self.header_offset();
        let maker_code_offset = header_offset + 0x19;
        let maker_code_bytes = &self.bytes[maker_code_offset..maker_code_offset + 2];
        let mut maker_code_vec = Vec::new();
        maker_code_vec.extend_from_slice(maker_code_bytes);
        String::from_utf8(maker_code_vec).unwrap()
    }

    pub fn game_code(&self) -> String {
        let header_offset = self.header_offset();
        let game_code_offset = header_offset + 0x1b;
        let game_code_bytes = &self.bytes[game_code_offset..game_code_offset + 2];
        let mut game_code_vec = Vec::new();
        game_code_vec.extend_from_slice(game_code_bytes);
        String::from_utf8(game_code_vec).unwrap()
    }

    pub fn game_version_byte(&self) -> u8 {
        let header_offset = self.header_offset();
        self.bytes[header_offset + 0x1f]
    }
}