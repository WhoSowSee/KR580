use super::*;

#[test]
fn shld_lhld_xchg_and_xthl_roundtrip_memory_and_pairs() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x22, 0x00, 0x20, 0x2A, 0x00, 0x20, 0xEB, 0xE3]);
    cpu.registers.h = 0x12;
    cpu.registers.l = 0x34;
    cpu.registers.d = 0xAB;
    cpu.registers.e = 0xCD;
    cpu.sp = 0x3000;
    cpu.memory.write(0x3000, 0x78);
    cpu.memory.write(0x3001, 0x56);

    step(&mut cpu);
    assert_eq!(cpu.memory.read(0x2000), 0x34);
    assert_eq!(cpu.memory.read(0x2001), 0x12);

    cpu.registers.h = 0;
    cpu.registers.l = 0;
    step(&mut cpu);
    assert_eq!(cpu.registers.h, 0x12);
    assert_eq!(cpu.registers.l, 0x34);

    step(&mut cpu);
    assert_eq!(cpu.registers.d, 0x12);
    assert_eq!(cpu.registers.e, 0x34);
    assert_eq!(cpu.registers.h, 0xAB);
    assert_eq!(cpu.registers.l, 0xCD);

    step(&mut cpu);
    assert_eq!(cpu.registers.h, 0x56);
    assert_eq!(cpu.registers.l, 0x78);
    assert_eq!(cpu.memory.read(0x3000), 0xCD);
    assert_eq!(cpu.memory.read(0x3001), 0xAB);
}
