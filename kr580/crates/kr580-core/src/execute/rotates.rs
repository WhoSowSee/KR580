//! Rotate family: RLC, RRC, RAL, RAR.
//!
//! Per `prompt/02_cpu_core.md`, rotates only change carry and the rotated
//! accumulator bits. S/Z/AC/P are NOT touched.

use crate::state::Cpu8080State;

/// `RLC` — A is rotated left through carry-out; CY = bit 7 before rotate.
pub fn rlc(cpu: &mut Cpu8080State) {
    let high = (cpu.a & 0x80) >> 7;
    cpu.a = (cpu.a << 1) | high;
    cpu.flags.cy = high == 1;
}

/// `RRC` — A is rotated right; CY = bit 0 before rotate.
pub fn rrc(cpu: &mut Cpu8080State) {
    let low = cpu.a & 0x01;
    cpu.a = (cpu.a >> 1) | (low << 7);
    cpu.flags.cy = low == 1;
}

/// `RAL` — A is rotated left through CY; new CY = old bit 7.
pub fn ral(cpu: &mut Cpu8080State) {
    let high = (cpu.a & 0x80) >> 7;
    cpu.a = (cpu.a << 1) | (cpu.flags.cy as u8);
    cpu.flags.cy = high == 1;
}

/// `RAR` — A is rotated right through CY; new CY = old bit 0.
pub fn rar(cpu: &mut Cpu8080State) {
    let low = cpu.a & 0x01;
    cpu.a = (cpu.a >> 1) | ((cpu.flags.cy as u8) << 7);
    cpu.flags.cy = low == 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NullIoBus;

    #[test]
    fn rlc_circular_through_carry_out() {
        let mut c = Cpu8080State::new();
        c.a = 0b1000_0001;
        c.flags.s = true;
        c.flags.z = true;
        c.flags.ac = true;
        c.flags.p = true;
        c.ram.write(0, 0x07);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0b0000_0011);
        assert!(c.flags.cy);
        // unchanged
        assert!(c.flags.s);
        assert!(c.flags.z);
        assert!(c.flags.ac);
        assert!(c.flags.p);
    }

    #[test]
    fn rar_through_carry() {
        let mut c = Cpu8080State::new();
        c.a = 0b0000_0001;
        c.flags.cy = true;
        c.ram.write(0, 0x1F);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.a, 0b1000_0000);
        assert!(c.flags.cy);
    }
}
