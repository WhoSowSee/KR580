//! Control flow: JMP, Jcc, CALL, Ccc, RET, Rcc, RST, PCHL.

use crate::decode::Cond;
use crate::execute::cond_holds;
use crate::state::Cpu8080State;

/// `JMP a16`.
pub fn jmp(cpu: &mut Cpu8080State) {
    let target = cpu.fetch_imm16();
    cpu.pc = target;
}

/// `Jcc a16`. Always reads the immediate so PC advances either way.
pub fn jcc(cpu: &mut Cpu8080State, cond: Cond) {
    let target = cpu.fetch_imm16();
    if cond_holds(cond, &cpu.flags) {
        cpu.pc = target;
    }
}

/// `CALL a16`.
pub fn call(cpu: &mut Cpu8080State) {
    let target = cpu.fetch_imm16();
    let ret = cpu.pc;
    cpu.push_word(ret);
    cpu.pc = target;
}

/// `Ccc a16`. Returns true if branch taken.
pub fn ccc(cpu: &mut Cpu8080State, cond: Cond) -> bool {
    let target = cpu.fetch_imm16();
    if cond_holds(cond, &cpu.flags) {
        let ret = cpu.pc;
        cpu.push_word(ret);
        cpu.pc = target;
        true
    } else {
        false
    }
}

/// `RET`.
pub fn ret(cpu: &mut Cpu8080State) {
    let ret = cpu.pop_word();
    cpu.pc = ret;
}

/// `Rcc`. Returns true if return taken.
pub fn rcc(cpu: &mut Cpu8080State, cond: Cond) -> bool {
    if cond_holds(cond, &cpu.flags) {
        let ret = cpu.pop_word();
        cpu.pc = ret;
        true
    } else {
        false
    }
}

/// `RST n` — bits 5-3 of the opcode select n (0..7). Branches to `n*8`.
pub fn rst(cpu: &mut Cpu8080State, op: u8) {
    let n = ((op >> 3) & 0b111) as u16;
    let ret = cpu.pc;
    cpu.push_word(ret);
    cpu.pc = n * 8;
}

/// `PCHL` — copy HL into PC.
pub fn pchl(cpu: &mut Cpu8080State) {
    cpu.pc = cpu.hl();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NullIoBus;

    #[test]
    fn jc_branches_only_when_carry_is_set() {
        // Per prompt: do not swap JC and JNC.
        let mut c = Cpu8080State::new();
        c.flags.cy = false;
        c.ram.write(0, 0xDA); // JC 0x1234
        c.ram.write(1, 0x34);
        c.ram.write(2, 0x12);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 3, "JC must NOT branch when CY clear");

        c = Cpu8080State::new();
        c.flags.cy = true;
        c.ram.write(0, 0xDA);
        c.ram.write(1, 0x34);
        c.ram.write(2, 0x12);
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x1234);
    }

    #[test]
    fn jnc_branches_only_when_carry_is_clear() {
        let mut c = Cpu8080State::new();
        c.flags.cy = true;
        c.ram.write(0, 0xD2); // JNC 0x1000
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x10);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 3);

        c = Cpu8080State::new();
        c.flags.cy = false;
        c.ram.write(0, 0xD2);
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x10);
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x1000);
    }

    #[test]
    fn call_ret_roundtrip() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        // CALL 0x0100
        c.ram.write(0, 0xCD);
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x01);
        // at 0x0100: RET
        c.ram.write(0x0100, 0xC9);
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x0100);
        assert_eq!(c.sp, 0x1FFE);
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x0003);
        assert_eq!(c.sp, 0x2000);
    }

    #[test]
    fn rst_pushes_and_branches() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        c.ram.write(0, 0xFF); // RST 7 → branch to 0x38
        let mut bus = NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x38);
        assert_eq!(c.sp, 0x1FFE);
    }

    #[test]
    fn cc_taken_uses_taken_timing() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        c.flags.cy = true;
        c.ram.write(0, 0xDC); // CC 0x0500
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x05);
        let mut bus = NullIoBus;
        let t = c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 0x0500);
        assert_eq!(t, 17);
    }

    #[test]
    fn cc_not_taken_uses_not_taken_timing() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        c.flags.cy = false;
        c.ram.write(0, 0xDC);
        c.ram.write(1, 0x00);
        c.ram.write(2, 0x05);
        let mut bus = NullIoBus;
        let t = c.step_instruction(&mut bus).unwrap();
        assert_eq!(c.pc, 3);
        assert_eq!(t, 11);
    }
}
