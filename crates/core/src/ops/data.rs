use crate::ops::operand::RegPair;
use crate::{Cpu8080State, InstructionOutcome};

pub(crate) fn handles(opcode: u8) -> bool {
    (0x40..=0x7F).contains(&opcode)
        || opcode & 0xC7 == 0x06
        || opcode & 0xCF == 0x01
        || opcode & 0xCF == 0x03
        || opcode & 0xCF == 0x09
        || opcode & 0xCF == 0x0B
        || matches!(
            opcode,
            0x02 | 0x0A | 0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0xE3 | 0xEB | 0xF9
        )
}

impl Cpu8080State {
    pub(crate) fn execute_data_opcode(
        &mut self,
        opcode: u8,
        mnemonic: String,
        pc_before: u16,
        t_states: u8,
    ) -> InstructionOutcome {
        if (0x40..=0x7F).contains(&opcode) {
            if opcode == 0x76 {
                self.pc = self.pc.wrapping_add(1);
                self.halted = true;
                return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
            }
            let dst = (opcode >> 3) & 7;
            let src = opcode & 7;
            let value = self.read_reg_code(src);
            self.write_reg_code(dst, value);
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xC7 == 0x06 {
            let reg = (opcode >> 3) & 7;
            // `MVI r, d8`: fetch_byte теперь `&mut self`, и
            // write_reg_code тоже — нельзя в одном выражении из-за
            // двойного заимствования. Локальная переменная для
            // immediate-байта.
            let value = self.fetch_byte(1);
            self.write_reg_code(reg, value);
            self.pc = self.pc.wrapping_add(2);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x01 {
            // `LXI rp, d16`: the operand passes through W/Z on its way
            // into the destination pair. Real microcode reads the low
            // byte into Z, then the high byte into W, then transfers
            // WZ → rp; we record the residue so the schematic shows
            // what the user just loaded (matches the reference
            // emulator's "Регистры временного хранения" readout).
            let value = self.fetch_word(1);
            self.registers.set_wz(value);
            self.write_pair(RegPair::from_code((opcode >> 4) & 3), value);
            self.pc = self.pc.wrapping_add(3);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x03 {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_add(1));
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x09 {
            let rhs = self.read_pair(RegPair::from_code((opcode >> 4) & 3));
            let hl = self.registers.hl();
            let sum = hl as u32 + rhs as u32;
            self.registers.set_hl(sum as u16);
            self.flags.carry = sum > 0xFFFF;
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        if opcode & 0xCF == 0x0B {
            let pair = RegPair::from_code((opcode >> 4) & 3);
            self.write_pair(pair, self.read_pair(pair).wrapping_sub(1));
            self.pc = self.pc.wrapping_add(1);
            return self.outcome(Some(opcode), mnemonic, pc_before, t_states, false);
        }

        match opcode {
            // `STAX (BC)` / `LDAX (BC)`: addressing through a register
            // pair does NOT route the address through W/Z on the real
            // chip — the pair is already on the address latch, so the
            // microcode goes straight to memory. Same for `(DE)`. We
            // intentionally leave WZ alone here so the schematic keeps
            // showing whatever the previous WZ-touching instruction
            // left, exactly like the reference emulator.
            0x02 => self.bus_write(self.registers.bc(), self.registers.a),
            0x0A => self.registers.a = self.bus_read(self.registers.bc()),
            0x12 => self.bus_write(self.registers.de(), self.registers.a),
            0x1A => self.registers.a = self.bus_read(self.registers.de()),
            // `SHLD a16` / `LHLD a16`: 16-bit immediate address goes
            // through W/Z. We record it before performing the memory
            // accesses so the residue reflects the address operand
            // even on the LHLD case where the helper itself only
            // holds locals.
            0x22 => self.shld(),
            0x2A => self.lhld(),
            // `STA a16` / `LDA a16`: same story — a16 is parked in WZ,
            // then a single byte transfer happens against (WZ).
            0x32 => {
                let address = self.fetch_word(1);
                self.registers.set_wz(address);
                self.bus_write(address, self.registers.a);
            }
            0x3A => {
                let address = self.fetch_word(1);
                self.registers.set_wz(address);
                self.registers.a = self.bus_read(address);
            }
            // `XTHL`: top of stack is read into WZ (lo, hi), then
            // swapped with HL. The reference emulator shows that
            // residue, so we record it.
            0xE3 => self.xthl(),
            // `XCHG`: HL ↔ DE. The 8080 microcode does this through
            // the W/Z scratch pair (HL → WZ, DE → HL, WZ → DE). We
            // record the previous HL there to match the residue
            // displayed by the reference emulator.
            0xEB => self.xchg(),
            // `SPHL`: HL → SP. Goes through WZ on the real chip.
            0xF9 => {
                let value = self.registers.hl();
                self.registers.set_wz(value);
                self.sp = value;
            }
            _ => unreachable!("data dispatch reached non-data opcode {opcode:#04X}"),
        }
        self.advance_data_pc(opcode);
        self.outcome(Some(opcode), mnemonic, pc_before, t_states, false)
    }

    fn advance_data_pc(&mut self, opcode: u8) {
        match opcode {
            0x02 | 0x0A | 0x12 | 0x1A | 0xE3 | 0xEB | 0xF9 => self.pc = self.pc.wrapping_add(1),
            0x22 | 0x2A | 0x32 | 0x3A => self.pc = self.pc.wrapping_add(3),
            _ => {}
        }
    }

    fn shld(&mut self) {
        let address = self.fetch_word(1);
        self.registers.set_wz(address);
        self.bus_write(address, self.registers.l);
        self.bus_write(address.wrapping_add(1), self.registers.h);
    }

    fn lhld(&mut self) {
        let address = self.fetch_word(1);
        self.registers.set_wz(address);
        self.registers.l = self.bus_read(address);
        self.registers.h = self.bus_read(address.wrapping_add(1));
    }

    fn xchg(&mut self) {
        // Real microcode: HL → WZ; DE → HL; WZ → DE. Net effect on
        // the programmer-visible state is the same as a straight swap
        // (which is what the previous implementation did via
        // `core::mem::swap`), but we record the residue so the
        // schematic shows the previous HL value sitting in WZ — that
        // is what the reference emulator displays after `XCHG`.
        let prev_hl = self.registers.hl();
        let prev_de = self.registers.de();
        self.registers.set_wz(prev_hl);
        self.registers.set_hl(prev_de);
        self.registers.set_de(prev_hl);
    }

    fn xthl(&mut self) {
        // `(SP)` is read into Z (low byte) and W (high byte), then
        // swapped with HL. Recording the WZ residue mirrors the
        // reference emulator and the 8080 microcode listing.
        let lo = self.bus_read(self.sp);
        let hi = self.bus_read(self.sp.wrapping_add(1));
        self.registers.w = hi;
        self.registers.z = lo;
        self.bus_write(self.sp, self.registers.l);
        self.bus_write(self.sp.wrapping_add(1), self.registers.h);
        self.registers.h = hi;
        self.registers.l = lo;
    }
}
