extern crate encoding;

#[macro_use]
extern crate nom;

mod rom;
mod interconnect;

use nom::IResult;

use rom::*;
use interconnect::*;

use std::env;
use std::io::{stdin, stdout, Write};
use std::borrow::Cow;
use std::str::{self, FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Exit,
    Repeat,
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match command(s.as_bytes()) {
            IResult::Done(_, c) => Ok(c),
            err => Err(format!("Unable to parse command: {:?}", err).into()),
        }
    }
}

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

    let interconnect = Interconnect::new(rom);

    let mut cursor = 0xfffffff0;
    let mut last_command = None;

    loop {
        print!("(vb {:#08x}) ", cursor);
        stdout().flush().unwrap();

        let command = match (read_stdin().parse(), last_command) {
            (Ok(Command::Repeat), Some(c)) => Ok(c),
            (Ok(Command::Repeat), None) => Err("No last command".into()),
            (Ok(c), _) => Ok(c),
            (Err(e), _) => Err(e),
        };

        match command {
            Ok(Command::Exit) => break,
            Ok(Command::Repeat) => unreachable!(),
            Err(ref e) => println!("{}", e),
        }

        last_command = command.ok();
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}

named!(
    command<Command>,
    terminated!(
        alt_complete!(
            exit |
            repeat),
        eof!()));

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("quit") | tag!("e") | tag!("q")),
        |_| Command::Exit));

named!(
    repeat<Command>,
    value!(Command::Repeat));
