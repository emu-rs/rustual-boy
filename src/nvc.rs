use video_driver::*;
use instruction::*;
use interconnect::*;

use std::collections::HashSet;

pub struct Nvc {
    reg_pc: u32,

    _reg_gpr: Box<[u32; 32]>,
    reg_gpr_ptr: *mut u32,

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
        let mut reg_gpr = Box::new([0xdeadbeef; 32]);
        reg_gpr[0] = 0;
        let reg_gpr_ptr = reg_gpr.as_mut_ptr();

        Nvc {
            reg_pc: 0xfffffff0,

            _reg_gpr: reg_gpr,
            reg_gpr_ptr: reg_gpr_ptr,

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
        unsafe {
            let reg_ptr = self.reg_gpr_ptr.offset(index as _);
            *reg_ptr
        }
    }

    fn set_reg_gpr(&mut self, index: usize, value: u32) {
        if index != 0 {
            unsafe {
                let reg_ptr = self.reg_gpr_ptr.offset(index as _);
                *reg_ptr = value;
            }
        }
    }

    // TODO: Come up with a more portable way to do this conversion
    fn reg_gpr_float(&self, index: usize) -> f32 {
        unsafe {
            let reg_ptr = self.reg_gpr_ptr.offset(index as _);
            let reg_float_ptr = reg_ptr as *const f32;
            *reg_float_ptr
        }
    }

    fn set_reg_gpr_float(&mut self, index: usize, value: f32) {
        if index != 0 {
            unsafe {
                let reg_ptr = self.reg_gpr_ptr.offset(index as _);
                let reg_float_ptr = reg_ptr as *mut f32;
                *reg_float_ptr = value;
            }
        }
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

        let mut num_cycles = 1;
        let mut trigger_watchpoint = false;

        if first_halfword >> 13 == OPCODE_BITS_BCOND_PREFIX {
            let cond_bits = (first_halfword >> 9) & 0x0f;
            let take_branch = match cond_bits {
                OPCODE_BITS_BCOND_BV => self.psw_overflow,
                OPCODE_BITS_BCOND_BC => self.psw_carry,
                OPCODE_BITS_BCOND_BZ => self.psw_zero,
                OPCODE_BITS_BCOND_BNH => self.psw_carry | self.psw_zero,
                OPCODE_BITS_BCOND_BN => self.psw_sign,
                OPCODE_BITS_BCOND_BR => true,
                OPCODE_BITS_BCOND_BLT => self.psw_sign != self.psw_overflow,
                OPCODE_BITS_BCOND_BLE => (self.psw_sign != self.psw_overflow) || self.psw_zero,
                OPCODE_BITS_BCOND_BNV => !self.psw_overflow,
                OPCODE_BITS_BCOND_BNC => !self.psw_carry,
                OPCODE_BITS_BCOND_BNZ => !self.psw_zero,
                OPCODE_BITS_BCOND_BH => !(self.psw_carry || self.psw_zero),
                OPCODE_BITS_BCOND_BP => !self.psw_sign,
                OPCODE_BITS_BCOND_NOP => false,
                OPCODE_BITS_BCOND_BGE => !(self.psw_sign != self.psw_overflow),
                OPCODE_BITS_BCOND_BGT => !((self.psw_sign != self.psw_overflow) || self.psw_zero),
                _ => panic!("Unrecognized cond bits: {:04b} (halfword: 0b{:016b})", cond_bits, first_halfword)
            };

            if take_branch {
                let disp9 = first_halfword & 0x01ff;
                let disp = (disp9 as u32) | if disp9 & 0x0100 == 0 { 0x00000000 } else { 0xfffffe00 };
                next_pc = self.reg_pc.wrapping_add(disp);
                num_cycles = 3;
            }
        } else {
            macro_rules! format_i {
                ($f:expr) => ({
                    let reg1 = (first_halfword & 0x1f) as usize;
                    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                    $f(reg1, reg2);
                });
            }

            macro_rules! format_ii {
                ($f:expr) => ({
                    let imm5 = (first_halfword & 0x1f) as usize;
                    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
                    $f(imm5, reg2);
                })
            }

            macro_rules! format_iv {
                ($f:expr) => ({
                    let second_halfword = interconnect.read_halfword(next_pc);
                    next_pc = next_pc.wrapping_add(2);

                    let disp26 = (((first_halfword as u32) & 0x03ff) << 16) | (second_halfword as u32);
                    let disp = disp26 | if (disp26 & 0x02000000) == 0 { 0x00000000 } else { 0xfc000000 };
                    let target = self.reg_pc.wrapping_add(disp);
                    $f(target);
                })
            }

            macro_rules! format_v {
                ($f:expr) => ({
                    let second_halfword = interconnect.read_halfword(next_pc);
                    next_pc = next_pc.wrapping_add(2);

                    let reg1 = (first_halfword & 0x1f) as usize;
                    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
                    let imm16 = second_halfword;
                    $f(reg1, reg2, imm16);
                })
            }

            macro_rules! format_vi {
                ($f:expr) => ({
                    let second_halfword = interconnect.read_halfword(next_pc);
                    next_pc = next_pc.wrapping_add(2);

                    let reg1 = (first_halfword & 0x1f) as usize;
                    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
                    let disp16 = second_halfword as i16;
                    $f(reg1, reg2, disp16);
                })
            }

            let opcode_bits = first_halfword >> 10;
            match opcode_bits {
                OPCODE_BITS_MOV_REG => format_i!(|reg1, reg2| {
                    let value = self.reg_gpr(reg1);
                    self.set_reg_gpr(reg2, value);
                }),
                OPCODE_BITS_ADD_REG => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    self.add(lhs, rhs, reg2);
                }),
                OPCODE_BITS_SUB => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = self.sub_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_CMP_REG => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    self.sub_and_set_flags(lhs, rhs);
                }),
                OPCODE_BITS_SHL_REG => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = self.shl_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_SHR_REG => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = self.shr_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_JMP => format_i!(|reg1, _| {
                    next_pc = self.reg_gpr(reg1);
                    num_cycles = 3;
                }),
                OPCODE_BITS_SAR_REG => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = self.sar_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_MUL => format_i!(|reg1, reg2| {
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
                    num_cycles = 13;
                }),
                OPCODE_BITS_DIV => format_i!(|reg1, reg2| {
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
                    num_cycles = 38;
                }),
                OPCODE_BITS_MUL_U => format_i!(|reg1, reg2| {
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
                    num_cycles = 13;
                }),
                OPCODE_BITS_DIV_U => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = lhs / rhs;
                    let r30 = lhs % rhs;
                    self.set_reg_gpr(30, r30);
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                    num_cycles = 36;
                }),
                OPCODE_BITS_OR => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = lhs | rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_AND => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = lhs & rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_XOR => format_i!(|reg1, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = self.reg_gpr(reg1);
                    let res = lhs ^ rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_NOT => format_i!(|reg1, reg2| {
                    let res = !self.reg_gpr(reg1);
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_MOV_IMM => format_ii!(|imm5, reg2| {
                    let value = sign_extend_imm5(imm5);
                    self.set_reg_gpr(reg2, value);
                }),
                OPCODE_BITS_ADD_IMM_5 => format_ii!(|imm5, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = sign_extend_imm5(imm5);
                    self.add(lhs, rhs, reg2);
                }),
                OPCODE_BITS_SETF => format_ii!(|imm5, reg2| {
                    let set = match imm5 {
                        OPCODE_CONDITION_BITS_V => self.psw_overflow,
                        OPCODE_CONDITION_BITS_C => self.psw_carry,
                        OPCODE_CONDITION_BITS_Z => self.psw_zero,
                        OPCODE_CONDITION_BITS_NH => self.psw_carry || self.psw_zero,
                        OPCODE_CONDITION_BITS_N => self.psw_sign,
                        OPCODE_CONDITION_BITS_T => true,
                        OPCODE_CONDITION_BITS_LT => self.psw_sign != self.psw_overflow,
                        OPCODE_CONDITION_BITS_LE => (self.psw_sign != self.psw_overflow) || self.psw_zero,
                        OPCODE_CONDITION_BITS_NV => !self.psw_overflow,
                        OPCODE_CONDITION_BITS_NC => !self.psw_carry,
                        OPCODE_CONDITION_BITS_NZ => !self.psw_zero,
                        OPCODE_CONDITION_BITS_H => !(self.psw_carry || self.psw_zero),
                        OPCODE_CONDITION_BITS_P => !self.psw_sign,
                        OPCODE_CONDITION_BITS_F => false,
                        OPCODE_CONDITION_BITS_GE => !(self.psw_sign != self.psw_overflow),
                        OPCODE_CONDITION_BITS_GT => !((self.psw_sign != self.psw_overflow) || self.psw_zero),
                        _ => panic!("Unrecognized condition: {}", imm5),
                    };
                    self.set_reg_gpr(reg2, if set { 1 } else { 0 });
                }),
                OPCODE_BITS_CMP_IMM => format_ii!(|imm5, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = sign_extend_imm5(imm5);
                    self.sub_and_set_flags(lhs, rhs);
                }),
                OPCODE_BITS_SHL_IMM => format_ii!(|imm5, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = imm5 as u32;
                    let res = self.shl_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_SHR_IMM => format_ii!(|imm5, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = sign_extend_imm5(imm5);
                    let res = self.shr_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_CLI => format_ii!(|_, _| {
                    self.psw_interrupt_disable = false;

                    num_cycles = 12;
                }),
                OPCODE_BITS_SAR_IMM => format_ii!(|imm5, reg2| {
                    let lhs = self.reg_gpr(reg2);
                    let rhs = imm5 as u32;
                    let res = self.sar_and_set_flags(lhs, rhs);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_RETI => format_ii!(|_, _| {
                    next_pc = self.return_from_exception();
                    num_cycles = 10;
                }),
                OPCODE_BITS_HALT => format_ii!(|_, _| {
                    next_pc = original_pc;
                }),
                OPCODE_BITS_LDSR => format_ii!(|imm5, reg2| {
                    let value = self.reg_gpr(reg2);
                    match imm5 {
                        OPCODE_SYSTEM_REGISTER_ID_EIPC => {
                            self.reg_eipc = value;
                        }
                        OPCODE_SYSTEM_REGISTER_ID_EIPSW => {
                            self.reg_eipsw = value;
                        }
                        OPCODE_SYSTEM_REGISTER_ID_FEPC => {
                            logln!("WARNING: ldsr fepc not yet implemented (value: 0x{:08x})", value);
                        }
                        OPCODE_SYSTEM_REGISTER_ID_FEPSW => {
                            logln!("WARNING: ldsr fepsw not yet implemented (value: 0x{:08x})", value);
                        }
                        OPCODE_SYSTEM_REGISTER_ID_ECR => {
                            logln!("WARNING: Attempted ldsr ecr (value: 0x{:08x})", value);
                        }
                        OPCODE_SYSTEM_REGISTER_ID_PSW => self.set_reg_psw(value),
                        OPCODE_SYSTEM_REGISTER_ID_CHCW => {
                            logln!("WARNING: ldsr chcw not yet implemented (value: 0x{:08x})", value);
                        }
                        _ => logln!("WARNING: Unrecognized system register: {}", imm5),
                    }
                }),
                OPCODE_BITS_STSR => format_ii!(|imm5, reg2| {
                    let value = match imm5 {
                        OPCODE_SYSTEM_REGISTER_ID_EIPC => self.reg_eipc,
                        OPCODE_SYSTEM_REGISTER_ID_EIPSW => self.reg_eipsw,
                        OPCODE_SYSTEM_REGISTER_ID_FEPC => {
                            logln!("WARNING: stsr fepc not yet implemented");
                            0
                        }
                        OPCODE_SYSTEM_REGISTER_ID_FEPSW => {
                            logln!("WARNING: stsr fepsw not yet implemented");
                            0
                        }
                        OPCODE_SYSTEM_REGISTER_ID_ECR => {
                            logln!("WARNING: stsr ecr not yet implemented");
                            0
                        }
                        OPCODE_SYSTEM_REGISTER_ID_PSW => self.reg_psw(),
                        OPCODE_SYSTEM_REGISTER_ID_CHCW => {
                            logln!("WARNING: stsr chcw not yet implemented");
                            0
                        }
                        _ => {
                            logln!("WARNING: Unrecognized system register: {}", imm5);
                            0
                        }
                    };
                    self.set_reg_gpr(reg2, value);
                }),
                OPCODE_BITS_SEI => format_ii!(|_, _| {
                    self.psw_interrupt_disable = true;

                    num_cycles = 12;
                }),
                OPCODE_BITS_MOVEA => format_v!(|reg1, reg2, imm16| {
                    let res = self.reg_gpr(reg1).wrapping_add((imm16 as i16) as u32);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_ADD_IMM_16 => format_v!(|reg1, reg2, imm16| {
                    let lhs = self.reg_gpr(reg1);
                    let rhs = (imm16 as i16) as u32;
                    self.add(lhs, rhs, reg2);
                }),
                OPCODE_BITS_JR => format_iv!(|target| {
                    next_pc = target;
                    num_cycles = 3;
                }),
                OPCODE_BITS_JAL => format_iv!(|target| {
                    self.set_reg_gpr(31, original_pc.wrapping_add(4));
                    next_pc = target;
                    num_cycles = 3;
                }),
                OPCODE_BITS_OR_I => format_v!(|reg1, reg2, imm16| {
                    let lhs = self.reg_gpr(reg1);
                    let rhs = imm16 as u32;
                    let res = lhs | rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_AND_I => format_v!(|reg1, reg2, imm16| {
                    let lhs = self.reg_gpr(reg1);
                    let rhs = imm16 as u32;
                    let res = lhs & rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_XOR_I => format_v!(|reg1, reg2, imm16| {
                    let lhs = self.reg_gpr(reg1);
                    let rhs = imm16 as u32;
                    let res = lhs ^ rhs;
                    self.set_reg_gpr(reg2, res);
                    self.set_zero_sign_flags(res);
                    self.psw_overflow = false;
                }),
                OPCODE_BITS_MOVHI => format_v!(|reg1, reg2, imm16| {
                    let res = self.reg_gpr(reg1).wrapping_add((imm16 as u32) << 16);
                    self.set_reg_gpr(reg2, res);
                }),
                OPCODE_BITS_LDB => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = (interconnect.read_byte(addr) as i8) as u32;
                    self.set_reg_gpr(reg2, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_LDH => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = (interconnect.read_halfword(addr) as i16) as u32;
                    self.set_reg_gpr(reg2, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_LDW | OPCODE_BITS_INW => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = (interconnect.read_halfword(addr) as u32) | ((interconnect.read_halfword(addr + 2) as u32) << 16);
                    self.set_reg_gpr(reg2, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_STB | OPCODE_BITS_OUTB => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = self.reg_gpr(reg2) as u8;
                    interconnect.write_byte(addr, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_STH | OPCODE_BITS_OUTH => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = self.reg_gpr(reg2) as u16;
                    interconnect.write_halfword(addr, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_STW | OPCODE_BITS_OUTW => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = self.reg_gpr(reg2);
                    interconnect.write_halfword(addr, value as _);
                    interconnect.write_halfword(addr + 2, (value >> 16) as _);
                    num_cycles = 4;
                }),
                OPCODE_BITS_INB => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = interconnect.read_byte(addr) as u32;
                    self.set_reg_gpr(reg2, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_INH => format_vi!(|reg1, reg2, disp16| {
                    let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                    trigger_watchpoint |= self.check_watchpoints(addr);
                    let value = interconnect.read_halfword(addr) as u32;
                    self.set_reg_gpr(reg2, value);
                    num_cycles = 4;
                }),
                OPCODE_BITS_EXTENDED => {
                    let second_halfword = interconnect.read_halfword(next_pc);
                    next_pc = next_pc.wrapping_add(2);

                    let reg1 = (first_halfword & 0x1f) as usize;
                    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;

                    let subop_bits = second_halfword >> 10;

                    match subop_bits {
                        OPCODE_BITS_SUB_OP_CMPF_S => {
                            let lhs = self.reg_gpr_float(reg2);
                            let rhs = self.reg_gpr_float(reg1);
                            let value = lhs - rhs;

                            self.set_fp_flags(value);

                            num_cycles = 10;
                        }
                        OPCODE_BITS_SUB_OP_CVT_WS => {
                            let value = (self.reg_gpr(reg1) as i32) as f32;
                            self.set_reg_gpr_float(reg2, value);

                            self.set_fp_flags(value);

                            num_cycles = 16;
                        }
                        OPCODE_BITS_SUB_OP_CVT_SW => {
                            let value = (self.reg_gpr_float(reg1).round() as i32) as u32;
                            self.set_reg_gpr(reg2, value);

                            self.psw_overflow = false;
                            self.set_zero_sign_flags(value);

                            num_cycles = 14;
                        }
                        OPCODE_BITS_SUB_OP_ADDF_S => {
                            let lhs = self.reg_gpr_float(reg2);
                            let rhs = self.reg_gpr_float(reg1);
                            let value = lhs + rhs;
                            self.set_reg_gpr_float(reg2, value);

                            self.set_fp_flags(value);

                            num_cycles = 28;
                        }
                        OPCODE_BITS_SUB_OP_SUBF_S => {
                            let lhs = self.reg_gpr_float(reg2);
                            let rhs = self.reg_gpr_float(reg1);
                            let value = lhs - rhs;
                            self.set_reg_gpr_float(reg2, value);

                            self.set_fp_flags(value);

                            num_cycles = 28;
                        }
                        OPCODE_BITS_SUB_OP_MULF_S => {
                            let lhs = self.reg_gpr_float(reg2);
                            let rhs = self.reg_gpr_float(reg1);
                            let value = lhs * rhs;
                            self.set_reg_gpr_float(reg2, value);

                            self.set_fp_flags(value);

                            num_cycles = 30;
                        }
                        OPCODE_BITS_SUB_OP_DIVF_S => {
                            let lhs = self.reg_gpr_float(reg2);
                            let rhs = self.reg_gpr_float(reg1);
                            let value = lhs / rhs;
                            self.set_reg_gpr_float(reg2, value);

                            self.set_fp_flags(value);

                            num_cycles = 44;
                        }
                        OPCODE_BITS_SUB_OP_XB => {
                            let original = self.reg_gpr(reg2);
                            let value = (original & 0xffff0000) | ((original & 0x0000ff00) >> 8) | ((original & 0x000000ff) << 8);
                            self.set_reg_gpr(reg2, value);
                        }
                        OPCODE_BITS_SUB_OP_XH => {
                            let original = self.reg_gpr(reg2);
                            let value = (original >> 16) | ((original & 0xffff) << 16);
                            self.set_reg_gpr(reg2, value);
                        }
                        OPCODE_BITS_SUB_OP_TRNC_SW => {
                            let value = (self.reg_gpr_float(reg1).trunc() as i32) as u32;
                            self.set_reg_gpr(reg2, value);

                            self.psw_overflow = false;
                            self.set_zero_sign_flags(value);

                            num_cycles = 14;
                        }
                        OPCODE_BITS_SUB_OP_MPYHW => {
                            let lhs = self.reg_gpr(reg2) as i32;
                            let rhs = ((self.reg_gpr(reg1) as i32) << 15) >> 15;
                            let value = (lhs * rhs) as u32;
                            self.set_reg_gpr(reg2, value);

                            num_cycles = 9;
                        }
                        _ => panic!("Unrecognized subop bits: {:06b}", subop_bits)
                    }
                }
                _ => panic!("Unrecognized opcode bits: {:06b} (halfword: 0b{:016b})", opcode_bits, first_halfword),
            }
        }

        self.reg_pc = next_pc;

        if let Some(exception_code) = interconnect.cycles(num_cycles, video_driver) {
            self.request_exception(exception_code);
        }

        (num_cycles, trigger_watchpoint)
    }

    fn check_watchpoints(&self, addr: u32) -> bool {
        self.watchpoints.len() != 0 && self.watchpoints.contains(&addr)
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
            carry = (res & 0x80000000) != 0;
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
            carry = (res & 0x00000001) != 0;
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
            carry = (res & 0x00000001) != 0;
            res = sign | res.wrapping_shr(1);
        }
        self.set_zero_sign_flags(res);
        self.psw_overflow = false;
        self.psw_carry = carry;
        res
    }

    fn set_zero_sign_flags(&mut self, value: u32) {
        self.psw_zero = value == 0;
        self.psw_sign = (value & 0x80000000) != 0;
    }

    fn set_fp_flags(&mut self, value: f32) {
        self.psw_carry = value.is_sign_negative();
        self.psw_overflow = false;
        self.psw_sign = self.psw_carry;
        self.psw_zero = value == 0.0;
    }

    fn request_exception(&mut self, exception_code: u16) {
        if self.psw_nmi_pending || self.psw_exception_pending || self.psw_interrupt_disable {
            return;
        }

        logln!("Entering exception (code: 0x{:04x})", exception_code);
        self.reg_eipc = self.reg_pc;
        self.reg_eipsw = self.reg_psw();
        self.reg_ecr = exception_code;
        self.psw_exception_pending = true;
        self.reg_pc = 0xffff0000 | (exception_code as u32);
    }

    fn return_from_exception(&mut self) -> u32 {
        logln!("Returning from exception (code: 0x{:04x})", self.reg_ecr);
        let psw = self.reg_eipsw;
        self.set_reg_psw(psw);
        self.reg_eipc
    }
}

fn sign_extend_imm5(imm5: usize) -> u32 {
    (imm5 as u32) | (if (imm5 & 0x10) == 0 { 0x00000000 } else { 0xffffffe0 })
}