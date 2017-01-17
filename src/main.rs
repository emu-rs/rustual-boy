extern crate encoding;

#[macro_use]
extern crate nom;

extern crate minifb;

extern crate rodio;

#[macro_use]
mod logging;
mod video_driver;
mod audio_driver;
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
mod rodio_driver;
mod command;
mod emulator;

use rom::*;
use sram::*;
use emulator::*;

use std::env;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    logln!("Loading ROM file {}", rom_file_name);

    let rom = Rom::load(&rom_file_name).unwrap();

    log!("ROM size: ");
    if rom.size() >= 1024 * 1024 {
        logln!("{}MB", rom.size() / 1024 / 1024);
    } else {
        logln!("{}KB", rom.size() / 1024);
    }

    logln!("Header info:");
    logln!(" name: \"{}\"", rom.name().unwrap());
    logln!(" maker code: \"{}\"", rom.maker_code().unwrap());
    logln!(" game code: \"{}\"", rom.game_code().unwrap());
    logln!(" game version: 1.{:#02}", rom.game_version_byte());

    let sram_file_name = rom_file_name.replace(".vb", ".srm");
    logln!("Attempting to load SRAM file: {}", sram_file_name);
    let sram = match Sram::load(&sram_file_name) {
        Ok(sram) => {
            logln!(" SRAM loaded successfully");

            sram
        }
        Err(err) => {
            logln!(" Couldn't load SRAM file: {}", err);

            Sram::new()
        }
    };

    let mut emulator = Emulator::new(rom, sram);
    emulator.run();

    if emulator.virtual_boy.interconnect.sram.size() > 0 {
        logln!("SRAM used, saving to {}", sram_file_name);
        emulator.virtual_boy.interconnect.sram.save(sram_file_name).unwrap();
    }
}
