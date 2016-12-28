use minifb::{WindowOptions, Window, Key, KeyRepeat};

use video_driver::*;
use rom::*;
use instruction::*;
use game_pad::*;
use virtual_boy::*;
use command::*;

use std::time;
use std::thread::{self, JoinHandle};
use std::io::{stdin, stdout, Write};
use std::collections::{HashSet, HashMap};
use std::sync::mpsc::{channel, Receiver};

const NS_TO_MS: u32 = 1000000;

struct SimpleVideoDriver {
    next: Option<(Box<[u8]>, Box<[u8]>)>,
}

impl VideoDriver for SimpleVideoDriver {
    fn output_frame(&mut self, frame: (Box<[u8]>, Box<[u8]>)) {
        self.next = Some(frame);
    }
}

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
}

pub struct Emulator {
    window: Window,

    virtual_boy: VirtualBoy,
    mode: Mode,

    breakpoints: HashSet<u32>,

    labels: HashMap<String, u32>,
    cursor: u32,
    last_command: Option<Command>,

    stdin_receiver: Receiver<String>,
    _stdin_thread: JoinHandle<()>,

    frame_cycles: usize,
}

impl Emulator {
    pub fn new(rom: Rom) -> Emulator {
        let (stdin_sender, stdin_receiver) = channel();
        let stdin_thread = thread::spawn(move || {
            loop {
                stdin_sender.send(read_stdin()).unwrap();
            }
        });

        Emulator {
            window: Window::new("vb-rs", 384, 224, WindowOptions::default()).unwrap(),

            virtual_boy: VirtualBoy::new(rom),
            mode: Mode::Running,

            breakpoints: HashSet::new(),

            labels: HashMap::new(),
            cursor: 0,
            last_command: None,

            stdin_receiver: stdin_receiver,
            _stdin_thread: stdin_thread,

            frame_cycles: 0,
        }
    }

    pub fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let frame_start_time = time::Instant::now();

            let mut video_driver = SimpleVideoDriver {
                next: None
            };

            match self.mode {
                Mode::Running => {
                    self.run_frame(&mut video_driver);
                }
                Mode::Debugging => {
                    if self.run_debugger_commands(&mut video_driver) {
                        break;
                    }
                }
            }

            match video_driver.next {
                Some((left_buffer, right_buffer)) => {
                    let mut buffer = vec![0; 384 * 224];
                    for i in 0..384 * 224 {
                        let left = left_buffer[i] as u32;
                        let right = right_buffer[i] as u32;
                        buffer[i] = (left << 16) | (right << 8) | right;
                    }
                    self.window.update_with_buffer(&buffer);
                }
                _ => self.window.update()
            }

            let frame_duration = frame_start_time.elapsed();
            if self.mode == Mode::Running {
                println!("Frame duration: {}ms", frame_duration.subsec_nanos() / NS_TO_MS);
            }
            let target_frame_duration = time::Duration::from_millis(20);
            if frame_duration < target_frame_duration {
                let sleep_ms = (target_frame_duration - frame_duration).subsec_nanos() / NS_TO_MS;
                thread::sleep(time::Duration::from_millis(sleep_ms as _));
            }
        }
    }

    fn run_frame(&mut self, video_driver: &mut VideoDriver) {
        let mut start_debugger = false;

        if self.window.is_key_pressed(Key::F12, KeyRepeat::No) {
            start_debugger = true;
        }

        self.read_input_keys();

        while self.frame_cycles < CPU_CYCLES_PER_FRAME {
            self.frame_cycles += self.virtual_boy.step(video_driver);
            if self.breakpoints.contains(&self.virtual_boy.cpu.reg_pc()) {
                start_debugger = true;
                break;
            }
        }
        if self.frame_cycles >= CPU_CYCLES_PER_FRAME {
            self.frame_cycles -= CPU_CYCLES_PER_FRAME;
        }

        if start_debugger {
            self.start_debugger();
        }
    }

    fn read_input_keys(&mut self) {
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::A, self.window.is_key_down(Key::F));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::B, self.window.is_key_down(Key::H));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::Start, self.window.is_key_down(Key::Enter));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::Select, self.window.is_key_down(Key::Space));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::L, self.window.is_key_down(Key::E));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::R, self.window.is_key_down(Key::U));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::LeftDPadUp, self.window.is_key_down(Key::W));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::LeftDPadDown, self.window.is_key_down(Key::S));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::LeftDPadLeft, self.window.is_key_down(Key::A));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::LeftDPadRight, self.window.is_key_down(Key::D));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::RightDPadUp, self.window.is_key_down(Key::I));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::RightDPadDown, self.window.is_key_down(Key::K));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::RightDPadLeft, self.window.is_key_down(Key::J));
        self.virtual_boy.interconnect.game_pad.set_button_pressed(Button::RightDPadRight, self.window.is_key_down(Key::L));
    }

    fn start_debugger(&mut self) {
        self.mode = Mode::Debugging;

        self.cursor = self.virtual_boy.cpu.reg_pc();
        self.disassemble_instruction();

        self.print_cursor();
    }

    fn run_debugger_commands(&mut self, video_driver: &mut VideoDriver) -> bool {
        while let Ok(command_string) = self.stdin_receiver.try_recv() {
            let command = match (command_string.parse(), self.last_command.clone()) {
                (Ok(Command::Repeat), Some(c)) => Ok(c),
                (Ok(Command::Repeat), None) => Err("No last command".into()),
                (Ok(c), _) => Ok(c),
                (Err(e), _) => Err(e),
            };

            match command {
                Ok(Command::ShowRegs) => {
                    println!("pc: 0x{:08x}", self.virtual_boy.cpu.reg_pc());
                    println!("gpr:");
                    for i in 0..32 {
                        println!(" r{}: 0x{:08x}", i, self.virtual_boy.cpu.reg_gpr(i));
                    }
                    println!("psw: 0x{:08x}", self.virtual_boy.cpu.reg_psw());
                    println!("eipc: 0x{:08x}", self.virtual_boy.cpu.reg_eipc());
                    println!("eipsw: 0x{:08x}", self.virtual_boy.cpu.reg_eipsw());
                    println!("ecr: 0x{:08x}", self.virtual_boy.cpu.reg_ecr());
                }
                Ok(Command::Step) => {
                    self.frame_cycles += self.virtual_boy.step(video_driver);
                    if self.frame_cycles >= CPU_CYCLES_PER_FRAME {
                        self.frame_cycles -= CPU_CYCLES_PER_FRAME;
                    }
                    self.cursor = self.virtual_boy.cpu.reg_pc();
                    self.disassemble_instruction();
                }
                Ok(Command::Continue) => {
                    self.mode = Mode::Running;
                }
                Ok(Command::Goto(addr)) => {
                    self.cursor = addr;
                }
                Ok(Command::ShowMem(addr)) => {
                    if let Some(addr) = addr {
                        self.cursor = addr;
                    }

                    self.print_labels_at_cursor();

                    const NUM_ROWS: usize = 16;
                    const NUM_COLS: usize = 16;
                    for _ in 0..NUM_ROWS {
                        print!("0x{:08x}  ", self.cursor);
                        for x in 0..NUM_COLS {
                            let byte = self.virtual_boy.interconnect.read_byte(self.cursor);
                            self.cursor = self.cursor.wrapping_add(1);
                            print!("{:02x}", byte);
                            if x < NUM_COLS - 1 {
                                print!(" ");
                            }
                        }
                        println!();
                    }
                }
                Ok(Command::Disassemble(count)) => {
                    for _ in 0..count {
                        self.cursor = self.disassemble_instruction();
                    }
                }
                Ok(Command::Label) => {
                    for (name, addr) in self.labels.iter() {
                        println!(".{}: 0x{:08x}", name, addr);
                    }
                }
                Ok(Command::AddLabel(ref name, addr)) => {
                    self.labels.insert(name.clone(), addr);
                }
                Ok(Command::RemoveLabel(ref name)) => {
                    if let None = self.labels.remove(name) {
                        println!("Label .{} does not exist", name);
                    }
                }
                Ok(Command::Breakpoint) => {
                    for addr in self.breakpoints.iter() {
                        println!("* 0x{:08x}", addr);
                    }
                }
                Ok(Command::AddBreakpoint(addr)) => {
                    self.breakpoints.insert(addr);
                }
                Ok(Command::RemoveBreakpoint(addr)) => {
                    if !self.breakpoints.remove(&addr) {
                        println!("Breakpoint at 0x{:08x} does not exist", addr);
                    }
                }
                Ok(Command::Exit) => {
                    return true;
                }
                Ok(Command::Repeat) => unreachable!(),
                Err(ref e) => println!("{}", e),
            }

            if let Ok(c) = command {
                self.last_command = Some(c);
            }

            self.print_cursor();
        }

        return false;
    }

    fn print_cursor(&self) {
        print!("(vb-rs 0x{:08x}) > ", self.cursor);
        stdout().flush().unwrap();
    }

    fn disassemble_instruction(&mut self) -> u32 {
        self.print_labels_at_cursor();

        if self.breakpoints.contains(&self.cursor) {
            print!("* ");
        } else {
            print!("  ");
        }

        print!("0x{:08x}  ", self.cursor);

        let first_halfword = self.virtual_boy.interconnect.read_halfword(self.cursor);
        let mut next_cursor = self.cursor.wrapping_add(2);
        print!("{:02x}{:02x}", first_halfword & 0xff, first_halfword >> 8);

        let opcode = Opcode::from_halfword(first_halfword);
        let instruction_format = opcode.instruction_format();

        let second_halfword = if instruction_format.has_second_halfword() {
            let second_halfword = self.virtual_boy.interconnect.read_halfword(next_cursor);
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
                    Opcode::Cli | Opcode::Reti | Opcode::Sei => println!("{}", opcode),
                    Opcode::Ldsr | Opcode::Stsr => println!("{} r{}, {}", opcode, reg2, opcode.system_register(imm5)),
                    _ => println!("{} {}, r{}", opcode, imm5, reg2)
                }
            }
            InstructionFormat::III => {
                let disp9 = first_halfword & 0x01ff;
                let disp = (disp9 as u32) | if disp9 & 0x0100 == 0 { 0x00000000 } else { 0xfffffe00 };
                let target = self.cursor.wrapping_add(disp);
                println!("{} {:#x} (0x{:08x})", opcode, disp9, target);
            }
            InstructionFormat::IV => {
                let disp26 = (((first_halfword as u32) & 0x03ff) << 16) | (second_halfword as u32);
                let disp = disp26 | if disp26 & 0x02000000 == 0 { 0x00000000 } else { 0xfc000000 };
                let target = self.cursor.wrapping_add(disp);
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

        next_cursor
    }

    fn print_labels_at_cursor(&mut self) {
        for (name, _) in self.labels.iter().filter(|x| *x.1 == self.cursor) {
            println!(".{}:", name);
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
