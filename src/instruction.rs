use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    Jmp,
    Movea,
    Movhi,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b000110 => Opcode::Jmp,
            0b101000 => Opcode::Movea,
            0b101111 => Opcode::Movhi,
            0b111111 => Opcode::Outw,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits),
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }
}


impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::Jmp => "jmp",
            &Opcode::Movea => "movea",
            &Opcode::Movhi => "movhi",
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum InstructionFormat {
    I,
    V,
    VI,
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}