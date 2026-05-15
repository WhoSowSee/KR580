//! Pure, deterministic Intel 8080 / KR580VM80 CPU core.
//!
//! No I/O, no UI, no async. The core exposes:
//!
//! * [`Cpu8080State`]      — owned register / flag / RAM / interrupt / timing state
//! * [`Flags`]              — 8080 PSW flags
//! * [`Memory64K`]          — flat 64 KiB RAM
//! * [`IoBus`]              — single trait that the executor calls for `IN` / `OUT`
//! * [`InstructionTiming`]  — taken / not-taken T-state metadata
//! * [`CoreError`]          — typed core errors
//!
//! The execution API is [`Cpu8080State::step_instruction`],
//! [`Cpu8080State::run_for_t_states`] and a debug-only
//! [`Cpu8080State::step_tact`].
//!
//! Semantics follow `prompt/02_cpu_core.md`, `prompt/07_cpu_opcode_semantics.md`,
//! and `prompt/opcode_dispatch.md` strictly. Undocumented opcode slots raise
//! [`DecodeError::UndocumentedOpcode`] and stop execution.

#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod commands;
pub mod decode;
pub mod error;
pub mod execute;
pub mod flags;
pub mod interrupt;
pub mod io;
pub mod memory;
pub mod state;
pub mod timing;

pub use commands::{CoreCommand, CoreEvent};
pub use error::{CoreError, DecodeError};
pub use flags::Flags;
pub use io::{IoBus, NullIoBus};
pub use memory::Memory64K;
pub use state::{Cpu8080State, Reg8, RegPair};
pub use timing::InstructionTiming;
