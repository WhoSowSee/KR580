use super::*;

#[test]
fn all_opcode_slots_are_classified() {
    let undocumented = [
        0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD,
    ];
    for opcode in 0u8..=255 {
        let decoded = decode_opcode(opcode);
        if undocumented.contains(&opcode) {
            assert!(
                matches!(decoded, Err(DecodeError::UndocumentedOpcode(found)) if found == opcode)
            );
        } else {
            let info = decoded.unwrap();
            assert_eq!(info.opcode, opcode);
            assert!((1..=3).contains(&info.size));
            assert!(info.timing.t_states_taken > 0);
            if let Some(not_taken) = info.timing.t_states_not_taken {
                assert!(not_taken > 0);
            }
        }
    }
}

#[test]
fn every_documented_opcode_executes_from_controlled_state() {
    for opcode in 0u8..=255 {
        if decode_opcode(opcode).is_err() {
            continue;
        }
        let mut cpu = Cpu8080State::default();
        cpu.sp = 0x4000;
        cpu.registers.a = 0x12;
        cpu.registers.b = 0x34;
        cpu.registers.c = 0x56;
        cpu.registers.d = 0x78;
        cpu.registers.e = 0x9A;
        cpu.registers.h = 0x20;
        cpu.registers.l = 0x00;
        cpu.memory.write(0x2000, 0x5A);
        cpu.memory.write(cpu.sp, 0xCD);
        cpu.memory.write(cpu.sp.wrapping_add(1), 0xAB);
        put_program(&mut cpu, &[opcode, 0x34, 0x12]);
        let mut bus = NullBus::default();
        bus.set_input(0x34, 0xA5);
        let result = cpu.step_instruction(&mut bus);
        assert!(result.is_ok(), "opcode {opcode:#04X} failed: {result:?}");
    }
}

#[test]
fn undocumented_opcode_returns_decode_error_and_does_not_advance_pc() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0, 0x08);
    let mut bus = NullBus::default();
    let err = cpu.step_instruction(&mut bus).unwrap_err();
    assert!(matches!(
        err,
        CoreError::Decode(DecodeError::UndocumentedOpcode(0x08))
    ));
    assert_eq!(cpu.pc, 0);
}

#[test]
fn psw_materialization_forces_reserved_bits() {
    let flags = Flags {
        sign: true,
        zero: false,
        auxiliary_carry: true,
        parity: true,
        carry: true,
    };
    assert_eq!(flags.to_psw(), 0b1001_0111);
    assert_eq!(Flags::from_psw(0xFF).to_psw(), 0b1101_0111);
}
