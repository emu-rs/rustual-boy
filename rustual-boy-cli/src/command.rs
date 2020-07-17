use combine::{choice, eof, many1, optional, Parser, parser, try, value};
use combine::char::{alpha_num, digit, hex_digit, space, spaces, string};
use combine::primitives::{ParseResult, Stream};

use std::str::{self, FromStr};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Command {
    ShowCpuCache,
    ShowRegs,
    Step(u32),
    Continue,
    Goto(u32),
    ShowMem(Option<u32>),
    Disassemble(u32),
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
        match parser(command).parse(s) {
            Ok((c, _)) => Ok(c),
            err => Err(format!("Unable to parse command: {:?}", err).into()),
        }
    }
}

fn command<I: Stream<Item=char>>(input: I) -> ParseResult<Command, I> {
    let show_cpu_cache =
        choice([try(string("show_cpu_cache")), try(string("scc"))])
        .map(|_| Command::ShowCpuCache)
        .boxed();

    let show_regs =
        choice([try(string("showregs")), try(string("r"))])
        .map(|_| Command::ShowRegs)
        .boxed();

    let step =
        (choice([try(string("step")), try(string("s"))]),
            optional((spaces(), u32_()).map(|x| x.1)))
        .map(|(_, count)| Command::Step(count.unwrap_or(1)))
        .boxed();

    let continue_ =
        choice([try(string("continue")), try(string("c"))])
        .map(|_| Command::Continue)
        .boxed();

    let goto =
        (choice([try(string("goto")), try(string("g"))]), spaces(), u32_hex())
        .map(|(_, _, addr)| Command::Goto(addr))
        .boxed();

    let show_mem =
        (choice([try(string("showmem")), try(string("m"))]),
            optional((spaces(), u32_hex()).map(|x| x.1)))
        .map(|(_, addr)| Command::ShowMem(addr))
        .boxed();

    let disassemble =
        (choice([try(string("disassemble")), try(string("d"))]),
            optional((spaces(), u32_()).map(|x| x.1)))
        .map(|(_, count)| Command::Disassemble(count.unwrap_or(4)))
        .boxed();

    let label =
        choice([try(string("label")), try(string("l"))])
        .map(|_| Command::Label)
        .boxed();

    let add_label =
        (choice([try(string("addlabel")), try(string("al"))]),
            space(),
            label_name(),
            space(),
            u32_hex())
        .map(|(_, _, name, _, addr)| Command::AddLabel(name, addr))
        .boxed();

    let remove_label =
        (choice([try(string("removelabel")), try(string("rl"))]),
            space(),
            label_name())
        .map(|(_, _, name)| Command::RemoveLabel(name))
        .boxed();

    let breakpoint =
        choice([try(string("breakpoint")), try(string("b"))])
        .map(|_| Command::Breakpoint)
        .boxed();

    let add_breakpoint =
        (choice([try(string("addbreakpoint")), try(string("ab"))]),
            space(),
            u32_hex())
        .map(|(_, _, addr)| Command::AddBreakpoint(addr))
        .boxed();

    let remove_breakpoint =
        (choice([try(string("removebreakpoint")), try(string("rb"))]),
            space(),
            u32_hex())
        .map(|(_, _, addr)| Command::RemoveBreakpoint(addr))
        .boxed();

    let watchpoint =
        choice([try(string("watchpoint")), try(string("w"))])
        .map(|_| Command::Watchpoint)
        .boxed();

    let add_watchpoint =
        (choice([try(string("addwatchpoint")), try(string("aw"))]),
            space(),
            u32_hex())
        .map(|(_, _, addr)| Command::AddWatchpoint(addr))
        .boxed();

    let remove_watchpoint =
        (choice([try(string("removewatchpoint")), try(string("rw"))]),
            space(),
            u32_hex())
        .map(|(_, _, addr)| Command::RemoveWatchpoint(addr))
        .boxed();

    let exit =
        choice([try(string("exit")), try(string("quit")), try(string("e")), try(string("x")), try(string("q"))])
        .map(|_| Command::Exit)
        .boxed();

    let repeat = value(Command::Repeat).boxed();

    choice(
        vec![
            show_cpu_cache,
            show_regs,
            step,
            continue_,
            goto,
            show_mem,
            disassemble,
            label,
            add_label,
            remove_label,
            breakpoint,
            add_breakpoint,
            remove_breakpoint,
            watchpoint,
            add_watchpoint,
            remove_watchpoint,
            exit,
            repeat,
        ]
        .into_iter()
        .map(|parser| (parser, eof()).map(|x| x.0))
        .map(|parser| try(parser))
        .collect::<Vec<_>>())
    .parse_stream(input)
}

fn u32_<'a, I: Stream<Item=char> + 'a>() -> Box<dyn Parser<Input=I, Output=u32> + 'a> {
    many1(digit())
        .and_then(|s: String| s.parse::<u32>())
        .boxed()
}

fn u32_hex<'a, I: Stream<Item=char> + 'a>() -> Box<dyn Parser<Input=I, Output=u32> + 'a> {
    let hex_prefix = choice([try(string("0x")), try(string("$"))]);
    (optional(hex_prefix), many1(hex_digit()))
        .map(|x| x.1)
        .and_then(|s: String| u32::from_str_radix(&s, 16))
        .boxed()
}

fn label_name<'a, I: Stream<Item=char> + 'a>() -> Box<dyn Parser<Input=I, Output=String> + 'a> {
    many1::<String, _>(alpha_num()).boxed()
}
