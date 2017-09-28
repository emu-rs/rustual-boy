extern crate wisegui;

use self::wisegui::*;

use rustual_boy_core::mem_map::*;
use rustual_boy_core::virtual_boy::*;

use minifb::{MouseButton, MouseMode, WindowOptions, Window, Scale};

/*struct DefaultPalette;

impl Palette for DefaultPalette {
    fn color(&self, color: Color) -> u32 {
        match color {
            Color::Darkest => 0x000000,
            Color::Dark => 0x555555,
            Color::Light => 0xaaaaaa,
            Color::Lightest => 0xffffff,
        }
    }
}*/

struct VirtualBoyPalette;

impl Palette for VirtualBoyPalette {
    fn color(&self, color: Color) -> u32 {
        match color {
            Color::Darkest => 0x000000,
            Color::Dark => 0x550000,
            Color::Light => 0xaa0000,
            Color::Lightest => 0xff0000,
        }
    }
}

pub struct Inspector {
    window: Window,
    width: i32,
    height: i32,
    buffer: Vec<u32>,

    context: Context,

    wrams: Vec<Vec<u8>>,
    wrams_index: u32,
    wram_access_color: Vec<u8>,

    vrams: Vec<Vec<u8>>,
    vrams_index: u32,
    vram_access_color: Vec<u8>,
}

impl Inspector {
    pub fn new() -> Inspector {
        let width = 780;
        let height = 540;

        Inspector {
            window: Window::new("Rustual Boy", width as _, height as _, WindowOptions {
                borderless: false,
                title: true,
                resize: true,
                scale: Scale::X2,
            }).unwrap(),
            width: width,
            height: height,
            buffer: vec![0; (width * height) as usize],

            context: Context::new(Box::new(VirtualBoyPalette)),

            wrams: vec![vec![0; 0x10000]; 2],
            wrams_index: 0,
            wram_access_color: vec![0; 0x10000],

            vrams: vec![vec![0; 0x40000]; 2],
            vrams_index: 0,
            vram_access_color: vec![0; 0x40000],
        }
    }

    pub fn update(&mut self, virtual_boy: &mut VirtualBoy) {
        if !self.window.is_open() {
            return;
        }

        let mouse_pos = {
            let p = self.window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
            (p.0 as i32, p.1 as i32)
        };
        let is_left_mouse_down = self.window.get_mouse_down(MouseButton::Left);
        self.context.update(mouse_pos, is_left_mouse_down);

        let (width, height) = self.window.get_size();
        if width as i32 != self.width || height as i32 != self.height {
            self.width = width as i32;
            self.height = height as i32;
            self.buffer = vec![0; width * height];
        }

        {
            let mut painter = Painter::new(&self.context, &mut self.buffer, self.width as _, self.height as _);

            painter.clear(Color::Dark);
        }

        self.wram_view(virtual_boy);
        self.vram_view(virtual_boy);

        self.window.update_with_buffer(&self.buffer);
    }

    fn wram_view(&mut self, virtual_boy: &mut VirtualBoy) {
        let margin = 4;

        {
            let mut painter = Painter::new(&self.context, &mut self.buffer, self.width as _, self.height as _);

            painter.rect(margin, margin, 256, 256, None, Some(Color::Lightest));
        }

        for i in 0..0x10000 {
            self.wrams[self.wrams_index as usize][i as usize] = virtual_boy.interconnect.read_byte(WRAM_START + i);
        }

        for y in 0..256 {
            let pixel_y = y + margin;
            if pixel_y >= self.height {
                break;
            }

            for x in 0..256 {
                let pixel_x = x + margin;
                if pixel_x >= self.width {
                    break;
                }

                let buffer_index = (y * 256 + x) as usize;
                let byte = self.wrams[self.wrams_index as usize][buffer_index];
                let last_byte = self.wrams[1 - self.wrams_index as usize][buffer_index];
                self.wram_access_color[buffer_index] = if byte != last_byte {
                    255
                } else if self.wram_access_color[buffer_index] > 0 {
                    self.wram_access_color[buffer_index] - 1
                } else {
                    0
                };
                let color = ((byte as u32) << 16) | (self.wram_access_color[buffer_index] as u32);

                self.buffer[(pixel_y * self.width + pixel_x) as usize] = color;
            }
        }

        self.wrams_index = 1 - self.wrams_index;
    }

    fn vram_view(&mut self, virtual_boy: &mut VirtualBoy) {
        let margin = 4;

        let start_x = margin * 2 + 256;
        let start_y = margin;

        {
            let mut painter = Painter::new(&self.context, &mut self.buffer, self.width as _, self.height as _);

            painter.rect(start_x, start_y, 512, 512, None, Some(Color::Lightest));
        }

        for i in 0..0x40000 {
            self.vrams[self.vrams_index as usize][i as usize] = virtual_boy.interconnect.read_byte(i);
        }

        for y in 0..512 {
            let pixel_y = y + start_y;
            if pixel_y >= self.height {
                break;
            }

            for x in 0..512 {
                let pixel_x = x + start_x;
                if pixel_x >= self.width {
                    break;
                }

                let buffer_index = (y * 512 + x) as usize;
                let byte = self.vrams[self.vrams_index as usize][buffer_index];
                let last_byte = self.vrams[1 - self.vrams_index as usize][buffer_index];
                self.vram_access_color[buffer_index] = if byte != last_byte {
                    255
                } else if self.vram_access_color[buffer_index] > 0 {
                    self.vram_access_color[buffer_index] - 1
                } else {
                    0
                };
                let color = ((byte as u32) << 16) | (self.vram_access_color[buffer_index] as u32);

                self.buffer[(pixel_y * self.width + pixel_x) as usize] = color;
            }
        }

        self.vrams_index = 1 - self.vrams_index;
    }
}
