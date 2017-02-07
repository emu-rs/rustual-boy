use std::fmt;

pub const OPCODE_BITS_BCOND_PREFIX: u16 = 0b100;

pub const OPCODE_BITS_BCOND_BV: u16 = 0b0000;
pub const OPCODE_BITS_BCOND_BC: u16 = 0b0001;
pub const OPCODE_BITS_BCOND_BZ: u16 = 0b0010;
pub const OPCODE_BITS_BCOND_BNH: u16 = 0b0011;
pub const OPCODE_BITS_BCOND_BN: u16 = 0b0100;
pub const OPCODE_BITS_BCOND_BR: u16 = 0b0101;
pub const OPCODE_BITS_BCOND_BLT: u16 = 0b0110;
pub const OPCODE_BITS_BCOND_BLE: u16 = 0b0111;
pub const OPCODE_BITS_BCOND_BNV: u16 = 0b1000;
pub const OPCODE_BITS_BCOND_BNC: u16 = 0b1001;
pub const OPCODE_BITS_BCOND_BNZ: u16 = 0b1010;
pub const OPCODE_BITS_BCOND_BH: u16 = 0b1011;
pub const OPCODE_BITS_BCOND_BP: u16 = 0b1100;
pub const OPCODE_BITS_BCOND_NOP: u16 = 0b1101;
pub const OPCODE_BITS_BCOND_BGE: u16 = 0b1110;
pub const OPCODE_BITS_BCOND_BGT: u16 = 0b1111;

pub const OPCODE_BITS_MOV_REG: u16 = 0b000000;
pub const OPCODE_BITS_ADD_REG: u16 = 0b000001;
pub const OPCODE_BITS_SUB: u16 = 0b000010;
pub const OPCODE_BITS_CMP_REG: u16 = 0b000011;
pub const OPCODE_BITS_SHL_REG: u16 = 0b000100;
pub const OPCODE_BITS_SHR_REG: u16 = 0b000101;
pub const OPCODE_BITS_JMP: u16 = 0b000110;
pub const OPCODE_BITS_SAR_REG: u16 = 0b000111;
pub const OPCODE_BITS_MUL: u16 = 0b001000;
pub const OPCODE_BITS_DIV: u16 = 0b001001;
pub const OPCODE_BITS_MUL_U: u16 = 0b001010;
pub const OPCODE_BITS_DIV_U: u16 = 0b001011;
pub const OPCODE_BITS_OR: u16 = 0b001100;
pub const OPCODE_BITS_AND: u16 = 0b001101;
pub const OPCODE_BITS_XOR: u16 = 0b001110;
pub const OPCODE_BITS_NOT: u16 = 0b001111;
pub const OPCODE_BITS_MOV_IMM: u16 = 0b010000;
pub const OPCODE_BITS_ADD_IMM_5: u16 = 0b010001;
pub const OPCODE_BITS_SETF: u16 = 0b010010;
pub const OPCODE_BITS_CMP_IMM: u16 = 0b010011;
pub const OPCODE_BITS_SHL_IMM: u16 = 0b010100;
pub const OPCODE_BITS_SHR_IMM: u16 = 0b010101;
pub const OPCODE_BITS_CLI: u16 = 0b010110;
pub const OPCODE_BITS_SAR_IMM: u16 = 0b010111;
pub const OPCODE_BITS_RETI: u16 = 0b011001;
pub const OPCODE_BITS_HALT: u16 = 0b011010;
pub const OPCODE_BITS_LDSR: u16 = 0b011100;
pub const OPCODE_BITS_STSR: u16 = 0b011101;
pub const OPCODE_BITS_SEI: u16 = 0b011110;
pub const OPCODE_BITS_MOVEA: u16 = 0b101000;
pub const OPCODE_BITS_ADD_IMM_16: u16 = 0b101001;
pub const OPCODE_BITS_JR: u16 = 0b101010;
pub const OPCODE_BITS_JAL: u16 = 0b101011;
pub const OPCODE_BITS_OR_I: u16 = 0b101100;
pub const OPCODE_BITS_AND_I: u16 = 0b101101;
pub const OPCODE_BITS_XOR_I: u16 = 0b101110;
pub const OPCODE_BITS_MOVHI: u16 = 0b101111;
pub const OPCODE_BITS_LDB: u16 = 0b110000;
pub const OPCODE_BITS_LDH: u16 = 0b110001;
pub const OPCODE_BITS_LDW: u16 = 0b110011;
pub const OPCODE_BITS_STB: u16 = 0b110100;
pub const OPCODE_BITS_STH: u16 = 0b110101;
pub const OPCODE_BITS_STW: u16 = 0b110111;
pub const OPCODE_BITS_INB: u16 = 0b111000;
pub const OPCODE_BITS_INH: u16 = 0b111001;
pub const OPCODE_BITS_INW: u16 = 0b111011;
pub const OPCODE_BITS_OUTB: u16 = 0b111100;
pub const OPCODE_BITS_OUTH: u16 = 0b111101;
pub const OPCODE_BITS_EXTENDED: u16 = 0b111110;
pub const OPCODE_BITS_OUTW: u16 = 0b111111;

pub const OPCODE_BITS_SUB_OP_CMPF_S: u16 = 0b000000;
pub const OPCODE_BITS_SUB_OP_CVT_WS: u16 = 0b000010;
pub const OPCODE_BITS_SUB_OP_CVT_SW: u16 = 0b000011;
pub const OPCODE_BITS_SUB_OP_ADDF_S: u16 = 0b000100;
pub const OPCODE_BITS_SUB_OP_SUBF_S: u16 = 0b000101;
pub const OPCODE_BITS_SUB_OP_MULF_S: u16 = 0b000110;
pub const OPCODE_BITS_SUB_OP_DIVF_S: u16 = 0b000111;
pub const OPCODE_BITS_SUB_OP_XB: u16 = 0b001000;
pub const OPCODE_BITS_SUB_OP_XH: u16 = 0b001001;
pub const OPCODE_BITS_SUB_OP_REV: u16 = 0b001010;
pub const OPCODE_BITS_SUB_OP_TRNC_SW: u16 = 0b001011;
pub const OPCODE_BITS_SUB_OP_MPYHW: u16 = 0b001100;

pub const OPCODE_SYSTEM_REGISTER_ID_EIPC: usize = 0;
pub const OPCODE_SYSTEM_REGISTER_ID_EIPSW: usize = 1;
pub const OPCODE_SYSTEM_REGISTER_ID_FEPC: usize = 2;
pub const OPCODE_SYSTEM_REGISTER_ID_FEPSW: usize = 3;
pub const OPCODE_SYSTEM_REGISTER_ID_ECR: usize = 4;
pub const OPCODE_SYSTEM_REGISTER_ID_PSW: usize = 5;
pub const OPCODE_SYSTEM_REGISTER_ID_CHCW: usize = 24;

pub const OPCODE_CONDITION_BITS_V: usize = 0x00;
pub const OPCODE_CONDITION_BITS_C: usize = 0x01;
pub const OPCODE_CONDITION_BITS_Z: usize = 0x02;
pub const OPCODE_CONDITION_BITS_NH: usize = 0x03;
pub const OPCODE_CONDITION_BITS_N: usize = 0x04;
pub const OPCODE_CONDITION_BITS_T: usize = 0x05;
pub const OPCODE_CONDITION_BITS_LT: usize = 0x06;
pub const OPCODE_CONDITION_BITS_LE: usize = 0x07;
pub const OPCODE_CONDITION_BITS_NV: usize = 0x08;
pub const OPCODE_CONDITION_BITS_NC: usize = 0x09;
pub const OPCODE_CONDITION_BITS_NZ: usize = 0x0a;
pub const OPCODE_CONDITION_BITS_H: usize = 0x0b;
pub const OPCODE_CONDITION_BITS_P: usize = 0x0c;
pub const OPCODE_CONDITION_BITS_F: usize = 0x0d;
pub const OPCODE_CONDITION_BITS_GE: usize = 0x0e;
pub const OPCODE_CONDITION_BITS_GT: usize = 0x0f;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    MovReg,
    AddReg,
    Sub,
    CmpReg,
    ShlReg,
    ShrReg,
    Jmp,
    SarReg,
    Mul,
    Div,
    MulU,
    DivU,
    Or,
    And,
    Xor,
    Not,
    MovImm,
    AddImm5,
    Setf,
    CmpImm,
    ShlImm,
    ShrImm,
    Cli,
    SarImm,
    Reti,
    Halt,
    Ldsr,
    Stsr,
    Sei,
    Bv,
    Bc,
    Bz,
    Bnh,
    Bn,
    Br,
    Blt,
    Ble,
    Bnv,
    Bnc,
    Bnz,
    Bh,
    Bp,
    Nop,
    Bge,
    Bgt,
    Movea,
    AddImm16,
    Jr,
    Jal,
    OrI,
    AndI,
    XorI,
    Movhi,
    Ldb,
    Ldh,
    Ldw,
    Stb,
    Sth,
    Stw,
    Inb,
    Inh,
    Inw,
    Outb,
    Outh,
    Extended,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        if halfword >> 13 == OPCODE_BITS_BCOND_PREFIX {
            let cond_bits = (halfword >> 9) & 0x0f;
            match cond_bits {
                OPCODE_BITS_BCOND_BV => Opcode::Bv,
                OPCODE_BITS_BCOND_BC => Opcode::Bc,
                OPCODE_BITS_BCOND_BZ => Opcode::Bz,
                OPCODE_BITS_BCOND_BNH => Opcode::Bnh,
                OPCODE_BITS_BCOND_BN => Opcode::Bn,
                OPCODE_BITS_BCOND_BR => Opcode::Br,
                OPCODE_BITS_BCOND_BLT => Opcode::Blt,
                OPCODE_BITS_BCOND_BLE => Opcode::Ble,
                OPCODE_BITS_BCOND_BNV => Opcode::Bnv,
                OPCODE_BITS_BCOND_BNC => Opcode::Bnc,
                OPCODE_BITS_BCOND_BNZ => Opcode::Bnz,
                OPCODE_BITS_BCOND_BH => Opcode::Bh,
                OPCODE_BITS_BCOND_BP => Opcode::Bp,
                OPCODE_BITS_BCOND_NOP => Opcode::Nop,
                OPCODE_BITS_BCOND_BGE => Opcode::Bge,
                OPCODE_BITS_BCOND_BGT => Opcode::Bgt,
                _ => panic!("Unrecognized cond bits: {:04b} (halfword: 0b{:016b})", cond_bits, halfword)
            }
        } else {
            let opcode_bits = halfword >> 10;
            match opcode_bits {
                OPCODE_BITS_MOV_REG => Opcode::MovReg,
                OPCODE_BITS_ADD_REG => Opcode::AddReg,
                OPCODE_BITS_SUB => Opcode::Sub,
                OPCODE_BITS_CMP_REG => Opcode::CmpReg,
                OPCODE_BITS_SHL_REG => Opcode::ShlReg,
                OPCODE_BITS_SHR_REG => Opcode::ShrReg,
                OPCODE_BITS_JMP => Opcode::Jmp,
                OPCODE_BITS_SAR_REG => Opcode::SarReg,
                OPCODE_BITS_MUL => Opcode::Mul,
                OPCODE_BITS_DIV => Opcode::Div,
                OPCODE_BITS_MUL_U => Opcode::MulU,
                OPCODE_BITS_DIV_U => Opcode::DivU,
                OPCODE_BITS_OR => Opcode::Or,
                OPCODE_BITS_AND => Opcode::And,
                OPCODE_BITS_XOR => Opcode::Xor,
                OPCODE_BITS_NOT => Opcode::Not,
                OPCODE_BITS_MOV_IMM => Opcode::MovImm,
                OPCODE_BITS_ADD_IMM_5 => Opcode::AddImm5,
                OPCODE_BITS_SETF => Opcode::Setf,
                OPCODE_BITS_CMP_IMM => Opcode::CmpImm,
                OPCODE_BITS_SHL_IMM => Opcode::ShlImm,
                OPCODE_BITS_SHR_IMM => Opcode::ShrImm,
                OPCODE_BITS_CLI => Opcode::Cli,
                OPCODE_BITS_SAR_IMM => Opcode::SarImm,
                OPCODE_BITS_RETI => Opcode::Reti,
                OPCODE_BITS_HALT => Opcode::Halt,
                OPCODE_BITS_LDSR => Opcode::Ldsr,
                OPCODE_BITS_STSR => Opcode::Stsr,
                OPCODE_BITS_SEI => Opcode::Sei,
                OPCODE_BITS_MOVEA => Opcode::Movea,
                OPCODE_BITS_ADD_IMM_16 => Opcode::AddImm16,
                OPCODE_BITS_JR => Opcode::Jr,
                OPCODE_BITS_JAL => Opcode::Jal,
                OPCODE_BITS_OR_I => Opcode::OrI,
                OPCODE_BITS_AND_I => Opcode::AndI,
                OPCODE_BITS_XOR_I => Opcode::XorI,
                OPCODE_BITS_MOVHI => Opcode::Movhi,
                OPCODE_BITS_LDB => Opcode::Ldb,
                OPCODE_BITS_LDH => Opcode::Ldh,
                OPCODE_BITS_LDW => Opcode::Ldw,
                OPCODE_BITS_STB => Opcode::Stb,
                OPCODE_BITS_STH => Opcode::Sth,
                OPCODE_BITS_STW => Opcode::Stw,
                OPCODE_BITS_INB => Opcode::Inb,
                OPCODE_BITS_INH => Opcode::Inh,
                OPCODE_BITS_INW => Opcode::Inw,
                OPCODE_BITS_OUTB => Opcode::Outb,
                OPCODE_BITS_OUTH => Opcode::Outh,
                OPCODE_BITS_EXTENDED => Opcode::Extended,
                OPCODE_BITS_OUTW => Opcode::Outw,
                _ => panic!("Unrecognized opcode bits: {:06b} (halfword: 0b{:016b})", opcode_bits, halfword),
            }
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::MovReg => InstructionFormat::I,
            &Opcode::AddReg => InstructionFormat::I,
            &Opcode::Sub => InstructionFormat::I,
            &Opcode::CmpReg => InstructionFormat::I,
            &Opcode::ShlReg => InstructionFormat::I,
            &Opcode::ShrReg => InstructionFormat::I,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::SarReg => InstructionFormat::I,
            &Opcode::Mul => InstructionFormat::I,
            &Opcode::Div => InstructionFormat::I,
            &Opcode::MulU => InstructionFormat::I,
            &Opcode::DivU => InstructionFormat::I,
            &Opcode::Or => InstructionFormat::I,
            &Opcode::And => InstructionFormat::I,
            &Opcode::Xor => InstructionFormat::I,
            &Opcode::Not => InstructionFormat::I,
            &Opcode::MovImm => InstructionFormat::II,
            &Opcode::AddImm5 => InstructionFormat::II,
            &Opcode::Setf => InstructionFormat::II,
            &Opcode::CmpImm => InstructionFormat::II,
            &Opcode::ShlImm => InstructionFormat::II,
            &Opcode::ShrImm => InstructionFormat::II,
            &Opcode::Cli => InstructionFormat::II,
            &Opcode::SarImm => InstructionFormat::II,
            &Opcode::Reti => InstructionFormat::II,
            &Opcode::Halt => InstructionFormat::II,
            &Opcode::Ldsr => InstructionFormat::II,
            &Opcode::Stsr => InstructionFormat::II,
            &Opcode::Sei => InstructionFormat::II,
            &Opcode::Bv => InstructionFormat::III,
            &Opcode::Bc => InstructionFormat::III,
            &Opcode::Bz => InstructionFormat::III,
            &Opcode::Bnh => InstructionFormat::III,
            &Opcode::Bn => InstructionFormat::III,
            &Opcode::Br => InstructionFormat::III,
            &Opcode::Blt => InstructionFormat::III,
            &Opcode::Ble => InstructionFormat::III,
            &Opcode::Bnv => InstructionFormat::III,
            &Opcode::Bnc => InstructionFormat::III,
            &Opcode::Bnz => InstructionFormat::III,
            &Opcode::Bh => InstructionFormat::III,
            &Opcode::Bp => InstructionFormat::III,
            &Opcode::Nop => InstructionFormat::III,
            &Opcode::Bge => InstructionFormat::III,
            &Opcode::Bgt => InstructionFormat::III,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::AddImm16 => InstructionFormat::V,
            &Opcode::Jr => InstructionFormat::IV,
            &Opcode::OrI => InstructionFormat::V,
            &Opcode::AndI => InstructionFormat::V,
            &Opcode::XorI => InstructionFormat::V,
            &Opcode::Jal => InstructionFormat::IV,
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Ldb => InstructionFormat::VI,
            &Opcode::Ldh => InstructionFormat::VI,
            &Opcode::Ldw => InstructionFormat::VI,
            &Opcode::Stb => InstructionFormat::VI,
            &Opcode::Sth => InstructionFormat::VI,
            &Opcode::Stw => InstructionFormat::VI,
            &Opcode::Inb => InstructionFormat::VI,
            &Opcode::Inh => InstructionFormat::VI,
            &Opcode::Inw => InstructionFormat::VI,
            &Opcode::Outb => InstructionFormat::VI,
            &Opcode::Outh => InstructionFormat::VI,
            &Opcode::Extended => InstructionFormat::VII,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }

    pub fn subop(&self, subop: u16) -> SubOp {
        match subop {
            OPCODE_BITS_SUB_OP_CMPF_S => SubOp::CmpfS,
            OPCODE_BITS_SUB_OP_CVT_WS => SubOp::CvtWs,
            OPCODE_BITS_SUB_OP_CVT_SW => SubOp::CvtSw,
            OPCODE_BITS_SUB_OP_ADDF_S => SubOp::AddfS,
            OPCODE_BITS_SUB_OP_SUBF_S => SubOp::SubfS,
            OPCODE_BITS_SUB_OP_MULF_S => SubOp::MulfS,
            OPCODE_BITS_SUB_OP_DIVF_S => SubOp::DivfS,
            OPCODE_BITS_SUB_OP_XB => SubOp::Xb,
            OPCODE_BITS_SUB_OP_XH => SubOp::Xh,
            OPCODE_BITS_SUB_OP_REV => SubOp::Rev,
            OPCODE_BITS_SUB_OP_TRNC_SW => SubOp::TrncSw,
            OPCODE_BITS_SUB_OP_MPYHW => SubOp::Mpyhw,
            _ => panic!("Unrecognized subop bits: {:06b}", subop),
        }
    }

    pub fn system_register(&self, imm5: usize) -> SystemRegister {
        match imm5 {
            OPCODE_SYSTEM_REGISTER_ID_EIPC => SystemRegister::Eipc,
            OPCODE_SYSTEM_REGISTER_ID_EIPSW => SystemRegister::Eipsw,
            OPCODE_SYSTEM_REGISTER_ID_FEPC => SystemRegister::Fepc,
            OPCODE_SYSTEM_REGISTER_ID_FEPSW => SystemRegister::Fepsw,
            OPCODE_SYSTEM_REGISTER_ID_ECR => SystemRegister::Ecr,
            OPCODE_SYSTEM_REGISTER_ID_PSW => SystemRegister::Psw,
            OPCODE_SYSTEM_REGISTER_ID_CHCW => SystemRegister::Chcw,
            _ => SystemRegister::Unknown(imm5),
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::MovReg | &Opcode::MovImm => "mov",
            &Opcode::AddReg | &Opcode::AddImm5 => "add",
            &Opcode::Sub => "sub",
            &Opcode::CmpReg | &Opcode::CmpImm => "cmp",
            &Opcode::ShlReg | &Opcode::ShlImm => "shl",
            &Opcode::ShrReg | &Opcode::ShrImm => "shr",
            &Opcode::Jmp => "jmp",
            &Opcode::SarReg | &Opcode::SarImm => "sar",
            &Opcode::Mul => "mul",
            &Opcode::Div => "div",
            &Opcode::MulU => "mulu",
            &Opcode::DivU => "divu",
            &Opcode::Or => "or",
            &Opcode::And => "and",
            &Opcode::Xor => "xor",
            &Opcode::Not => "not",
            &Opcode::Setf => "setf",
            &Opcode::Cli => "cli",
            &Opcode::Reti => "reti",
            &Opcode::Halt => "halt",
            &Opcode::Ldsr => "ldsr",
            &Opcode::Stsr => "stsr",
            &Opcode::Sei => "sei",
            &Opcode::Bv => "bv",
            &Opcode::Bc => "bc",
            &Opcode::Bz => "bz",
            &Opcode::Bnh => "bnh",
            &Opcode::Bn => "bn",
            &Opcode::Br => "br",
            &Opcode::Blt => "blt",
            &Opcode::Ble => "ble",
            &Opcode::Bnv => "bnv",
            &Opcode::Bnc => "bnc",
            &Opcode::Bnz => "bnz",
            &Opcode::Bh => "bh",
            &Opcode::Bp => "bp",
            &Opcode::Nop => "nop",
            &Opcode::Bge => "bge",
            &Opcode::Bgt => "bgt",
            &Opcode::Movea => "movea",
            &Opcode::AddImm16 => "addi",
            &Opcode::Jr => "jr",
            &Opcode::Jal => "jal",
            &Opcode::OrI => "ori",
            &Opcode::AndI => "andi",
            &Opcode::XorI => "xori",
            &Opcode::Movhi => "movhi",
            &Opcode::Ldb => "ld.b",
            &Opcode::Ldh => "ld.h",
            &Opcode::Ldw => "ld.w",
            &Opcode::Stb => "st.b",
            &Opcode::Sth => "st.h",
            &Opcode::Stw => "st.w",
            &Opcode::Inb => "in.b",
            &Opcode::Inh => "in.h",
            &Opcode::Inw => "in.w",
            &Opcode::Outb => "out.b",
            &Opcode::Outh => "out.h",
            &Opcode::Extended => unreachable!(), // TODO: Better pattern
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum InstructionFormat {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::II => false,
            &InstructionFormat::III => false,
            &InstructionFormat::IV => true,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
            &InstructionFormat::VII => true,
        }
    }
}

pub enum SubOp {
    CmpfS,
    CvtWs,
    CvtSw,
    AddfS,
    SubfS,
    MulfS,
    DivfS,
    Xb,
    Xh,
    Rev,
    TrncSw,
    Mpyhw,
}

impl fmt::Display for SubOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &SubOp::CmpfS => "cmpf.s",
            &SubOp::CvtWs => "cvt.ws",
            &SubOp::CvtSw => "cvt.sw",
            &SubOp::AddfS => "addf.s",
            &SubOp::SubfS => "subf.s",
            &SubOp::MulfS => "mulf.s",
            &SubOp::DivfS => "divf.s",
            &SubOp::Xb => "xb",
            &SubOp::Xh => "xh",
            &SubOp::Rev => "rev",
            &SubOp::TrncSw => "trnc.sw",
            &SubOp::Mpyhw => "mpyhw",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum SystemRegister {
    Eipc,
    Eipsw,
    Fepc,
    Fepsw,
    Ecr,
    Psw,
    Chcw,
    Unknown(usize),
}

impl fmt::Display for SystemRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SystemRegister::Eipc => write!(f, "{}", "eipc"),
            &SystemRegister::Eipsw => write!(f, "{}", "eipsw"),
            &SystemRegister::Fepc => write!(f, "{}", "fepc"),
            &SystemRegister::Fepsw => write!(f, "{}", "fepsw"),
            &SystemRegister::Ecr => write!(f, "{}", "ecr"),
            &SystemRegister::Psw => write!(f, "{}", "psw"),
            &SystemRegister::Chcw => write!(f, "{}", "chcw"),
            &SystemRegister::Unknown(imm5) => write!(f, "??? ({})", imm5),
        }
    }
}
