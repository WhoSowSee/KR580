mod constants;
mod handlers;
mod keymap;
mod messages;
mod modal;
mod register_inline;
mod speed;
mod state;
mod subscription;
mod undo;
mod update;

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID, parse_register_name,
    register_name,
};
pub(crate) use messages::{MenuId, Message, RegisterInlineTarget, SpeedTier};
pub(crate) use modal::DiscardModalButton;
pub(crate) use register_inline::RegisterMove;
pub(crate) use speed::tier_hz;
pub(crate) use state::{DesktopApp, PendingAction};
pub(crate) use undo::UndoEntry;
