use crate::cpu_core::flag_register::FlagEffect;

#[derive(Default)]
pub struct Insn {
    pub size: u16,              // length in bytes
    pub cycles: u16,            // duration in cycles
    pub flags: [FlagEffect; 4], // bit string: Z N H C
}

impl Insn {
    pub fn invalid() -> Insn {
        Insn {
            ..Default::default()
        }
    }

    pub fn nop() -> Insn {
        Insn {
            size: 1,
            cycles: 4, // interesting
            ..Default::default()
        }
    }
}
