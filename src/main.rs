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
use sram::*;
use emulator::*;

use std::env;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    println!("Loading ROM file {}", rom_file_name);

    let rom = Rom::load(&rom_file_name).unwrap();

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

    let sram_file_name = rom_file_name.replace(".vb", ".srm");
    println!("Attempting to load SRAM file: {}", sram_file_name);
    let sram = match Sram::load(&sram_file_name) {
        Ok(sram) => {
            println!(" SRAM loaded successfully");

            sram
        }
        Err(err) => {
            println!(" Couldn't load SRAM file: {}", err);

            Sram::new()
        }
    };

    let mut emulator = Emulator::new(rom, sram);
    emulator.run();

    if emulator.virtual_boy.interconnect.sram.size() > 0 {
        println!("SRAM used, saving to {}", sram_file_name);
        emulator.virtual_boy.interconnect.sram.save(sram_file_name).unwrap();
    }
}
