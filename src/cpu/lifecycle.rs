use super::memory;
use super::operations::CPUStatus;
use super::CPU;

impl CPU {
    /// Creates a new instance of a CPU
    pub fn new() -> Self {
        CPU {
            a: 0,
            status: CPUStatus::from_bits_truncate(0b100100),
            counter: 0,
            x: 0,
            y: 0,
            memory: [0; 0xFFFF],
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
        self.a = 0;
        self.x = 0;
        self.status = CPUStatus::from_bits_truncate(0b100100);
        self.counter = self.mem_read_u16(0xFFFC);
    }

    /// Executes the instructions stored on the CPU's PRG ROM
    pub fn run(&mut self) -> Result<(), &str> {
        loop {
            let op = self.mem_read(self.counter);
            self.counter += 1;
            match op {
                0x69 => {
                    self.adc(memory::AddressingMode::Immediate);
                }

                0x65 => {
                    self.adc(memory::AddressingMode::ZeroPage);
                }

                0x75 => {
                    self.adc(memory::AddressingMode::ZeroPageX);
                }

                0x6d => {
                    self.adc(memory::AddressingMode::Absolute);
                }

                0x7d => {
                    self.adc(memory::AddressingMode::AbsoluteX);
                }

                0x79 => {
                    self.adc(memory::AddressingMode::AbsoluteY);
                }

                0x61 => {
                    self.adc(memory::AddressingMode::IndirectX);
                }

                0x71 => {
                    self.adc(memory::AddressingMode::IndirectY);
                }

                0x29 => {
                    self.and(memory::AddressingMode::Immediate);
                }

                0x25 => {
                    self.and(memory::AddressingMode::ZeroPage);
                }

                0x35 => {
                    self.and(memory::AddressingMode::ZeroPageX);
                }

                0x2d => {
                    self.and(memory::AddressingMode::Absolute);
                }

                0x3d => {
                    self.and(memory::AddressingMode::AbsoluteX);
                }

                0x39 => {
                    self.and(memory::AddressingMode::AbsoluteY);
                }

                0x21 => {
                    self.and(memory::AddressingMode::IndirectX);
                }

                0x31 => {
                    self.and(memory::AddressingMode::IndirectY);
                }

                0x0a => {
                    self.asl_on_accumulator();
                }

                0x06 => {
                    self.asl(memory::AddressingMode::ZeroPage);
                }

                0x16 => {
                    self.asl(memory::AddressingMode::ZeroPageX);
                }

                0x0e => {
                    self.asl(memory::AddressingMode::Absolute);
                }

                0x1e => {
                    self.asl(memory::AddressingMode::AbsoluteX);
                }

                /* BCC - Branch if Carry Clear */
                0x90 => self.branch(!self.status.contains(CPUStatus::CARRY)),

                /* BCS - Branch if Carry Set */
                0xb0 => self.branch(self.status.contains(CPUStatus::CARRY)),

                /* BEQ - Branch if Equal */
                0xf0 => self.branch(self.status.contains(CPUStatus::ZERO)),

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

                0x85 => {
                    self.sta(memory::AddressingMode::ZeroPage);
                }

                0x95 => {
                    self.sta(memory::AddressingMode::ZeroPageX);
                }

                0x8D => {
                    self.sta(memory::AddressingMode::Absolute);
                }

                0x9D => {
                    self.sta(memory::AddressingMode::AbsoluteX);
                }

                0x99 => {
                    self.sta(memory::AddressingMode::AbsoluteY);
                }

                0x81 => {
                    self.sta(memory::AddressingMode::IndirectX);
                }

                0x91 => {
                    self.sta(memory::AddressingMode::IndirectY);
                }

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
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.status, CPUStatus::from_bits_truncate(0b100100),);
        assert_eq!(cpu.counter, 0x8000);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode found.")]
    fn run_panics() {
        let mut cpu = CPU::new();
        let program = vec![0xff];
        cpu.load_and_run(program).unwrap();
    }
}
