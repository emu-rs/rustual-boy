extern crate encoding;

#[macro_use]
extern crate nom;

extern crate minifb;

mod video_driver;
mod rom;
mod wram;
mod sram;
mod vip;
mod vsu;
mod timer;
mod game_pad;
mod mem_map;
mod interconnect;
mod instruction;
mod nvc;
mod virtual_boy;
mod command;
mod emulator;

use rom::*;
use emulator::*;

use std::env;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("Loading ROM file {}", rom_file_name);

    let rom = Rom::load(rom_file_name).unwrap();

    print!("ROM size: ");
    if rom.size() >= 1024 * 1024 {
        println!("{}MB", rom.size() / 1024 / 1024);
    } else {
        println!("{}KB", rom.size() / 1024);
    }

    println!("Header info:");
    println!(" name: \"{}\"", rom.name().unwrap());
    println!(" maker code: \"{}\"", rom.maker_code().unwrap());
    println!(" game code: \"{}\"", rom.game_code().unwrap());
    println!(" game version: 1.{:#02}", rom.game_version_byte());

    let mut emulator = Emulator::new(rom);
    emulator.run();
}
