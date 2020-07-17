extern crate minifb;

extern crate cpal;

#[macro_use]
extern crate clap;

extern crate combine;

extern crate rustual_boy_core;

mod argparse;
mod audio_dest;
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

fn main() {
    let config = argparse::parse_args();

    logln!("Loading ROM file {}", config.rom_path);

    let rom = Rom::load(&config.rom_path).unwrap();

    log!("ROM size: ");
    if rom.size >= 1024 * 1024 {
        logln!("{}MB", rom.size / 1024 / 1024);
    } else {
        logln!("{}KB", rom.size / 1024);
    }

    logln!("Header info:");
    logln!(" name: \"{}\"", rom.name().unwrap());
    logln!(" maker code: \"{}\"", rom.maker_code().unwrap());
    logln!(" game code: \"{}\"", rom.game_code().unwrap());
    logln!(" game version: 1.{:#02}", rom.game_version_byte());

    logln!("Attempting to load SRAM file: {}", config.sram_path);
    let sram = match Sram::load(&config.sram_path) {
        Ok(sram) => {
            logln!(" SRAM loaded successfully");

            sram
        }
        Err(err) => {
            logln!(" Couldn't load SRAM file: {}", err);

            Sram::new()
        }
    };

    let audio_driver = CpalDriver::new(SAMPLE_RATE, 100).unwrap();

    let mut audio_dest = audio_driver.audio_dest();
    let time_source = audio_driver.time_source();

    let mut emulator = Emulator::new(rom, sram, time_source);
    emulator.run(&mut *audio_dest);

    if emulator.virtual_boy.interconnect.sram.size > 0 {
        logln!("SRAM used, saving to {}", config.sram_path);
        emulator.virtual_boy.interconnect.sram.save(config.sram_path).unwrap();
    }
}
