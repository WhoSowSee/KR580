use k580_core::RegisterName;

pub(crate) const MEMORY_ADDRESS_COUNT: usize = 0x1_0000;
pub(crate) const MEMORY_OVERSCAN_ROWS: usize = 12;
pub(crate) const MEMORY_RENDER_ROWS: usize = 96;
pub(crate) const MEMORY_ROW_HEIGHT: f32 = 28.0;
pub(crate) const MEMORY_SCROLL_ID: &str = "memory-scroll";

pub(crate) const MEMORY_ADDRESS_INPUT_ID: &str = "memory-address-input";
pub(crate) const MEMORY_VALUE_INPUT_ID: &str = "memory-value-input";
pub(crate) const REGISTER_NAME_INPUT_ID: &str = "register-name-input";
pub(crate) const REGISTER_VALUE_INPUT_ID: &str = "register-value-input";
pub(crate) const REGISTER_INLINE_INPUT_ID: &str = "register-inline-input";
pub(crate) const MEMORY_INLINE_INPUT_ID: &str = "memory-inline-input";
pub(crate) const OPCODE_SEARCH_INPUT_ID: &str = "opcode-search-input";

/// Number of 100 ms ticks the memory scrollbar stays visible after the
/// last scroll event.
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
