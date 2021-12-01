pub enum FlagEffect {
    Reset,
    Set,
    None,   // do nothing
    Result, // the flag effect depends on the result of the operation
}

impl Default for FlagEffect {
    fn default() -> Self {
        FlagEffect::None
    }
}

/// Enum that presents the bit position of the
/// conditional flag in the Flag register
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum FlagRegister {
    //Zero = 7,      // Z
    Subtract = 6,  // N
    HalfCarry = 5, // H
    Carry = 4,     // C
}
