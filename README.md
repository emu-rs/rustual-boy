# Rustual Boy

[![Build Status](https://travis-ci.org/emu-rs/rustual-boy.svg?branch=master)](https://travis-ci.org/emu-rs/rustual-boy) [![Build status](https://ci.appveyor.com/api/projects/status/ec29vne6uuh7tjtu/branch/master?svg=true)](https://ci.appveyor.com/project/yupferris/rustual-boy/branch/master) [![rustualboy.com](https://img.shields.io/badge/www-rustualboy.com-red.svg)](https://rustualboy.com/) [![Compabitility](https://img.shields.io/badge/compatibility-100%25-green.svg)](https://github.com/emu-rs/rustual-boy/blob/master/README.md#known-game-compatibility) [![Chat on discord](https://img.shields.io/badge/chat-on%20discord-7289DA.svg)](https://discord.gg/65j9YMA) [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/emu-rs/rustual-boy/blob/master/README.md#license)

![Rustual Boy](media/logo.png)

## Description

Rustual Boy is a [Virtual Boy](https://en.wikipedia.org/wiki/Virtual_Boy) emulator. It can be used to play existing Virtual Boy games, as well as be a helpful development/debugging tool for homebrew for this platform.

The name "Rustual Boy" is a [portmanteau](https://en.wikipedia.org/wiki/Portmanteau) of the words "Rust" and "Virtual Boy". Ok, the "Virtual Boy" part was obvious, but why "Rust"? In fact, Rustual Boy is written in the [Rust programming language](https://www.rust-lang.org/en-US/) - a "systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety." Rust is a wonderful language with a thriving community, and as such provides a fantastic platform for an emulation project!

## Status

Rustual Boy is currently able to emulate [all commercial Virtual Boy titles](https://github.com/emu-rs/rustual-boy/blob/master/README.md#known-game-compatibility), as well as many homebrew ones. It supports basic video/audio output and keyboard input, and also a simple CLI debugger. It runs on Windows, MacOS, Linux, and likely some other platforms Rust supports.

While Rustual Boy's core emulation is complete [compatibility-wise](https://github.com/emu-rs/rustual-boy/blob/master/README.md#known-game-compatibility), and in some respects is already one of the more accurate VB emulators available, the project is still young, and needs some time to mature. Particularly, its user interface is lacking (read: virtually non-existent), and there are some known (and probably unknown) stability/compatibility issues. Also, performance isn't fantastic yet, but the emulator should be playable at least.

That said, these things are bound to improve with time, and everyone is encouraged to [grab a build](https://github.com/emu-rs/rustual-boy/releases) or [build the emulator](https://github.com/emu-rs/rustual-boy/blob/master/README.md#building-and-running), give it a shot, and [report any bugs/feature requests](CONTRIBUTING.md)!

## Screenshots

![screenie](media/screenshot.png)
![screenie](media/screenshot2.png)
![screenie](media/screenshot3.png)
![screenie](media/screenshot4.png)

## Known game compatibility

Key:
 - 💖: Fully compatible. The game is fully playable with no known bugs (to our knowledge).
 - 💔: Playable. The game can be played, but has some minor known glitches (described in the notes field).
 - 🚫: Incompatible. The reason is described in the notes field.
 - `N/A`: Game does not exist for this region.

Games marked 💖 and 💔 are all included in the compatibility metric.

| Game | USA | JP | Notes |
| --- | --- | --- | --- |
| 3-D Tetris | 💖 | N/A | |
| Bound High! | 💖 | N/A | ROM is both USA+JP |
| Galactic Pinball | 💔 | 💔 | Vocal samples throughout the game play at the wrong rate, likely related to CPU clock cycle inaccuracy |
| Golf | 💔 | 💔 | Game transitions to demo mode WAY too fast, sometimes softlocks when transitioning to/from demo mode |
| Insmouse No Yakata | N/A | 💖 | |
| Jack Bros. | 💖 | 💖 | |
| Mario Clash | 💖 | 💖 | |
| Mario's Tennis | 💖 | 💖 | |
| Nester's Funky Bowling | 💖 | N/A | |
| Panic Bomber | 💖 | 💖 | |
| Red Alarm | 💖 | 💖 | |
| SD Gundam Dimension War | N/A | 💖 | |
| Space Invaders Virtual Collection | N/A | 💖 | |
| Space Squash | N/A | 💖 | |
| Teleroboxer | 💔 | 💔 | Small audio glitches during screen transitions after demo mode wins games |
| V-Tetris | N/A | 💖 | |
| Vertical Force | 💖 | 💖 | |
| Virtual Bowling | N/A | 💔 | Samples play at the wrong rate |
| Virtual Boy Wario Land | 💔 | 💔 | Game is fully playable, but some parts of the intro cutscene are too dark, due to inaccurate emulation of the physical LED periods used in the display |
| Virtual Fishing | N/A | 💖 | |
| Virtual Lab | N/A | 💖 | |
| Virtual League Baseball | 💖 | 💖 | |
| Waterworld | 💖 | N/A | |

## Building and running

Currently, the only dependency for building is Rust itself, which can be downloaded [here](https://www.rust-lang.org/downloads.html). Once that's installed, you can clone the repo, and simply `cargo build`/`cargo run` your way to victory! It's recommended to use the `--release` flag as well, as emulation can require a lot of CPU power, so we'll want all the compiler help we can get.

> Note: If you're new to using Cargo (Rust's build system), it's recommended to give the [Cargo Guide](http://doc.crates.io/guide.html) a quick skim.

Rustual Boy has a very simple CLI interface:

```
$ rustual-boy-cli --help
Rustual Boy 0.2.0
ferris <yupferris@gmail.com>, The Rustual Boy contributors
A CLI frontend to the Rustual Boy emulator

USAGE:
    rustual-boy-cli.exe [FLAGS] <ROM>

FLAGS:
    -s, --sram       Path to an SRAM
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <ROM>    The name of the ROM to load
```

The easiest way to run a game is to supply a single file argument:

```
rustual-boy-cli bound-high.vb
```

## Input keymap

The Virtual Boy game pad consists of two d-pads, 4 buttons, and two shoulder buttons. These are mapped to the following keyboard keys:

| Button | Key |
| --- | --- |
| Left D-Pad Up | <kbd>W</kbd> |
| Left D-Pad Down | <kbd>S</kbd> |
| Left D-Pad Left | <kbd>A</kbd> |
| Left D-Pad Right | <kbd>D</kbd> |
| Right D-Pad Up | <kbd>I</kbd> |
| Right D-Pad Down | <kbd>K</kbd> |
| Right D-Pad Left | <kbd>J</kbd> |
| Right D-Pad Right | <kbd>L</kbd> |
| Select | <kbd>space</kbd> |
| Start | <kbd>enter/return</kbd> |
| A | <kbd>F</kbd> |
| B | <kbd>H</kbd> |
| Left bumper | <kbd>E</kbd> |
| Right bumper | <kbd>U</kbd> |

For game pad layout reference, refer to [this image](https://en.wikipedia.org/wiki/Virtual_Boy#/media/File:Virtual-Boy-Set.jpg). This key map is currently non-configurable.

## Contributing

Rustual Boy aims to be an open project where anyone can contribute. If you're interested, check [CONTRIBUTING.md](CONTRIBUTING.md)!

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
