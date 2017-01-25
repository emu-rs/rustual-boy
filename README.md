# Rustual Boy [![Build Status](https://travis-ci.org/emu-rs/rustual-boy.svg?branch=master)](https://travis-ci.org/emu-rs/rustual-boy) [![Build status](https://ci.appveyor.com/api/projects/status/ec29vne6uuh7tjtu/branch/master?svg=true)](https://ci.appveyor.com/project/yupferris/rustual-boy/branch/master) [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/emu-rs/rustual-boy#license)

![Rustual Boy](logo.png)

A WIP Virtual Boy emulator in Rust.

## Status

This emulator is not quite complete, but covers quite a bit of the CPU, memory mapping, and video hardware. It also contains a basic command-line debugger supporting disassembly, mem dump, breakpoints, etc. Many ROM's are fully compatible so far, but there's still some fundamental things that aren't yet implemented, such as some floating point flags/exceptions, bit string op's, etc. There are also a few bugs lurking around :)

Performance isn't fantastic yet, but the emulator should be playable at least. YMMV. Naturally, I'd like to get the core emulation logic in place before focusing too much on that.

A more detailed status/compatibility section will be written when the emu is complete enough for it to be relevant.

## Screenshots

![screenie](screenshot.png)
![screenie](screenshot2.png)
![screenie](screenshot3.png)
![screenie](screenshot4.png)

## Known game compatibility

| Game | USA | JP | Notes |
| --- | --- | --- | --- |
| 3-D Tetris | | N/A | Uses bit string ops |
| Bound High! | x | N/A | ROM is both USA+JP |
| Galactic Pinball | x | x | |
| Golf | | | Uses bit string ops |
| Insmouse No Yakata | N/A | x | |
| Jack Bros. | x | x | |
| Mario Clash | x | x | |
| Mario's Tennis | x | x | |
| Nester's Funky Bowling | | N/A | Uses bit string ops |
| Panic Bomber | x | x | |
| Red Alarm | | | Uses bit string ops |
| SD Gundam Dimension War | N/A | x | |
| Space Invaders Virtual Collection | N/A | | Lazers aren't visible on most of the screen in "Virtual 3D" mode |
| Space Squash | N/A | | Game doesn't start; seems to be waiting on an interrupt (see commits [7423524](https://github.com/emu-rs/rustual-boy/commit/74235249a1abfca8d4b3d80e8c3c6b37230679a2) and [0ff9c61](https://github.com/emu-rs/rustual-boy/commit/0ff9c61efb188832680292a11c1a24c5c4f25360) for some investigation) |
| Teleroboxer | x | x | |
| V-Tetris | N/A | x | |
| Vertical Force | | | Enemies/powerups appear/disappear, first boss disappears and the game softlocks |
| Virtual Bowling | N/A | | Some graphical glitches and slowdowns in the intro sequence, possibly other bugs |
| Virtual Boy Wario Land | x | x | |
| Virtual Fishing | N/A | x | |
| Virtual Lab | N/A | x | |
| Virtual League Baseball | x | x | |
| Waterworld | x | N/A | |

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
