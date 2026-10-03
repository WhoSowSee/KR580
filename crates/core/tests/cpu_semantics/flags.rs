use super::*;

#[test]
fn add_adc_and_inr_flags_follow_8080_rules() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x80, 0x88, 0x3C]);
    cpu.registers.a = 0x0F;
    cpu.registers.b = 0x01;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x10);
    assert!(cpu.flags.auxiliary_carry);
    assert!(!cpu.flags.carry);

    cpu.flags.carry = true;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x12);
    assert!(!cpu.flags.carry);

    cpu.flags.carry = true;
    cpu.registers.a = 0xFF;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.flags.zero);
    assert!(cpu.flags.auxiliary_carry);
    assert!(cpu.flags.carry, "INR must not touch carry");
}

/// Datasheet-correct flag expectations for `ADD B` with `A=0x0F`,
/// `B=0x37` (the case from `t2.580` that diverged from the reference
/// emulator). Pins down `S/Z/AC/P/C` and the PSW byte so any future
/// regression in `add`, `set_sign_zero_parity`, AC or carry shows up
/// here instead of as a UI mismatch.

#[test]
fn add_b_zero_f_plus_three_seven_matches_datasheet_flags() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x80]); // ADD B
    cpu.registers.a = 0x0F;
    cpu.registers.b = 0x37;
    step(&mut cpu);

    assert_eq!(cpu.registers.a, 0x46, "ADD result must be 0x46");
    assert!(!cpu.flags.sign, "S=0 because bit 7 of 0x46 is 0");
    assert!(!cpu.flags.zero, "Z=0 because A != 0");
    assert!(
        cpu.flags.auxiliary_carry,
        "AC=1: half-carry from bit 3 (0xF + 0x7 = 0x16)"
    );
    assert!(
        !cpu.flags.parity,
        "P=0: 0x46 = 0b0100_0110 has three 1-bits (odd parity)"
    );
    assert!(
        !cpu.flags.carry,
        "C=0: 0x0F + 0x37 = 0x46, no carry out of bit 7"
    );
    // PSW byte: bit 4 (AC) + bit 1 (always set) = 0x12.
    assert_eq!(cpu.flags.to_psw(), 0x12);
}

#[test]
fn subtract_auxiliary_carry_matches_prompt_edge_cases() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x90, 0x90]);
    cpu.registers.a = 1;
    cpu.registers.b = 0;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 1);
    assert!(cpu.flags.auxiliary_carry, "1-0 yields AC=1 under plain SUB");

    cpu.registers.a = 0;
    cpu.registers.b = 1;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0xFF);
    assert!(
        !cpu.flags.auxiliary_carry,
        "0-1 yields AC=0 under plain SUB"
    );
    assert!(cpu.flags.carry);
}

#[test]
fn dcr_ana_cmp_and_dad_have_documented_flag_behavior() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x05, 0xA0, 0xB8, 0x09]);
    cpu.registers.b = 0;
    cpu.flags.carry = true;
    step(&mut cpu);
    assert_eq!(cpu.registers.b, 0xFF);
    assert!(!cpu.flags.auxiliary_carry);
    assert!(cpu.flags.carry, "DCR must not touch carry");

    cpu.registers.a = 0x08;
    cpu.registers.b = 0x00;
    cpu.flags.carry = true;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0);
    assert!(cpu.flags.auxiliary_carry);
    assert!(!cpu.flags.carry);

    cpu.registers.a = 0x22;
    cpu.registers.b = 0x22;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x22, "CMP must not change A");
    assert!(cpu.flags.zero);

    cpu.registers.set_hl(0xFFFF);
    cpu.registers.set_bc(0x0001);
    cpu.flags.zero = true;
    step(&mut cpu);
    assert_eq!(cpu.registers.hl(), 0);
    assert!(cpu.flags.carry);
    assert!(cpu.flags.zero, "DAD updates only carry");
}

#[test]
fn daa_adjusts_after_bcd_addition() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x80, 0x27]);
    cpu.registers.a = 0x09;
    cpu.registers.b = 0x09;
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x12);
    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0x18);
    assert!(!cpu.flags.carry);
}

#[test]
fn rotate_complement_and_carry_ops_touch_only_documented_state() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x07, 0x0F, 0x17, 0x1F, 0x2F, 0x37, 0x3F]);
    cpu.registers.a = 0b1000_0001;
    cpu.flags = Flags {
        sign: true,
        zero: true,
        auxiliary_carry: true,
        parity: false,
        carry: false,
    };

    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0b0000_0011);
    assert!(cpu.flags.carry);
    assert!(cpu.flags.sign);
    assert!(cpu.flags.zero);
    assert!(cpu.flags.auxiliary_carry);
    assert!(!cpu.flags.parity);

    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0b1000_0001);
    assert!(cpu.flags.carry);

    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0b0000_0011);
    assert!(cpu.flags.carry);

    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0b1000_0001);
    assert!(cpu.flags.carry);

    step(&mut cpu);
    assert_eq!(cpu.registers.a, 0b0111_1110);
    assert!(cpu.flags.sign);
    assert!(cpu.flags.zero);
    assert!(cpu.flags.auxiliary_carry);
    assert!(!cpu.flags.parity);

    step(&mut cpu);
    assert!(cpu.flags.carry);
    step(&mut cpu);
    assert!(!cpu.flags.carry);
}
