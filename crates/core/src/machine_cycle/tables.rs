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

    if (0x80..=0xBF).contains(&opcode) {
        return if (opcode & 7) == 6 {
            &[M1Fetch, MemoryRead]
        } else {
            &[M1Fetch]
        };
    }

    if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
        return if ((opcode >> 3) & 7) == 6 {
            &[M1Fetch, MemoryRead, MemoryWrite]
        } else {
            &[M1Fetch]
        };
    }

    if opcode & 0xC7 == 0x06 {
        return if ((opcode >> 3) & 7) == 6 {
            &[M1Fetch, MemoryRead, MemoryWrite]
        } else {
            &[M1Fetch, MemoryRead]
        };
    }

    if opcode & 0xCF == 0x01 {
        return &[M1Fetch, MemoryRead, MemoryRead];
    }
    if opcode & 0xCF == 0x03 {
        return &[M1Fetch];
    }
    if opcode & 0xCF == 0x09 {
        return &[M1Fetch, BusIdle, BusIdle];
    }
    if opcode & 0xCF == 0x0B {
        return &[M1Fetch];
    }

    if opcode & 0xC7 == 0xC0 {
        return if branch_taken {
            &[M1Fetch, StackRead, StackRead]
        } else {
            &[M1Fetch]
        };
    }
    if opcode & 0xC7 == 0xC2 {
        return &[M1Fetch, MemoryRead, MemoryRead];
    }
    if opcode & 0xC7 == 0xC4 {
        return if branch_taken {
            &[M1Fetch, MemoryRead, MemoryRead, StackWrite, StackWrite]
        } else {
            &[M1Fetch, MemoryRead, MemoryRead]
        };
    }
    if opcode & 0xC7 == 0xC7 {
        return &[M1Fetch, StackWrite, StackWrite];
    }
    if opcode & 0xCF == 0xC1 {
        return &[M1Fetch, StackRead, StackRead];
    }
    if opcode & 0xCF == 0xC5 {
        return &[M1Fetch, StackWrite, StackWrite];
    }

    match opcode {
        0x00 => &[M1Fetch],
        0x02 | 0x12 => &[M1Fetch, MemoryWrite],
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => &[M1Fetch],
        0x0A | 0x1A => &[M1Fetch, MemoryRead],
        0x22 => &[M1Fetch, MemoryRead, MemoryRead, MemoryWrite, MemoryWrite],
        0x2A => &[M1Fetch, MemoryRead, MemoryRead, MemoryRead, MemoryRead],
        0x32 => &[M1Fetch, MemoryRead, MemoryRead, MemoryWrite],
        0x3A => &[M1Fetch, MemoryRead, MemoryRead, MemoryRead],
        0x76 => &[M1Fetch],
        0xC3 => &[M1Fetch, MemoryRead, MemoryRead],
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => &[M1Fetch, MemoryRead],
        0xC9 => &[M1Fetch, StackRead, StackRead],
        0xCD => &[M1Fetch, MemoryRead, MemoryRead, StackWrite, StackWrite],
        0xD3 => &[M1Fetch, MemoryRead, IoWrite],
        0xDB => &[M1Fetch, MemoryRead, IoRead],
        0xE3 => &[
            M1Fetch, StackRead, StackRead, StackWrite, StackWrite, BusIdle,
        ],
        0xE9 => &[M1Fetch],
        0xEB => &[M1Fetch],
        0xF3 | 0xFB => &[M1Fetch],
        0xF9 => &[M1Fetch],
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
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    if opcode & 0xCF == 0x03 {
        return MachineCycleLayout::fixed(&[5]);
    }
    if opcode & 0xCF == 0x09 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    if opcode & 0xCF == 0x0B {
        return MachineCycleLayout::fixed(&[5]);
    }

    if opcode & 0xC7 == 0xC0 {
        return MachineCycleLayout::branch(&[5, 3, 3], &[5]);
    }
    if opcode & 0xC7 == 0xC2 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    if opcode & 0xC7 == 0xC4 {
        return MachineCycleLayout::branch(&[5, 3, 3, 3, 3], &[5, 3, 3]);
    }
    if opcode & 0xC7 == 0xC7 {
        return MachineCycleLayout::fixed(&[5, 3, 3]);
    }
    if opcode & 0xCF == 0xC1 {
        return MachineCycleLayout::fixed(&[4, 3, 3]);
    }
    if opcode & 0xCF == 0xC5 {
        return MachineCycleLayout::fixed(&[5, 3, 3]);
    }

    match opcode {
        0x00 => MachineCycleLayout::fixed(&[4]),
        0x02 | 0x12 => MachineCycleLayout::fixed(&[4, 3]),
        0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F => MachineCycleLayout::fixed(&[4]),
        0x0A | 0x1A => MachineCycleLayout::fixed(&[4, 3]),
        0x22 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]),
        0x2A => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3]),
        0x32 => MachineCycleLayout::fixed(&[4, 3, 3, 3]),
        0x3A => MachineCycleLayout::fixed(&[4, 3, 3, 3]),
        // HLT exposes a 4T M1 while instruction metadata retains the 7T total.
        0x76 => MachineCycleLayout::fixed(&[4]),
        0xC3 => MachineCycleLayout::fixed(&[4, 3, 3]),
        0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => MachineCycleLayout::fixed(&[4, 3]),
        0xC9 => MachineCycleLayout::fixed(&[4, 3, 3]),
        0xCD => MachineCycleLayout::fixed(&[5, 3, 3, 3, 3]),
        0xD3 => MachineCycleLayout::fixed(&[4, 3, 3]),
        0xDB => MachineCycleLayout::fixed(&[4, 3, 3]),
        0xE3 => MachineCycleLayout::fixed(&[4, 3, 3, 3, 3, 2]),
        0xE9 => MachineCycleLayout::fixed(&[5]),
        0xEB => MachineCycleLayout::fixed(&[5]),
        0xF3 | 0xFB => MachineCycleLayout::fixed(&[4]),
        0xF9 => MachineCycleLayout::fixed(&[5]),
        _ => MachineCycleLayout::fixed(&[]),
    }
}
