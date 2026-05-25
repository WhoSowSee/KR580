use k580_core::{
    CoreError, Cpu8080State, DecodeError, Flags, NullBus, RegisterName, decode_opcode,
};

fn step(cpu: &mut Cpu8080State) {
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
}

fn put_program(cpu: &mut Cpu8080State, bytes: &[u8]) {
    for (offset, byte) in bytes.iter().copied().enumerate() {
        cpu.memory.write(offset as u16, byte);
    }
}

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

/// Точная фиксация флагов для конкретного `ADD B` из программы
/// `t2.580`, на которой пользователь сравнивал наш эмулятор со
/// школьным референсным. Раньше пользователь видел в нашем
/// «Регистре флагов» строку `010010`, а в школьном — `01001`, и
/// возникло подозрение, что наша ALU неправильно выставляет
/// чётность (P) и/или знак (S). Этот тест фиксирует datasheet-
/// корректные ожидания для `0x0F + 0x37`:
///
/// - результат: `0x46` (66 в десятичном)
/// - бит 7 = 0 → S = 0
/// - результат ≠ 0 → Z = 0
/// - полупереполнение из бита 3 (`0xF + 0x7 = 0x16`) → AC = 1
/// - двоичная запись `0100 0110` содержит **три** единицы
///   (нечётно) → P = 0 (8080 ставит P=1 только при чётном числе
///   единиц)
/// - `0x0F + 0x37 = 0x46 < 0x100`, переноса из бита 7 нет → C = 0
///
/// Если этот тест начнёт падать, значит регрессировала одна из
/// четырёх ALU-веток (`add`, `set_sign_zero_parity`, AC-вычисление,
/// или формирование C). Раскладка PSW отдельно проверяется
/// `psw_materialization_forces_reserved_bits` выше.
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

#[test]
fn run_for_t_states_advances_exact_quantum() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00, 0x00]);
    let mut bus = NullBus::default();
    cpu.run_for_t_states(&mut bus, 3).unwrap();
    assert_eq!(cpu.cycle_count, 3);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.tact_phase, Some(3));

    cpu.run_for_t_states(&mut bus, 1).unwrap();
    assert_eq!(cpu.cycle_count, 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.tact_phase, None);
}

/// Холодный старт: до первого `step_*` ни одна T-фаза не выполнена,
/// поэтому `last_completed_tact_phase == None`. UI в этом случае
/// рисует `-` в строке «Такт» — это и есть ожидаемое поведение
/// «никогда не было инструкции», в отличие от `Some(_)` который
/// означает «была, и вот её последняя выполненная T».
#[test]
fn last_completed_tact_phase_is_none_on_cold_start() {
    let cpu = Cpu8080State::default();
    assert_eq!(cpu.last_completed_tact_phase, None);

    let mut cpu2 = Cpu8080State::default();
    put_program(&mut cpu2, &[0x00]);
    let mut bus = NullBus::default();
    cpu2.step_instruction(&mut bus).unwrap();
    cpu2.reset_cpu();
    assert_eq!(cpu2.last_completed_tact_phase, None);
}

/// Атомарный путь `step_instruction` без предварительного walking:
/// после выполнения NOP (4 такта) `last_completed_tact_phase` должен
/// быть `Some(3)` — линейная фаза `total - 1`. Это и есть «последний
/// горящий такт» на школьном табло после завершения инструкции.
#[test]
fn last_completed_tact_phase_after_step_instruction_equals_total_minus_one() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]); // NOP, 4 T-states
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
    assert_eq!(cpu.tact_phase, None);
    assert_eq!(cpu.last_completed_tact_phase, Some(3));
}

/// Walking-режим через `step_tact`: на каждом такте обновляется
/// `last_completed_tact_phase = phase`. После 4 тактов NOP она
/// должна быть `Some(3)`, и параллельно `tact_phase == None` (граница
/// инструкции). Это закрывает разрыв «активная фаза vs последняя
/// выполненная» который раньше прятал позицию между нажатиями.
#[test]
fn last_completed_tact_phase_walks_with_step_tact() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]); // NOP
    let mut bus = NullBus::default();
    for expected in 0u8..4 {
        cpu.step_tact(&mut bus).unwrap();
        assert_eq!(cpu.last_completed_tact_phase, Some(expected));
    }
    assert_eq!(cpu.tact_phase, None);
    assert_eq!(cpu.last_completed_tact_phase, Some(3));
}

/// HLT и run_until_halt: после остановки последняя выполненная фаза
/// должна совпадать со школьным эталоном — `total - 1` HLT-инструкции
/// (7 тактов → Some(6)). Раньше UI после HLT падал в `-`/`1`, теперь
/// «застывает» на правильной позиции.
#[test]
fn last_completed_tact_phase_after_halt_run() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x76]); // HLT, 7 T-states
    let mut bus = NullBus::default();
    cpu.run_until_halt(&mut bus, 1).unwrap();
    assert!(cpu.halted);
    assert_eq!(cpu.last_completed_tact_phase, Some(6));
}

/// TACT-COMPLETE flush: если walking-режим оборвали через
/// `step_instruction` посреди инструкции, `last_completed_tact_phase`
/// должна перенести позицию `total - 1` той инструкции которую
/// доисполнили flush'ем, а не сбрасываться в `None`.
#[test]
fn last_completed_tact_phase_after_flush_carries_total_minus_one() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]); // NOP, 4 T-states
    let mut bus = NullBus::default();
    cpu.step_tact(&mut bus).unwrap(); // запустили walking, выполнен phase=0
    assert_eq!(cpu.last_completed_tact_phase, Some(0));
    cpu.step_instruction(&mut bus).unwrap(); // flush остатка
    assert_eq!(cpu.tact_phase, None);
    assert_eq!(cpu.last_completed_tact_phase, Some(3));
}
