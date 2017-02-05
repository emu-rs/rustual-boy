use clap::{App, Arg};

pub struct CommandLineConfig {
    pub rom_path: String,
    pub sram_path: String,
}

pub fn parse_args() -> CommandLineConfig {
    let app = App::new("Rustual Boy")
        .version("0.1.0")
        .author(crate_authors!(", "))
        .about("A CLI frontend to the Rustual Boy emulator")
        .arg(Arg::with_name("ROM")
             .help("The name of the ROM to load")
             .required(true)
             .index(1)
        ).arg(Arg::with_name("SRAM")
              .help("Path to an SRAM")
              .short("s")
              .long("sram")
        );

    let matches = app.get_matches();
    //
    // unwrap is safe here becuase clap guarantees that required arguments are never None
    let rom_path = matches.value_of("ROM").unwrap();

    CommandLineConfig {
        rom_path: rom_path.into(),
        sram_path: match matches.value_of("SRAM") {
            Some(v) => v.into(),
            None => rom_path.replace(".vb", ".srm")
        },
    }
}
