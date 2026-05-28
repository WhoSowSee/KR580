use super::*;
use crate::decode::is_undocumented_opcode;

/// HLT (0x76) is the deliberate exception: school layout `[4]` (visible
/// M1 only) vs datasheet 7T (M1 fetch + M2 halt-ack).
#[test]
fn layout_sums_match_decode_timing_for_all_documented_opcodes() {
    for opcode in 0u8..=255 {
        if is_undocumented_opcode(opcode) {
            continue;
        }
        let info = crate::decode::decode_opcode(opcode).unwrap();
        let layout = layout_for(opcode);
        assert!(
            !layout.taken.is_empty(),
            "documented opcode {opcode:#04X} must have a layout"
        );
        if opcode == 0x76 {
            assert_eq!(layout.total_t_states(true), 4);
            assert_eq!(info.timing.t_states_taken, 7);
            continue;
        }
        assert_eq!(
            layout.total_t_states(true),
            info.timing.t_states_taken,
            "taken-sum mismatch for {opcode:#04X}"
        );
        if let Some(not_taken) = info.timing.t_states_not_taken {
            assert_eq!(
                layout.total_t_states(false),
                not_taken,
                "not-taken-sum mismatch for {opcode:#04X}"
            );
        }
    }
}

#[test]
fn mov_register_register_maps_to_single_m_cycle() {
    let layout = layout_for(0x78);
    for t in 0..5 {
        let pos = position_for(layout, true, t).unwrap();
        assert_eq!(pos.m_cycle, 1);
        assert_eq!(pos.t_in_cycle, t + 1);
        assert_eq!(pos.m_cycle_length, 5);
    }
    assert!(position_for(layout, true, 5).is_none());
}

#[test]
fn lxi_three_machine_cycles() {
    let layout = layout_for(0x01);
    assert_eq!(position_for(layout, true, 0).unwrap().m_cycle, 1);
    assert_eq!(position_for(layout, true, 3).unwrap().m_cycle, 1);
    assert_eq!(position_for(layout, true, 4).unwrap().m_cycle, 2);
    assert_eq!(position_for(layout, true, 6).unwrap().m_cycle, 2);
    assert_eq!(position_for(layout, true, 7).unwrap().m_cycle, 3);
    assert_eq!(position_for(layout, true, 9).unwrap().m_cycle, 3);
    let last = position_for(layout, true, 9).unwrap();
    assert_eq!(last.t_in_cycle, 3);
    assert_eq!(last.m_cycle_length, 3);
}

#[test]
fn rcond_branch_layouts_differ() {
    let layout = layout_for(0xC8);
    assert_eq!(layout.total_t_states(true), 11);
    assert_eq!(layout.total_t_states(false), 5);
    assert_eq!(position_for(layout, false, 4).unwrap().m_cycle, 1);
    assert!(position_for(layout, false, 5).is_none());
    assert_eq!(position_for(layout, true, 5).unwrap().m_cycle, 2);
    assert_eq!(position_for(layout, true, 8).unwrap().m_cycle, 3);
}

#[test]
fn kinds_length_matches_layout_for_all_documented_opcodes() {
    for opcode in 0u8..=255 {
        if is_undocumented_opcode(opcode) {
            continue;
        }
        let layout = layout_for(opcode);
        for taken in [true, false] {
            let layout_len = if taken {
                layout.taken.len()
            } else {
                layout.not_taken.unwrap_or(layout.taken).len()
            };
            let kinds_len = kinds_for(opcode, taken).len();
            assert_eq!(
                layout_len, kinds_len,
                "layout/kinds length mismatch for {opcode:#04X} (taken={taken})"
            );
        }
    }
}

#[test]
fn first_machine_cycle_is_always_m1_fetch() {
    for opcode in 0u8..=255 {
        if is_undocumented_opcode(opcode) {
            continue;
        }
        assert_eq!(
            kind_at(opcode, 0, true),
            Some(MachineCycleKind::M1Fetch),
            "first M-cycle of {opcode:#04X} (taken) is not M1Fetch"
        );
        assert_eq!(
            kind_at(opcode, 0, false),
            Some(MachineCycleKind::M1Fetch),
            "first M-cycle of {opcode:#04X} (not taken) is not M1Fetch"
        );
    }
}

#[test]
fn status_bytes_match_intel_8080a_datasheet() {
    use MachineCycleKind::*;
    let cases: &[(MachineCycleKind, u8)] = &[
        (M1Fetch, 0b1010_0010),
        (MemoryRead, 0b1000_0010),
        (MemoryWrite, 0b0000_0000),
        (StackRead, 0b1000_0110),
        (StackWrite, 0b0000_0100),
        (IoRead, 0b0100_0010),
        (IoWrite, 0b0001_0000),
        (InterruptAck, 0b0010_0011),
        (HaltAck, 0b1000_1010),
        (BusIdle, 0),
    ];
    for (kind, expected) in cases.iter().copied() {
        assert_eq!(
            kind.status_byte(),
            expected,
            "status byte mismatch for {kind:?}"
        );
    }
}
