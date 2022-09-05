use super::memory;
use super::Flags;
use super::CPU;

impl CPU {
    /// Updates zero and negative flags with a given value
    fn update_flags_zero_neg(&mut self, val: u8) {
        self.flags.set(Flags::ZERO, val == 0);
        self.flags.set(Flags::NEGATIVE, val & 0b1000_0000 != 0);
    }

    /// Updates the accumulator with a given value
    /// Also updates zero and negative flags
    fn set_a(&mut self, value: u8) {
        self.a = value;
        self.update_flags_zero_neg(self.a);
    }

    /// Checks for carry and overflow when adding
    /// to the accumulator's current value
    /// After those checks, calls set_a()
    fn add_to_a(&mut self, data: u8) {
        // perform sum with carry bit
        let sum = self.a as u16
            + data as u16
            + (if self.flags.contains(Flags::CARRY) {
                1
            } else {
                0
            }) as u16;

        // check for carry
        self.flags.set(Flags::CARRY, sum > 0xff);

        // check for overflow
        let result = sum as u8;
        self.flags.set(
            Flags::OVERFLOW,
            (data ^ result) & (result ^ self.a) & 0x80 != 0,
        );

        self.set_a(result);
    }
}

impl CPU {
    /// ## ADC - Add with Carry
    ///
    /// This instruction adds the contents of a memory location
    /// to the accumulator together with the carry bit. If overflow
    /// occurs the carry bit is set, this enables multiple byte addition
    /// to be performed.
    pub(super) fn adc(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        let param = self.mem_read_incr(addr);
        self.add_to_a(param);
    }

    /// ## AND - Logical AND
    ///
    /// A logical AND is performed, bit by bit, on the accumulator contents
    /// using the contents of a byte of memory.
    pub(super) fn and(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        let param = self.mem_read_incr(addr);
        self.set_a(self.a & param);
    }

    /// ## ASL - Arithmetic Shift Left
    ///
    /// This operation shifts all the bits of
    /// the accumulator or memory contents one
    /// bit left. Bit 0 is set to 0 and bit 7
    /// is placed in the carry flag. The effect
    /// of this operation is to multiply the memory
    /// contents by 2 (ignoring 2's complement considerations),
    /// setting the carry if the result will not fit in 8 bit
    pub(super) fn asl_on_accumulator(&mut self) {
        let param = self.a;
        self.flags.set(Flags::CARRY, param & 0b1000_0000 != 0);
        self.set_a(param << 1);
    }

    /// ## ASL - Arithmetic Shift Left
    ///
    /// This operation shifts all the bits of
    /// the accumulator or memory contents one
    /// bit left. Bit 0 is set to 0 and bit 7
    /// is placed in the carry flag. The effect
    /// of this operation is to multiply the memory
    /// contents by 2 (ignoring 2's complement considerations),
    /// setting the carry if the result will not fit in 8 bit
    pub(super) fn asl(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        let param = self.mem_read_incr(addr);
        self.flags.set(Flags::CARRY, param & 0b1000_0000 != 0);
        self.set_a(param << 1);
    }

    /// ## Branch
    ///
    /// If the condition is true then add
    /// the relative displacement to the program
    /// counter to cause a branch to a new location.
    ///
    /// Used in:
    /// - BCC - Branch if Carry Clear
    pub(super) fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read_incr(self.counter) as i8;
            let jump_addr = self.counter.wrapping_add(1).wrapping_add(jump as u16);
            self.counter = jump_addr;
        }
    }

    /// ## BIT - Bit Test
    ///
    /// This instructions is used to test if one or more
    /// bits are set in a target memory location. The mask
    /// pattern in A is ANDed with the value in memory to
    /// set or clear the zero flag, but the result is not
    /// kept. Bits 7 and 6 of the value from memory are copied
    /// into the N and V flags.
    pub(super) fn bit(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        let data = self.mem_read_incr(addr);
        self.a &= data;
        self.flags.set(Flags::ZERO, self.a == 0);
        self.flags.set(Flags::NEGATIVE, (data & 0b1000_0000) != 0);
        self.flags.set(Flags::OVERFLOW, (data & 0b0100_0000) != 0);
    }

    /// ## CLC - Clear Carry Flag
    ///
    /// Set the carry flag to zero.
    pub(super) fn clc(&mut self) {
        self.flags.remove(Flags::CARRY);
    }

    /// ## CLC - Clear Carry Flag
    ///
    /// Set the carry flag to zero.
    pub(super) fn cld(&mut self) {
        self.flags.remove(Flags::DECIMAL);
    }

    /// ## CLI - Clear Interrupt Disable
    ///
    /// Clears the interrupt disable flag allowing normal
    /// interrupt requests to be serviced.
    pub(super) fn cli(&mut self) {
        self.flags.remove(Flags::NO_INTERRUPT);
    }

    /// ## CLI - Clear Interrupt Disable
    ///
    /// Clears the interrupt disable flag allowing normal
    /// interrupt requests to be serviced.
    pub(super) fn clv(&mut self) {
        self.flags.remove(Flags::OVERFLOW);
    }

    /// ## Compare
    ///
    /// This instruction compares the contents of a
    /// register with another memory held value and
    /// sets the zero and carry flags as appropriate.
    ///
    /// Used in:
    /// - CMP - Compare
    pub(super) fn cmp(&mut self, mode: memory::AddressingMode, compare_with: u8) {
        let addr = self.get_oper_addr(mode);
        let data = self.mem_read_incr(addr);
        self.flags.set(Flags::CARRY, data <= compare_with);
        self.update_flags_zero_neg(compare_with.wrapping_sub(data));
    }

    pub(super) fn lda(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        let param = self.mem_read_incr(addr);
        self.set_a(param);
    }

    pub(super) fn tax(&mut self) {
        self.x = self.a;
        self.update_flags_zero_neg(self.x);
    }

    pub(super) fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_flags_zero_neg(self.x);
    }

    pub(super) fn sta(&mut self, mode: memory::AddressingMode) {
        let addr = self.get_oper_addr(mode);
        self.mem_write(addr, self.a);
        self.counter += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::vec;

    #[test]
    fn updates_zero_flag() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_neg(0);
        assert!(cpu.flags.contains(Flags::ZERO));
    }

    #[test]
    fn updates_neg_flag() {
        let mut cpu = CPU::new();
        cpu.update_flags_zero_neg(0b1000_0000);
        assert!(cpu.flags.contains(Flags::NEGATIVE));
    }

    #[test]
    fn updates_overflow_flag_on_accumulator_add() {
        let mut cpu = CPU::new();
        cpu.a = 0x7f;
        cpu.add_to_a(1);
        assert!(cpu.flags.contains(Flags::OVERFLOW));
        assert!(!cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn updates_carry_flag_on_accumulator_add() {
        let mut cpu = CPU::new();
        cpu.a = 0xff;
        cpu.add_to_a(1);
        assert!(cpu.flags.contains(Flags::CARRY));
        assert!(!cpu.flags.contains(Flags::OVERFLOW));
    }

    #[test]
    fn adc_adds_with_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x1]);
        cpu.reset();
        cpu.flags.insert(Flags::CARRY);
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
    fn asl_on_accumulator_shifts_and_clears_carry_flag() {
        let mut cpu = CPU::new();
        cpu.a = 0x1;
        cpu.asl_on_accumulator();
        assert_eq!(cpu.a, 0x2);
        assert!(!cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn asl_on_accumulator_sets_carry_flag() {
        let mut cpu = CPU::new();
        cpu.a = 0x80;
        cpu.asl_on_accumulator();
        assert!(cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn asl_shifts_and_clears_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x1]);
        cpu.reset();
        cpu.asl(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x2);
        assert!(!cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn asl_sets_carry_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x80]);
        cpu.reset();
        cpu.asl(memory::AddressingMode::Immediate);
        assert!(cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn branch_branches() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xa]);
        cpu.reset();
        cpu.branch(true);
        assert_eq!(cpu.counter, 0x800c);
    }

    #[test]
    fn bit_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa]);
        cpu.reset();
        cpu.a = 0x55;
        cpu.bit(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x0);
        assert!(cpu.flags.contains(Flags::ZERO));
    }

    #[test]
    fn bit_sets_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x80]);
        cpu.reset();
        cpu.a = 0x0;
        cpu.bit(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x0);
        assert!(cpu.flags.contains(Flags::NEGATIVE));
    }

    #[test]
    fn bit_sets_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x40]);
        cpu.reset();
        cpu.a = 0x40;
        cpu.bit(memory::AddressingMode::Immediate);
        assert_eq!(cpu.a, 0x40);
        assert!(cpu.flags.contains(Flags::OVERFLOW));
    }

    #[test]
    fn clc_clears_carry_flag() {
        let mut cpu = CPU::new();
        cpu.flags.insert(Flags::CARRY);
        cpu.clc();
        assert!(!cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn cld_clears_decimal_flag() {
        let mut cpu = CPU::new();
        cpu.flags.insert(Flags::DECIMAL);
        cpu.cld();
        assert!(!cpu.flags.contains(Flags::DECIMAL));
    }

    #[test]
    fn cli_clears_no_interrupt_flag() {
        let mut cpu = CPU::new();
        cpu.flags.insert(Flags::NO_INTERRUPT);
        cpu.cli();
        assert!(!cpu.flags.contains(Flags::NO_INTERRUPT));
    }

    #[test]
    fn clv_clears_overflow_flag() {
        let mut cpu = CPU::new();
        cpu.flags.insert(Flags::OVERFLOW);
        cpu.clv();
        assert!(!cpu.flags.contains(Flags::OVERFLOW));
    }

    #[test]
    fn compare_sets_carry_when_less() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x1]);
        cpu.reset();
        cpu.cmp(memory::AddressingMode::Immediate, 0x2);
        assert!(cpu.flags.contains(Flags::CARRY));
    }

    #[test]
    fn compare_clears_carry_when_greater() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x2]);
        cpu.reset();
        cpu.flags.insert(Flags::CARRY);
        cpu.cmp(memory::AddressingMode::Immediate, 0x1);
        assert!(!cpu.flags.contains(Flags::CARRY));
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
