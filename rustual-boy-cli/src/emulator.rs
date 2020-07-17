use minifb::{WindowOptions, Window, Key, KeyRepeat, Scale, ScaleMode};

use audio_dest::*;
use command::*;

use rustual_boy_core::game_pad::Button;
use rustual_boy_core::instruction::*;
use rustual_boy_core::rom::Rom;
use rustual_boy_core::sinks::*;
use rustual_boy_core::sram::Sram;
use rustual_boy_core::time_source::TimeSource;
use rustual_boy_core::vip::*;
use rustual_boy_core::virtual_boy::VirtualBoy;
use rustual_boy_core::vsu::*;

use std::collections::{HashSet, HashMap};
use std::io::{stdin, stdout, Write};
use std::mem;
use std::thread::{self, JoinHandle};
use std::time;
use std::sync::mpsc::{channel, Receiver};

const CPU_CYCLE_TIME_NS: u64 = 50;

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
}

pub struct Emulator {
    window: Window,

    pub virtual_boy: VirtualBoy,
    mode: Mode,

    breakpoints: HashSet<u32>,

    labels: HashMap<String, u32>,
    cursor: u32,
    last_command: Option<Command>,

    stdin_receiver: Receiver<String>,
    _stdin_thread: JoinHandle<()>,

    time_source: Box<dyn TimeSource>,
    time_source_start_time_ns: u64,

    emulated_cycles: u64,
}

impl Emulator {
    pub fn new(rom: Rom, sram: Sram, time_source: Box<dyn TimeSource>) -> Emulator {
        let (stdin_sender, stdin_receiver) = channel();
        let stdin_thread = thread::spawn(move || {
            loop {
                stdin_sender.send(read_stdin()).unwrap();
            }
        });

        Emulator {
            window: Window::new("Rustual Boy", 384, 224, WindowOptions {
                borderless: false,
                title: true,
                resize: false,
                scale: Scale::X2,
                scale_mode: ScaleMode::AspectRatioStretch,
                topmost: false,
                transparency: false
            }).unwrap(),

            virtual_boy: VirtualBoy::new(rom, sram),
            mode: Mode::Running,

            breakpoints: HashSet::new(),

            labels: HashMap::new(),
            cursor: 0,
            last_command: None,

            stdin_receiver: stdin_receiver,
            _stdin_thread: stdin_thread,

            time_source: time_source,
            time_source_start_time_ns: 0,

            emulated_cycles: 0,
        }
    }

    pub fn run(&mut self, audio_dest: &mut dyn AudioDest) {
        let mut video_output_frame_buffer = vec![0; DISPLAY_PIXELS as usize];
        let mut audio_frame_buffer = vec![(0, 0); (SAMPLE_RATE as usize) / 50 * 2]; // double space needed for 1 frame for lots of skid room

        self.time_source_start_time_ns = self.time_source.time_ns();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let mut audio_sink = AudioSink {
                buffer: &mut audio_frame_buffer,
                buffer_pos: 0,
            };

            let is_video_output_frame_buffer_populated = {
                let pixel_buffer = PixelBuffer::Xrgb8888(&mut video_output_frame_buffer, (DISPLAY_RESOLUTION_X as usize) * mem::size_of::<u32>());
                let mut video_output_sink = VideoSink {
                    buffer: pixel_buffer,
                    format: StereoVideoFormat::AnaglyphRedElectricCyan,
                    gamma_correction: GammaCorrection::TwoPointTwo,
                    is_populated: false,
                };

                let target_emulated_time_ns = self.time_source.time_ns() - self.time_source_start_time_ns;
                let target_emulated_cycles = target_emulated_time_ns / CPU_CYCLE_TIME_NS;

                match self.mode {
                    Mode::Running => {
                        let mut start_debugger = false;

                        while self.emulated_cycles < target_emulated_cycles && !start_debugger {
                            let (_, trigger_watchpoint) = self.step(&mut video_output_sink, &mut audio_sink);
                            if trigger_watchpoint || (self.breakpoints.len() != 0 && self.breakpoints.contains(&self.virtual_boy.cpu.reg_pc())) {
                                start_debugger = true;
                            }
                        }

                        if start_debugger {
                            self.start_debugger();
                        }
                    }
                    Mode::Debugging => {
                        if self.run_debugger_commands(&mut video_output_sink, &mut audio_sink) {
                            break;
                        }

                        self.window.update();
                    }
                }

                video_output_sink.is_populated
            };

            if is_video_output_frame_buffer_populated {
                self.window.update_with_buffer(&mut video_output_frame_buffer, 384, 224).unwrap();

                if self.mode == Mode::Running {
                    // We only want to update the key state when a frame is actually pushed
                    // Otherwise some games break.
                    self.read_input_keys();
                    if self.window.is_key_pressed(Key::F12, KeyRepeat::No) {
                        self.start_debugger();
                    }
                }
            }

            audio_dest.append(&audio_sink.buffer[..audio_sink.buffer_pos]);

            thread::sleep(time::Duration::from_millis(3));
        }
    }

    fn step(&mut self, video_sink: &mut VideoSink, audio_sink: &mut AudioSink) -> (u32, bool) {
        let ret = self.virtual_boy.step(video_sink, audio_sink);

        self.emulated_cycles += ret.0 as u64;

        ret
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

    fn run_debugger_commands(&mut self, video_sink: &mut VideoSink, audio_sink: &mut AudioSink) -> bool {
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
                Ok(Command::ShowCpuCache) => {
                    println!("CPU Instruction Cached enable: {}", self.virtual_boy.cpu.cache.is_enabled());
                    let (hits, misses) = self.virtual_boy.cpu.cache.stats();
                    let percent_hit = (hits as f64 / (hits + misses) as f64) * 100.0;
                    println!("Cache Hits: {}, Cache Misses: {} ({:.1}% hit rate)", hits, misses, percent_hit);
                    for i in 0..128 {
                        println!("Entry {:3}: {}", i, self.virtual_boy.cpu.cache.entry(i));
                    }
                },
                Ok(Command::Step(count)) => {
                    for _ in 0..count {
                        self.step(video_sink, audio_sink);
                        self.cursor = self.virtual_boy.cpu.reg_pc();
                        self.disassemble_instruction();
                    }
                }
                Ok(Command::Continue) => {
                    self.mode = Mode::Running;
                    self.time_source_start_time_ns = self.time_source.time_ns() - (self.emulated_cycles * CPU_CYCLE_TIME_NS);
                }
                Ok(Command::Goto(addr)) => {
                    self.cursor = addr;
                }
                Ok(Command::ShowMem(addr)) => {
                    if let Some(addr) = addr {
                        self.cursor = addr;
                    }

                    self.print_labels_at_cursor();

                    const NUM_ROWS: u32 = 16;
                    const NUM_COLS: u32 = 16;
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
                Ok(Command::Watchpoint) => {
                    for addr in self.virtual_boy.cpu.watchpoints.iter() {
                        println!("* 0x{:08x}", addr);
                    }
                }
                Ok(Command::AddWatchpoint(addr)) => {
                    self.virtual_boy.cpu.watchpoints.insert(addr);
                }
                Ok(Command::RemoveWatchpoint(addr)) => {
                    if !self.virtual_boy.cpu.watchpoints.remove(&addr) {
                        println!("Watchpoint at 0x{:08x} does not exist", addr);
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

            if self.mode == Mode::Debugging {
                self.print_cursor();
            }
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
                let reg1 = (first_halfword & 0x1f) as u32;
                let reg2 = ((first_halfword >> 5) & 0x1f) as u32;
                if opcode == Opcode::Jmp {
                    println!("jmp [r{}]", reg1);
                } else {
                    println!("{} r{}, r{}", opcode, reg1, reg2);
                }
            }
            InstructionFormat::II => {
                let imm5 = (first_halfword & 0x1f) as u32;
                let reg2 = ((first_halfword >> 5) & 0x1f) as u32;
                match opcode {
                    Opcode::BitString => {
                        let bit_string_op = opcode.bit_string_op(imm5);

                        println!("{}", bit_string_op);
                    }
                    Opcode::Cli | Opcode::Reti | Opcode::Halt | Opcode::Sei => println!("{}", opcode),
                    Opcode::Ldsr => println!("{} r{}, {}", opcode, reg2, opcode.system_register(imm5)),
                    Opcode::Stsr => println!("{} {}, r{}", opcode, opcode.system_register(imm5), reg2),
                    Opcode::MovImm | Opcode::AddImm5 | Opcode::CmpImm => println!("{} {}, r{}", opcode, (((imm5 as i32) << 27) >> 27), reg2),
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
                let reg1 = (first_halfword & 0x1f) as u32;
                let reg2 = ((first_halfword >> 5) & 0x1f) as u32;

                let imm16 = second_halfword;

                println!("{} {:#x}, r{}, r{}", opcode, imm16, reg1, reg2);
            }
            InstructionFormat::VI => {
                let reg1 = (first_halfword & 0x1f) as u32;
                let reg2 = ((first_halfword >> 5) & 0x1f) as u32;

                let disp16 = second_halfword as i16;

                match opcode {
                    Opcode::Stb | Opcode::Sth | Opcode::Stw | Opcode::Outb | Opcode::Outh | Opcode::Outw => println!("{} r{}, {}[r{}]", opcode, reg2, disp16, reg1),
                    _ => println!("{} {}[r{}], r{}", opcode, disp16, reg1, reg2),
                }
            }
            InstructionFormat::VII => {
                let reg1 = (first_halfword & 0x1f) as u32;
                let reg2 = ((first_halfword >> 5) & 0x1f) as u32;

                let subop_bits = second_halfword >> 10;

                let subop = opcode.subop(subop_bits);

                println!("{} r{}, r{}", subop, reg1, reg2);
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
