use super::memory;
use super::CPU;

impl CPU {
    /// Creates a new instance of a CPU
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            proc_status: 0,
            prog_counter: 0,
            reg_x: 0,
            reg_y: 0,

            memory: [0; 0xFFFF], // [0x8000 .. 0xFFFF] is reserved for Program ROM
        }
    }

    /// Loads a program into PRG ROM space and saves the reference to the
    /// beginning code into 0xFFFC memory cell
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    /// Restores the state of all registers, and initializes `prog_counter` by the 2-byte value stored at 0xFFFC
    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.reg_x = 0;
        self.proc_status = 0;
        self.prog_counter = self.mem_read_u16(0xFFFC);
    }

    /// Executes the instructions stored on the CPU's PRG ROM
    pub fn run(&mut self) -> Result<(), &str> {
        loop {
            let op = self.mem_read(self.prog_counter);
            self.prog_counter += 1;
            match op {
                0xa9 => {
                    self.lda(memory::AddressingMode::Immediate);
                }
                0xa5 => {
                    self.lda(memory::AddressingMode::ZeroPage);
                }
                0xb5 => {
                    self.lda(memory::AddressingMode::ZeroPageX);
                }
                0xad => {
                    self.lda(memory::AddressingMode::Absolute);
                }
                0xbd => {
                    self.lda(memory::AddressingMode::AbsoluteX);
                }
                0xb9 => {
                    self.lda(memory::AddressingMode::AbsoluteY);
                }
                0xa1 => {
                    self.lda(memory::AddressingMode::IndirectX);
                }
                0xb1 => {
                    self.lda(memory::AddressingMode::IndirectY);
                }

                0xaa => self.tax(),
                0xe8 => self.inx(),
                0x00 => return Ok(()),

                _ => return Err("Unknown opcode found."),
            }
        }
    }

    /// Combines `load()`, `reset()` and `run()` associated functions.
    /// This is the primary method to be used by client code
    pub fn load_and_run(&mut self, program: Vec<u8>) -> Result<(), &str> {
        self.load(program);
        self.reset();
        self.run()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loads() {
        let mut cpu = CPU::new();
        let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        let prog_len = program.len();
        cpu.load(program);
        assert_eq!(
            cpu.memory[0x8000..(0x8000 + prog_len)],
            vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]
        )
    }

    #[test]
    fn resets() {
        let mut cpu = CPU::new();
        cpu.memory[0xFFFC] = 0x00;
        cpu.memory[0xFFFD] = 0x80;
        cpu.reset();
        assert_eq!(cpu.accumulator, 0);
        assert_eq!(cpu.reg_x, 0);
        assert_eq!(cpu.proc_status, 0);
        assert_eq!(cpu.prog_counter, 0x8000);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode found.")]
    fn run_panics() {
        let mut cpu = CPU::new();
        let program = vec![0xff];
        cpu.load_and_run(program).unwrap();
    }
}
