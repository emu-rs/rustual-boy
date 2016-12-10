use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Opcode {
    Sub,
    Jmp,
    MovImm,
    Movea,
    Movhi,
    Stb,
    Outw,
}

impl Opcode {
    pub fn from_halfword(halfword: u16) -> Opcode {
        let opcode_bits = halfword >> 10;
        match opcode_bits {
            0b000010 => Opcode::Sub,
            0b000110 => Opcode::Jmp,
            0b010000 => Opcode::MovImm,
            0b101000 => Opcode::Movea,
            0b101111 => Opcode::Movhi,
            0b110100 => Opcode::Stb,
            0b111111 => Opcode::Outw,
            _ => panic!("Unrecognized opcode bits: {:06b}", opcode_bits),
        }
    }

    pub fn instruction_format(&self) -> InstructionFormat {
        match self {
            &Opcode::Sub => InstructionFormat::I,
            &Opcode::Jmp => InstructionFormat::I,
            &Opcode::MovImm => InstructionFormat::II,
            &Opcode::Movea => InstructionFormat::V,
            &Opcode::Movhi => InstructionFormat::V,
            &Opcode::Stb => InstructionFormat::VI,
            &Opcode::Outw => InstructionFormat::VI,
        }
    }

    pub fn num_cycles(&self) -> usize {
        match self {
            &Opcode::Sub => 1,
            &Opcode::Jmp => 3,
            &Opcode::MovImm => 1,
            &Opcode::Movea => 1,
            &Opcode::Movhi => 1,
            &Opcode::Stb => 1,
            &Opcode::Outw => 1,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mnemonic = match self {
            &Opcode::Sub => "sub",
            &Opcode::Jmp => "jmp",
            &Opcode::MovImm => "mov",
            &Opcode::Movea => "movea",
            &Opcode::Movhi => "movhi",
            &Opcode::Stb => "st.b",
            &Opcode::Outw => "out.w",
        };
        write!(f, "{}", mnemonic)
    }
}

pub enum InstructionFormat {
    I,
    II,
    V,
    VI,
}

impl InstructionFormat {
    pub fn has_second_halfword(&self) -> bool {
        match self {
            &InstructionFormat::I => false,
            &InstructionFormat::II => false,
            &InstructionFormat::V => true,
            &InstructionFormat::VI => true,
        }
    }
}