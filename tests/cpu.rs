use nes::cpu::CPU;

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    let program = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
    cpu.load_and_run(program).unwrap();
    assert_eq!(cpu.x, 0xc1)
}
