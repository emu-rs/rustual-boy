use clap::{App, Arg};

pub struct CommandLineConfig {
    pub rom_path: String,
    pub sram_path: String,
}

pub fn parse_args() -> CommandLineConfig {
    let app = App::new("Rustual-boy")
        .version("0.1.0")
        .author(crate_authors!(", "))
        .about("A CLI frontend to the rustual-boy emulator")
        .arg(Arg::with_name("ROM")
             .help("The name of the ROM to load")
             .required(true)
             .index(1)
        );

    let matches = app.get_matches();

    CommandLineConfig {
        rom_path: matches.value_of("ROM").unwrap().into(),
        sram_path: matches.value_of("ROM").unwrap().replace(".vb", ".srm").into()
    }
}
