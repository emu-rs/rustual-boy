extern crate encoding;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use std::env;
use std::io::Read;
use std::fs::File;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    let mut rom_buf = Vec::new();
    let mut rom_file = File::open(&rom_file_name).unwrap();
    rom_file.read_to_end(&mut rom_buf).unwrap();

    let header_offset = rom_buf.len() - 544;
    let name_bytes = &rom_buf[header_offset..header_offset + 20];

    let encoding = encoding_from_whatwg_label("shift-jis").unwrap();
    let name = encoding.decode(name_bytes, DecoderTrap::Strict).unwrap();

    println!("\"{}\" ({})", name, rom_file_name);
}
