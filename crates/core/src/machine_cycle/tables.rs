use super::{MachineCycleKind, MachineCycleKinds, MachineCycleLayout};
use crate::decode::is_undocumented_opcode;

pub fn kind_at(opcode: u8, m_cycle_idx: usize, branch_taken: bool) -> Option<MachineCycleKind> {
    kinds_for(opcode, branch_taken).get(m_cycle_idx).copied()
}

/// `HaltAck` / `InterruptAck` are not in this table – they depend on
/// runtime state, the UI raises them via `derive_status_kind`.
pub(crate) fn kinds_for(opcode: u8, branch_taken: bool) -> MachineCycleKinds {
    use MachineCycleKind::{
        BusIdle, IoRead, IoWrite, M1Fetch, MemoryRead, MemoryWrite, StackRead, StackWrite,
    };

    if is_undocumented_opcode(opcode) {
        return &[];
    }

    // MOV r1,r2 (excluding HLT=0x76).
    if (0x40..=0x7F).contains(&opcode) && opcode != 0x76 {
        let dst = (opcode >> 3) & 7;
        let src = opcode & 7;
        return match (dst == 6, src == 6) {
            (false, false) => &[M1Fetch],
            (false, true) => &[M1Fetch, MemoryRead],
            (true, false) => &[M1Fetch, MemoryWrite],
            (true, true) => &[M1Fetch],
        };
    }

    // ALU r.
    if (0x80..=0xBF).contains(&opcode) {
        return if (opcode & 7) == 6 {
            &[M1Fetch, MemoryRead]
        } else {
            &[M1Fetch]
        };
    }

    // INR/DCR r.
    if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
        return if ((opcode >> 3) & 7) == 6 {
            &[M1Fetch, MemoryRead, MemoryWrite]
        } else {
            &[M1Fetch]
        };
    }

    // MVI r,d8.
    if opcode & 0xC7 == 0x06 {
        return if ((opcode >> 3) & 7) == 6 {
            &[M1Fetch, MemoryRead, MemoryWrite]
        } else {
            &[M1Fetch, MemoryRead]
        };
    }

    if opcode & 0xCF == 0x01 {
        return &[M1Fetch, MemoryRead, MemoryRead]; // LXI rp,d16
    }
    if opcode & 0xCF == 0x03 {
        return &[M1Fetch]; // INX rp
    }
    if opcode & 0xCF == 0x09 {
        return &[M1Fetch, BusIdle, BusIdle]; // DAD rp
    }
    if opcode & 0xCF == 0x0B {
        return &[M1Fetch]; // DCX rp
    }

    if opcode & 0xC7 == 0xC0 {
        return if branch_taken {
            &[M1Fetch, StackRead, StackRead]
        } else {
            &[M1Fetch]
        };
    }
    if opcode & 0xC7 == 0xC2 {
        return &[M1Fetch, MemoryRead, MemoryRead]; // Jcond – operand always read
    }
    if opcode & 0xC7 == 0xC4 {
        return if branch_taken {
            &[M1Fetch, MemoryRead, MemoryRead, StackWrite, StackWrite]
        } else {
            &[M1Fetch, MemoryRead, MemoryRead]
        };
    }
    if opcode & 0xC7 == 0xC7 {
        return &[M1Fetch, StackWrite, StackWrite]; // RST n
    }
    if opcode & 0xCF == 0xC1 {
        return &[M1Fetch, StackRead, StackRead]; // POP rp
    }
    if opcode & 0xCF == 0xC5 {
        return &[M1Fetch, StackWrite, StackWrite]; // PUSH rp
    }

    match opcode {
        0x00 => &[M1Fetch],                                                   // NOP
        0x02 | 0x12 => &[M1Fetch, MemoryWrite],                               // STAX B/D
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => &[M1Fetch],  // RLC..CMC
        0x0A | 0x1A => &[M1Fetch, MemoryRead],                                // LDAX B/D
        0x22 => &[M1Fetch, MemoryRead, MemoryRead, MemoryWrite, MemoryWrite], // SHLD
        0x2A => &[M1Fetch, MemoryRead, MemoryRead, MemoryRead, MemoryRead],   // LHLD
        0x32 => &[M1Fetch, MemoryRead, MemoryRead, MemoryWrite],              // STA
        0x3A => &[M1Fetch, MemoryRead, MemoryRead, MemoryRead],               // LDA
        // HLT: visible M1 only; halt-ack is handled out of band.
        0x76 => &[M1Fetch],
        0xC3 => &[M1Fetch, MemoryRead, MemoryRead], // JMP
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => &[M1Fetch, MemoryRead],
        0xC9 => &[M1Fetch, StackRead, StackRead], // RET
        0xCD => &[M1Fetch, MemoryRead, MemoryRead, StackWrite, StackWrite], // CALL
        0xD3 => &[M1Fetch, MemoryRead, IoWrite],  // OUT
        0xDB => &[M1Fetch, MemoryRead, IoRead],   // IN
        0xE3 => &[
            M1Fetch, StackRead, StackRead, StackWrite, StackWrite, BusIdle,
        ], // XTHL
        0xE9 => &[M1Fetch],                       // PCHL
        0xEB => &[M1Fetch],                       // XCHG
        0xF3 | 0xFB => &[M1Fetch],                // DI / EI
        0xF9 => &[M1Fetch],                       // SPHL
        _ => &[],
    }
}

pub fn layout_for(opcode: u8) -> MachineCycleLayout {
    if is_undocumented_opcode(opcode) {
        return MachineCycleLayout::fixed(&[]);
    }

    if (0x40..=0x7F).contains(&opcode) && opcode != 0x76 {
        let dst = (opcode >> 3) & 7;
        let src = opcode & 7;
        return if dst == 6 || src == 6 {
            MachineCycleLayout::fixed(&[4, 3])
        } else {
            MachineCycleLayout::fixed(&[5])
        };
    }

    if (0x80..=0xBF).contains(&opcode) {
        return if (opcode & 7) == 6 {
            MachineCycleLayout::fixed(&[4, 3])
        } else {
            MachineCycleLayout::fixed(&[4])
        };
    }

    if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
        let reg = (opcode >> 3) & 7;
        return if reg == 6 {
            MachineCycleLayout::fixed(&[4, 3, 3])
        } else {
            MachineCycleLayout::fixed(&[5])
        };
    }

    if opcode & 0xC7 == 0x06 {
        let reg = (opcode >> 3) & 7;
        return if reg == 6 {
            MachineCycleLayout::fixed(&[4, 3, 3])
        } else {
            MachineCycleLayout::fixed(&[4, 3])
        };
    }

    if opcode & 0xCF == 0x01 {
        return MachineCycleLayout::fixed(&[4, 3, 3]); // LXI rp,d16
    }
    if opcode & 0xCF == 0x03 {
        return MachineCycleLayout::fixed(&[5]); // INX rp
    }
    if opcode & 0xCF == 0x09 {
        return MachineCycleLayout::fixed(&[4, 3, 3]); // DAD rp
    }
    if opcode & 0xCF == 0x0B {
        return MachineCycleLayout::fixed(&[5]); // DCX rp
    }

    if opcode & 0xC7 == 0xC0 {
        return MachineCycleLayout::branch(&[5, 3, 3], &[5]); // Rcond
    }
    if opcode & 0xC7 == 0xC2 {
        return MachineCycleLayout::fixed(&[4, 3, 3]); // Jcond
    }
    if opcode & 0xC7 == 0xC4 {
        return MachineCycleLayout::branch(&[5, 3, 3, 3, 3], &[5, 3, 3]); // Ccond
    }
    if opcode & 0xC7 == 0xC7 {
        return MachineCycleLayout::fixed(&[5, 3, 3]); // RST n
    }
    if opcode & 0xCF == 0xC1 {
        return MachineCycleLayout::fixed(&[4, 3, 3]); // POP rp
    }
    if opcode & 0xCF == 0xC5 {
        return MachineCycleLayout::fixed(&[5, 3, 3]); // PUSH rp
    }

    match opcode {
        0x00 => MachineCycleLayout::fixed(&[4]),           // NOP
        0x02 | 0x12 => MachineCycleLayout::fixed(&[4, 3]), // STAX B/D
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => MachineCycleLayout::fixed(&[4]),
        0x0A | 0x1A => MachineCycleLayout::fixed(&[4, 3]), // LDAX B/D
        0x22 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]), // SHLD
        0x2A => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]), // LHLD
        0x32 => MachineCycleLayout::fixed(&[4, 3, 3, 3]),  // STA
        0x3A => MachineCycleLayout::fixed(&[4, 3, 3, 3]),  // LDA
        // HLT layout is intentionally `[4]` (school convention) while
        // `decode.rs` keeps the datasheet 7T total – guarded by a test.
        0x76 => MachineCycleLayout::fixed(&[4]),
        0xC3 => MachineCycleLayout::fixed(&[4, 3, 3]), // JMP
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => MachineCycleLayout::fixed(&[4, 3]),
        0xC9 => MachineCycleLayout::fixed(&[4, 3, 3]), // RET
        0xCD => MachineCycleLayout::fixed(&[5, 3, 3, 3, 3]), // CALL
        0xD3 => MachineCycleLayout::fixed(&[4, 3, 3]), // OUT
        0xDB => MachineCycleLayout::fixed(&[4, 3, 3]), // IN
        0xE3 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3, 2]), // XTHL
        0xE9 => MachineCycleLayout::fixed(&[5]),       // PCHL
        0xEB => MachineCycleLayout::fixed(&[5]),       // XCHG
        0xF3 | 0xFB => MachineCycleLayout::fixed(&[4]), // DI / EI
        0xF9 => MachineCycleLayout::fixed(&[5]),       // SPHL
        _ => MachineCycleLayout::fixed(&[]),
    }
}
