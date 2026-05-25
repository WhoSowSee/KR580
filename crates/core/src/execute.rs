use crate::ops::{alu, control, data, misc, stack};
use crate::{CoreError, Cpu8080State, DecodeError, InstructionOutcome, PortBus, decode_opcode};

impl Cpu8080State {
    pub(crate) fn can_accept_interrupt(&self) -> bool {
        self.interrupt_request_pending
            && self.interrupt_enable
            && self.interrupt_vector_byte.is_some()
    }

    pub(crate) fn execute_instruction_boundary<B: PortBus>(
        &mut self,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        if self.can_accept_interrupt() {
            return self.accept_interrupt();
        }
        if self.halted {
            return Ok(self.outcome(None, "HALTED", self.pc, 0, false));
        }

        let pending_before = self.interrupt_enable_pending;
        // Через `fetch_opcode`, не `memory.read`: M1-цикл должен
        // зафиксировать байт в РК (`last_fetched_opcode`) и пройти
        // через буферы адреса/данных, иначе четыре readout'а на
        // схематике (РК, Д/Ш команд, Буфер данных, Буфер адреса)
        // показывали бы look-ahead в RAM по PC, а не реальный
        // последний байт через шину. После `HLT` это особенно
        // заметно: PC шагает на адрес после HLT, RAM там лежит
        // 00 (NOP), а РК должен продолжать держать 76.
        let opcode = self.fetch_opcode();
        let outcome = self.execute_opcode(opcode, bus)?;
        if pending_before && self.interrupt_enable_pending {
            self.interrupt_enable = true;
            self.interrupt_enable_pending = false;
        }
        Ok(outcome)
    }

    fn accept_interrupt(&mut self) -> Result<InstructionOutcome, CoreError> {
        let vector = self
            .interrupt_vector_byte
            .take()
            .expect("checked by can_accept_interrupt");
        if vector & 0xC7 != 0xC7 {
            return Err(DecodeError::InvalidInterruptVector(vector).into());
        }
        self.interrupt_enable = false;
        self.interrupt_enable_pending = false;
        self.interrupt_request_pending = false;
        self.halted = false;

        let pc_before = self.pc;
        let rst = (vector >> 3) & 7;
        self.push_word(self.pc);
        self.pc = u16::from(rst) * 8;
        Ok(InstructionOutcome {
            opcode: Some(vector),
            mnemonic: format!("RST {}", rst),
            pc_before,
            pc_after: self.pc,
            t_states: 11,
            halted: false,
            interrupt_accepted: true,
        })
    }

    fn execute_opcode<B: PortBus>(
        &mut self,
        opcode: u8,
        bus: &mut B,
    ) -> Result<InstructionOutcome, CoreError> {
        let pc_before = self.pc;
        let info = decode_opcode(opcode)?;
        let mnemonic = info.mnemonic;
        let t_states = info.timing.t_states_taken;

        if data::handles(opcode) {
            return Ok(self.execute_data_opcode(opcode, mnemonic, pc_before, t_states));
        }

        if alu::handles(opcode) {
            return Ok(self.execute_alu_opcode(opcode, mnemonic, pc_before, t_states));
        }

        if stack::handles(opcode) {
            return Ok(self.execute_stack_opcode(opcode, mnemonic, pc_before, t_states));
        }

        if control::handles(opcode) {
            return Ok(self.execute_control_opcode(opcode, mnemonic, pc_before, t_states));
        }

        debug_assert!(misc::handles(opcode));
        self.execute_misc(opcode, bus, pc_before, mnemonic)
    }
}
