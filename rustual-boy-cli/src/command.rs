use nom::{IResult, eof, space, digit, hex_digit, alphanumeric};

use std::str::{self, FromStr};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Command {
    ShowCpuCache,
    ShowRegs,
    Step(usize),
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
    Watchpoint,
    AddWatchpoint(u32),
    RemoveWatchpoint(u32),
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

named!(
    command<Command>,
    complete!(
        terminated!(
        alt_complete!(
            show_cpu_cache|
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
            watchpoint |
            add_watchpoint |
            remove_watchpoint |
            exit |
            show_regs |
            repeat),
        eof)));

named!(
    show_cpu_cache<Command>,
    map!(
        alt_complete!(tag!("showcpucache") | tag!("scc")),
    |_| Command::ShowCpuCache));

named!(
    step<Command>,
    chain!(
        alt_complete!(tag!("step") | tag!("s")) ~
        count: opt!(preceded!(space, usize_parser)),
    || Command::Step(count.unwrap_or(1))));

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
    watchpoint<Command>,
    map!(
        alt_complete!(tag!("watchpoint") | tag!("w")),
    |_| Command::Watchpoint));

named!(
    add_watchpoint<Command>,
    chain!(
        alt_complete!(tag!("addwatchpoint") | tag!("aw")) ~
        space ~
        addr: hex_u32_parser,
    || Command::AddWatchpoint(addr)));

named!(
    remove_watchpoint<Command>,
    chain!(
        alt_complete!(tag!("removewatchpoint") | tag!("rw")) ~
        space ~
        addr: hex_u32_parser,
    || Command::RemoveWatchpoint(addr)));

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
