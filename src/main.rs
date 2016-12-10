extern crate encoding;

#[macro_use]
extern crate nom;

mod rom;
mod interconnect;

use nom::{IResult, eof, space, digit};

use rom::*;
use interconnect::*;

use std::env;
use std::io::{stdin, stdout, Write};
use std::borrow::Cow;
use std::str::{self, FromStr};

#[derive(Debug)]
enum Opcode {
    MovHi,
}

impl Opcode {
    fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b101111 => Opcode::MovHi,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits),
        }
    }

    fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::MovHi => InstructionFormat::V,
        }
    }
}

#[derive(Debug)]
enum InstructionFormat {
    V,
}

impl InstructionFormat {
    fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::V => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Disassemble(usize),
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
        print!("(vb-rs {:#08x}) > ", cursor);
        stdout().flush().unwrap();

        let command = match (read_stdin().parse(), last_command) {
            (Ok(Command::Repeat), Some(c)) => Ok(c),
            (Ok(Command::Repeat), None) => Err("No last command".into()),
            (Ok(c), _) => Ok(c),
            (Err(e), _) => Err(e),
        };

        match command {
            Ok(Command::Disassemble(count)) => {
                for _ in 0..count {
                    print!("0x{:08x}  ", cursor);

                    let first_halfword = interconnect.read_halfword(cursor);
                    cursor = cursor.wrapping_add(2);
                    print!("{:02x}{:02x}", first_halfword & 0xff, first_halfword >> 8);

                    let opcode = Opcode::from_halfword(first_halfword);
                    let instruction_format = opcode.instruction_format();

                    let second_halfword = if instruction_format.has_second_halfword() {
                        let second_halfword = interconnect.read_halfword(cursor);
                        print!("{:02x}{:02x}", second_halfword & 0xff, second_halfword >> 8);
                        cursor = cursor.wrapping_add(2);
                        second_halfword
                    } else {
                        print!("    ");
                        0
                    };

                    print!("    ");

                    match instruction_format {
                        InstructionFormat::V => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            let imm16 = second_halfword;

                            match opcode {
                                Opcode::MovHi => {
                                    println!("movhi {:#x}, r{}, r{}", imm16, reg1, reg2)
                                }
                            }
                        }
                    }
                }
            }
            Ok(Command::Exit) => break,
            Ok(Command::Repeat) => unreachable!(),
            Err(ref e) => println!("{}", e),
        }

        if let Ok(c) = command {
            last_command = Some(c);
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}

named!(
    command<Command>,
    complete!(
        terminated!(
        alt_complete!(
            disassemble |
            exit |
            repeat),
        eof)));

named!(
    disassemble<Command>,
    chain!(
        alt_complete!(tag!("disassemble") | tag!("d")) ~
            count: opt!(preceded!(space, usize_parser)),
    || Command::Disassemble(count.unwrap_or(1))));

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("quit") | tag!("e") | tag!("q")),
        |_| Command::Exit));

named!(
    repeat<Command>,
    value!(Command::Repeat));

named!(
    usize_parser<usize>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8),
    FromStr::from_str));
