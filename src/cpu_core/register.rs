pub trait RegisterOperation {
    fn read(&self) -> u16;
    fn read_upper(&self) -> u8;
    fn read_lower(&self) -> u8;

    fn write(&mut self, value: u16);
    fn write_upper(&mut self, value: u8);
    fn write_lower(&mut self, value: u8);

    // Increment or decrement by a delta value
    fn increment(&mut self, delta: u16) -> Option<u16>;
    fn decrement(&mut self, delta: u16) -> Option<u16>;
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
        let masked = self.value & 0b1111_1111_0000_0000;
        (masked >> 8) as u8
    }

    /// Return the least-significant byte in the register
    fn read_lower(&self) -> u8 {
        // Shift the bits to the most-significant bits
        // to zero out the least-significant bits
        let shifted = self.value << 8;
        // Apply mask
        let masked = shifted & 0b1111_1111_0000_0000;
        // Shift the bits back to the least-significant bits
        (masked >> 8) as u8
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

    // If overflow occurs, return None
    fn increment(&mut self, delta: u16) -> Option<u16> {
        let overflow_check = self.value.checked_add(delta);
        self.value = self.value.wrapping_add(delta);
        overflow_check
    }

    fn decrement(&mut self, delta: u16) -> Option<u16> {
        let overflow_check = self.value.checked_sub(delta);
        self.value = self.value.wrapping_sub(delta);
        overflow_check
    }
}

#[cfg(test)]
mod tests {
    use super::*; // use the same imports as outer scope
    use log::debug;
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
        let overflow_check = reg.increment(delta);

        assert_eq!(overflow_check, None);
        // delta - <distance from u16::MAX> - 1
        assert_eq!(reg.read(), delta - 1000 - 1);
    }

    #[test]
    fn test_increment_u16_max_overflow() {
        let val = u16::MAX;
        let mut reg = Register { value: val };
        assert_eq!(reg.read(), val);

        let delta = 432;
        let overflow_check = reg.increment(delta);

        assert_eq!(overflow_check, None);
        assert_eq!(reg.read(), delta - 1);
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
    fn test_decrement_overflow() {
        let mut reg: Register = Default::default();
        assert_eq!(reg.read(), 0);

        let delta = 432;
        let overflow_check = reg.decrement(delta);

        assert_eq!(overflow_check, None);
        debug!("u16::MAX: {}", u16::MAX);
        assert_eq!(reg.read(), u16::MAX - delta + 1);
    }
}
