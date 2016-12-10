extern crate encoding;

#[macro_use]
extern crate nom;

mod rom;
mod interconnect;

use nom::{IResult, eof, space, digit, hex_digit, alphanumeric};

use rom::*;
use interconnect::*;

use std::env;
use std::io::{stdin, stdout, Write};
use std::borrow::Cow;
use std::str::{self, FromStr};
use std::fmt;
use std::collections::HashMap;

#[derive(PartialEq, Eq)]
enum Opcode {
    Jmp,
    Movea,
    Movhi,
    Outw,
}

impl Opcode {
    fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b000110 => Opcode::Jmp,
            0b101000 => Opcode::Movea,
            0b101111 => Opcode::Movhi,
            0b111111 => Opcode::Outw,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits),
        }
    }

    fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }
}


impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::Jmp => "jmp",
            &Opcode::Movea => "movea",
            &Opcode::Movhi => "movhi",
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

enum InstructionFormat {
    I,
    V,
    VI,
}

impl InstructionFormat {
    fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Goto(u32),
    ShowMem(Option<u32>),
    Disassemble(usize),
    Label,
    AddLabel(String, u32),
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

    let mut labels = HashMap::new();
    let mut cursor = 0xfffffff0;
    let mut last_command = None;

    loop {
        print!("(vb-rs 0x{:08x}) > ", cursor);
        stdout().flush().unwrap();

        let command = match (read_stdin().parse(), last_command.clone()) {
            (Ok(Command::Repeat), Some(c)) => Ok(c),
            (Ok(Command::Repeat), None) => Err("No last command".into()),
            (Ok(c), _) => Ok(c),
            (Err(e), _) => Err(e),
        };

        match command {
            Ok(Command::Goto(addr)) => {
                cursor = addr;
            }
            Ok(Command::ShowMem(addr)) => {
                if let Some(addr) = addr {
                    cursor = addr;
                }

                print_labels(&labels, cursor);

                const NUM_ROWS: usize = 16;
                const NUM_COLS: usize = 16;
                for _ in 0..NUM_ROWS {
                    print!("0x{:08x}  ", cursor);
                    for x in 0..NUM_COLS {
                        let byte = interconnect.read_byte(cursor);
                        cursor = cursor.wrapping_add(1);
                        print!("{:02x}", byte);
                        if x < NUM_COLS - 1 {
                            print!(" ");
                        }
                    }
                    println!("");
                }
            }
            Ok(Command::Disassemble(count)) => {
                for _ in 0..count {
                    print_labels(&labels, cursor);

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
                        InstructionFormat::I => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
                            if opcode == Opcode::Jmp {
                                println!("jmp [r{}]", reg1);
                            } else {
                                println!("{}, r{}, r{}", opcode, reg1, reg2);
                            }
                        }
                        InstructionFormat::V => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            let imm16 = second_halfword;

                            println!("{} {:#x}, r{}, r{}", opcode, imm16, reg1, reg2);
                        }
                        InstructionFormat::VI => {
                            let reg1 = (first_halfword & 0x1f) as usize;
                            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                            let disp16 = second_halfword as i16;

                            println!("{} {}[r{}], r{}", opcode, disp16, reg1, reg2);
                        }
                    }
                }
            }
            Ok(Command::Label) => {
                for (name, addr) in labels.iter() {
                    println!(".{}: 0x{:08x}", name, addr);
                }
            }
            Ok(Command::AddLabel(ref name, addr)) => {
                labels.insert(name.clone(), addr);
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

fn print_labels(labels: &HashMap<String, u32>, addr: u32) {
    for (name, _) in labels.iter().filter(|x| *x.1 == addr) {
        println!(".{}:", name);
    }
}

named!(
    command<Command>,
    complete!(
        terminated!(
        alt_complete!(
            goto |
            show_mem |
            disassemble |
            label |
            add_label |
            exit |
            repeat),
        eof)));

named!(
    goto<Command>,
    chain!(
        alt_complete!(tag!("goto") | tag!("g")) ~
        addr: preceded!(space, hex_u32_parser),
    || Command::Goto(addr)));

named!(
    show_mem<Command>,
    chain!(
        alt_complete!(tag!("showmem") | tag!("mem") | tag!("m")) ~
        addr: opt!(preceded!(space, hex_u32_parser)),
    || Command::ShowMem(addr)));

named!(
    hex_u32_parser<u32>,
    map_res!(
        map_res!(
            preceded!(opt!(alt_complete!(tag!("0x") | tag!("$"))), hex_digit),
            str::from_utf8),
    |s| u32::from_str_radix(s, 16)));

named!(
    disassemble<Command>,
    chain!(
        alt_complete!(tag!("disassemble") | tag!("d")) ~
        count: opt!(preceded!(space, usize_parser)),
    || Command::Disassemble(count.unwrap_or(4))));

named!(
    usize_parser<usize>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8),
    FromStr::from_str));

named!(
    label<Command>,
    map!(
        alt_complete!(tag!("label") | tag!("l")),
    |_| Command::Label));

named!(
    add_label<Command>,
    chain!(
        alt_complete!(tag!("addlabel") | tag!("al")) ~
        space ~
        name: label_name ~
        space ~
        addr: hex_u32_parser,
    || Command::AddLabel(name, addr)));

named!(
    label_name<String>,
    preceded!(
        char!('.'),
        map_res!(
            map_res!(
                alphanumeric,
                str::from_utf8),
        FromStr::from_str)));

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("quit") | tag!("e") | tag!("x") | tag!("q")),
        |_| Command::Exit));

named!(
    repeat<Command>,
    value!(Command::Repeat));
