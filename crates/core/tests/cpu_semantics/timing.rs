use super::*;

#[test]
fn run_for_t_states_advances_exact_quantum() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00, 0x00]);
    let mut bus = NullBus::default();
    cpu.run_for_t_states(&mut bus, 3).unwrap();
    assert_eq!(cpu.cycle_count, 3);
    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.tact_phase, Some(3));

    cpu.run_for_t_states(&mut bus, 1).unwrap();
    assert_eq!(cpu.cycle_count, 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.tact_phase, None);
}

/// Cold start: before any `step_*` no T-phase has run, so
/// `last_completed_tact_phase == None`. The UI shows `-` in the tact
/// row – distinct from `Some(_)`, which means "an instruction
/// ran, and here is its last completed T".

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

/// Atomic `step_instruction` path with no preceding walking: after a
/// NOP (4 T-states) `last_completed_tact_phase` must be `Some(3)` –
/// the linear phase `total - 1`. That is the "last lit T" the
/// reference panel freezes on once the instruction completes.

#[test]
fn last_completed_tact_phase_after_step_instruction_equals_total_minus_one() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]); // NOP, 4 T-states
    let mut bus = NullBus::default();
    cpu.step_instruction(&mut bus).unwrap();
    assert_eq!(cpu.tact_phase, None);
    assert_eq!(cpu.last_completed_tact_phase, Some(3));
}

/// Walking mode through `step_tact`: every tact updates
/// `last_completed_tact_phase = phase`. After 4 NOP tacts it must be
/// `Some(3)` while `tact_phase == None` (instruction boundary). This
/// closes the "active phase vs last completed" gap that used to hide
/// the position between key presses.

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

/// HLT + `run_until_halt`: once halted, the last completed phase
/// must match the reference – `total - 1` of the HLT instruction
/// (7 T-states → `Some(6)`). The UI used to fall to `-`/`1` after
/// HLT; now it freezes on the right slot.

#[test]
fn last_completed_tact_phase_after_halt_run() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x76]); // HLT, 7 T-states
    let mut bus = NullBus::default();
    cpu.run_until_halt(&mut bus, 1).unwrap();
    assert!(cpu.halted);
    assert_eq!(cpu.last_completed_tact_phase, Some(6));
}

/// TACT-COMPLETE flush: when walking mode is interrupted by a
/// `step_instruction` mid-instruction, `last_completed_tact_phase`
/// must carry the `total - 1` of the flushed instruction instead of
/// resetting to `None`.

#[test]
fn last_completed_tact_phase_after_flush_carries_total_minus_one() {
    let mut cpu = Cpu8080State::default();
    put_program(&mut cpu, &[0x00]); // NOP, 4 T-states
    let mut bus = NullBus::default();
    cpu.step_tact(&mut bus).unwrap(); // start walking, phase=0 done
    assert_eq!(cpu.last_completed_tact_phase, Some(0));
    cpu.step_instruction(&mut bus).unwrap(); // flush remainder
    assert_eq!(cpu.tact_phase, None);
    assert_eq!(cpu.last_completed_tact_phase, Some(3));
}
