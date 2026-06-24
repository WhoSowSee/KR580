//! Disassembly-aware classification of operand bytes in the memory list.

use std::collections::HashSet;

use k580_core::{Memory64K, decode_opcode};

/// Classified operand addresses in a visible memory range.
pub(super) struct OperandKinds {
    pub addresses: HashSet<u16>,
    pub data: HashSet<u16>,
    pub ports: HashSet<u16>,
}

/// Returns the operand addresses in `[start, start + count)` grouped by
/// kind, using a local disassembly scan.
///
/// The scan walks back up to two bytes to find the most likely opcode
/// boundary, then advances instruction by instruction and marks every
/// byte after an opcode as an operand. Operands are split into:
///
/// - `addresses` — 16-bit memory addresses/16-bit immediates (3-byte
///   instructions such as `LXI`, `JMP`, `CALL`, `SHLD`, `LHLD`, `STA`,
///   `LDA`, and the conditional branch/call family).
/// - `data` — 8-bit generic immediate operands (`MVI`, `ADI`, `CPI`, etc.).
/// - `ports` — port numbers of `IN`/`OUT`.
pub(super) fn classify_operands(start: u16, count: usize, memory: &Memory64K) -> OperandKinds {
    let mut operands = OperandKinds {
        addresses: HashSet::new(),
        data: HashSet::new(),
        ports: HashSet::new(),
    };
    if count == 0 {
        return operands;
    }

    let boundary = find_scan_boundary(start, memory);
    let mut address = boundary;
    let mut classified = 0usize;

    while classified < count {
        let value = memory.read(address);
        let size = decode_opcode(value).map(|info| info.size).unwrap_or(1);
        let kind = operand_kind(value);

        for offset in 0..size {
            let addr = address.wrapping_add(offset as u16);
            if in_range(addr, start, count) {
                classified += 1;
                if offset > 0 {
                    match kind {
                        OperandKind::Address => operands.addresses.insert(addr),
                        OperandKind::Data => operands.data.insert(addr),
                        OperandKind::Port => operands.ports.insert(addr),
                    };
                }
            }
        }

        address = address.wrapping_add(size as u16);
    }

    operands
}

/// 16-bit jump target for an address operand at `address`, if the
/// byte at `address` is the low or high half of a 3-byte address
/// instruction's operand (e.g. `LXI`, `JMP`, `CALL`, `SHLD`, `LHLD`,
/// `STA`, `LDA`, conditional branch/call). Returns `None` when
/// `address` is an opcode byte, a data/port operand, or outside any
/// address instruction. Independent of the highlight toggle.
pub(crate) fn operand_jump_target(address: u16, memory: &Memory64K) -> Option<u16> {
    let boundary = find_scan_boundary(address, memory);
    let mut cursor = boundary;
    for _ in 0..3 {
        let opcode = memory.read(cursor);
        let size = decode_opcode(opcode).map(|info| info.size).unwrap_or(1);
        if instruction_covers(cursor, size, address) {
            if address != cursor && is_address_opcode(opcode) && size == 3 {
                let low = memory.read(cursor.wrapping_add(1));
                let high = memory.read(cursor.wrapping_add(2));
                return Some(u16::from_le_bytes([low, high]));
            }
            return None;
        }
        cursor = cursor.wrapping_add(size as u16);
    }
    None
}

/// Port number encoded by an `IN`/`OUT` operand at `address`, if the
/// byte at `address` is the port-number half of a 2-byte `IN`/`OUT`
/// instruction. Returns `None` for opcode bytes, data operands, and
/// address operands. The caller maps the returned port to the device
/// window it opens (see `crate::devices::IoBus` port constants).
pub(crate) fn operand_port_number(address: u16, memory: &Memory64K) -> Option<u8> {
    let boundary = find_scan_boundary(address, memory);
    let mut cursor = boundary;
    for _ in 0..3 {
        let opcode = memory.read(cursor);
        let size = decode_opcode(opcode).map(|info| info.size).unwrap_or(1);
        if instruction_covers(cursor, size, address) {
            if address != cursor && is_port_opcode(opcode) && size == 2 {
                return Some(memory.read(cursor.wrapping_add(1)));
            }
            return None;
        }
        cursor = cursor.wrapping_add(size as u16);
    }
    None
}

fn instruction_covers(start: u16, size: u8, address: u16) -> bool {
    if size == 0 {
        return false;
    }
    let end = start.wrapping_add((size - 1) as u16);
    if end >= start {
        (start..=end).contains(&address)
    } else {
        address >= start || address <= end
    }
}

enum OperandKind {
    Address,
    Data,
    Port,
}

fn operand_kind(opcode: u8) -> OperandKind {
    if is_port_opcode(opcode) {
        return OperandKind::Port;
    }
    if is_address_opcode(opcode) {
        return OperandKind::Address;
    }
    OperandKind::Data
}

fn is_address_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        0x01 | 0x11 | 0x21 | 0x31 | // LXI rp,d16
        0x22 | 0x2A | 0x32 | 0x3A | // SHLD, LHLD, STA, LDA
        0xC3 | 0xCD | // JMP, CALL
        0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA | // Jcond
        0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC // Ccond
    )
}

fn is_port_opcode(opcode: u8) -> bool {
    matches!(opcode, 0xD3 | 0xDB)
}

fn find_scan_boundary(target: u16, memory: &Memory64K) -> u16 {
    for back in [1, 2] {
        let candidate = target.wrapping_sub(back);
        let size = decode_opcode(memory.read(candidate))
            .map(|info| info.size)
            .unwrap_or(1);
        if target.wrapping_sub(candidate) < size as u16 {
            return candidate;
        }
    }
    target
}

fn in_range(address: u16, start: u16, count: usize) -> bool {
    (address.wrapping_sub(start) as usize) < count
}

#[cfg(test)]
mod tests {
    use k580_core::Memory64K;

    use super::{classify_operands, operand_jump_target, operand_port_number};

    #[test]
    fn one_byte_instructions_have_no_operands() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x00); // NOP
        memory.write(1, 0x07); // RLC
        memory.write(2, 0x76); // HLT
        let operands = classify_operands(0, 3, &memory);
        assert!(operands.addresses.is_empty());
        assert!(operands.data.is_empty());
        assert!(operands.ports.is_empty());
    }

    #[test]
    fn eight_bit_data_operand_is_marked() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x06); // MVI B
        memory.write(1, 0x42);
        memory.write(2, 0x00);
        let operands = classify_operands(0, 3, &memory);
        assert!(operands.data.contains(&1));
        assert!(!operands.addresses.contains(&1));
        assert!(!operands.ports.contains(&1));
    }

    #[test]
    fn sixteen_bit_address_operands_are_marked() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x01); // LXI B
        memory.write(1, 0x34);
        memory.write(2, 0x12);
        memory.write(3, 0xC3); // JMP
        memory.write(4, 0x00);
        memory.write(5, 0x01);
        let operands = classify_operands(0, 6, &memory);
        assert!(operands.addresses.contains(&1));
        assert!(operands.addresses.contains(&2));
        assert!(operands.addresses.contains(&4));
        assert!(operands.addresses.contains(&5));
        assert!(operands.data.is_empty());
    }

    #[test]
    fn operand_classification_wraps_across_64k_boundary() {
        let mut memory = Memory64K::default();
        memory.write(0xFFFF, 0x06); // MVI B
        memory.write(0x0000, 0x42);
        let operands = classify_operands(0, 1, &memory);
        assert!(operands.data.contains(&0));
    }

    #[test]
    fn in_and_out_port_operands_are_purple() {
        let mut memory = Memory64K::default();
        memory.write(0, 0xD3); // OUT
        memory.write(1, 0x04); // port 4
        memory.write(2, 0xDB); // IN
        memory.write(3, 0x00); // port 0
        let operands = classify_operands(0, 4, &memory);
        assert!(operands.ports.contains(&1));
        assert!(operands.ports.contains(&3));
        assert!(!operands.data.contains(&1));
        assert!(!operands.addresses.contains(&1));
    }

    #[test]
    fn jump_target_from_low_operand_byte_is_little_endian_address() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x01); // LXI B,d16
        memory.write(1, 0x34); // low
        memory.write(2, 0x12); // high -> 0x1234
        assert_eq!(operand_jump_target(1, &memory), Some(0x1234));
    }

    #[test]
    fn jump_target_from_high_operand_byte_matches_low_half() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x01); // LXI B,d16
        memory.write(1, 0x34);
        memory.write(2, 0x12);
        assert_eq!(operand_jump_target(2, &memory), Some(0x1234));
    }

    #[test]
    fn jump_target_is_none_on_the_opcode_byte_itself() {
        let mut memory = Memory64K::default();
        memory.write(0, 0xC3); // JMP
        memory.write(1, 0x00);
        memory.write(2, 0x10);
        assert_eq!(operand_jump_target(0, &memory), None);
    }

    #[test]
    fn jump_target_resolves_after_a_preceding_two_byte_instruction() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x06); // MVI B (2 bytes)
        memory.write(1, 0xFF);
        memory.write(2, 0xC3); // JMP 0x0200
        memory.write(3, 0x00);
        memory.write(4, 0x02);
        assert_eq!(operand_jump_target(3, &memory), Some(0x0200));
        assert_eq!(operand_jump_target(4, &memory), Some(0x0200));
    }

    #[test]
    fn jump_target_is_none_for_data_and_port_operands() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x06); // MVI B,d8
        memory.write(1, 0x42);
        memory.write(2, 0xD3); // OUT
        memory.write(3, 0x04);
        assert_eq!(operand_jump_target(1, &memory), None);
        assert_eq!(operand_jump_target(3, &memory), None);
    }

    #[test]
    fn jump_target_wraps_across_64k_boundary() {
        let mut memory = Memory64K::default();
        memory.write(0xFFFF, 0x01); // LXI B,d16 wrapping
        memory.write(0x0000, 0x78);
        memory.write(0x0001, 0x56);
        assert_eq!(operand_jump_target(0x0000, &memory), Some(0x5678));
        assert_eq!(operand_jump_target(0x0001, &memory), Some(0x5678));
    }

    #[test]
    fn port_number_from_out_operand_byte() {
        let mut memory = Memory64K::default();
        memory.write(0, 0xD3); // OUT
        memory.write(1, 0x04); // port 4
        assert_eq!(operand_port_number(1, &memory), Some(0x04));
    }

    #[test]
    fn port_number_from_in_operand_byte() {
        let mut memory = Memory64K::default();
        memory.write(0, 0xDB); // IN
        memory.write(1, 0x00); // port 0 (monitor)
        assert_eq!(operand_port_number(1, &memory), Some(0x00));
    }

    #[test]
    fn port_number_is_none_on_opcode_byte() {
        let mut memory = Memory64K::default();
        memory.write(0, 0xD3); // OUT
        memory.write(1, 0x04);
        assert_eq!(operand_port_number(0, &memory), None);
    }

    #[test]
    fn port_number_is_none_for_data_and_address_operands() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x06); // MVI B,d8
        memory.write(1, 0x42);
        memory.write(2, 0x01); // LXI B,d16
        memory.write(3, 0x34);
        memory.write(4, 0x12);
        assert_eq!(operand_port_number(1, &memory), None);
        assert_eq!(operand_port_number(3, &memory), None);
        assert_eq!(operand_port_number(4, &memory), None);
    }

    #[test]
    fn port_number_resolves_after_preceding_instruction() {
        let mut memory = Memory64K::default();
        memory.write(0, 0x3E); // MVI A (2 bytes)
        memory.write(1, 0xFF);
        memory.write(2, 0xD3); // OUT 0x02
        memory.write(3, 0x02);
        assert_eq!(operand_port_number(3, &memory), Some(0x02));
    }
}
