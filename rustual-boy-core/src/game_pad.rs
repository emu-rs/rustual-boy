pub enum Button {
    A,
    B,
    Start,
    Select,
    L,
    R,
    LeftDPadUp,
    LeftDPadDown,
    LeftDPadLeft,
    LeftDPadRight,
    RightDPadUp,
    RightDPadDown,
    RightDPadLeft,
    RightDPadRight,
}

pub struct GamePad {
    a_pressed: bool,
    b_pressed: bool,
    start_pressed: bool,
    select_pressed: bool,
    l_pressed: bool,
    r_pressed: bool,
    left_d_pad_up_pressed: bool,
    left_d_pad_down_pressed: bool,
    left_d_pad_left_pressed: bool,
    left_d_pad_right_pressed: bool,
    right_d_pad_up_pressed: bool,
    right_d_pad_down_pressed: bool,
    right_d_pad_left_pressed: bool,
    right_d_pad_right_pressed: bool,
}

impl GamePad {
    pub fn new() -> GamePad {
        GamePad {
            a_pressed: false,
            b_pressed: false,
            start_pressed: false,
            select_pressed: false,
            l_pressed: false,
            r_pressed: false,
            left_d_pad_up_pressed: false,
            left_d_pad_down_pressed: false,
            left_d_pad_left_pressed: false,
            left_d_pad_right_pressed: false,
            right_d_pad_up_pressed: false,
            right_d_pad_down_pressed: false,
            right_d_pad_left_pressed: false,
            right_d_pad_right_pressed: false,
        }
    }

    pub fn read_scr(&self) -> u8 {
        logln!(Log::GamePad, "WARNING: Read SCR not yet implemented");
        0
    }

    pub fn write_scr(&mut self, value: u8) {
        logln!(Log::GamePad, "WARNING: Write SCR not yet implemented (value: 0x{:02x})", value);
    }

    pub fn read_sdlr(&self) -> u8 {
        let version = 1;
        // TODO: Would be cool to be able to toggle this at startup/runtime :)
        let low_battery = false;
        (if self.right_d_pad_right_pressed { 1 } else { 0 } << 7) |
        (if self.right_d_pad_up_pressed { 1 } else { 0 } << 6) |
        (if self.l_pressed { 1 } else { 0 } << 5) |
        (if self.r_pressed { 1 } else { 0 } << 4) |
        (if self.b_pressed { 1 } else { 0 } << 3) |
        (if self.a_pressed { 1 } else { 0 } << 2) |
        (version << 1) |
        if low_battery { 1 } else { 0 }
    }

    pub fn read_sdhr(&self) -> u8 {
        (if self.right_d_pad_down_pressed { 1 } else { 0 } << 7) |
        (if self.right_d_pad_left_pressed { 1 } else { 0 } << 6) |
        (if self.select_pressed { 1 } else { 0 } << 5) |
        (if self.start_pressed { 1 } else { 0 } << 4) |
        (if self.left_d_pad_up_pressed { 1 } else { 0 } << 3) |
        (if self.left_d_pad_down_pressed { 1 } else { 0 } << 2) |
        (if self.left_d_pad_left_pressed { 1 } else { 0 } << 1) |
        if self.left_d_pad_right_pressed { 1 } else { 0 }
    }

    pub fn set_button_pressed(&mut self, button: Button, pressed: bool) {
        match button {
            Button::A => self.a_pressed = pressed,
            Button::B => self.b_pressed = pressed,
            Button::Start => self.start_pressed = pressed,
            Button::Select => self.select_pressed = pressed,
            Button::L => self.l_pressed = pressed,
            Button::R => self.r_pressed = pressed,
            Button::LeftDPadUp => self.left_d_pad_up_pressed = pressed,
            Button::LeftDPadDown => self.left_d_pad_down_pressed = pressed,
            Button::LeftDPadLeft => self.left_d_pad_left_pressed = pressed,
            Button::LeftDPadRight => self.left_d_pad_right_pressed = pressed,
            Button::RightDPadUp => self.right_d_pad_up_pressed = pressed,
            Button::RightDPadDown => self.right_d_pad_down_pressed = pressed,
            Button::RightDPadLeft => self.right_d_pad_left_pressed = pressed,
            Button::RightDPadRight => self.right_d_pad_right_pressed = pressed,
        }
    }
}
