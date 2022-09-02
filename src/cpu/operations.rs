use super::memory;
use super::CPU;

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
    pub(super) fn lda(&mut self, mode: memory::AddressingMode) {
        let addr = self.operand_address(mode);
        let param = self.mem_read(addr);
        self.accumulator = param;
        self.prog_counter += 1;
        self.update_flags_zero_and_neg(self.accumulator);
    }

    pub(super) fn tax(&mut self) {
        self.reg_x = self.accumulator;
        self.update_flags_zero_and_neg(self.reg_x);
    }

    pub(super) fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_flags_zero_and_neg(self.reg_x);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl CPU {
        fn flag_zero(&self) -> bool {
            (self.proc_status & 0b0000_0010) != 0
        }
        fn flag_neg(&self) -> bool {
            (self.proc_status & 0b1000_0000) != 0
        }
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
    fn lda_loads_data() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x05]);
        cpu.reset();
        cpu.lda(memory::AddressingMode::Immediate);
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
