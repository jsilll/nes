use super::CPU;

#[derive(Debug)]
#[allow(dead_code)]
pub(super) enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

impl CPU {
    pub(super) fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub(super) fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub(super) fn mem_read_u16(&self, addr: u16) -> u16 {
        let low = self.mem_read(addr) as u16;
        let high = self.mem_read(addr + 1) as u16;
        (high << 8) | (low as u16)
    }

    pub(super) fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let high = (data >> 8) as u8;
        let low = (data & 0xff) as u8;
        self.mem_write(addr, low);
        self.mem_write(addr + 1, high);
    }

    pub(super) fn operand_address(&self, mode: AddressingMode) -> u16 {
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
                let addr: u8 = base.wrapping_add(self.reg_x);
                let lo = self.mem_read(addr as u16);
                let hi = self.mem_read(addr.wrapping_add(1) as u16);
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
        }
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
    fn operand_address_immediate() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xA]);
        cpu.reset();
        assert_eq!(
            cpu.prog_counter,
            cpu.operand_address(AddressingMode::Immediate)
        );
    }

    #[test]
    fn operand_address_zero_page() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa]);
        cpu.reset();
        assert_eq!(cpu.operand_address(AddressingMode::ZeroPage), 0x00aa);
    }

    #[test]
    fn operand_address_zero_page_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa]);
        cpu.reset();
        cpu.reg_x = 1;
        assert_eq!(cpu.operand_address(AddressingMode::ZeroPageX), 0x000b);
    }

    #[test]
    fn operand_address_zero_page_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa]);
        cpu.reset();
        cpu.reg_y = 1;
        assert_eq!(cpu.operand_address(AddressingMode::ZeroPageY), 0x000b);
    }

    #[test]
    fn operand_address_absolute() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa]);
        cpu.reset();
        assert_eq!(cpu.operand_address(AddressingMode::Absolute), 0x00aa);
    }

    #[test]
    fn operand_address_absolute_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xef, 0xbe]);
        cpu.reset();
        cpu.reg_x = 1;
        assert_eq!(cpu.operand_address(AddressingMode::AbsoluteX), 0xbef0);
    }

    #[test]
    fn operand_address_absolute_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xef, 0xbe]);
        cpu.reset();
        cpu.reg_y = 1;
        assert_eq!(cpu.operand_address(AddressingMode::AbsoluteY), 0xbef0);
    }

    #[test]
    fn operand_address_indirect_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![]);
        cpu.reset();
        cpu.reg_x = 1;
        cpu.memory[0x8000] = 0xde;
        cpu.memory[0x00df] = 0xef;
        cpu.memory[0x00e0] = 0xbe;
        assert_eq!(cpu.operand_address(AddressingMode::IndirectX), 0xbeef);
    }

    #[test]
    fn operand_address_indirect_y() {
        let mut cpu = CPU::new();
        cpu.load(vec![]);
        cpu.reset();
        cpu.reg_y = 1;
        cpu.memory[0x8000] = 0xde;
        cpu.memory[0x00de] = 0xef;
        cpu.memory[0x00df] = 0xbe;
        assert_eq!(cpu.operand_address(AddressingMode::IndirectY), 0xbef0);
    }
}
