mod lifecycle;
mod memory;
mod operations;

bitflags::bitflags! {
    /// Internal representation of the status
    /// flags for the 6502 CPU
    pub struct Flags: u8 {
        const CARRY        = 0b00000001;
        const ZERO         = 0b00000010;
        const NO_INTERRUPT = 0b00000100;
        const DECIMAL      = 0b00001000;
        const BREAK1       = 0b00010000;
        const BREAK2       = 0b00100000;
        const OVERFLOW     = 0b01000000;
        const NEGATIVE     = 0b10000000;
    }
}

/// Internal representation of the 6502 CPU
pub struct CPU {
    /// Stores the result of arithmetic, logic and memory operations
    pub a: u8,
    /// Represents 7 status flags that can be set or unset depending on the result of
    /// the last instruction
    pub flags: Flags,
    /// Holds the address for the next instruction
    pub counter: u16,
    /// Used as an offset in specific memory addressing modes, can be used for temporary
    /// values or used as a counter
    pub x: u8,
    /// Used as an offset in specific memory addressing modes, can be used for temporary
    /// values or used as a counter
    pub y: u8,

    /// Continuous array of 1-byte cells. NES CPU uses 16-bit for memory addressing which means
    /// that it can address 65536 different memory cells
    memory: [u8; 0xFFFF],
}
