extern crate rustual_boy_core;
use rustual_boy_core::game_pad::{GamePad, Button};

use callbacks::Callbacks;

pub enum JoypadButton {
    B = 0,
    Y = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
    A = 8,
    X = 9,
    L = 10,
    R = 11,
    L2 = 12,
    R2 = 13,
    L3 = 14,
    R3 = 15,
}

#[derive(Clone)]
pub enum AnalogAxis {
    X = 0,
    Y = 1,
}

#[derive(Clone)]
pub enum AnalogStick {
    Left = 0,
    Right = 1,
}


#[allow(dead_code)]
pub enum RetroDeviceType {
    None = 0,
    Joypad = 1,
    Mouse = 2,
    Keyboard = 3,
    Lightgun = 4,
    Analog = 5,
    Pointer = 6,
}

impl Callbacks {
    pub fn joypad_button(&self, button: JoypadButton) -> bool {
        0 != self.input_state(0, RetroDeviceType::Joypad as u32, 0, button as u32)
    }

    pub fn analog_xy(&self, stick: AnalogStick) -> (i16, i16) {
        let x = self.input_state(0,
                                 RetroDeviceType::Analog as u32,
                                 stick.clone() as u32,
                                 AnalogAxis::X as u32);
        let y = self.input_state(0,
                                 RetroDeviceType::Analog as u32,
                                 stick as u32,
                                 AnalogAxis::Y as u32);

        (x, y)
    }
}
