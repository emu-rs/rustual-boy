extern crate encoding;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use std::env;
use std::io::Read;
use std::fs::File;

const MIN_ROM_SIZE: usize = 512 * 1024;
const MAX_ROM_SIZE: usize = 16 * 1024 * 1024;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("Loading ROM file {}", rom_file_name);

    let mut rom_buf = Vec::new();
    let mut rom_file = File::open(&rom_file_name).unwrap();
    rom_file.read_to_end(&mut rom_buf).unwrap();
    let rom_size = rom_buf.len();

    if rom_size < MIN_ROM_SIZE || rom_size > MAX_ROM_SIZE || rom_size.count_ones() != 1 {
        panic!("Invalid ROM size");
    }

    print!("ROM size: ");
    if rom_size >= 1024 * 1024 {
        println!("{}MB", rom_size / 1024 / 1024);
    } else {
        println!("{}KB", rom_size / 1024);
    }

    let header_offset = rom_size - 544;

    println!("Header info:");

    let name_offset = header_offset;
    let name_bytes = &rom_buf[name_offset..name_offset + 0x14];
    let shift_jis_encoding = encoding_from_whatwg_label("shift-jis").unwrap();
    let name = shift_jis_encoding.decode(name_bytes, DecoderTrap::Strict).unwrap();

    let maker_code_offset = header_offset + 0x19;
    let maker_code_bytes = &rom_buf[maker_code_offset..maker_code_offset + 2];
    let mut maker_code_vec = Vec::new();
    maker_code_vec.extend_from_slice(maker_code_bytes);
    let maker_code = String::from_utf8(maker_code_vec).unwrap();

    let game_code_offset = header_offset + 0x1b;
    let game_code_bytes = &rom_buf[game_code_offset..game_code_offset + 2];
    let mut game_code_vec = Vec::new();
    game_code_vec.extend_from_slice(game_code_bytes);
    let game_code = String::from_utf8(game_code_vec).unwrap();

    let game_version_byte = rom_buf[header_offset + 0x1f];

    println!(" name: \"{}\"", name);
    println!(" maker code: \"{}\"", maker_code);
    println!(" game code: \"{}\"", game_code);
    println!(" game version: 1.{:#02}", game_version_byte);
}
