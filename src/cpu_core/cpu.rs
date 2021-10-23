use log::{debug, info, warn};
use std::fmt;
use std::format;
use std::fs;
use std::path::PathBuf;

use crate::cli::Subcommand;
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

    // Loaded ROM
    rom: Vec<u8>,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let registers = format!(
            "
            ROM: {} bytes
            Registers
            AF: {} {}
            BC: {} {}
            DE: {} {}
            HL: {} {}
            stack_pointer: {}
            program_counter: {}
            ",
            self.rom.len(),
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
    pub fn new(rom_path: PathBuf) -> Cpu {
        let mut cpu: Cpu = Default::default();
        if rom_path.exists() {
            cpu.rom = fs::read(rom_path).unwrap();
            debug!(
                "Loaded ROM (byte preview): {:#02x} {:#02x} {:#02x}",
                cpu.rom[0], cpu.rom[1], cpu.rom[2]
            );
        } else {
            warn!("ROM file does not exist! Nothing was loaded.");
        }

        cpu
    }

    pub fn start(&self, subcommand: Subcommand) {
        info!("Subcommand: {:?}", subcommand);
    }
}
