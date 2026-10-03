use super::*;

#[test]
fn conditional_jumps_use_normal_carry_meanings() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xDA, 0x34, 0x12, 0xD2, 0x78, 0x56]);
    cpu.flags.carry = true;
    let out = cpu.step_instruction(&mut NullBus::default()).unwrap();
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(out.t_states, 10);

    cpu.pc = 3;
    cpu.flags.carry = false;
    cpu.step_instruction(&mut NullBus::default()).unwrap();
    assert_eq!(cpu.pc, 0x5678);
}

#[test]
fn conditional_calls_use_documented_taken_and_not_taken_timing() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xDC, 0x00, 0x10]);
    cpu.sp = 0x8000;
    cpu.flags.carry = false;
    let out = cpu.step_instruction(&mut NullBus::default()).unwrap();
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.sp, 0x8000);
    assert_eq!(out.t_states, 11);

    cpu.pc = 0;
    cpu.flags.carry = true;
    let out = cpu.step_instruction(&mut NullBus::default()).unwrap();
    assert_eq!(cpu.pc, 0x1000);
    assert_eq!(cpu.sp, 0x7FFE);
    assert_eq!(cpu.memory.read_word(cpu.sp), 3);
    assert_eq!(out.t_states, 17);
}

#[test]
fn call_ret_and_stack_roundtrip() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xCD, 0x00, 0x10]);
    cpu.memory.write(0x1000, 0xC9);
    cpu.sp = 0x8000;
    step(&mut cpu);
    assert_eq!(cpu.pc, 0x1000);
    assert_eq!(cpu.sp, 0x7FFE);
    assert_eq!(cpu.memory.read_word(cpu.sp), 3);
    step(&mut cpu);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.sp, 0x8000);
}

#[test]
fn push_pop_psw_roundtrips_flags_with_reserved_bits_normalized() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xF5, 0xF1]);
    cpu.sp = 0x9000;
    cpu.registers.a = 0xA5;
    cpu.flags = Flags {
        sign: true,
        zero: true,
        auxiliary_carry: true,
        parity: false,
        carry: true,
    };
    step(&mut cpu);
    cpu.registers.a = 0;
    cpu.flags = Flags::default();
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0xA5);
    assert_eq!(cpu.flags.to_psw(), 0b1101_0011);
}
