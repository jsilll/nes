use nes::cpu::CPU;

fn main() {
    let cpu = CPU::new();
    print!("{}", cpu.flag_zero());
}
