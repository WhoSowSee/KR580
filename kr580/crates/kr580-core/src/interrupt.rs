//! Interrupt servicing.
//!
//! Per `prompt/02_cpu_core.md`:
//!
//! * an accepted interrupt clears `interrupt_enable`, clears `halted`, and
//!   consumes the pending vector byte;
//! * an interrupt requested while `interrupt_enable=false` stays pending;
//! * `EI` does not enable interrupts immediately — `interrupt_enable_pending`
//!   is set, then promoted to live at the *next* instruction boundary;
//! * `interrupt_vector_byte` carries one single-byte opcode (typically
//!   `RST n`); multi-byte sequences are out of scope.

use crate::state::Cpu8080State;

/// If an interrupt is pending and accepted at this instruction boundary,
/// service it and return the consumed T-states. Otherwise return `None`.
pub fn handle_pending_interrupt(cpu: &mut Cpu8080State) -> Option<u32> {
    if !cpu.interrupt_request_pending {
        return None;
    }
    if !cpu.interrupt_enable {
        // Pending but masked — leave the request pending.
        return None;
    }
    let vector = cpu.interrupt_vector_byte.take()?;

    // Acceptance: clear enable, exit halt, drop request.
    cpu.interrupt_enable = false;
    cpu.interrupt_enable_pending = false;
    cpu.interrupt_request_pending = false;
    cpu.halted = false;

    // Only `RST n` (encoding 0b11_xxx_111) is supported as a single-byte
    // interrupt vector. Anything else is rejected to avoid silently
    // emulating undocumented multi-byte acknowledge sequences.
    if (vector & 0b11_000_111) == 0b11_000_111 {
        let n = ((vector >> 3) & 0b111) as u16;
        let ret = cpu.pc;
        cpu.push_word(ret);
        cpu.pc = n * 8;
        // RST is documented as 11 T-states; interrupt acknowledge adds
        // ~4 more on real hardware. We charge 11 and leave acknowledge
        // overhead as out-of-scope.
        return Some(11);
    }

    // Unknown single-byte vector: ignore for now and consume one T-state.
    Some(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ei_is_deferred_one_instruction() {
        // Per prompt: EI does not enable interrupts until *after* the next
        // instruction boundary.
        let mut c = Cpu8080State::new();
        c.ram.write(0, 0xFB); // EI
        c.ram.write(1, 0x00); // NOP
        c.ram.write(2, 0x00); // NOP
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert!(
            !c.interrupt_enable,
            "EI must not enable until next boundary"
        );
        assert!(c.interrupt_enable_pending);
        // Boundary 2: pending becomes live before fetching next instruction.
        c.step_instruction(&mut bus).unwrap();
        assert!(c.interrupt_enable);
        assert!(!c.interrupt_enable_pending);
    }

    #[test]
    fn pending_irq_with_disabled_stays_pending() {
        let mut c = Cpu8080State::new();
        c.interrupt_enable = false;
        c.interrupt_request_pending = true;
        c.interrupt_vector_byte = Some(0xFF); // RST 7
        c.ram.write(0, 0x00); // NOP
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert!(c.interrupt_request_pending);
        assert_eq!(c.pc, 1, "must execute regular NOP");
    }

    #[test]
    fn enabled_irq_routes_through_rst_vector() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        c.interrupt_enable = true;
        c.interrupt_request_pending = true;
        c.interrupt_vector_byte = Some(0xFF); // RST 7
        c.pc = 0x1000;
        let mut bus = crate::io::NullIoBus;
        let t = c.step_instruction(&mut bus).unwrap();
        assert_eq!(t, 11);
        assert_eq!(c.pc, 0x38);
        assert_eq!(c.sp, 0x1FFE);
        assert!(!c.interrupt_enable);
        assert!(!c.interrupt_request_pending);
        assert_eq!(c.interrupt_vector_byte, None);
    }

    #[test]
    fn irq_unhalts_cpu() {
        let mut c = Cpu8080State::new();
        c.sp = 0x2000;
        c.halted = true;
        c.interrupt_enable = true;
        c.interrupt_request_pending = true;
        c.interrupt_vector_byte = Some(0xCF); // RST 1
        let mut bus = crate::io::NullIoBus;
        c.step_instruction(&mut bus).unwrap();
        assert!(!c.halted);
        assert_eq!(c.pc, 0x08);
    }
}
