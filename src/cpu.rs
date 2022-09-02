pub struct CPU {
    pub accumulator: u8,
    pub proc_status: u8,
    pub prog_counter: u16,
    pub reg_x: u8,
    pub reg_y: u8,

    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            accumulator: 0,
            proc_status: 0,
            prog_counter: 0,
            reg_x: 0,
            reg_y: 0,

            memory: [0; 0xFFFF],
            // [0x8000 .. 0xFFFF] is reserved for Program ROM
        }
    }
}

impl CPU {
    pub fn flag_zero(&self) -> bool {
        (self.proc_status & 0b0000_0010) != 0
    }
    pub fn flag_neg(&self) -> bool {
        (self.proc_status & 0b1000_0000) != 0
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

impl CPU {
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let low = self.mem_read(pos) as u16;
        let high = self.mem_read(pos + 1) as u16;
        (high << 8) | (low as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let high = (data >> 8) as u8;
        let low = (data & 0xff) as u8;
        self.mem_write(pos, low);
        self.mem_write(pos + 1, high);
    }
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

impl CPU {
    fn operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.prog_counter,
            AddressingMode::ZeroPage => self.mem_read(self.prog_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.prog_counter),

            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.prog_counter);
                let addr = pos.wrapping_add(self.reg_x) as u16;
                addr
            }

            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.prog_counter);
                let addr = pos.wrapping_add(self.reg_y) as u16;
                addr
            }

            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.prog_counter);
                let addr = base.wrapping_add(self.reg_x as u16);
                addr
            }

            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.prog_counter);
                let addr = base.wrapping_add(self.reg_y as u16);
                addr
            }

            AddressingMode::IndirectX => {
                let base = self.mem_read(self.prog_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.reg_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::IndirectY => {
                let base = self.mem_read(self.prog_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }
}

impl CPU {
    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.reg_x = 0;
        self.proc_status = 0;
        self.prog_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.prog_counter);
            self.prog_counter += 1;
            match opcode {
                0xa9 => {
                    self.lda(AddressingMode::Immediate);
                    self.prog_counter += 1;
                }
                0xa5 => {
                    self.lda(AddressingMode::ZeroPage);
                    self.prog_counter += 1;
                }
                0xad => {
                    self.lda(AddressingMode::Absolute);
                    self.prog_counter += 2;
                }

                0xaa => self.tax(),
                0xe8 => self.inx(),
                0x00 => return,
                _ => todo!(),
            }
        }
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }
}

impl CPU {
    fn update_flags_zero_and_neg(&mut self, val: u8) {
        // updating zero flag
        if val == 0 {
            self.proc_status = self.proc_status | 0b0000_0010;
        } else {
            self.proc_status = self.proc_status & 0b1111_1101;
        }

        // updating neg flag
        if val & 0b1000_0000 != 0 {
            self.proc_status = self.proc_status | 0b1000_0000;
        } else {
            self.proc_status = self.proc_status & 0b0111_1111;
        }
    }
}

impl CPU {
    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.operand_address(mode);
        let param = self.mem_read(addr);
        self.accumulator = param;
        self.update_flags_zero_and_neg(self.accumulator);
    }

    fn tax(&mut self) {
        self.reg_x = self.accumulator;
        self.update_flags_zero_and_neg(self.reg_x);
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_flags_zero_and_neg(self.reg_x);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_mem_u16() {
        let mut cpu = CPU::new();
        cpu.memory[0x0] = 0xef;
        cpu.memory[0x1] = 0xbe;
        assert_eq!(cpu.mem_read_u16(0x0), 0xbeef);
    }

    #[test]
    fn writes_mem_u16() {
        let mut cpu = CPU::new();
        cpu.mem_write_u16(0x0, 0xbeef);
        assert_eq!(cpu.memory[0x0], 0xef);
        assert_eq!(cpu.memory[0x1], 0xbe);
    }

    #[test]
    fn updates_flag_zero() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_and_neg(0);
        assert!(cpu.flag_zero());
    }

    #[test]
    fn updates_flag_neg() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_and_neg(0b1000_0000);
        assert!(cpu.flag_neg());
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
    fn lda_loads_data() {
        let mut cpu = CPU::new();
        cpu.lda(0x05);
        assert_eq!(cpu.accumulator, 0x05);
    }

    #[test]
    fn tax_moves_a_to_x() {
        let mut cpu = CPU::new();
        cpu.accumulator = 10;
        cpu.tax();
        assert_eq!(cpu.reg_x, 10);
    }

    #[test]
    fn inx_increments() {
        let mut cpu = CPU::new();
        cpu.inx();
        assert_eq!(cpu.reg_x, 1);
    }

    #[test]
    fn inx_overflows() {
        let mut cpu = CPU::new();
        cpu.reg_x = 0xff;
        cpu.inx();
        assert_eq!(cpu.reg_x, 0);
    }
}
