use crate::{CoreError, Cpu8080State, PortBus, TactOutcome, decode_opcode};

struct TactSetup {
    opcode: Option<u8>,
    t_states: u8,
    branch_taken: bool,
}

impl Cpu8080State {
    pub fn step_tact<B: PortBus>(&mut self, bus: &mut B) -> Result<TactOutcome, CoreError> {
        if self.active_tacts_remaining == 0 {
            self.start_tact_walk()?;
        }

        let phase = self.active_tacts_total - self.active_tacts_remaining;
        self.active_tacts_remaining -= 1;
        self.cycle_count += 1;
        self.last_completed_tact_phase = Some(phase);

        let boundary = self.active_tacts_remaining == 0;
        if boundary {
            if self.active_opcode.is_some() || self.can_accept_interrupt() {
                self.execute_instruction_boundary(bus)?;
            }
            self.clear_active_tact();
        } else {
            self.tact_phase = Some(phase + 1);
        }

        Ok(TactOutcome {
            tact_phase: phase,
            instruction_boundary: boundary,
            cycle_count: self.cycle_count,
        })
    }

    fn start_tact_walk(&mut self) -> Result<(), CoreError> {
        let setup = self.tact_setup()?;
        self.active_tacts_total = setup.t_states;
        self.active_tacts_remaining = setup.t_states;
        self.active_opcode = setup.opcode;
        self.active_branch_taken = setup.branch_taken;
        self.tact_phase = Some(0);

        if let Some(opcode) = setup.opcode {
            self.last_fetched_opcode = opcode;
            self.last_address_bus = self.pc;
            self.last_data_bus_byte = opcode;
        }

        Ok(())
    }

    fn tact_setup(&self) -> Result<TactSetup, CoreError> {
        if self.can_accept_interrupt() {
            return Ok(TactSetup {
                opcode: self.interrupt_vector_byte,
                t_states: 11,
                branch_taken: true,
            });
        }
        if self.halted {
            return Ok(TactSetup {
                opcode: None,
                t_states: 1,
                branch_taken: true,
            });
        }

        let opcode = self.peek(self.pc);
        let info = decode_opcode(opcode)?;
        let branch_taken = self.branch_taken_for_tact(opcode);
        Ok(TactSetup {
            opcode: Some(opcode),
            t_states: info.timing.for_branch(branch_taken),
            branch_taken,
        })
    }

    fn branch_taken_for_tact(&self, opcode: u8) -> bool {
        if opcode & 0xC7 == 0xC0 || opcode & 0xC7 == 0xC2 || opcode & 0xC7 == 0xC4 {
            self.condition((opcode >> 3) & 7)
        } else {
            true
        }
    }
}
