use instruction::*;
use interconnect::*;

pub struct Nvc {
    reg_pc: u32,
    reg_gpr: [u32; 31],
}

impl Nvc {
    pub fn new() -> Nvc {
        Nvc {
            reg_pc: 0xfffffff0,
            reg_gpr: [0; 31],
        }
    }

    pub fn reg_pc(&self) -> u32 {
        self.reg_pc
    }

    pub fn reg_gpr(&self, index: usize) -> u32 {
        if index == 0 {
            0
        } else {
            self.reg_gpr[index - 1]
        }
    }

    fn set_reg_gpr(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.reg_gpr[index - 1] = value;
        }
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) {
        let first_halfword = interconnect.read_halfword(self.reg_pc);
        self.reg_pc = self.reg_pc.wrapping_add(2);

        let opcode = Opcode::from_halfword(first_halfword);
        let instruction_format = opcode.instruction_format();

        let second_halfword = if instruction_format.has_second_halfword() {
            let second_halfword = interconnect.read_halfword(self.reg_pc);
            self.reg_pc = self.reg_pc.wrapping_add(2);
            second_halfword
        } else {
            0
        };

        match opcode {
            Opcode::Jmp => format_i(|reg1, _| {
                self.reg_pc = self.reg_gpr(reg1);
            }, first_halfword),
            Opcode::MovImm => format_ii(|imm5, reg2| {
                let value = sign_extend_imm5(imm5);
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::Movea => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as i16) as u32);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::Movhi => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as u32) << 16);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::Stb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = self.reg_gpr(reg2) as u8;
                interconnect.write_byte(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Outw => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = self.reg_gpr(reg2);
                interconnect.write_word(addr, value);
            }, first_halfword, second_halfword),
        }

        interconnect.cycles(opcode.num_cycles());
    }
}

fn format_i<F: FnOnce(usize, usize)>(f: F, first_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    f(reg1, reg2);
}

fn format_ii<F: FnOnce(usize, usize)>(f: F, first_halfword: u16) {
    let imm5 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    f(imm5, reg2);
}

fn format_v<F: FnOnce(usize, usize, u16)>(f: F, first_halfword: u16, second_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    let imm16 = second_halfword;
    f(reg1, reg2, imm16);
}

fn format_vi<F: FnOnce(usize, usize, i16)>(f: F, first_halfword: u16, second_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    let disp16 = second_halfword as i16;
    f(reg1, reg2, disp16);
}

fn sign_extend_imm5(imm5: usize) -> u32 {
    let imm5 = imm5 | (if imm5 & 0x10 == 0 { 0x00 } else { 0xe0 });
    (imm5 as i8) as u32
}