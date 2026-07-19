mod constants;
mod export_modal;
mod export_modal_state;
mod export_modal_targets;
#[cfg(test)]
mod export_modal_tests;
mod handlers;
#[cfg(test)]
mod handlers_tests;
mod help;
mod help_routing;
mod hex_stream_filter;
mod import_modal;
mod import_modal_state;
#[cfg(test)]
mod import_modal_tests;
mod keymap;
pub(crate) mod messages;
mod modal;
mod network;
mod opcode_picker;
mod printer;
mod register_inline;
pub(crate) mod settings_modal;
mod settings_saved_notice;
pub(crate) mod shortcuts;
mod speed;
mod state;
mod state_helpers;
mod status;
mod subscription;
mod undo;
mod update;
mod update_overlays;
mod update_routes;
mod update_settings;
#[cfg(test)]
mod window_tests;
mod windows;

pub(crate) use constants::{
    MEMORY_ADDRESS_COUNT, MEMORY_ADDRESS_INPUT_ID, MEMORY_INLINE_INPUT_ID, MEMORY_OVERSCAN_ROWS,
    MEMORY_RENDER_ROWS, MEMORY_ROW_HEIGHT, MEMORY_SCROLL_ID, MEMORY_SCROLL_VISIBLE_TICKS,
    MEMORY_VALUE_INPUT_ID, OPCODE_SEARCH_INPUT_ID, REGISTER_INLINE_INPUT_ID,
    REGISTER_NAME_INPUT_ID, REGISTER_ORDER, REGISTER_VALUE_INPUT_ID, SETTINGS_SEARCH_INPUT_ID,
    STACK_VIEW_SIZE, STACK_VIEW_START, parse_register_name, register_name,
};
pub(crate) use export_modal_state::{
    ExportFlagSelection, ExportMemoryColumns, ExportModalFocus, ExportRegisterSelection,
    ExportTargetSettings,
};
pub(crate) use help::{
    HelpDialog, HelpMarkdownHighlight, HelpMarkdownHighlighter, HelpMarkdownLine, HelpNode,
    parse_help_markdown_line,
};
pub(crate) use hex_stream_filter::HexStreamFilter;
pub(crate) use import_modal_state::{ImportFileFormat, ImportModalFocus};
pub(crate) use messages::{
    ExportFlag, ExportMemoryColumn, ExportRegister, ExportTab, MenuId, Message,
    RegisterInlineTarget, SettingsCategory, SpeedTier, ToolWindowKind,
};
pub(crate) use modal::DiscardModalButton;
pub(crate) use opcode_picker::{OpcodeChoice, filtered_opcode_choices};
pub(crate) use printer::{
    PRINTER_PROPERTIES_PRESET_INPUT_ID, PrinterPropertiesDialog, PrinterPropertiesFocus,
    PrinterPropertiesTab, PrinterPropertyDropdown, PrinterSetupDialog, PrinterSetupDropdown,
    PrinterSetupFocus, printer_property_parameter_input_id,
};
pub(crate) use register_inline::RegisterMove;
pub(crate) use settings_modal::{
    ContentFocus, FooterFocus, ResetConfirmFocus, SettingsDialog, SettingsSection,
};
pub(crate) use settings_saved_notice::{SettingsSavedNotice, SettingsSavedNoticePresentation};
pub(crate) use speed::tier_hz;
pub(crate) use state::{DesktopApp, PendingAction};
pub(crate) use status::{StatusKind, shorten_status_for_width};
pub(crate) use undo::UndoEntry;
