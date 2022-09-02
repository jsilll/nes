use nes::cpu::CPU;
use std::process;

fn main() {
    let mut cpu = CPU::new();
    if let Err(msg) = cpu.load_and_run(vec![0xAA, 0x00]) {
        eprintln!("Application error: {}", msg);
        process::exit(1);
    }
}
