use log::{debug, warn};

/// The return value of a arithmetic operatiomn
// which indicates whether a carry or a half-carry occurred
pub struct CarryState {
    pub carry: bool,
    pub half_carry: bool,
}

/*
    Helper functions
*/

fn _read_upper(value: u16) -> u8 {
    let masked = value & 0b1111_1111_0000_0000;
    (masked >> 8) as u8
}

fn _read_lower(value: u16) -> u8 {
    // Shift the bits to the most-significant bits
    // to zero out the least-significant bits
    let shifted = value << 8;
    // Apply mask
    let masked = shifted & 0b1111_1111_0000_0000;
    // Shift the bits back to the least-significant bits
    (masked >> 8) as u8
}

pub trait RegisterOperation {
    fn read(&self) -> u16;
    fn read_upper(&self) -> u8;
    fn read_lower(&self) -> u8;

    fn write(&mut self, value: u16);
    fn write_upper(&mut self, value: u8);
    fn write_lower(&mut self, value: u8);

    fn set_bit(&mut self, bit_index: u8);
    fn set_bit_upper(&mut self, bit_index: u8);
    fn set_bit_lower(&mut self, bit_index: u8);

    fn clear_bit(&mut self, bit_index: u8);
    fn clear_bit_upper(&mut self, bit_index: u8);
    fn clear_bit_lower(&mut self, bit_index: u8);

    fn is_half_carry(&self, delta: u16) -> bool;

    // Increment or decrement by a delta value
    fn increment(&mut self, delta: u16) -> CarryState;
    fn decrement(&mut self, delta: u16) -> CarryState;
}

/// 16-bit register
#[derive(Default)]
pub struct Register {
    value: u16, // derive(Default) sets value to 0
}

impl RegisterOperation for Register {
    fn read(&self) -> u16 {
        self.value
    }

    /// Return the most-significant byte in the register
    fn read_upper(&self) -> u8 {
        _read_upper(self.value)
    }

    /// Return the least-significant byte in the register
    fn read_lower(&self) -> u8 {
        _read_lower(self.value)
    }

    fn write(&mut self, value: u16) {
        self.value = value;
    }

    fn write_upper(&mut self, value: u8) {
        // Zero out the most-significant bits
        self.value &= 0b0000_0000_1111_1111;
        // Write the bits
        self.value |= (value as u16) << 8;
    }

    fn write_lower(&mut self, value: u8) {
        // Zero out the least-significant bits
        self.value &= 0b1111_1111_0000_0000;
        // Write the bits
        self.value |= value as u16;
    }

    fn set_bit(&mut self, bit_index: u8) {
        if bit_index > 15 {
            warn!("Bit index {} is out of range!", bit_index);
            return;
        }
        debug!("Before set_bit: {:b}", self.value);
        let mask = 1 << bit_index;
        self.value |= mask;
        debug!("After set_bit: {:b}", self.value);
    }

    /// Set a bit in the upper register (of a 16-bit register)
    /// by passing in a logical bit_index (range 0-7)
    fn set_bit_upper(&mut self, logical_bit_index: u8) {
        if logical_bit_index > 7 {
            warn!("Bit index {} is out of range!", logical_bit_index);
            return;
        }
        let bit_index = logical_bit_index + 8;
        self.set_bit(bit_index);
    }

    /// Same as set_bit() but with a stricter range
    /// within the lower 8-bit register
    fn set_bit_lower(&mut self, bit_index: u8) {
        if bit_index > 7 {
            warn!("Bit index {} is out of range!", bit_index);
            return;
        }
        self.set_bit(bit_index);
    }

    fn clear_bit(&mut self, bit_index: u8) {
        if bit_index > 15 {
            warn!("Bit index {} is out of range!", bit_index);
            return;
        }
        debug!("Before clear_bit: {:b}", self.value);
        let mask = 1 << bit_index;
        self.value &= !mask;
        debug!("After clear_bit: {:b}", self.value);
    }

    /// Clear a bit in the upper register (of a 16-bit register)
    /// by passing in a logical bit_index (range 0-7)
    fn clear_bit_upper(&mut self, logical_bit_index: u8) {
        if logical_bit_index > 7 {
            warn!("Bit index {} is out of range!", logical_bit_index);
            return;
        }
        let bit_index = logical_bit_index + 8;
        self.clear_bit(bit_index);
    }

    fn clear_bit_lower(&mut self, bit_index: u8) {
        if bit_index > 7 {
            warn!("Bit index {} is out of range!", bit_index);
            return;
        }
        self.clear_bit(bit_index);
    }

    fn is_half_carry(&self, delta: u16) -> bool {
        // Extract the upper byte
        let a: u8 = self.read_upper();
        let b: u8 = _read_lower(delta);

        // Check if adding the lower 4-bits (nibble) produces a carry
        (a & 0b000_1111) + (b & 0b0000_1111) == (0b0001_0000)
    }

    // If overflow occurs, return None
    fn increment(&mut self, delta: u16) -> CarryState {
        let overflow_check = self.value.checked_add(delta);
        self.value = self.value.wrapping_add(delta);
        CarryState {
            carry: overflow_check == None,
            half_carry: self.is_half_carry(delta),
        }
    }

    fn decrement(&mut self, delta: u16) -> CarryState {
        let overflow_check = self.value.checked_sub(delta);
        self.value = self.value.wrapping_sub(delta);
        CarryState {
            carry: overflow_check == None, // might not apply to sub...
            half_carry: self.is_half_carry(delta),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // use the same imports as outer scope
    use test_env_log::test;

    #[test]
    fn test_read_default() {
        let reg: Register = Default::default();
        assert_eq!(reg.read(), 0);
        assert_eq!(reg.read_upper(), 0);
        assert_eq!(reg.read_lower(), 0);
    }

    #[test]
    fn test_read() {
        let val = 37485;
        let reg = Register { value: val };
        assert_eq!(reg.read(), val);
    }

    #[test]
    fn test_read_upper_lower() {
        let val = 0b1001_0010_0110_1101;
        let reg = Register { value: val };
        assert_eq!(reg.read(), val);
        assert_eq!(reg.read_upper(), 0b1001_0010);
        assert_eq!(reg.read_lower(), 0b0110_1101);
    }

    #[test]
    fn test_write() {
        let val = 12985;
        let mut reg: Register = Default::default();
        assert_eq!(reg.read(), 0);
        reg.write(val);
        assert_eq!(reg.read(), val);
    }

    #[test]
    fn test_write_upper() {
        let val = 0b0011_0010_1011_1001;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);
        assert_eq!(reg.read_upper(), 0b0011_0010);

        let upper = 0b0010_0000;
        reg.write_upper(upper);
        assert_eq!(reg.read_upper(), upper);
        assert_eq!(reg.read(), 0b0010_0000_1011_1001);
    }

    #[test]
    fn test_write_lower() {
        let val = 0b0011_0010_1011_1001;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);
        assert_eq!(reg.read_lower(), 0b1011_1001);

        let lower = 0b1111_0101;
        reg.write_lower(lower);
        assert_eq!(reg.read_lower(), lower);
        assert_eq!(reg.read(), 0b0011_0010_1111_0101);
    }

    #[test]
    fn test_set_bit() {
        let val = 0b0000_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.set_bit(27);
        // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        reg.set_bit(13);
        assert_eq!(reg.read(), 0b0010_0010_0100_1111);
    }

    #[test]
    fn test_set_bit_upper() {
        let val = 0b0000_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.set_bit_upper(9); // bit index 0-7 are only allowed
                              // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        // A logical index of an 8-bit register is passed in,
        // which then gets mapped to the actual bit index
        // in the 16-bit register
        reg.set_bit_upper(2);
        assert_eq!(reg.read(), 0b0000_0110_0100_1111);
    }

    #[test]
    fn test_set_bit_lower() {
        let val = 0b0000_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.set_bit_lower(14); // bit index 0-7 are only allowed
                               // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        reg.set_bit_lower(5);
        assert_eq!(reg.read(), 0b0000_0010_0110_1111);
    }

    #[test]
    fn test_clear_bit() {
        let val = 0b1011_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.clear_bit(38);
        // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        reg.clear_bit(15);
        assert_eq!(reg.read(), 0b0011_0010_0100_1111);
    }

    #[test]
    fn clear_bit_upper() {
        let val = 0b1111_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.clear_bit_upper(9); // bit index 0-7 are only allowed
                                // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        // A logical index of an 8-bit register is passed in,
        // which then gets mapped to the actual bit index
        // in the 16-bit register
        reg.clear_bit_upper(1);
        assert_eq!(reg.read(), 0b1111_0000_0100_1111);
    }

    #[test]
    fn clear_bit_lower(&mut self, bit_index: u8) {
        let val = 0b1111_0010_0100_1111;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        // Test out of range bit index
        reg.clear_bit_lower(14); // bit index 0-7 are only allowed
                                 // The value should be unchanged
        assert_eq!(reg.read(), val);

        // Test a valid bit index
        // A logical index of an 8-bit register is passed in,
        // which then gets mapped to the actual bit index
        // in the 16-bit register
        reg.clear_bit_lower(3);
        assert_eq!(reg.read(), 0b1111_0010_0100_0111);
    }

    #[test]
    fn test_increment() {
        let val = 34521;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 432;
        reg.increment(delta);

        assert_eq!(reg.read(), val + delta);
    }

    #[test]
    fn test_increment_max_overflow() {
        let val = u16::MAX - 1000;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 2000;
        let carry_state = reg.increment(delta);

        assert_eq!(carry_state.carry, true);
        // wrapping_add() is equivalent mod u16::max
        // Need to compute in u32 as the temporary result cannot fit in u16
        let expected: u16 = ((val as u32) + (delta as u32) % (u16::MAX as u32 + 1)) as u16;
        assert_eq!(reg.read(), expected);
    }

    #[test]
    fn test_increment_u16_max_overflow() {
        let val = u16::MAX;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 432;
        let carry_state = reg.increment(delta);

        assert_eq!(carry_state.carry, true);
        let expected: u16 = ((val as u32) + (delta as u32) % (u16::MAX as u32 + 1)) as u16;
        assert_eq!(reg.read(), expected);
    }

    #[test]
    fn test_decrement() {
        let val = 34521;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 432;
        reg.decrement(delta);

        assert_eq!(reg.read(), val - delta);
    }

    #[test]
    fn test_decrement_from_zero_overflow() {
        let mut reg: Register = Default::default();
        assert_eq!(reg.read(), 0);

        let delta = 432;
        let carry_state = reg.decrement(delta);

        assert_eq!(carry_state.carry, true);
        assert_eq!(reg.read(), u16::MAX - delta + 1);
    }

    #[test]
    fn test_decrement_overflow() {
        let val = 500;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 1000;
        assert!(delta > val);
        let carry_state = reg.decrement(delta);

        assert_eq!(carry_state.carry, true);
        let expected = u16::MAX - (delta - val) + 1;
        assert_eq!(reg.read(), expected);
    }
}
