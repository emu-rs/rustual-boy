# Contributing

Want to contribute to Rustual Boy? Fantastic! We could use your help!

## Code of conduct

Any contribution to/participation in the Rustual Boy project is expected to follow the [Contributor Covenant code of conduct](CODE_OF_CONDUCT.md). Rustual Boy aims to be an open project where anyone can contribute, and upholding this code is essential to that goal!

## Getting started/getting in touch

If you're interested in contributing in any way, shape, or form, feel free to [join our discord server](https://discord.gg/65j9YMA), [email @yupferris](mailto:yupferris@gmail.com), [file an issue](https://github.com/emu-rs/rustual-boy/issues) or [make a pull request](https://github.com/emu-rs/rustual-boy/pulls). For larger things it's obviously favorable to have a chat about it beforehand; for smaller bugfixes etc feel free to just fork/PR.

## Improvement areas/ideas

Most of the core emulation/known game compatibility for Rustual Boy is nearing completion, but that doesn't mean the project is finished by any means! There are many areas where Rustual Boy can improve:

**Accuracy**: Even though the emulator is capable of playing most commercial games (to our knowledge), that doesn't mean every nook and cranny of the Virtual Boy hardware is emulated, and it doesn't mean we've got all the details nailed down. For example, the CPU has a pipeline that we haven't even bothered with yet. Its floating point operations don't always take the same amount of cycles. And many of the hardware details of the virtual boy are completely unknown/unexplored! We're always super pumped to improve our collective understanding of this hardware oddity.

**User interface**: Currently, Rustual Boy expects a ROM as its first command line arg. That's it! No further options, nothing. Ideally, we'd have both an improved CLI and a GUI would be ideal. This could obviously use a bit of TLC!

**Additional frontend(s)** Integration with something like [libretro](https://www.libretro.com/)/[retroarch](https://www.libretro.com/index.php/retroarch-2/) would be a HUGE win for us in terms of having a great user experience and getting people actually using this thing :)

**Performance**: As we've primarily focused on nailing the core emulation before spending too much time on this, the emulator is nowhere near as fast as it could be!

**Documentation**: Both inside and out, this project currently lacks any and all documentation, really.

**Webdev**: We've got a [basic landing page](http://rustualboy.com/), but it would be fab incorporate other bits as well (such as a blog for bugfix logs, status updates, releases, etc). See [this repo](https://github.com/yupferris/rustual-boy-www) for the current state of affairs.

**Branding/logo**: We have a logo, but it's pretty basic! Would be cool to take this even further.

**Feature request/issue/bug report management**: This project is so young, not even these details are hammered out yet! We'd love to get your input regarding this.

These are just _some_ areas we thought of off the top of our heads, and there are surely other areas to improve as well!

## Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
