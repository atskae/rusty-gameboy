use log::{debug, info, warn};
use std::fmt;
use std::format;
use std::fs;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;

use crate::cli::Subcommand;
use crate::cpu_core::register::{Register, RegisterOperation};

// Indices into Cpu::registers vector
#[derive(PartialEq, Clone, Copy, Debug)]
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
    /// Custom for debugging
    Invalid,
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
    cycle: u16,
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
            == Cycle {} ==
            ROM: {} bytes
            Registers
            AF: {} {}
            BC: {} {}
            DE: {} {}
            HL: {} {}
            stack_pointer: {}
            program_counter: {}
            ",
            self.cycle,
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
    pub fn new() -> Cpu {
        let mut cpu: Cpu = Cpu {
            regs: Vec::with_capacity(RegIndex::NumRegs as usize), // only sets upper bound
            ..Default::default()
        };

        // Initialize registers
        for _ in 0..RegIndex::NumRegs as usize {
            cpu.regs.push(Default::default());
        }
        debug!("Initialized {} registers", cpu.regs.len());

        cpu
    }
    /// Create a Cpu from a Rom as a vector of bytes
    pub fn new_from_vec(rom: Vec<u8>) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.rom = rom;
        cpu
    }

    /// Create a Cpu from a Rom path
    pub fn new_from_path(rom_path: PathBuf) -> Cpu {
        // Load ROM
        if rom_path.exists() {
            let cpu = Cpu::new_from_vec(fs::read(rom_path).unwrap());
            debug!(
                "Loaded ROM (byte preview): {:#02x} {:#02x} {:#02x}",
                cpu.rom[0], cpu.rom[1], cpu.rom[2]
            );
            cpu
        } else {
            warn!("ROM file does not exist! Nothing was loaded.");
            Cpu::new() // return default
        }
    }

    /*
        Register helper methods
    */

    // Update register
    fn increment_reg(&mut self, reg_index: RegIndex, delta: u16) {
        // Set the carry flag since the operation overflowed
        if self.regs[reg_index].increment(delta) == None {
            unimplemented!("Setting carry flag on overflow is not implemented!");
        }
    }

    fn read_pc(&self) -> u16 {
        self.regs[RegIndex::PC].read()
    }

    // Uncomment when actually used
    //fn read_sp(&self) -> u16 {
    //    self.regs[RegIndex::SP].read()
    //}

    /*
        Helper methods for executing instructions.
        Methods are named after the tables/logic defined here:
            https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
    */

    // cc[index]
    fn cc(&self, index: u8) -> bool {
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

    fn rp(&self, index: u8) -> RegIndex {
        match index {
            0 => RegIndex::BC,
            1 => RegIndex::DE,
            2 => RegIndex::HL,
            3 => RegIndex::SP,
            _ => RegIndex::Invalid,
        }
    }

    /*
        Actual instruction execution. Modifies Cpu state.
        Each function returns the number of bytes to increment the program counter.
        Usually this is the instruction size in bytes, but for control-flow intructions
            (such as Jump), the program counter increment is zero.
    */

    // Loads a 16-bit value into a register
    fn ld_d16_rp(&mut self, index: u8) -> u16 {
        let pc = self.read_pc() as usize; // points to the opcode
        let mut imm16: u16 = self.rom[pc + 1] as u16;
        imm16 <<= 8;
        imm16 |= self.rom[pc + 2] as u16;

        let reg_index: RegIndex = self.rp(index);
        self.regs[reg_index].write(imm16);

        debug!("LD {:?}, {:#02x}", reg_index, imm16);
        self.cycle += 12;
        3
    }

    /// Load a 16-bit value into the stack pointer
    fn ld_d16_sp(&mut self) -> u16 {
        // 3 is the index into rp that corresponds to the SP register
        self.ld_d16_rp(3)
    }

    /// If cond is true, check condition of the flag (specified by y)
    /// to decide whether to jump or not
    fn jr_d8(&mut self, cond: bool, y: u8) -> u16 {
        debug!("Jump, cond={}, y={}", cond, y);
        let pc_increment = 0;
        if cond && !self.cc(y - 4) {
            return pc_increment;
        }

        let pc = self.read_pc() as usize; // points to the opcode
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

        self.cycle += 12;
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
        let p: u8 = (y & 0b110) >> 1;
        let q: u8 = y & 0b001;

        // Unprefixed opcodes
        let pc_increment: u16 = match x {
            0 => {
                match z {
                    0 => match y {
                        0 => 1,                    // NOP
                        1 => self.ld_d16_sp(),     // Load immediate into SP
                        2 => 2,                    // STOP
                        3 => self.jr_d8(false, y), // Jump
                        _ => self.jr_d8(true, y),  // Conditional jump
                    },
                    1 => match q {
                        0 => self.ld_d16_rp(p),
                        _ => unimplemented!("Not implemented this case of q!"),
                    },
                    _ => unimplemented!("Not implemented this case of z!"),
                }
            }
            _ => unimplemented!("Not implemented this case of x!"),
        };

        // Increment the program counter
        self.increment_reg(RegIndex::PC, pc_increment);
    }

    pub fn start(&mut self, subcommand: Subcommand) {
        info!("Subcommand: {:?}", subcommand);

        info!("Running execute()");
        self.execute();
        debug!("{}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*; // use the same imports as outer scope
    use test_case::test_case; // parameterized tests

    // Used until cpu.read_sp() is actually used somewhere
    // outside the test environment...
    fn read_sp(cpu: &Cpu) -> u16 {
        cpu.regs[RegIndex::SP].read()
    }

    // Checks that AL, BC, DE, and HL are zero
    fn check_regs_are_zero(cpu: &Cpu) {
        assert_eq!(cpu.regs[RegIndex::AF].read(), 0);
        assert_eq!(cpu.regs[RegIndex::BC].read(), 0);
        assert_eq!(cpu.regs[RegIndex::DE].read(), 0);
        assert_eq!(cpu.regs[RegIndex::HL].read(), 0);
    }

    /*
        Test that execute() correctly decodes each instruction.
        0xFF unused bytes; these are inserted to
        test ROM reads at arbitrary PC values
    */

    #[test]
    fn test_nop() {
        // 0x00 = Opcode
        let rom: Vec<u8> = vec![0x00, 0xFF, 0xFF, 0x00, 0xFF];

        let mut cpu = Cpu::new_from_vec(rom);
        let start_pc = 3;
        cpu.regs[RegIndex::PC].write(start_pc);
        cpu.execute();

        assert_eq!(cpu.read_pc(), start_pc + 1); // size of instruction
        check_regs_are_zero(&cpu);
    }

    // This opcode specifically loads into SP always.
    // test_ld_d16_rp on the other hand, can load into any register
    #[test]
    fn test_ld_d16_sp() {
        let rom: Vec<u8> = vec![
            0xFF, 0xFF, 0x08, // 0x08 = Opcode
            0xFF, // First byte of 16-bit data
            0xA7, // Second byte of 16-bit data
            0xFF, 0xFF,
        ];
        let mut cpu = Cpu::new_from_vec(rom);
        let start_pc = 2;
        cpu.regs[RegIndex::PC].write(start_pc);
        cpu.execute();

        assert_eq!(cpu.read_pc(), start_pc + 3); // size of instruction
        assert_eq!(read_sp(&cpu), 0xFFA7);
        check_regs_are_zero(&cpu);
    }

    #[test_case(0x01, RegIndex::BC; "bc register")]
    #[test_case(0x11, RegIndex::DE; "de register")]
    #[test_case(0x21, RegIndex::HL; "hl register")]
    #[test_case(0x31, RegIndex::SP; "stack pointer")]
    fn test_ld_d16_rp(opcode: u8, reg: RegIndex) {
        let mut rom: Vec<u8> = vec![
            0xFF, 0xFF, 0x00, 0x41, // First byte of 16-bit data
            0x23, // Second byte of 16-bit data
            0xFF, 0xFF,
        ];
        rom[2] = opcode;

        let mut cpu = Cpu::new_from_vec(rom);
        let start_pc = 2;
        cpu.regs[RegIndex::PC].write(start_pc);
        cpu.execute();

        assert_eq!(cpu.read_pc(), start_pc + 3); // size of instruction
        assert_eq!(cpu.regs[reg].read(), 0x4123);

        // Check that other registers were not modified
        let regs_to_check = [RegIndex::AF, RegIndex::BC, RegIndex::DE, RegIndex::HL];
        for reg_to_check in regs_to_check.iter() {
            // reg_to_check is a reference, so must de-reference
            if reg == *reg_to_check {
                continue;
            }
            assert_eq!(cpu.regs[*reg_to_check].read(), 0);
        }
    }
}
