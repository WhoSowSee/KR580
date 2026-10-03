use k580_core::{Cpu8080State, NullBus, RegisterName};

fn run_until_halt(cpu: &mut Cpu8080State) {
    let executed = cpu.run_until_halt(&mut NullBus::default(), 100).unwrap();
    assert!(
        cpu.halted,
        "not halted after {executed} instructions at {:04X}",
        cpu.pc
    );
}

#[test]
fn conditional_returns_obey_each_flag_in_both_directions() {
    for (opcode, mask, expected_set) in [
        (0xC8, 0x40, true),
        (0xC0, 0x40, false),
        (0xD8, 0x01, true),
        (0xD0, 0x01, false),
        (0xE8, 0x04, true),
        (0xE0, 0x04, false),
        (0xF8, 0x80, true),
        (0xF0, 0x80, false),
    ] {
        for set in [false, true] {
            let mut cpu = Cpu8080State::default();
            cpu.flags = k580_core::Flags::from_psw(if set { mask } else { 0 });
            cpu.set_memory_block(
                0,
                &[
                    0x31, 0xF0, 0xFF, 0xCD, 0x0A, 0x00, 0x3E, 0x55, 0x76, 0x00, opcode, 0x3E, 0xFF,
                    0x76,
                ],
            )
            .unwrap();
            run_until_halt(&mut cpu);
            assert_eq!(
                cpu.registers.a,
                if set == expected_set { 0x55 } else { 0xFF },
                "return opcode {opcode:02X}, flag={set}"
            );
        }
    }
}

#[test]
fn conditional_calls_use_sign_independently_of_parity() {
    for (opcode, sign, parity, expected_a) in [
        (0xF4, false, true, 0x22),
        (0xFC, true, false, 0x22),
        (0xFC, false, true, 0x11),
        (0xF4, true, false, 0x11),
    ] {
        let mut cpu = Cpu8080State::default();
        cpu.flags.sign = sign;
        cpu.flags.parity = parity;
        cpu.set_memory_block(
            0,
            &[
                0x31, 0xF0, 0xFF, 0x3E, 0x11, opcode, 0x09, 0x00, 0x76, 0x3E, 0x22, 0xC9,
            ],
        )
        .unwrap();
        run_until_halt(&mut cpu);
        assert_eq!(
            cpu.registers.a, expected_a,
            "call opcode {opcode:02X}, sign={sign}"
        );
        assert_eq!(cpu.sp, 0xFFF0);
    }
}

#[test]
fn bug01_jp_uses_sign_not_parity() {
    let mut cpu = Cpu8080State::default();
    let prog = [
        0x3E, 0x03, 0xB7, 0xF2, 0x0C, 0x00, 0x3E, 0x11, 0x76, 0x00, 0x00, 0x00, 0x3E, 0x22, 0x76,
    ];
    for (i, b) in prog.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    run_until_halt(&mut cpu);
    assert_eq!(
        cpu.get_register(RegisterName::A),
        0x22,
        "JP must jump on S=0"
    );
}

#[test]
fn bug01_jm_uses_sign_not_parity() {
    let mut cpu = Cpu8080State::default();
    let prog = [
        0x3E, 0x80, 0xB7, 0xFA, 0x0C, 0x00, 0x3E, 0x11, 0x76, 0x00, 0x00, 0x00, 0x3E, 0x22, 0x76,
    ];
    for (i, b) in prog.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    run_until_halt(&mut cpu);
    assert_eq!(
        cpu.get_register(RegisterName::A),
        0x22,
        "JM must jump on S=1"
    );
}

#[test]
fn bug02_rrc_uses_bit0_not_old_carry() {
    let mut cpu = Cpu8080State::default();
    cpu.flags.carry = true;
    cpu.registers.a = 0x02;
    cpu.memory.write(0, 0x0F);
    cpu.memory.write(1, 0x76);
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x01, "RRC must use bit0");
    assert!(!cpu.flags.carry, "RRC of 0x02 sets CY=0 (bit0 of 0x02)");
}

#[test]
fn bug03_rar_preserves_data_and_uses_old_carry() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0xC3;
    cpu.flags.carry = true;
    cpu.memory.write(0, 0x1F);
    cpu.memory.write(1, 0x76);
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0xE1);
    assert!(cpu.flags.carry, "old bit0 of 0xC3 = 1 → new CY=1");
}

#[test]
fn bug04_daa_adjusts_bcd_addition() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x15;
    cpu.registers.b = 0x27;
    cpu.memory.write(0, 0x80);
    cpu.memory.write(1, 0x27);
    cpu.memory.write(2, 0x76);
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x42);
}

#[test]
fn bug05_in_returns_port_byte_and_advances() {
    let mut cpu = Cpu8080State::default();
    cpu.memory.write(0, 0xDB);
    cpu.memory.write(1, 0x05);
    cpu.memory.write(2, 0x76);
    let mut bus = NullBus::default();
    bus.set_input(0x05, 0x99);
    cpu.run_until_halt(&mut bus, 10).unwrap();
    assert!(cpu.halted);
    assert_eq!(cpu.get_register(RegisterName::A), 0x99);
}

#[test]
fn bug07_ana_ac_follows_8080_quirk() {
    let mut cpu = Cpu8080State::default();
    cpu.registers.a = 0x08;
    cpu.registers.b = 0x00;
    cpu.memory.write(0, 0xA0);
    cpu.memory.write(1, 0x76);
    run_until_halt(&mut cpu);
    assert_eq!(cpu.get_register(RegisterName::A), 0x00);
    assert!(cpu.flags.zero);
    assert!(
        cpu.flags.auxiliary_carry,
        "ANA must set AC per 8080 quirk: ((A|operand)&0x08)!=0"
    );
}
