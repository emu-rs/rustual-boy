extern crate encoding;

#[macro_use]
extern crate nom;

mod rom;
mod wram;
mod mem_map;
mod interconnect;
mod instruction;
mod nvc;

use nom::{IResult, eof, space, digit, hex_digit, alphanumeric};

use rom::*;
use interconnect::*;
use instruction::*;
use nvc::*;

use std::env;
use std::io::{stdin, stdout, Write};
use std::borrow::Cow;
use std::str::{self, FromStr};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone)]
pub enum Command {
    ShowRegs,
    Step,
    Continue,
    Goto(u32),
    ShowMem(Option<u32>),
    Disassemble(usize),
    Label,
    AddLabel(String, u32),
    RemoveLabel(String),
    Breakpoint,
    AddBreakpoint(u32),
    RemoveBreakpoint(u32),
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

struct VirtualBoy {
    pub interconnect: Interconnect,
    pub cpu: Nvc,
}

impl VirtualBoy {
    pub fn new(rom: Rom) -> VirtualBoy {
        VirtualBoy {
            interconnect: Interconnect::new(rom),
            cpu: Nvc::new()
        }
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.interconnect);
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

    let mut virtual_boy = VirtualBoy::new(rom);

    let mut labels = HashMap::new();
    let mut breakpoints = HashSet::new();
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
            Ok(Command::ShowRegs) => {
                println!("pc: 0x{:08x}", virtual_boy.cpu.reg_pc());
                println!("gpr:");
                for i in 0..32 {
                    println!(" r{}: 0x{:08x}", i, virtual_boy.cpu.reg_gpr(i));
                }
                println!("psw: 0x{:08x}", virtual_boy.cpu.reg_psw());
            }
            Ok(Command::Step) => {
                virtual_boy.step();
                cursor = virtual_boy.cpu.reg_pc();
                disassemble_instruction(&mut virtual_boy, &labels, &breakpoints, &mut cursor);
                cursor = virtual_boy.cpu.reg_pc();
            }
            Ok(Command::Continue) => {
                // TODO: Main loop shouldn't be here probably; should
                //  break out to something else
                loop {
                    virtual_boy.step();
                    cursor = virtual_boy.cpu.reg_pc();
                    if breakpoints.contains(&cursor) {
                        break;
                    }
                }
                disassemble_instruction(&mut virtual_boy, &labels, &breakpoints, &mut cursor);
                cursor = virtual_boy.cpu.reg_pc();
            }
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
                        let byte = virtual_boy.interconnect.read_byte(cursor);
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
                    disassemble_instruction(&mut virtual_boy, &labels, &breakpoints, &mut cursor);
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
            Ok(Command::RemoveLabel(ref name)) => {
                if let None = labels.remove(name) {
                    println!("Label .{} does not exist", name);
                }
            }
            Ok(Command::Breakpoint) => {
                for addr in breakpoints.iter() {
                    println!("* 0x{:08x}", addr);
                }
            }
            Ok(Command::AddBreakpoint(addr)) => {
                breakpoints.insert(addr);
            }
            Ok(Command::RemoveBreakpoint(addr)) => {
                if !breakpoints.remove(&addr) {
                    println!("Breakpoint at 0x{:08x} does not exist", addr);
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

fn print_labels(labels: &HashMap<String, u32>, addr: u32) {
    for (name, _) in labels.iter().filter(|x| *x.1 == addr) {
        println!(".{}:", name);
    }
}

fn disassemble_instruction(virtual_boy: &mut VirtualBoy, labels: &HashMap<String, u32>, breakpoints: &HashSet<u32>, cursor: &mut u32) {
    print_labels(labels, *cursor);

    if breakpoints.contains(cursor) {
        print!("* ");
    } else {
        print!("  ");
    }

    print!("0x{:08x}  ", cursor);

    let first_halfword = virtual_boy.interconnect.read_halfword(*cursor);
    let mut next_cursor = cursor.wrapping_add(2);
    print!("{:02x}{:02x}", first_halfword & 0xff, first_halfword >> 8);

    let opcode = Opcode::from_halfword(first_halfword);
    let instruction_format = opcode.instruction_format();

    let second_halfword = if instruction_format.has_second_halfword() {
        let second_halfword = virtual_boy.interconnect.read_halfword(next_cursor);
        print!("{:02x}{:02x}", second_halfword & 0xff, second_halfword >> 8);
        next_cursor = next_cursor.wrapping_add(2);
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
                println!("{} r{}, r{}", opcode, reg1, reg2);
            }
        }
        InstructionFormat::II => {
            let imm5 = (first_halfword & 0x1f) as usize;
            let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
            match opcode {
                Opcode::Cli | Opcode::Sei => println!("{}", opcode),
                Opcode::Ldsr => println!("{} r{}, {}", opcode, reg2, opcode.system_register(imm5)),
                _ => println!("{} {}, r{}", opcode, imm5, reg2)
            }
        }
        InstructionFormat::III => {
            let disp9 = first_halfword & 0x01ff;
            let disp = (disp9 as u32) | if disp9 & 0x0100 == 0 { 0x00000000 } else { 0xfffffe00 };
            let target = cursor.wrapping_add(disp);
            println!("{} {:#x} (0x{:08x})", opcode, disp9, target);
        }
        InstructionFormat::IV => {
            let disp26 = (((first_halfword as u32) & 0x03ff) << 16) | (second_halfword as u32);
            let disp = disp26 | if disp26 & 0x02000000 == 0 { 0x00000000 } else { 0xfc000000 };
            let target = cursor.wrapping_add(disp);
            println!("{} {} (0x{:08x})", opcode, disp26 as i32, target);
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

    *cursor = next_cursor;
}

named!(
    command<Command>,
    complete!(
        terminated!(
        alt_complete!(
            step |
            continue_ |
            goto |
            show_mem |
            disassemble |
            label |
            add_label |
            remove_label |
            breakpoint |
            add_breakpoint |
            remove_breakpoint |
            exit |
            show_regs |
            repeat),
        eof)));

named!(
    step<Command>,
    map!(
        alt_complete!(tag!("step") | tag!("s")),
    |_| Command::Step));

named!(
    continue_<Command>,
    map!(
        alt_complete!(tag!("continue") | tag!("c")),
    |_| Command::Continue));

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
    remove_label<Command>,
    chain!(
        alt_complete!(tag!("removelabel") | tag!("rl")) ~
        space ~
        name: label_name,
    || Command::RemoveLabel(name)));

named!(
    breakpoint<Command>,
    map!(
        alt_complete!(tag!("breakpoint") | tag!("b")),
    |_| Command::Breakpoint));

named!(
    add_breakpoint<Command>,
    chain!(
        alt_complete!(tag!("addbreakpoint") | tag!("ab")) ~
        space ~
        addr: hex_u32_parser,
    || Command::AddBreakpoint(addr)));

named!(
    remove_breakpoint<Command>,
    chain!(
        alt_complete!(tag!("removebreakpoint") | tag!("rb")) ~
        space ~
        addr: hex_u32_parser,
    || Command::RemoveBreakpoint(addr)));

named!(
    exit<Command>,
    map!(
        alt_complete!(tag!("exit") | tag!("quit") | tag!("e") | tag!("x") | tag!("q")),
        |_| Command::Exit));

named!(
    show_regs<Command>,
    map!(
        alt_complete!(tag!("showregs") | tag!("r")),
    |_| Command::ShowRegs));

named!(
    repeat<Command>,
    value!(Command::Repeat));
