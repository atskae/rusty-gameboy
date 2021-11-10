use log::{debug, error, info, warn};
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

/// Enum that presents the bit position of the
/// conditional flag in the Flag register
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FlagRegister {
    //Zero = 7,      // Z
    Subtract = 6,  // N
    HalfCarry = 5, // H
    Carry = 4,     // C
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
        if self.regs[reg_index].increment(delta).carry {
            unimplemented!("Setting carry flag on overflow is not implemented!");
        }
    }

    fn read_pc(&self) -> u16 {
        self.regs[RegIndex::PC].read()
    }

    fn read_zero_flag(&self) -> u8 {
        let flag_reg_val: u8 = self.regs[RegIndex::AF].read_lower();
        debug!("Read flag register: {:#010b}", flag_reg_val);
        let zero_flag_val: u8 = (flag_reg_val & 0b1000_0000) >> 7;
        debug!("Retrieved zero flag: {}", zero_flag_val);
        zero_flag_val
    }

    fn read_carry_flag(&self) -> u8 {
        let flag_reg_val: u8 = self.regs[RegIndex::AF].read_lower();
        (flag_reg_val & 0b0001_0000) >> 4
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

    fn invalid_opcode(&self, opcode: u8) -> u16 {
        error!("Invalid opcode! {:#02x}", opcode);
        0
    }

    // cc[index]
    fn cc(&self, index: u8) -> bool {
        debug!("Condition table index={}", index);
        let condition: bool = match index {
            0 => self.read_zero_flag() == 0,  // NZ
            1 => self.read_zero_flag() == 1,  // Z
            2 => self.read_carry_flag() == 0, // NC
            3 => self.read_carry_flag() == 1, // C
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

    /// Jump using an 8-bit offset
    fn jr_d8(&mut self) -> u16 {
        let pc = self.read_pc() as usize; // points to the opcode
        debug!(
            "displacement as u8: {:#02x} = {}",
            self.rom[pc + 1],
            self.rom[pc + 1]
        );
        let displacement: i8 = self.rom[pc + 1] as i8;
        debug!(
            "displacement as i8: {:#02x} = {}",
            displacement, displacement
        );

        let mut new_pc = pc as u16;
        debug!("pc={}, new_pc={}", pc, new_pc);
        if displacement < 0 {
            new_pc -= displacement.abs() as u16;
        } else {
            new_pc += displacement.abs() as u16;
        }
        self.regs[RegIndex::PC].write(new_pc);

        self.cycle += 12;
        0 // pc_increment
    }

    /// Conditional jump using an 8-bit offset
    fn jr_d8_cond(&mut self, y: u8) -> u16 {
        if y > 7 {
            warn!("y={} is too large.", y);
            return 0;
        }

        if self.cc(y - 4) {
            return self.jr_d8();
        }
        info!("Jump condition not satisfied.");
        0 // pc_increment
    }

    /// Add a 16-bit value from a register to HL
    fn add_hl_rp(&mut self, p: u8) -> u16 {
        let reg_val = self.regs[self.rp(p)].read();
        let carry_state = self.regs[RegIndex::HL].increment(reg_val);

        // Set the condition flags
        let af_reg = &mut self.regs[RegIndex::AF];
        af_reg.clear_bit_lower(FlagRegister::Subtract as u8);
        if carry_state.half_carry {
            debug!("Setting the half-carry flag.");
            af_reg.set_bit_lower(FlagRegister::HalfCarry as u8);
        }
        if carry_state.carry {
            debug!("Setting the carry flag.");
            af_reg.set_bit_lower(FlagRegister::Carry as u8);
        }

        1
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
                        0 => 1,                // NOP
                        1 => self.ld_d16_sp(), // Load immediate into SP
                        2 => {
                            // STOP
                            unimplemented!("STOP not implemented!");
                        }
                        3 => self.jr_d8(),           // Jump
                        4..=7 => self.jr_d8_cond(y), // Conditional jump
                        _ => self.invalid_opcode(opcode_byte),
                    },
                    1 => match q {
                        0 => self.ld_d16_rp(p),
                        1 => self.add_hl_rp(p),
                        _ => self.invalid_opcode(opcode_byte),
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

    // Checks that A (of AF), BC, DE, and HL are zero
    // The Flag register (F in AF) should be checked separately
    fn check_scratch_regs_are_zero(cpu: &Cpu) {
        // Check the A (accumulator) register only
        // since the Flag register is not really a scratch register
        assert_eq!(cpu.regs[RegIndex::AF].read_upper(), 0);

        assert_eq!(cpu.regs[RegIndex::BC].read(), 0);
        assert_eq!(cpu.regs[RegIndex::DE].read(), 0);
        assert_eq!(cpu.regs[RegIndex::HL].read(), 0);
    }

    fn read_flag_reg(cpu: &Cpu) -> u8 {
        cpu.regs[RegIndex::AF].read_lower()
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
        check_scratch_regs_are_zero(&cpu);
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
        check_scratch_regs_are_zero(&cpu);
    }

    #[test]
    fn test_jr_d8() {
        // Opcode = 0x18
        let rom: Vec<u8> = vec![0xFF, 0x18, 0x05, 0xFF, 0xFF, 0xFF, 0xFF];

        let mut cpu = Cpu::new_from_vec(rom);
        let start_pc = 1;
        cpu.regs[RegIndex::PC].write(start_pc);
        cpu.execute();

        assert_eq!(cpu.read_pc(), start_pc + 0x05);
        check_scratch_regs_are_zero(&cpu);
    }

    // Test jump with a negative offset
    #[test_env_log::test]
    fn test_jr_d8_negative() {
        // Opcode = 0x18
        // 0xFC= -4
        // Signed integers, 2s complement
        let rom: Vec<u8> = vec![0xFF, 0x18, 0x05, 0xFF, 0xFF, 0x18, 0xFC];

        let mut cpu = Cpu::new_from_vec(rom);
        let start_pc = 5;
        cpu.regs[RegIndex::PC].write(start_pc);
        debug!("pc: {}", cpu.read_pc());
        cpu.execute();

        assert_eq!(cpu.read_pc(), start_pc - 0x04);
        check_scratch_regs_are_zero(&cpu);
        assert_eq!(read_flag_reg(&cpu), 0);
    }

    /*
        Opcode:
        0x20: Jump if the zero flag is NOT set
        0x28: Jump if the zero flag is set
        0x30: Jump if the carry flag is NOT set
        0x38: Jump if the carry flag is set
    */

    #[test_case(0x020, 0b0111_1111, 5, 1; "nz jump")] // zero flag is bit 7
    #[test_case(0x020, 0b1000_0000, 5, 5; "no nz jump")]
    #[test_case(0x028, 0b1000_0000, 5, 1; "z jump")]
    #[test_case(0x028, 0b0111_1111, 5, 5; "no z jump")]
    #[test_case(0x030, 0b1110_1111, 5, 1; "nc jump")] // carry flag is bit 4
    #[test_case(0x030, 0b0001_0000, 5, 5; "no nc jump")]
    #[test_case(0x038, 0b0001_0000, 5, 1; "c jump")]
    #[test_case(0x038, 0b1110_1111, 5, 5; "no c jump")]
    fn test_jr_d8_cond(opcode: u8, flag_reg_val: u8, start_pc: u16, expected_pc: u16) {
        // The flag and condition to expect is written in the opcode
        // 0xFC= -4 ; signed integers, 2s complement
        let mut rom: Vec<u8> = vec![0xFF, 0x18, 0x05, 0xFF, 0xFF, 0x00, 0xFC];
        rom[start_pc as usize] = opcode; // Cpu will read the instruction from here
        let mut cpu = Cpu::new_from_vec(rom);
        cpu.regs[RegIndex::PC].write(start_pc as u16);
        debug!("pc: {}", cpu.read_pc());

        // Set the condition flag values
        cpu.regs[RegIndex::AF].write_lower(flag_reg_val);
        debug!("flag reg: {:#010b}", cpu.regs[RegIndex::AF].read_lower());
        cpu.execute();

        // Check if the jump occurred or not, based on the condition
        assert_eq!(cpu.read_pc(), expected_pc);
        check_scratch_regs_are_zero(&cpu);
        assert_eq!(read_flag_reg(&cpu), flag_reg_val);
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

    #[test_case(0x09, RegIndex::BC, 151, 75, 0b0000_0000; "bc register")]
    #[test_case(0x09, RegIndex::BC, 4095, 10, 0b0010_0000; "bc register half carry")]
    #[test_case(0x09, RegIndex::BC, 65535, 25, 0b0011_0000; "bc register half carry and carry")]
    #[test_case(0x19, RegIndex::DE, 151, 75, 0b0000_0000; "de register")]
    #[test_case(0x29, RegIndex::HL, 151, 75, 0b0000_0000; "hl register")]
    #[test_case(0x39, RegIndex::SP, 151, 75, 0b0000_0000; "sp register")]
    #[test_env_log::test]
    fn test_add_hl_rp(
        opcode: u8,
        reg_op: RegIndex,
        hl_val: u16,
        reg_op_val: u16,
        expected_flag_reg_val: u8,
    ) {
        let start_pc = 2; // arbitrary value
                          // All the bytes except at start_pc are arbitrary and not used
        let mut rom: Vec<u8> = vec![0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00];
        rom[start_pc as usize] = opcode; // Cpu will read the instruction from here
        let mut cpu = Cpu::new_from_vec(rom);
        // Set up register values
        cpu.regs[RegIndex::PC].write(start_pc as u16);
        cpu.regs[RegIndex::HL].write(hl_val);
        cpu.regs[reg_op].write(reg_op_val);
        debug!("pc: {}", cpu.read_pc());

        cpu.execute();

        let overflow_check = hl_val.checked_add(reg_op_val);
        if reg_op == RegIndex::HL {
            if overflow_check != None {
                assert_eq!(cpu.regs[RegIndex::HL].read(), reg_op_val + reg_op_val);
            }
        } else {
            if overflow_check != None {
                assert_eq!(cpu.regs[RegIndex::HL].read(), hl_val + reg_op_val);
            }
            assert_eq!(cpu.regs[reg_op].read(), reg_op_val); // check that it is unchanged
        }
        assert_eq!(cpu.regs[RegIndex::AF].read_lower(), expected_flag_reg_val); // check that it is unchanged
    }
} // tests module ; end
