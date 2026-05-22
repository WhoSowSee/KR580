//! Compile-time identifiers and small lookup helpers shared by the UI
//! state, the view code, and the runtime helpers.
//!
//! These live in their own module so neither `app/mod.rs` nor `runtime`
//! has to host an unrelated soup of static strings, and so the constants
//! can be re-exported from `crate::app::*` without dragging the rest of
//! the application state along.

use k580_core::RegisterName;

pub(crate) const MEMORY_ADDRESS_COUNT: usize = 0x1_0000;
pub(crate) const MEMORY_OVERSCAN_ROWS: usize = 12;
pub(crate) const MEMORY_RENDER_ROWS: usize = 96;
pub(crate) const MEMORY_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MEMORY_SCROLL_ID: &str = "memory-scroll";

/// Stable widget identifiers for every text input we want to drive with
/// keyboard navigation. They define isolated focus rings so that Tab/Shift+Tab
/// only cycles inside the panel that currently owns the focus instead of
/// walking through every focusable widget in the application.
pub(crate) const MEMORY_ADDRESS_INPUT_ID: &str = "memory-address-input";
pub(crate) const MEMORY_VALUE_INPUT_ID: &str = "memory-value-input";
pub(crate) const REGISTER_NAME_INPUT_ID: &str = "register-name-input";
pub(crate) const REGISTER_VALUE_INPUT_ID: &str = "register-value-input";
/// The inline value editor inside the memory list. Only one such input is
/// rendered at a time (for the currently selected address), so a single ID
/// keeps focus continuity when the user steps from one row to the next.
pub(crate) const MEMORY_INLINE_INPUT_ID: &str = "memory-inline-input";
/// Search field inside the floating opcode picker. Carries an id so the
/// hotkey-driven open path (E from "no-focus") can chain a focus task
/// straight after toggling the dropdown — the user expects to start typing
/// the mnemonic immediately, the same way the click-driven open feels.
pub(crate) const OPCODE_SEARCH_INPUT_ID: &str = "opcode-search-input";

/// Number of 100 ms ticks the memory scrollbar stays visible after the last
/// scroll event. 12 ticks ≈ 1.2 seconds.
pub(crate) const MEMORY_SCROLL_VISIBLE_TICKS: u8 = 12;

pub(crate) const REGISTER_ORDER: [RegisterName; 7] = [
    RegisterName::A,
    RegisterName::B,
    RegisterName::C,
    RegisterName::D,
    RegisterName::E,
    RegisterName::H,
    RegisterName::L,
];

pub(crate) fn register_name(register: RegisterName) -> &'static str {
    match register {
        RegisterName::A => "A",
        RegisterName::B => "B",
        RegisterName::C => "C",
        RegisterName::D => "D",
        RegisterName::E => "E",
        RegisterName::H => "H",
        RegisterName::L => "L",
    }
}

pub(crate) fn parse_register_name(input: &str) -> Option<RegisterName> {
    REGISTER_ORDER
        .iter()
        .copied()
        .find(|register| register_name(*register).eq_ignore_ascii_case(input.trim()))
}
