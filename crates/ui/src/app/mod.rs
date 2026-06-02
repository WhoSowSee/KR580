mod constants;
mod handlers;
mod help;
mod help_routing;
mod keymap;
pub(crate) mod messages;
mod modal;
mod register_inline;
pub(crate) mod settings_modal;
mod speed;
mod state;
mod status;
mod subscription;
mod undo;
mod update;
mod update_overlays;
mod update_settings;

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID, SETTINGS_SEARCH_INPUT_ID,
    parse_register_name, register_name,
};
pub(crate) use help::{
    HelpDialog, HelpMarkdownHighlight, HelpMarkdownHighlighter, HelpMarkdownLine, HelpNode,
    parse_help_markdown_line,
};
pub(crate) use messages::{MenuId, Message, RegisterInlineTarget, SettingsCategory, SpeedTier};
pub(crate) use modal::DiscardModalButton;
pub(crate) use register_inline::RegisterMove;
pub(crate) use settings_modal::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsDialog, SettingsSection,
};
pub(crate) use speed::tier_hz;
pub(crate) use state::{DesktopApp, HexStreamFilter, PendingAction};
pub(crate) use status::{StatusKind, shorten_status_for_width};
pub(crate) use undo::UndoEntry;
