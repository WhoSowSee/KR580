//! Per-instruction timing metadata.
//!
//! Conditional control-flow instructions store *both* taken and not-taken
//! T-state counts (per `prompt/01_architecture.md`).

use serde::{Deserialize, Serialize};

/// T-state cost for an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionTiming {
    /// T-states when the instruction proceeds normally / branch is taken.
    pub t_states_taken: u8,
    /// T-states when a conditional branch is *not* taken. Equals
    /// `t_states_taken` for unconditional instructions.
    pub t_states_not_taken: u8,
    /// Optional machine-cycle count, included where it is well-defined for
    /// the family.
    pub machine_cycle_count: Option<u8>,
}

impl InstructionTiming {
    /// Build a fixed-cost timing.
    pub const fn fixed(t: u8) -> Self {
        Self {
            t_states_taken: t,
            t_states_not_taken: t,
            machine_cycle_count: None,
        }
    }

    /// Build a fixed-cost timing with explicit machine-cycle count.
    pub const fn fixed_m(t: u8, m: u8) -> Self {
        Self {
            t_states_taken: t,
            t_states_not_taken: t,
            machine_cycle_count: Some(m),
        }
    }

    /// Build a conditional timing.
    pub const fn cond(taken: u8, not_taken: u8) -> Self {
        Self {
            t_states_taken: taken,
            t_states_not_taken: not_taken,
            machine_cycle_count: None,
        }
    }
}
