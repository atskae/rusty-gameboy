#[derive(Default)]
pub struct Insn {
    pub size: u16,   // length in bytes
    pub cycles: u16, // duration in cycles
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
        }
    }
}
