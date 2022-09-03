use super::memory;
use super::CPU;

bitflags::bitflags! {
    pub struct CPUStatus: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK1            = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

impl CPU {
    fn update_flags_zero_and_neg(&mut self, val: u8) {
        // updating zero flag
        if val == 0 {
            self.status.insert(CPUStatus::ZERO);
        } else {
            self.status.remove(CPUStatus::ZERO);
        }

        // updating neg flag
        if val & 0b1000_0000 != 0 {
            self.status.insert(CPUStatus::NEGATIVE);
        } else {
            self.status.remove(CPUStatus::NEGATIVE);
        }
    }

    fn set_accumulator(&mut self, value: u8) {
        self.a = value;
        self.update_flags_zero_and_neg(self.a);
    }

    fn add_to_accumulator(&mut self, data: u8) {
        // perform sum with carry bit
        let sum = self.a as u16
            + data as u16
            + (if self.status.contains(CPUStatus::CARRY) {
                1
            } else {
                0
            }) as u16;

        // check for carry
        if sum > 0xff {
            self.status.insert(CPUStatus::CARRY);
        } else {
            self.status.remove(CPUStatus::CARRY);
        }

        // check for overflow
        let result = sum as u8;
        if (data ^ result) & (result ^ self.a) & 0x80 != 0 {
            self.status.insert(CPUStatus::OVERFLOW);
        } else {
            self.status.remove(CPUStatus::OVERFLOW)
        }

        self.set_accumulator(result);
    }
}

impl CPU {
    /// # ADC - Add with Carry
    ///
    /// This instruction adds the contents of a memory location
    /// to the accumulator together with the carry bit. If overflow
    /// occurs the carry bit is set, this enables multiple byte addition
    /// to be performed.
    ///
    pub(super) fn adc(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_operand_address(mode);
        let param = self.mem_read_increment(addr);
        self.add_to_accumulator(param);
    }

    /// # AND - Logical AND
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    ///
    pub(super) fn and(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_operand_address(mode);
        let param = self.mem_read_increment(addr);
        self.set_accumulator(self.a & param);
    }

    /// # ASL - Arithmetic Shift Left
    ///
    /// This operation shifts all the bits of
    /// the accumulator or memory contents one
    /// bit left. Bit 0 is set to 0 and bit 7
    /// is placed in the carry flag. The effect
    /// of this operation is to multiply the memory
    /// contents by 2 (ignoring 2's complement considerations),
    /// setting the carry if the result will not fit in 8 bit
    ///
    pub(super) fn asl_on_accumulator(&mut self) {
        let param = self.a;
        if param & 0b1000_0000 != 0 {
            self.status.insert(CPUStatus::CARRY);
        } else {
            self.status.remove(CPUStatus::CARRY);
        }
        self.set_accumulator(param << 1);
    }

    /// # ASL - Arithmetic Shift Left
    ///
    /// This operation shifts all the bits of
    /// the accumulator or memory contents one
    /// bit left. Bit 0 is set to 0 and bit 7
    /// is placed in the carry flag. The effect
    /// of this operation is to multiply the memory
    /// contents by 2 (ignoring 2's complement considerations),
    /// setting the carry if the result will not fit in 8 bit
    ///
    pub(super) fn asl(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_operand_address(mode);
        let param = self.mem_read(addr);
        if param & 0b1000_0000 != 0 {
            self.status.insert(CPUStatus::CARRY);
        } else {
            self.status.remove(CPUStatus::CARRY);
        }
        self.set_accumulator(param << 1);
    }

    /// # Branch
    ///
    /// If the condition is true then add
    /// the relative displacement to the program
    /// counter to cause a branch to a new location.
    ///
    /// Used in:
    /// - BCC - Branch if Carry Clear
    ///
    pub(super) fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.counter) as i8;
            let jump_addr = self.counter.wrapping_add(1).wrapping_add(jump as u16);
            self.counter = jump_addr;
        }
    }

    /// # BIT - Bit Test
    ///
    /// This instructions is used to test if one or more
    /// bits are set in a target memory location. The mask
    /// pattern in A is ANDed with the value in memory to
    /// set or clear the zero flag, but the result is not
    /// kept. Bits 7 and 6 of the value from memory are copied
    /// into the N and V flags.
    ///
    /// TODO
    // pub(super) fn bit(&mut self, mode: memory::AddressingMode) {
    //     let addr = self.get_operand_address(mode);
    //     let data = self.mem_read(addr);
    //     let and = self.a & data;
    //     if and == 0 {
    //         self.status.insert(CPUStatus::ZERO);
    //     } else {
    //         self.status.remove(CPUStatus::ZERO);
    //     }
    //     self.status.set(CPUStatus::NEGATIVE, data & 0b1000_0000 > 0);
    //     self.status.set(CPUStatus::OVERFLOW, data & 0b0100_0000 > 0);
    // }

    pub(super) fn lda(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_operand_address(mode);
        let param = self.mem_read_increment(addr);
        self.set_accumulator(param);
    }

    pub(super) fn tax(&mut self) {
        self.x = self.a;
        self.update_flags_zero_and_neg(self.x);
    }

    pub(super) fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_flags_zero_and_neg(self.x);
    }

    pub(super) fn sta(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.a);
        self.counter += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::vec;

    #[test]
    fn updates_flag_zero() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_and_neg(0);
        assert!(cpu.status.contains(CPUStatus::ZERO));
    }

    #[test]
    fn updates_flag_neg() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_and_neg(0b1000_0000);
        assert!(cpu.status.contains(CPUStatus::NEGATIVE));
    }

    #[test]
    fn updates_overflow_flag_on_accumulator_add() {
        let mut cpu = CPU::new();
        cpu.a = 0x7f;
        cpu.add_to_accumulator(1);
        assert!(cpu.status.contains(CPUStatus::OVERFLOW));
        assert!(!cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn updates_carry_flag_on_accumulator_add() {
        let mut cpu = CPU::new();
        cpu.a = 0xff;
        cpu.add_to_accumulator(1);
        assert!(cpu.status.contains(CPUStatus::CARRY));
        assert!(!cpu.status.contains(CPUStatus::OVERFLOW));
    }

    #[test]
    fn adc_adds_with_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x1]);
        cpu.reset();
        cpu.status.insert(CPUStatus::CARRY);
        cpu.adc(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 2);
    }

    #[test]
    fn and_ands() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAA]);
        cpu.reset();
        cpu.a = 0x55;
        cpu.and(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x0);
    }

    #[test]
    fn asl_on_accumulator_shifts_and_clears_carry() {
        let mut cpu = CPU::new();
        cpu.a = 0x1;
        cpu.asl_on_accumulator();
        assert_eq!(cpu.a, 0x2);
        assert!(!cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn asl_on_accumulator_sets_carry() {
        let mut cpu = CPU::new();
        cpu.a = 0x80;
        cpu.asl_on_accumulator();
        assert!(cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn asl_shifts_and_clears_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x1]);
        cpu.reset();
        cpu.asl(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x2);
        assert!(!cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn asl_sets_carry() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x80]);
        cpu.reset();
        cpu.asl(memory::AddressingMode::Immediate);
        assert!(cpu.status.contains(CPUStatus::CARRY));
    }

    #[test]
    fn branch_branches() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa]);
        cpu.reset();
        cpu.branch(true);
        assert_eq!(cpu.counter, 0x800b);
    }

    #[test]
    fn lda_loads_data() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x05]);
        cpu.reset();
        cpu.lda(memory::AddressingMode::Immediate);
        assert_eq!(cpu.counter, 0x8001);
        assert_eq!(cpu.a, 0x05);
    }

    #[test]
    fn tax_moves_a_to_x() {
        let mut cpu = CPU::new();
        cpu.a = 10;
        cpu.tax();
        assert_eq!(cpu.x, 10);
    }

    #[test]
    fn inx_increments() {
        let mut cpu = CPU::new();
        cpu.inx();
        assert_eq!(cpu.x, 1);
    }

    #[test]
    fn inx_overflows() {
        let mut cpu = CPU::new();
        cpu.x = 0xff;
        cpu.inx();
        assert_eq!(cpu.x, 0);
    }

    #[test]
    fn sta_copies_from_a_to_mem() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa]);
        cpu.reset();
        cpu.a = 0xbe;
        cpu.sta(memory::AddressingMode::ZeroPage);
        assert_eq!(cpu.memory[0xaa], cpu.a);
    }
}
