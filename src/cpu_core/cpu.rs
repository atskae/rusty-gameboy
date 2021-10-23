use std::fmt;
use std::format;

use crate::cpu_core::register::{Register, RegisterOperation};

#[derive(Default)] // needed so Register initalizes to zero automatically
pub struct Cpu {
    /// Accumulator and Flag
    af_reg: Register,
    /// Scratch registers
    bc_reg: Register,
    de_reg: Register,
    hl_reg: Register,
    /// Stack pointer and program counter
    sp_reg: Register,
    pc_reg: Register,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let registers = format!(
            "
            Registers
            AF: {} {}
            BC: {} {}
            DE: {} {}
            HL: {} {}
            stack_pointer: {}
            program_counter: {}
            ",
            self.af_reg.read_upper(),
            self.af_reg.read_lower(),
            self.bc_reg.read_upper(),
            self.bc_reg.read_lower(),
            self.de_reg.read_upper(),
            self.de_reg.read_lower(),
            self.hl_reg.read_upper(),
            self.hl_reg.read_lower(),
            self.sp_reg.read(),
            self.pc_reg.read()
        );
        write!(f, "{}", registers)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Default::default()
    }
}
