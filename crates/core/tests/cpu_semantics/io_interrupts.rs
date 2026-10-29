use super::*;

#[test]
fn in_and_out_route_through_bus() {
    for tact_steps in [false, true] {
        for (input_port, output_port, value) in [(0x03, 0x04, 0xA5), (0x00, 0xFF, 0x5A)] {
            let mut cpu = Cpu8080State::default();
            put_program(&mut cpu, &[0xDB, input_port, 0xD3, output_port]);
            let mut bus = NullBus::default();
            bus.set_input(input_port, value);

            for (opcode, port) in [(0xDB, input_port), (0xD3, output_port)] {
                if tact_steps {
                    for _ in 0..10 {
                        cpu.step_tact(&mut bus).unwrap();
                    }
                } else {
                    cpu.step_instruction(&mut bus).unwrap();
                }
                assert_eq!(
                    (
                        cpu.last_fetched_opcode,
                        cpu.last_address_bus,
                        cpu.last_data_bus_byte
                    ),
                    (opcode, u16::from_le_bytes([port, port]), value),
                    "opcode={opcode:02X}, tact_steps={tact_steps}"
                );
                assert_eq!(cpu.get_register(RegisterName::A), value);
            }
            assert_eq!(bus.writes(), &[(output_port, value)]);
            assert_eq!((cpu.pc, cpu.cycle_count), (4, 20));
        }
    }
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
