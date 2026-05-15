#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstructionTiming {
    pub t_states_taken: u8,
    pub t_states_not_taken: Option<u8>,
    pub machine_cycles: Option<u8>,
}

impl InstructionTiming {
    pub const fn fixed(t_states: u8) -> Self {
        Self {
            t_states_taken: t_states,
            t_states_not_taken: None,
            machine_cycles: None,
        }
    }

    pub const fn conditional(taken: u8, not_taken: u8) -> Self {
        Self {
            t_states_taken: taken,
            t_states_not_taken: Some(not_taken),
            machine_cycles: None,
        }
    }

    pub fn for_branch(self, taken: bool) -> u8 {
        if taken {
            self.t_states_taken
        } else {
            self.t_states_not_taken.unwrap_or(self.t_states_taken)
        }
    }
}
