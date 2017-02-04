extern crate encoding;

#[macro_use]
extern crate nom;

extern crate minifb;

extern crate cpal;

extern crate futures;

extern crate rustual_boy_core;

#[macro_use]
mod logging;
mod command;
mod cpal_driver;
mod emulator;
mod system_time_source;
mod wave_file_buffer_sink;

use rustual_boy_core::rom::*;
use rustual_boy_core::sram::*;
use rustual_boy_core::vsu::*;
use cpal_driver::*;
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

    let audio_driver = CpalDriver::new(SAMPLE_RATE as _, 100).unwrap();

    let audio_buffer_sink = audio_driver.sink();
    let time_source = audio_driver.time_source();

    let mut emulator = Emulator::new(rom, sram, audio_buffer_sink, time_source);
    emulator.run();

    if emulator.virtual_boy.interconnect.sram.size() > 0 {
        logln!("SRAM used, saving to {}", sram_file_name);
        emulator.virtual_boy.interconnect.sram.save(sram_file_name).unwrap();
    }
}
