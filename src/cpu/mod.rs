/// Internal representation of the 6502 CPU
pub struct CPU {
    /// Stores the result of arithmetic, logic and memory operations
    pub accumulator: u8,
    /// Represents 7 status flags that can be set or unset depending on the result of
    /// the last instruction
    pub proc_status: u8,
    /// Holds the address for the next instruction
    pub prog_counter: u16,
    /// Used as an offset in specific memory addressing modes, can be used for temporary
    /// values or used as a counter
    pub reg_x: u8,
    /// Used as an offset in specific memory addressing modes, can be used for temporary
    /// values or used as a counter
    pub reg_y: u8,

    /// Continuous array of 1-byte cells. NES CPU uses 16-bit for memory addressing which means
    /// that it can address 65536 different memory cells
    memory: [u8; 0xFFFF],
}

mod lifecycle;
mod memory;
mod operations;
