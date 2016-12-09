extern crate encoding;

mod rom;

use std::env;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("Loading ROM file {}", rom_file_name);

    let rom = rom::Rom::load(rom_file_name);

    print!("ROM size: ");
    if rom.size() >= 1024 * 1024 {
        println!("{}MB", rom.size() / 1024 / 1024);
    } else {
        println!("{}KB", rom.size() / 1024);
    }

    println!("Header info:");

    println!(" name: \"{}\"", rom.name());
    println!(" maker code: \"{}\"", rom.maker_code());
    println!(" game code: \"{}\"", rom.game_code());
    println!(" game version: 1.{:#02}", rom.game_version_byte());
}
