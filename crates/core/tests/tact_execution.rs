use k580_core::{CoreError, Cpu8080State, DecodeError, NullBus};

fn put_program(cpu: &mut Cpu8080State, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
}

fn step_tacts(cpu: &mut Cpu8080State, bus: &mut NullBus, count: u8) {
    for _ in 0..count {
        cpu.step_tact(bus).unwrap();
    }
}

#[test]
fn step_tact_defers_nop_until_instruction_boundary() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]);
    let mut bus = NullBus::default();

    let first = cpu.step_tact(&mut bus).unwrap();
    assert_eq!(first.tact_phase, 0);
    assert!(!first.instruction_boundary);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycle_count, 1);

    step_tacts(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.tact_phase, Some(3));

    let last = cpu.step_tact(&mut bus).unwrap();
    assert_eq!(last.tact_phase, 3);
    assert!(last.instruction_boundary);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.tact_phase, None);
}

#[test]
fn step_tact_delays_port_output_until_final_tact() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xD3, 0x04]);
    cpu.registers.a = 0x77;
    let mut bus = NullBus::default();

    step_tacts(&mut cpu, &mut bus, 9);
    assert_eq!(cpu.pc, 0);
    assert!(bus.writes().is_empty());

    let last = cpu.step_tact(&mut bus).unwrap();
    assert!(last.instruction_boundary);
    assert_eq!(cpu.pc, 2);
    assert_eq!(bus.writes(), &[(0x04, 0x77)]);
}

#[test]
fn step_instruction_completes_active_tact_walk_once() {
    let mut atomic = Cpu8080State::default();
    put_program(&mut atomic, &[0xD3, 0x04]);
    atomic.registers.a = 0x77;
    let mut atomic_bus = NullBus::default();
    atomic.step_instruction(&mut atomic_bus).unwrap();

    let mut mixed = Cpu8080State::default();
    put_program(&mut mixed, &[0xD3, 0x04]);
    mixed.registers.a = 0x77;
    let mut mixed_bus = NullBus::default();

    step_tacts(&mut mixed, &mut mixed_bus, 4);
    assert_eq!(mixed.pc, 0);
    assert!(mixed_bus.writes().is_empty());

    mixed.step_instruction(&mut mixed_bus).unwrap();
    assert_eq!(mixed, atomic);
    assert_eq!(mixed_bus.writes(), atomic_bus.writes());
}

#[test]
fn interrupt_during_tact_walk_waits_for_the_next_instruction_boundary() {
    for finish_instruction in [false, true] {
        for split in 1..5 {
            let mut cpu = Cpu8080State::default();
            put_program(&mut cpu, &[0x3C]);
            cpu.sp = 0x9000;
            cpu.interrupt_enable = true;
            let mut bus = NullBus::default();
            step_tacts(&mut cpu, &mut bus, split);
            cpu.request_interrupt(0xCF);
            if finish_instruction {
                cpu.step_instruction(&mut bus).unwrap();
            } else {
                step_tacts(&mut cpu, &mut bus, 5 - split);
            }
            assert_eq!(
                (cpu.registers.a, cpu.pc, cpu.cycle_count, cpu.sp),
                (1, 1, 5, 0x9000),
                "split={split}, finish_instruction={finish_instruction}",
            );
            assert!(cpu.interrupt_request_pending);
            let interrupt = cpu.step_instruction_metadata(&mut bus).unwrap();
            assert!(interrupt.interrupt_accepted);
            assert_eq!(interrupt.t_states, 11);
            assert_eq!(
                (cpu.pc, cpu.cycle_count, cpu.memory.read_word(cpu.sp)),
                (8, 16, 1),
            );
        }
    }
}

#[test]
fn invalid_interrupt_preserves_state_until_replaced() {
    for tact_steps in [false, true] {
        for halted in [false, true] {
            for vector in [0x00, 0x08, 0x76, 0xCD] {
                let mut cpu = Cpu8080State::default();
                cpu.pc = 0x1234;
                cpu.sp = 0x9000;
                cpu.halted = halted;
                cpu.cycle_count = 23;
                cpu.interrupt_enable = true;
                cpu.request_interrupt(vector);
                let before = cpu.clone();
                let mut bus = NullBus::default();

                for _ in 0..2 {
                    let result = if tact_steps {
                        cpu.step_tact(&mut bus).map(|_| ())
                    } else {
                        cpu.step_instruction(&mut bus).map(|_| ())
                    };
                    assert_eq!(
                        result,
                        Err(CoreError::Decode(DecodeError::InvalidInterruptVector(
                            vector
                        )))
                    );
                    assert_eq!(
                        cpu, before,
                        "vector={vector:02X}, tact_steps={tact_steps}, halted={halted}"
                    );
                }

                cpu.request_interrupt(0xCF);
                if tact_steps {
                    step_tacts(&mut cpu, &mut bus, 11);
                } else {
                    let outcome = cpu.step_instruction_metadata(&mut bus).unwrap();
                    assert!(outcome.interrupt_accepted);
                    assert_eq!(outcome.t_states, 11);
                }
                assert_eq!((cpu.pc, cpu.sp, cpu.cycle_count), (8, 0x8FFE, 34));
                assert_eq!(cpu.memory.read_word(cpu.sp), 0x1234);
                assert!(!cpu.interrupt_request_pending);
                assert_eq!(cpu.interrupt_vector_byte, None);
                assert!(!cpu.interrupt_enable);
                assert!(!cpu.halted);
            }
        }
    }
}

#[test]
fn interrupt_tact_walk_matches_step_instruction() {
    let mut atomic = Cpu8080State::default();
    atomic.sp = 0x9000;
    atomic.pc = 0x1234;
    atomic.interrupt_enable = true;
    atomic.request_interrupt(0xCF);
    atomic.step_instruction(&mut NullBus::default()).unwrap();

    let mut walked = Cpu8080State::default();
    walked.sp = 0x9000;
    walked.pc = 0x1234;
    walked.interrupt_enable = true;
    walked.request_interrupt(0xCF);
    step_tacts(&mut walked, &mut NullBus::default(), 4);
    let mut restored = Cpu8080State::default();
    restored.apply_metadata(walked.metadata());
    step_tacts(&mut restored, &mut NullBus::default(), 7);

    assert_eq!(restored, atomic);
}

#[test]
fn full_tact_walk_matches_step_instruction_for_representative_opcodes() {
    let programs: &[(&[u8], u8)] = &[
        (&[0x00], 4),
        (&[0x01, 0x34, 0x12], 10),
        (&[0x32, 0x00, 0x20], 13),
        (&[0xD3, 0x04], 10),
        (&[0xDB, 0x03], 10),
        (&[0xCD, 0x00, 0x10], 17),
        (&[0x76], 7),
    ];

    for &(program, t_states) in programs {
        let mut atomic = Cpu8080State::default();
        put_program(&mut atomic, program);
        atomic.sp = 0x9000;
        atomic.registers.a = 0x66;
        let mut atomic_bus = NullBus::default();
        atomic_bus.set_input(0x03, 0xA5);
        atomic.step_instruction(&mut atomic_bus).unwrap();

        let mut walked = Cpu8080State::default();
        put_program(&mut walked, program);
        walked.sp = 0x9000;
        walked.registers.a = 0x66;
        let mut walked_bus = NullBus::default();
        walked_bus.set_input(0x03, 0xA5);
        step_tacts(&mut walked, &mut walked_bus, t_states);

        assert_eq!(walked, atomic, "program {program:02X?}");
        assert_eq!(
            walked_bus.writes(),
            atomic_bus.writes(),
            "program {program:02X?}"
        );
    }
}

#[test]
fn conditional_call_tact_count_follows_branch_state() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0xCC, 0x00, 0x10]);
    cpu.flags.zero = false;
    let mut bus = NullBus::default();

    step_tacts(&mut cpu, &mut bus, 10);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.tact_phase, Some(10));

    let last = cpu.step_tact(&mut bus).unwrap();
    assert!(last.instruction_boundary);
    assert_eq!(cpu.pc, 3);

    let mut taken = Cpu8080State::default();
    put_program(&mut taken, &[0xCC, 0x00, 0x10]);
    taken.sp = 0x9000;
    taken.flags.zero = true;
    let mut bus = NullBus::default();

    step_tacts(&mut taken, &mut bus, 16);
    assert_eq!(taken.pc, 0);
    assert_eq!(taken.tact_phase, Some(16));

    let last = taken.step_tact(&mut bus).unwrap();
    assert!(last.instruction_boundary);
    assert_eq!(taken.pc, 0x1000);
}
