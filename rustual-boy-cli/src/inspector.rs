extern crate wisegui;

use simd::*;

use self::wisegui::*;

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

    wram: Vec<u8x16>,
    wram_access_color: Vec<u8x16>,

    vram: Vec<u8x16>,
    vram_access_color: Vec<u8x16>,
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

            wram: vec![u8x16::splat(0); 0x10000 / 16],
            wram_access_color: vec![u8x16::splat(0); 0x10000 / 16],

            vram: vec![u8x16::splat(0); 0x40000 / 16],
            vram_access_color: vec![u8x16::splat(0); 0x40000 / 16],
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

        for i in 0..0x10000 / 16 {
            let bytes = u8x16::load(&virtual_boy.interconnect.wram.bytes(), i * 16);
            let last_bytes = self.wram[i as usize];

            let last_access_color = self.wram_access_color[i as usize];

            let different_bytes = bytes.ne(last_bytes);
            let last_access_color_gt_0 = last_access_color.gt(u8x16::splat(0));

            let _255s = u8x16::splat(255);
            let _0s = u8x16::splat(0);

            // different_bytes { 255 } else last_access_color_gt_0 { last_access_color_gt_0 - 1 } else { 0 }

            let tmp = last_access_color_gt_0.select(last_access_color - u8x16::splat(1), _0s);
            self.wram_access_color[i as usize] = different_bytes.select(_255s, tmp);

            self.wram[i as usize] = bytes;
        }

        for y in 0..256 {
            let pixel_y = y + margin;
            if pixel_y >= self.height {
                break;
            }

            for x in 0..256 / 16 {
                let pixel_x = x * 16 + margin;
                if pixel_x >= self.width {
                    break;
                }

                let buffer_index = (y * 256 / 16 + x) as usize;

                let reds = self.wram[buffer_index];
                let blues = self.wram_access_color[buffer_index];

                for i in 0..4 {
                    let reds =
                        u32x4::new(
                            reds.extract(i * 4 + 0) as _,
                            reds.extract(i * 4 + 1) as _,
                            reds.extract(i * 4 + 2) as _,
                            reds.extract(i * 4 + 3) as _);
                    let blues =
                        u32x4::new(
                            blues.extract(i * 4 + 0) as _,
                            blues.extract(i * 4 + 1) as _,
                            blues.extract(i * 4 + 2) as _,
                            blues.extract(i * 4 + 3) as _);

                    let colors: u32x4 = (reds << 16) | blues;

                    colors.store(&mut self.buffer, (pixel_y * self.width + pixel_x + (i as i32) * 4) as usize);
                }
            }
        }
    }

    fn vram_view(&mut self, virtual_boy: &mut VirtualBoy) {
        let margin = 4;

        let start_x = margin * 2 + 256;
        let start_y = margin;

        {
            let mut painter = Painter::new(&self.context, &mut self.buffer, self.width as _, self.height as _);

            painter.rect(start_x, start_y, 512, 512, None, Some(Color::Lightest));
        }

        for i in 0..0x40000 / 16 {
            let bytes = u8x16::load(&virtual_boy.interconnect.vip.vram(), i * 16);
            let last_bytes = self.vram[i as usize];

            let last_access_color = self.vram_access_color[i as usize];

            let different_bytes = bytes.ne(last_bytes);
            let last_access_color_gt_0 = last_access_color.gt(u8x16::splat(0));

            let _255s = u8x16::splat(255);
            let _0s = u8x16::splat(0);

            // different_bytes { 255 } else last_access_color_gt_0 { last_access_color_gt_0 - 1 } else { 0 }

            let tmp = last_access_color_gt_0.select(last_access_color - u8x16::splat(1), _0s);
            self.vram_access_color[i as usize] = different_bytes.select(_255s, tmp);

            self.vram[i as usize] = bytes;
        }

        for y in 0..512 {
            let pixel_y = y + start_y;
            if pixel_y >= self.height {
                break;
            }

            for x in 0..512 / 16 {
                let pixel_x = x * 16 + start_x;
                if pixel_x >= self.width {
                    break;
                }

                let buffer_index = (y * 512 / 16 + x) as usize;

                let reds = self.vram[buffer_index];
                let blues = self.vram_access_color[buffer_index];

                for i in 0..4 {
                    let reds =
                        u32x4::new(
                            reds.extract(i * 4 + 0) as _,
                            reds.extract(i * 4 + 1) as _,
                            reds.extract(i * 4 + 2) as _,
                            reds.extract(i * 4 + 3) as _);
                    let blues =
                        u32x4::new(
                            blues.extract(i * 4 + 0) as _,
                            blues.extract(i * 4 + 1) as _,
                            blues.extract(i * 4 + 2) as _,
                            blues.extract(i * 4 + 3) as _);

                    let colors: u32x4 = (reds << 16) | blues;

                    colors.store(&mut self.buffer, (pixel_y * self.width + pixel_x + (i as i32) * 4) as usize);
                }
            }
        }
    }
}
