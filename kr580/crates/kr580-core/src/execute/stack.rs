//! Stack family: PUSH / POP for BC, DE, HL, PSW.

use crate::decode::decode_rp_psw;
use crate::flags::Flags;
use crate::state::Cpu8080State;

/// `PUSH rp` (BC/DE/HL or PSW). Bits 5-4 select the pair (PSW slot is 11).
pub fn push(cpu: &mut Cpu8080State, op: u8) {
    let pp = (op >> 4) & 0b11;
    match decode_rp_psw(pp) {
        Some(rp) => {
            let v = cpu.get_pair(rp);
            cpu.push_word(v);
        }
        None => {
            // PUSH PSW: high byte = A, low byte = PSW with bit1=1, bits 3,5=0
            let psw = cpu.flags.to_psw_byte();
            let word = ((cpu.a as u16) << 8) | psw as u16;
            cpu.push_word(word);
        }
    }
}

/// `POP rp` (BC/DE/HL or PSW).
pub fn pop(cpu: &mut Cpu8080State, op: u8) {
    let pp = (op >> 4) & 0b11;
    let v = cpu.pop_word();
    match decode_rp_psw(pp) {
        Some(rp) => cpu.set_pair(rp, v),
        None => {
            // POP PSW: high byte → A, low byte → flags (mask bits 3 & 5,
            // force bit 1 to 1 — handled in `Flags::from_psw_byte` round-trip).
            cpu.a = (v >> 8) as u8;
            cpu.flags = Flags::from_psw_byte(v as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NullIoBus;

    #[test]
    fn push_pop_psw_roundtrip_masks_psw_bits() {
        let mut c = Cpu8080State::new();
        c.a = 0xAB;
        c.flags.s = true;
        c.flags.z = false;
        c.flags.ac = true;
        c.flags.p = false;
        c.flags.cy = true;
        c.sp = 0x2000;
        // PUSH PSW
        c.ram.write(0, 0xF5);
        // POP PSW
        c.ram.write(1, 0xF1);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        // Mangle A and flags between push and pop
        c.a = 0;
        c.flags = Flags::default();
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0xAB);
        assert!(c.flags.s);
        assert!(!c.flags.z);
        assert!(c.flags.ac);
        assert!(!c.flags.p);
        assert!(c.flags.cy);
    }

    #[test]
    fn push_pop_bc() {
        let mut c = Cpu8080State::new();
        c.b = 0x12;
        c.c = 0x34;
        c.sp = 0x1000;
        // PUSH B; POP D
        c.ram.write(0, 0xC5);
        c.ram.write(1, 0xD1);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        c.step_instruction(&mut bus).unwrap();
        assert_eq!((c.d, c.e), (0x12, 0x34));
    }
}
