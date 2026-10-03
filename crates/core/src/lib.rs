pub mod bus;
pub mod decode;
pub mod error;
pub mod flags;
pub mod machine_cycle;
pub mod memory;
pub mod registers;
pub mod state;
pub mod timing;

mod execute;
mod ops;
mod tact;

pub use bus::{NullBus, PortBus};
pub use decode::{
    InstructionInfo, InstructionMetadata, decode_metadata, decode_opcode, is_undocumented_opcode,
};
pub use error::{CoreError, DecodeError, PortError, ValidationError};
pub use flags::Flags;
pub use machine_cycle::{
    MachineCycleKind, MachineCycleKinds, MachineCycleLayout, MachineCycleLengths,
    MachineCyclePosition, kind_at, layout_for, position_for,
};
pub use memory::Memory64K;
pub use registers::{RegisterName, Registers};
pub use state::{Cpu8080State, CpuMetadata, InstructionOutcome, InstructionStep, TactOutcome};
pub use timing::InstructionTiming;
