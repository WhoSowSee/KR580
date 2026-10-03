use super::*;

#[test]
fn in_and_out_route_through_bus() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xDB, 0x03, 0xD3, 0x04]);
    let mut bus = NullBus::default();
    bus.set_input(0x03, 0x77);
    cpu.step_instruction(&mut bus).unwrap();
    assert_eq!(cpu.get_register(RegisterName::A), 0x77);
    cpu.step_instruction(&mut bus).unwrap();
    assert_eq!(bus.writes(), &[(0x04, 0x77)]);
}

#[test]
fn ei_delay_di_and_interrupt_acceptance_follow_prompt() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xFB, 0x00, 0xF3]);
    step(&mut cpu);
    assert!(!cpu.interrupt_enable);
    assert!(cpu.interrupt_enable_pending);
    step(&mut cpu);
    assert!(cpu.interrupt_enable);
    assert!(!cpu.interrupt_enable_pending);
    step(&mut cpu);
    assert!(!cpu.interrupt_enable);
    assert!(!cpu.interrupt_enable_pending);

    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x76]);
    cpu.sp = 0x9000;
    cpu.interrupt_enable = true;
    step(&mut cpu);
    assert!(cpu.halted);
    cpu.request_interrupt(0xCF);
    let out = cpu.step_instruction(&mut NullBus::default()).unwrap();
    assert!(out.interrupt_accepted);
    assert!(!cpu.halted);
    assert!(!cpu.interrupt_enable);
    assert_eq!(cpu.pc, 0x08);
    assert_eq!(cpu.memory.read_word(cpu.sp), 1);
}
