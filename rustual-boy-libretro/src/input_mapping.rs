extern crate rustual_boy_core;
use rustual_boy_core::game_pad::{Button, GamePad};

use callbacks::Callbacks;
use input::{JoypadButton, AnalogStick};

const ANALOG_THRESHOLD: i16 = 0x7fff / 2;

pub fn map_input(callbacks: &'static Callbacks, game_pad: &mut GamePad) {
    game_pad.set_button_pressed(Button::A, callbacks.joypad_button(JoypadButton::A));
    game_pad.set_button_pressed(Button::B, callbacks.joypad_button(JoypadButton::B));
    game_pad.set_button_pressed(Button::L, callbacks.joypad_button(JoypadButton::L));
    game_pad.set_button_pressed(Button::R, callbacks.joypad_button(JoypadButton::R));
    game_pad.set_button_pressed(Button::Start, callbacks.joypad_button(JoypadButton::Start));
    game_pad.set_button_pressed(Button::Select,
                                callbacks.joypad_button(JoypadButton::Select));

    let joypad_left_pressed = callbacks.joypad_button(JoypadButton::Left);
    let joypad_right_pressed = callbacks.joypad_button(JoypadButton::Right);
    let joypad_up_pressed = callbacks.joypad_button(JoypadButton::Up);
    let joypad_down_pressed = callbacks.joypad_button(JoypadButton::Down);

    let (left_x, left_y) = callbacks.analog_xy(AnalogStick::Left);
    let left_analog_left_pressed = left_x < -ANALOG_THRESHOLD;
    let left_analog_right_pressed = left_x > ANALOG_THRESHOLD;
    let left_analog_up_pressed = left_y < -ANALOG_THRESHOLD;
    let left_analog_down_pressed = left_y > ANALOG_THRESHOLD;

    game_pad.set_button_pressed(Button::LeftDPadLeft,
                                left_analog_left_pressed || joypad_left_pressed);
    game_pad.set_button_pressed(Button::LeftDPadRight,
                                left_analog_right_pressed || joypad_right_pressed);
    game_pad.set_button_pressed(Button::LeftDPadUp,
                                left_analog_up_pressed || joypad_up_pressed);
    game_pad.set_button_pressed(Button::LeftDPadDown,
                                left_analog_down_pressed || joypad_down_pressed);

    let (right_x, right_y) = callbacks.analog_xy(AnalogStick::Right);
    game_pad.set_button_pressed(Button::RightDPadLeft, right_x < -ANALOG_THRESHOLD);
    game_pad.set_button_pressed(Button::RightDPadRight, right_x > ANALOG_THRESHOLD);
    game_pad.set_button_pressed(Button::RightDPadUp, right_y < -ANALOG_THRESHOLD);
    game_pad.set_button_pressed(Button::RightDPadDown, right_y > ANALOG_THRESHOLD);
}
