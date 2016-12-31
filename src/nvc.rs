use video_driver::*;
use instruction::*;
use interconnect::*;

use std::collections::HashSet;

pub struct Nvc {
    reg_pc: u32,
    reg_gpr: [u32; 31],

    reg_eipc: u32,
    reg_eipsw: u32,
    reg_ecr: u16,

    psw_zero: bool,
    psw_sign: bool,
    psw_overflow: bool,
    psw_carry: bool,
    psw_fp_precision_degredation: bool,
    psw_fp_underflow: bool,
    psw_fp_overflow: bool,
    psw_fp_zero_division: bool,
    psw_fp_invalid_operation: bool,
    psw_fp_reserved_operand: bool,
    psw_interrupt_disable: bool,
    psw_address_trap_enable: bool,
    psw_exception_pending: bool,
    psw_nmi_pending: bool,
    psw_interrupt_mask_level: usize,

    pub watchpoints: HashSet<u32>,
}

impl Nvc {
    pub fn new() -> Nvc {
        Nvc {
            reg_pc: 0xfffffff0,
            reg_gpr: [0xdeadbeef; 31],

            reg_eipc: 0xdeadbeef,
            reg_eipsw: 0xdeadbeef,
            reg_ecr: 0xbeef,

            psw_zero: false,
            psw_sign: false,
            psw_overflow: false,
            psw_carry: false,
            psw_fp_precision_degredation: false,
            psw_fp_underflow: false,
            psw_fp_overflow: false,
            psw_fp_zero_division: false,
            psw_fp_invalid_operation: false,
            psw_fp_reserved_operand: false,
            psw_interrupt_disable: false,
            psw_address_trap_enable: false,
            psw_exception_pending: false,
            psw_nmi_pending: true,
            psw_interrupt_mask_level: 0,

            watchpoints: HashSet::new(),
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

    // TODO: Come up with a more portable way to do this conversion
    fn reg_gpr_float(&self, index: usize) -> f32 {
        let value = self.reg_gpr(index);
        let value_ptr = &value as *const _;
        let value_float_ptr = value_ptr as *const f32;
        unsafe { *value_float_ptr }
    }

    fn set_reg_gpr_float(&mut self, index: usize, value: f32) {
        let value_ptr = &value as *const _;
        let value_int_ptr = value_ptr as *const u32;
        self.set_reg_gpr(index, unsafe { *value_int_ptr });
    }

    pub fn reg_eipc(&self) -> u32 {
        self.reg_eipc
    }

    pub fn reg_eipsw(&self) -> u32 {
        self.reg_eipsw
    }

    pub fn reg_ecr(&self) -> u16 {
        self.reg_ecr
    }

    pub fn reg_psw(&self) -> u32 {
        (if self.psw_zero { 1 << 0 } else { 0 }) |
        (if self.psw_sign { 1 << 1 } else { 0 }) |
        (if self.psw_overflow { 1 << 2 } else { 0 }) |
        (if self.psw_carry { 1 << 3 } else { 0 }) |
        (if self.psw_fp_precision_degredation { 1 << 4 } else { 0 }) |
        (if self.psw_fp_underflow { 1 << 5 } else { 0 }) |
        (if self.psw_fp_overflow { 1 << 6 } else { 0 }) |
        (if self.psw_fp_zero_division { 1 << 7 } else { 0 }) |
        (if self.psw_fp_invalid_operation { 1 << 8 } else { 0 }) |
        (if self.psw_fp_reserved_operand { 1 << 9 } else { 0 }) |
        (if self.psw_interrupt_disable { 1 << 12 } else { 0 }) |
        (if self.psw_address_trap_enable { 1 << 13 } else { 0 }) |
        (if self.psw_exception_pending { 1 << 14 } else { 0 }) |
        (if self.psw_nmi_pending { 1 << 15 } else { 0 }) |
        (self.psw_interrupt_mask_level as u32) << 16
    }

    pub fn set_reg_psw(&mut self, value: u32) {
        self.psw_zero = ((value >> 0) & 0x01) != 0;
        self.psw_sign = ((value >> 1) & 0x01) != 0;
        self.psw_overflow = ((value >> 2) & 0x01) != 0;
        self.psw_carry = ((value >> 3) & 0x01) != 0;
        self.psw_fp_precision_degredation = ((value >> 4) & 0x01) != 0;
        self.psw_fp_underflow = ((value >> 5) & 0x01) != 0;
        self.psw_fp_overflow = ((value >> 6) & 0x01) != 0;
        self.psw_fp_zero_division = ((value >> 7) & 0x01) != 0;
        self.psw_fp_invalid_operation = ((value >> 8) & 0x01) != 0;
        self.psw_fp_reserved_operand = ((value >> 9) & 0x01) != 0;
        self.psw_interrupt_disable = ((value >> 12) & 0x01) != 0;
        self.psw_address_trap_enable = ((value >> 13) & 0x01) != 0;
        self.psw_exception_pending = ((value >> 14) & 0x01) != 0;
        self.psw_nmi_pending = ((value >> 15) & 0x01) != 0;
        self.psw_interrupt_mask_level = ((value as usize) >> 16) & 0x0f;
    }

    pub fn step(&mut self, interconnect: &mut Interconnect, video_driver: &mut VideoDriver) -> (usize, bool) {
        let original_pc = self.reg_pc;

        let first_halfword = interconnect.read_halfword(original_pc);
        let mut next_pc = original_pc.wrapping_add(2);

        let opcode = Opcode::from_halfword(first_halfword);
        let instruction_format = opcode.instruction_format();

        let second_halfword = if instruction_format.has_second_halfword() {
            let second_halfword = interconnect.read_halfword(next_pc);
            next_pc = next_pc.wrapping_add(2);
            second_halfword
        } else {
            0
        };

        let mut trigger_watchpoint = false;

        // TODO: Not too convinced of this pattern, but we'll see what else we
        //  may have to special case moving forward
        let mut take_branch = false;

        match opcode {
            Opcode::MovReg => format_i(|reg1, reg2| {
                let value = self.reg_gpr(reg1);
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::AddReg => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                self.add(lhs, rhs, reg2);
            }, first_halfword),
            Opcode::Sub => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = self.sub_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::CmpReg => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                self.sub_and_set_flags(lhs, rhs);
            }, first_halfword),
            Opcode::ShlReg => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = self.shl_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::ShrReg => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = self.shr_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::Jmp => format_i(|reg1, _| {
                next_pc = self.reg_gpr(reg1);
            }, first_halfword),
            Opcode::SarReg => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = self.sar_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::Mul => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2) as i64;
                let rhs = self.reg_gpr(reg1) as i64;
                let res = (lhs * rhs) as u64;
                let res_low = res as u32;
                let res_high = (res >> 32) as u32;
                let overflow = res != (res_low as i32) as u64;
                self.set_reg_gpr(30, res_high);
                self.set_reg_gpr(reg2, res_low);
                self.set_zero_sign_flags(res_low);
                self.psw_overflow = overflow;
            }, first_halfword),
            Opcode::Div => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let (res, r30, overflow) = if lhs == 0x80000000 && rhs == 0xffffffff {
                    (lhs, 0, true)
                } else {
                    let lhs = lhs as i32;
                    let rhs = rhs as i32;
                    let res = (lhs / rhs) as u32;
                    let r30 = (lhs % rhs) as u32;
                    (res, r30, false)
                };
                self.set_reg_gpr(30, r30);
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = overflow;
            }, first_halfword),
            Opcode::MulU => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2) as u64;
                let rhs = self.reg_gpr(reg1) as u64;
                let res = lhs * rhs;
                let res_low = res as u32;
                let res_high = (res >> 32) as u32;
                let overflow = res != res_low as u64;
                self.set_reg_gpr(30, res_high);
                self.set_reg_gpr(reg2, res_low);
                self.set_zero_sign_flags(res_low);
                self.psw_overflow = overflow;
            }, first_halfword),
            Opcode::DivU => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = lhs / rhs;
                let r30 = lhs % rhs;
                self.set_reg_gpr(30, r30);
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword),
            Opcode::Or => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = lhs | rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword),
            Opcode::And => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = lhs & rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword),
            Opcode::Xor => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);
                let res = lhs ^ rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword),
            Opcode::Not => format_i(|reg1, reg2| {
                let res = !self.reg_gpr(reg1);
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword),
            Opcode::MovImm => format_ii(|imm5, reg2| {
                let value = sign_extend_imm5(imm5);
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::AddImm5 => format_ii(|imm5, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = sign_extend_imm5(imm5);
                self.add(lhs, rhs, reg2);
            }, first_halfword),
            Opcode::Setf => format_ii(|imm5, reg2| {
                let condition = opcode.condition(imm5);
                // TODO: See if we can unify with branch code
                let set = match condition {
                    Condition::V => self.psw_overflow,
                    Condition::C => self.psw_carry,
                    Condition::Z => self.psw_zero,
                    Condition::Nh => self.psw_carry || self.psw_zero,
                    Condition::N => self.psw_sign,
                    Condition::T => true,
                    Condition::Lt => self.psw_sign != self.psw_overflow,
                    Condition::Le => (self.psw_sign != self.psw_overflow) || self.psw_zero,
                    Condition::Nv => !self.psw_overflow,
                    Condition::Nc => !self.psw_carry,
                    Condition::Nz => !self.psw_zero,
                    Condition::H => !(self.psw_carry || self.psw_zero),
                    Condition::P => !self.psw_sign,
                    Condition::F => false,
                    Condition::Ge => !(self.psw_sign != self.psw_overflow),
                    Condition::Gt => !((self.psw_sign != self.psw_overflow) || self.psw_zero),
                };
                self.set_reg_gpr(reg2, if set { 1 } else { 0 });
            }, first_halfword),
            Opcode::CmpImm => format_ii(|imm5, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = sign_extend_imm5(imm5);
                self.sub_and_set_flags(lhs, rhs);
            }, first_halfword),
            Opcode::ShlImm => format_ii(|imm5, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = sign_extend_imm5(imm5);
                let res = self.shl_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::ShrImm => format_ii(|imm5, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = sign_extend_imm5(imm5);
                let res = self.shr_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::Cli => format_ii(|_, _| {
                self.psw_interrupt_disable = false;
            }, first_halfword),
            Opcode::SarImm => format_ii(|imm5, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = sign_extend_imm5(imm5);
                let res = self.sar_and_set_flags(lhs, rhs);
                self.set_reg_gpr(reg2, res);
            }, first_halfword),
            Opcode::Reti => format_ii(|_, _| {
                next_pc = self.return_from_exception();
            }, first_halfword),
            Opcode::Ldsr => format_ii(|imm5, reg2| {
                let value = self.reg_gpr(reg2);
                let system_register = opcode.system_register(imm5);
                match system_register {
                    SystemRegister::Eipc => {
                        self.reg_eipc = value;
                    }
                    SystemRegister::Eipsw => {
                        self.reg_eipsw = value;
                    }
                    SystemRegister::Fepc => {
                        println!("WARNING: ldsr fepc not yet implemented (value: 0x{:08x})", value);
                    }
                    SystemRegister::Fepsw => {
                        println!("WARNING: ldsr fepsw not yet implemented (value: 0x{:08x})", value);
                    }
                    SystemRegister::Ecr => {
                        println!("WARNING: Attempted ldsr ecr (value: 0x{:08x})", value);
                    }
                    SystemRegister::Psw => self.set_reg_psw(value),
                    SystemRegister::Chcw => {
                        println!("WARNING: ldsr chcw not yet implemented (value: 0x{:08x})", value);
                    }
                }
            }, first_halfword),
            Opcode::Stsr => format_ii(|imm5, reg2| {
                let system_register = opcode.system_register(imm5);
                let value = match system_register {
                    SystemRegister::Eipc => self.reg_eipc,
                    SystemRegister::Eipsw => self.reg_eipsw,
                    SystemRegister::Fepc => {
                        println!("WARNING: stsr fepc not yet implemented");
                        0
                    }
                    SystemRegister::Fepsw => {
                        println!("WARNING: stsr fepsw not yet implemented");
                        0
                    }
                    SystemRegister::Ecr => {
                        println!("WARNING: stsr ecr not yet implemented");
                        0
                    }
                    SystemRegister::Psw => self.reg_psw(),
                    SystemRegister::Chcw => {
                        println!("WARNING: stsr chcw not yet implemented");
                        0
                    }
                };
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::Sei => format_ii(|_, _| {
                self.psw_interrupt_disable = true;
            }, first_halfword),
            Opcode::Bv => {
                take_branch = self.psw_overflow;
            },
            Opcode::Bc => {
                take_branch = self.psw_carry;
            },
            Opcode::Bz => {
                take_branch = self.psw_zero;
            },
            Opcode::Bnh => {
                take_branch = self.psw_carry | self.psw_zero;
            },
            Opcode::Bn => {
                take_branch = self.psw_sign;
            },
            Opcode::Br => {
                take_branch = true;
            },
            Opcode::Blt => {
                take_branch = self.psw_sign != self.psw_overflow;
            },
            Opcode::Ble => {
                take_branch = (self.psw_sign != self.psw_overflow) || self.psw_zero;
            },
            Opcode::Bnv => {
                take_branch = !self.psw_overflow;
            },
            Opcode::Bnc => {
                take_branch = !self.psw_carry;
            },
            Opcode::Bnz => {
                take_branch = !self.psw_zero;
            },
            Opcode::Bh => {
                take_branch = !(self.psw_carry || self.psw_zero);
            },
            Opcode::Bp => {
                take_branch = !self.psw_sign;
            },
            Opcode::Nop => (),
            Opcode::Bge => {
                take_branch = !(self.psw_sign != self.psw_overflow);
            },
            Opcode::Bgt => {
                take_branch = !((self.psw_sign != self.psw_overflow) || self.psw_zero);
            },
            Opcode::Movea => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as i16) as u32);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::AddImm16 => format_v(|reg1, reg2, imm16| {
                let lhs = self.reg_gpr(reg1);
                let rhs = (imm16 as i16) as u32;
                self.add(lhs, rhs, reg2);
            }, first_halfword, second_halfword),
            Opcode::Jr => format_iv(|target| {
                next_pc = target;
            }, first_halfword, second_halfword, original_pc),
            Opcode::Jal => format_iv(|target| {
                self.set_reg_gpr(31, original_pc.wrapping_add(4));
                next_pc = target;
            }, first_halfword, second_halfword, original_pc),
            Opcode::OrI => format_v(|reg1, reg2, imm16| {
                let lhs = self.reg_gpr(reg1);
                let rhs = imm16 as u32;
                let res = lhs | rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword, second_halfword),
            Opcode::AndI => format_v(|reg1, reg2, imm16| {
                let lhs = self.reg_gpr(reg1);
                let rhs = imm16 as u32;
                let res = lhs & rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword, second_halfword),
            Opcode::XorI => format_v(|reg1, reg2, imm16| {
                let lhs = self.reg_gpr(reg1);
                let rhs = imm16 as u32;
                let res = lhs ^ rhs;
                self.set_reg_gpr(reg2, res);
                self.set_zero_sign_flags(res);
                self.psw_overflow = false;
            }, first_halfword, second_halfword),
            Opcode::Movhi => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as u32) << 16);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::Ldb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = (interconnect.read_byte(addr) as i8) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Ldh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = (interconnect.read_halfword(addr) as i16) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Ldw | Opcode::Inw => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = interconnect.read_word(addr);
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Stb | Opcode::Outb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = self.reg_gpr(reg2) as u8;
                interconnect.write_byte(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Sth | Opcode::Outh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = self.reg_gpr(reg2) as u16;
                interconnect.write_halfword(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Stw | Opcode::Outw => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = self.reg_gpr(reg2);
                interconnect.write_word(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Inb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = interconnect.read_byte(addr) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Inh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                trigger_watchpoint |= self.check_watchpoints(addr);
                let value = interconnect.read_halfword(addr) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Extended => {
                let reg1 = (first_halfword & 0x1f) as usize;
                let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                let subop_bits = second_halfword >> 10;

                let subop = opcode.subop(subop_bits as _);

                match subop {
                    SubOp::CmpfS => {
                        let lhs = self.reg_gpr_float(reg2);
                        let rhs = self.reg_gpr_float(reg1);
                        let value = lhs - rhs;

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value.is_sign_negative();
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::CvtWs => {
                        let value = (self.reg_gpr(reg1) as i32) as f32;
                        self.set_reg_gpr_float(reg2, value);

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value.is_sign_negative();
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::CvtSw => {
                        let value = (self.reg_gpr_float(reg1).round() as i32) as u32;
                        self.set_reg_gpr(reg2, value);

                        self.psw_overflow = false;
                        self.set_zero_sign_flags(value);
                    }
                    SubOp::AddfS => {
                        let lhs = self.reg_gpr_float(reg2);
                        let rhs = self.reg_gpr_float(reg1);
                        let value = lhs + rhs;
                        self.set_reg_gpr_float(reg2, value);

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value.is_sign_negative();
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::SubfS => {
                        let lhs = self.reg_gpr_float(reg2);
                        let rhs = self.reg_gpr_float(reg1);
                        let value = lhs - rhs;
                        self.set_reg_gpr_float(reg2, value);

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value.is_sign_negative();
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::MulfS => {
                        let lhs = self.reg_gpr_float(reg2);
                        let rhs = self.reg_gpr_float(reg1);
                        let value = lhs * rhs;
                        self.set_reg_gpr_float(reg2, value);

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value < 0.0;
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::DivfS => {
                        let lhs = self.reg_gpr_float(reg2);
                        let rhs = self.reg_gpr_float(reg1);
                        let value = lhs / rhs;
                        self.set_reg_gpr_float(reg2, value);

                        self.psw_carry = false;
                        self.psw_overflow = false;
                        self.psw_sign = value.is_sign_negative();
                        self.psw_zero = value == 0.0;
                    }
                    SubOp::Xb => {
                        let original = self.reg_gpr(reg2);
                        let value = (original & 0xffff0000) | ((original & 0x0000ff00) >> 8) | ((original & 0x000000ff) << 8);
                        self.set_reg_gpr(reg2, value);
                    }
                    SubOp::Xh => {
                        let original = self.reg_gpr(reg2);
                        let value = (original >> 16) | ((original & 0xffff) << 16);
                        self.set_reg_gpr(reg2, value);
                    }
                    SubOp::TrncSw => {
                        let value = (self.reg_gpr_float(reg1).trunc() as i32) as u32;
                        self.set_reg_gpr(reg2, value);

                        self.psw_overflow = false;
                        self.set_zero_sign_flags(value);
                    }
                    SubOp::Mpyhw => {
                        let lhs = (self.reg_gpr(reg2) as i16) as i32;
                        let rhs = (self.reg_gpr(reg1) as i16) as i32;
                        let value = (lhs * rhs) as u32;
                        self.set_reg_gpr(reg2, value);
                    }
                }
            }
        }

        if take_branch {
            let disp9 = first_halfword & 0x01ff;
            let disp = (disp9 as u32) | if disp9 & 0x0100 == 0 { 0x00000000 } else { 0xfffffe00 };
            next_pc = self.reg_pc.wrapping_add(disp);
        }

        self.reg_pc = next_pc;

        let num_cycles = match opcode.instruction_format() {
            InstructionFormat::VII => {
                let subop_bits = second_halfword >> 10;

                let subop = opcode.subop(subop_bits as _);

                subop.num_cycles()
            }
            _ => opcode.num_cycles(take_branch)
        };

        if let Some(exception_code) = interconnect.cycles(num_cycles, video_driver) {
            self.request_exception(exception_code);
        }

        (num_cycles, trigger_watchpoint)
    }

    fn check_watchpoints(&self, addr: u32) -> bool {
        self.watchpoints.contains(&addr)
    }

    fn add(&mut self, lhs: u32, rhs: u32, reg2: usize) {
        let (res, carry) = lhs.overflowing_add(rhs);
        self.set_reg_gpr(reg2, res);
        self.set_zero_sign_flags(res);
        self.psw_overflow = ((!(lhs ^ rhs) & (rhs ^ res)) & 0x80000000) != 0;
        self.psw_carry = carry;
    }

    fn sub_and_set_flags(&mut self, lhs: u32, rhs: u32) -> u32 {
        let (res, carry) = lhs.overflowing_sub(rhs);
        self.set_zero_sign_flags(res);
        self.psw_overflow = (((lhs ^ rhs) & !(rhs ^ res)) & 0x80000000) != 0;
        self.psw_carry = carry;
        res
    }

    fn shl_and_set_flags(&mut self, lhs: u32, rhs: u32) -> u32 {
        let mut res = lhs;
        let mut carry = false;
        let shift = (rhs as usize) & 0x1f;
        for _ in 0..shift {
            carry = res & 0x80000000 != 0;
            res = res.wrapping_shl(1);
        }
        self.set_zero_sign_flags(res);
        self.psw_overflow = false;
        self.psw_carry = carry;
        res
    }

    fn shr_and_set_flags(&mut self, lhs: u32, rhs: u32) -> u32 {
        let mut res = lhs;
        let mut carry = false;
        let shift = (rhs as usize) & 0x1f;
        for _ in 0..shift {
            carry = res & 0x00000001 != 0;
            res = res.wrapping_shr(1);
        }
        self.set_zero_sign_flags(res);
        self.psw_overflow = false;
        self.psw_carry = carry;
        res
    }

    fn sar_and_set_flags(&mut self, lhs: u32, rhs: u32) -> u32 {
        let mut res = lhs;
        let mut carry = false;
        let shift = (rhs as usize) & 0x1f;
        for _ in 0..shift {
            let sign = res & 0x80000000;
            carry = res & 0x00000001 != 0;
            res = sign | res.wrapping_shr(1);
        }
        self.set_zero_sign_flags(res);
        self.psw_overflow = false;
        self.psw_carry = carry;
        res
    }

    fn set_zero_sign_flags(&mut self, value: u32) {
        self.psw_zero = value == 0;
        self.psw_sign = value & 0x80000000 != 0;
    }

    fn request_exception(&mut self, exception_code: u16) {
        if self.psw_nmi_pending || self.psw_exception_pending || self.psw_interrupt_disable {
            return;
        }

        println!("Entering exception (code: 0x{:04x})", exception_code);
        self.reg_eipc = self.reg_pc;
        self.reg_eipsw = self.reg_psw();
        self.reg_ecr = exception_code;
        self.psw_exception_pending = true;
        self.reg_pc = 0xffff0000 | (exception_code as u32);
    }

    fn return_from_exception(&mut self) -> u32 {
        println!("Returning from exception (code: 0x{:04x})", self.reg_ecr);
        let psw = self.reg_eipsw;
        self.set_reg_psw(psw);
        self.reg_eipc
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

fn format_iv<F: FnOnce(u32)>(f: F, first_halfword: u16, second_halfword: u16, reg_pc: u32) {
    let disp26 = (((first_halfword as u32) & 0x03ff) << 16) | (second_halfword as u32);
    let disp = disp26 | if disp26 & 0x02000000 == 0 { 0x00000000 } else { 0xfc000000 };
    let target = reg_pc.wrapping_add(disp);
    f(target);
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
    (imm5 as u32) | (if imm5 & 0x10 == 0 { 0x00000000 } else { 0xffffffe0 })
}