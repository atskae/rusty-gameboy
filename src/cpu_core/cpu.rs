use log::{debug, info, warn};
use std::fmt;
use std::format;
use std::fs;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::cli::Subcommand;
use crate::cpu_core::register::{Register, RegisterOperation};

// Indices into Cpu::registers vector
#[repr(u8)]
enum RegIndex {
    /// Accumulator and Flag
    AF = 0,
    /// Scratch registers
    BC,
    DE,
    HL,
    /// Stack pointer
    SP,
    /// Program counter
    PC,
    NumRegs,
}

impl Index<RegIndex> for Vec<Register> {
    type Output = Register;
    fn index(&self, register_index: RegIndex) -> &Self::Output {
        &self[register_index as usize]
    }
}

impl IndexMut<RegIndex> for Vec<Register> {
    fn index_mut(&mut self, register_index: RegIndex) -> &mut Self::Output {
        &mut self[register_index as usize]
    }
}

#[derive(Default)] // needed so Register initalizes to zero automatically
pub struct Cpu {
    regs: Vec<Register>,
    // Loaded ROM
    rom: Vec<u8>,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug!(
            "{} registers, AF={}",
            self.regs.len(),
            RegIndex::AF as usize
        );
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
            self.regs[RegIndex::AF].read_upper(),
            self.regs[RegIndex::AF].read_lower(),
            self.regs[RegIndex::BC].read_upper(),
            self.regs[RegIndex::BC].read_lower(),
            self.regs[RegIndex::DE].read_upper(),
            self.regs[RegIndex::DE].read_lower(),
            self.regs[RegIndex::HL].read_upper(),
            self.regs[RegIndex::HL].read_lower(),
            self.regs[RegIndex::SP].read(),
            self.regs[RegIndex::PC].read()
        );
        write!(f, "{}", registers)
    }
}

impl Cpu {
    pub fn new(rom_path: PathBuf) -> Cpu {
        let mut cpu: Cpu = Cpu {
            regs: Vec::with_capacity(RegIndex::NumRegs as usize), // only sets upper bound
            ..Default::default()
        };

        // Initialize registers
        for _ in 0..RegIndex::NumRegs as usize {
            cpu.regs.push(Default::default());
        }
        debug!("Initialized {} registers", cpu.regs.len());

        // Load ROM
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

    // Intruction logic
    // Each function returns the number of bytes to increment the program counter
    // Usually this is the instruction size in bytes, but for control-flow intructions
    // (such as Jump), the program counter increment is zero.

    fn load_imm16_sp(&mut self) -> u16 {
        let pc = self.regs[RegIndex::PC].read() as usize; // points to the opcode
        let mut imm16: u16 = self.rom[pc + 1] as u16;
        imm16 <<= 8;
        imm16 |= self.rom[pc + 2] as u16;

        // Update stack pointer
        self.regs[RegIndex::SP].write(imm16);

        2
    }

    // cc[index]
    // https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    fn cond_code(&self, index: u8) -> bool {
        let flag_reg_val: u8 = self.regs[RegIndex::AF].read_lower();
        let condition: bool = match index {
            0 => (flag_reg_val & 0b1000_0000) >> 7 == 0, // NZ
            1 => (flag_reg_val & 0b1000_0000) >> 7 == 1, // Z
            2 => (flag_reg_val & 0b0001_0000) >> 4 == 0, // NC
            3 => (flag_reg_val & 0b0001_0000) >> 4 == 1, // C
            _ => {
                warn!("Condition code index={}, case not covered!", index);
                false
            }
        };

        condition
    }

    /// If cond is true, check condition of the flag (specified by y)
    /// to decide whether to jump or not
    fn jr_d8(&mut self, cond: bool, y: u8) -> u16 {
        debug!("Jump, cond={}, y={}", cond, y);
        let pc_increment = 0;
        if cond && !self.cond_code(y - 4) {
            return pc_increment;
        }

        let pc = self.regs[RegIndex::PC].read() as usize; // points to the opcode
        let displacement: i8 = self.rom[pc + 1] as i8;
        debug!("displacement: {}", displacement);

        let mut new_pc = pc as u16;
        debug!("pc={}, new_pc={}", pc, new_pc);
        if displacement < 0 {
            new_pc -= displacement.abs() as u16;
        } else {
            new_pc += displacement.abs() as u16;
        }
        self.regs[RegIndex::PC].write(new_pc);

        pc_increment
    }

    /// Decodes then executes the instruction pointed to by the program_counter
    // Fields in the GameBoy manual label fields as single characters
    #[allow(clippy::many_single_char_names)]
    fn execute(&mut self) {
        // Decode the opcode byte by reading the subfields according to:
        // https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
        let opcode_byte: u8 = self.rom[self.regs[RegIndex::PC].read() as usize];
        debug!("program_counter: {}", self.regs[RegIndex::PC].read());
        debug!("Opcode {:b}", opcode_byte);

        let x: u8 = (opcode_byte & 0b1100_0000) >> 6;
        let y: u8 = (opcode_byte & 0b0011_1000) >> 3;
        let z: u8 = opcode_byte & 0b0000_0011;
        //let p: u8 = (y & 0b110) >> 1;
        //let q: u8 = y & 0b001;

        // Unprefixed opcodes
        let pc_increment: u16 = match x {
            0 => {
                match z {
                    0 => match y {
                        0 => 1,                    // NOP
                        1 => self.load_imm16_sp(), // Load immediate into SP
                        2 => 2,                    // STOP
                        3 => self.jr_d8(false, y), // Jump
                        _ => self.jr_d8(true, y),  // Conditional jump
                    },
                    _ => {
                        warn!("Not implemented this case of z!");
                        0
                    }
                }
            }
            _ => {
                debug!("Not implemented this case of x!");
                0
            }
        };

        // Increment the program counter
        self.regs[RegIndex::PC].increment(pc_increment);
    }

    pub fn start(&mut self, subcommand: Subcommand) {
        info!("Subcommand: {:?}", subcommand);

        info!("Running execute()");
        self.execute();
    }
}
