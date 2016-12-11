use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    MovReg,
    Sub,
    Jmp,
    MovImm,
    Cli,
    Ldsr,
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
    Movhi,
    Ldb,
    Ldh,
    Ldw,
    Stb,
    Inb,
    Inh,
    Inw,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        if halfword >> 13 == 0b100 {
            let cond_bits = (halfword >> 9) & 0x0f;
            match cond_bits {
                0b0000 => Opcode::Bv,
                0b0001 => Opcode::Bc,
                0b0010 => Opcode::Bz,
                0b0011 => Opcode::Bnh,
                0b0100 => Opcode::Bn,
                0b0101 => Opcode::Br,
                0b0110 => Opcode::Blt,
                0b0111 => Opcode::Ble,
                0b1000 => Opcode::Bnv,
                0b1001 => Opcode::Bnc,
                0b1010 => Opcode::Bnz,
                0b1011 => Opcode::Bh,
                0b1100 => Opcode::Bp,
                0b1101 => Opcode::Nop,
                0b1110 => Opcode::Bge,
                0b1111 => Opcode::Bgt,
                _ => panic!("Unrecognized cond bits: {:04b} (halfword: 0b{:016b})", cond_bits, halfword)
            }
        } else {
            let opcode_bits = halfword >> 10;
            match opcode_bits {
                0b000000 => Opcode::MovReg,
                0b000010 => Opcode::Sub,
                0b000110 => Opcode::Jmp,
                0b010000 => Opcode::MovImm,
                0b010110 => Opcode::Cli,
                0b011100 => Opcode::Ldsr,
                0b011110 => Opcode::Sei,
                0b101000 => Opcode::Movea,
                0b101111 => Opcode::Movhi,
                0b110000 => Opcode::Ldb,
                0b110001 => Opcode::Ldh,
                0b110011 => Opcode::Ldw,
                0b110100 => Opcode::Stb,
                0b111000 => Opcode::Inb,
                0b111001 => Opcode::Inh,
                0b111011 => Opcode::Inw,
                0b111111 => Opcode::Outw,
                _ => panic!("Unrecognized opcode bits: {:06b} (halfword: 0b{:016b})", opcode_bits, halfword),
            }
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::MovReg => InstructionFormat::I,
            &Opcode::Sub => InstructionFormat::I,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::MovImm => InstructionFormat::II,
            &Opcode::Cli => InstructionFormat::II,
            &Opcode::Ldsr => InstructionFormat::II,
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
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Ldb => InstructionFormat::VI,
            &Opcode::Ldh => InstructionFormat::VI,
            &Opcode::Ldw => InstructionFormat::VI,
            &Opcode::Stb => InstructionFormat::VI,
            &Opcode::Inb => InstructionFormat::VI,
            &Opcode::Inh => InstructionFormat::VI,
            &Opcode::Inw => InstructionFormat::VI,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }

    pub fn system_register(&self, imm5: usize) -> SystemRegister {
        match imm5 {
            5 => SystemRegister::Psw,
            _ => panic!("Unrecognized system register: {}", imm5),
        }
    }

    pub fn num_cycles(&self, branch_taken: bool) -> usize {
        match self {
            &Opcode::MovReg => 1,
            &Opcode::Sub => 1,
            &Opcode::Jmp => 3,
            &Opcode::MovImm => 1,
            &Opcode::Cli => 1,
            &Opcode::Ldsr => 1,
            &Opcode::Sei => 1,
            &Opcode::Bv |
            &Opcode::Bc |
            &Opcode::Bz |
            &Opcode::Bnh |
            &Opcode::Bn |
            &Opcode::Blt |
            &Opcode::Ble |
            &Opcode::Bnv |
            &Opcode::Bnc |
            &Opcode::Bnz |
            &Opcode::Bh |
            &Opcode::Bp |
            &Opcode::Bge |
            &Opcode::Bgt => if branch_taken { 3 } else { 1 },
            &Opcode::Br => 3,
            &Opcode::Nop => 1,
            &Opcode::Movea => 1,
            &Opcode::Movhi => 1,
            &Opcode::Ldb => 4,
            &Opcode::Ldh => 4,
            &Opcode::Ldw => 4,
            &Opcode::Stb => 1,
            &Opcode::Inb => 4,
            &Opcode::Inh => 4,
            &Opcode::Inw => 4,
            &Opcode::Outw => 1,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::MovReg | &Opcode::MovImm => "mov",
            &Opcode::Sub => "sub",
            &Opcode::Jmp => "jmp",
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
            &Opcode::Cli => "cli",
            &Opcode::Ldsr => "ldsr",
            &Opcode::Sei => "sei",
            &Opcode::Movea => "movea",
            &Opcode::Movhi => "movhi",
            &Opcode::Ldb => "ld.b",
            &Opcode::Ldh => "ld.h",
            &Opcode::Ldw => "ld.w",
            &Opcode::Stb => "st.b",
            &Opcode::Inb => "in.b",
            &Opcode::Inh => "in.h",
            &Opcode::Inw => "in.w",
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum InstructionFormat {
    I,
    II,
    III,
    V,
    VI,
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::II => false,
            &InstructionFormat::III => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}

pub enum SystemRegister {
    Psw
}

impl fmt::Display for SystemRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &SystemRegister::Psw => "psw",
        };
        write!(f, "{}", mnemonic)
    }
}