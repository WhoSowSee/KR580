use super::super::changelog::ChangelogDialog;
use super::super::help::HelpDialog;
use super::super::hex_stream_filter::HexStreamFilter;
use super::super::messages::{
    ExportTab, MenuId, Message, RegisterInlineTarget, SpeedTier, TopMenuFocus, TopMenuIndicator,
};
use super::super::modal::{DiscardModalButton, PendingAction};
use super::super::pending::PendingRequests;
use super::super::printer::PrinterSetupDialog;
use super::super::settings_modal::SettingsDialog;
use super::super::settings_notice::SettingsNotice;
use super::super::status::StatusKind;
use super::super::subprogram_modal::SubprogramDialog;
use super::super::undo::UndoStack;
use super::super::windows::ToolWindowState;
use super::super::{
    ExportFlagSelection, ExportMemoryColumns, ExportModalFocus, ExportRegisterSelection,
    ExportTarget, ImportFileFormat, ImportModalFocus,
};
use super::{DropdownState, MemoryView, OperandReturn};
use crate::i18n::Lang;
use crate::persistence::{ColorScheme, PrinterDialogMode, ShortcutSettings};
use crate::runtime::storage_files::ImagePreview;
use iced::{Point, Size, Task, keyboard};
use k580_core::RegisterName;
use std::path::PathBuf;
use std::time::Instant;

pub(crate) struct DocumentState {
    pub(crate) current_snapshot_path: Option<PathBuf>,
    pub(crate) current_subprogram_range: Option<(u16, u16)>,
    pub(crate) subprogram_dialog: Option<SubprogramDialog>,
    pub(crate) undo_stack: UndoStack,
    pub(crate) dirty: bool,
    pub(crate) saved_cpu: k580_core::Cpu8080State,
    pub(crate) discard_modal_focus: DiscardModalButton,
    pub(crate) discard_modal_keyboard_focus_visible: bool,
    pub(crate) pending_action: Option<PendingAction>,
    pub(crate) file_drag_hovered: bool,
    pub(crate) file_drag_cursor_position: Option<Point>,
    pub(crate) edit_epoch: u64,
}

pub(crate) struct MemoryEditorState {
    pub(crate) memory_scroll_first_row: u16,
    pub(crate) memory_scroll_offset: f32,
    pub(crate) memory_viewport_height: f32,
    pub(crate) memory_scroll_visible_ticks: u8,
    pub(crate) opcode_scroll_visible_ticks: u8,
    pub(crate) opcode_scroll_offset: f32,
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
    pub(crate) memory_inline_value_input: String,
    pub(crate) opcode_dropdown_address: Option<u16>,
    pub(crate) opcode_search_input: String,
    pub(crate) opcode_highlight_index: usize,
    /// Each search match changes the visible address; preserve the original query.
    pub(crate) memory_search_pattern: Option<String>,
    pub(crate) operand_return: Option<OperandReturn>,
    pub(crate) view: MemoryView,
}

pub(crate) struct RegisterEditorState {
    pub(crate) selected_register: RegisterName,
    pub(crate) register_name_input: String,
    pub(crate) register_value_input: String,
    pub(crate) active_register_target: Option<RegisterInlineTarget>,
    pub(crate) inline_register_target: Option<RegisterInlineTarget>,
    pub(crate) hovered_register_target: Option<RegisterInlineTarget>,
    pub(crate) inline_register_just_entered: bool,
}

pub(crate) struct InteractionState {
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
    /// iced 0.14 exposes no on_focus/on_blur callbacks for this marker.
    pub(crate) focused_input: Option<&'static str>,
    pub(crate) replacement_input: Option<&'static str>,
    pub(crate) replacement_placeholder: String,
    pub(crate) replacement_original_value: String,
    /// ButtonPressed supplies no coordinates; retain the last cursor position for hit tests.
    pub(crate) latest_cursor_position: Point,
    /// iced drops local click history when the first click swaps in a text input.
    pub(crate) previous_left_click: Option<iced::advanced::mouse::Click>,
    pub(crate) mouse_press_generation: u64,
    pub(crate) replacement_reconcile_guard: Option<(u64, &'static str)>,
}

pub(crate) struct ExecutionState {
    pub(crate) running: bool,
    /// A burst may auto-pause before Tick; the final PC still needs following.
    pub(crate) pending_follow_pc: bool,
    pub(crate) speed_tier: SpeedTier,
    pub(crate) halt_notice: Option<String>,
    pub(crate) halt_notice_dismiss_at: Option<Instant>,
    /// The post-HLT execution lock outlives the fading halt notice.
    pub(crate) run_blocked_after_halt: bool,
}

pub(crate) struct ExportDialogState {
    pub(crate) export_modal_open: bool,
    pub(crate) export_tab: ExportTab,
    pub(crate) export_modal_focus: ExportModalFocus,
    pub(crate) export_modal_keyboard_focus_visible: bool,
    pub(crate) export_xlsx_page_input: String,
    pub(crate) export_text_section_input: String,
    pub(crate) export_xlsx_pages: Vec<ExportTarget>,
    pub(crate) export_text_sections: Vec<ExportTarget>,
    pub(crate) export_target_dropdown: DropdownState,
    pub(crate) export_memory_start_input: String,
    pub(crate) export_memory_end_input: String,
    pub(crate) export_memory_columns: ExportMemoryColumns,
    pub(crate) export_registers: ExportRegisterSelection,
    pub(crate) export_flags: ExportFlagSelection,
}

pub(crate) struct ImportDialogState {
    pub(crate) generation: u64,
    pub(crate) loading: bool,
    pub(crate) import_modal_open: bool,
    pub(crate) import_modal_focus: ImportModalFocus,
    pub(crate) import_modal_keyboard_focus_visible: bool,
    pub(crate) import_file_drag_hovered: bool,
    pub(crate) import_file_path: Option<PathBuf>,
    pub(crate) import_file_display: String,
    pub(crate) import_file_format: Option<ImportFileFormat>,
    pub(crate) import_target_options: Vec<String>,
    pub(crate) import_target_input: String,
    pub(crate) import_target_dropdown: DropdownState,
    pub(crate) import_error: Option<String>,
}

pub(crate) struct PreferencesState {
    pub(crate) stored: crate::persistence::Settings,
    pub(crate) dialog_generation: u64,
    pub(crate) directory_generation: u64,
    pub(crate) lang: Lang,
    pub(crate) default_speed: SpeedTier,
    pub(crate) color_scheme: ColorScheme,
    pub(crate) shortcut_settings: ShortcutSettings,
    pub(crate) settings_dialog: Option<SettingsDialog>,
    pub(crate) settings_notice: Option<SettingsNotice>,
    pub(crate) file_association_pending: bool,
    pub(crate) follow_pc: bool,
    pub(crate) memory_operand_highlighting: bool,
    pub(crate) show_file_name: bool,
    pub(crate) menu_categories_visible: bool,
}

pub(crate) struct PrinterSetupState {
    pub(crate) printer_default_settings: Option<k580_ui::devices::printer::PrinterSettings>,
    pub(crate) printer_dialog_mode: PrinterDialogMode,
    pub(crate) printer_session_settings: Option<k580_ui::devices::printer::PrinterSettings>,
    pub(crate) printer_setup_dialog: Option<PrinterSetupDialog>,
    pub(crate) printer_setup_window_id: Option<iced::window::Id>,
    pub(crate) printer_properties_window_id: Option<iced::window::Id>,
    pub(crate) printer_setup_pending: bool,
}

pub(crate) struct DevicePanels {
    pub(crate) monitor_window: ToolWindowState,
    pub(crate) floppy_window: ToolWindowState,
    pub(crate) hdd_window: ToolWindowState,
    pub(crate) network_window: ToolWindowState,
    pub(crate) printer_window: ToolWindowState,
    pub(crate) monitor_open: bool,
    pub(crate) monitor_split: bool,
    pub(crate) monitor_hex_popup: bool,
    pub(crate) monitor_hex_scroll_visible_ticks: u8,
    pub(crate) monitor_hex_scroll_offset: f32,
    pub(crate) monitor_hex_filter: HexStreamFilter,
    pub(crate) floppy_open: bool,
    pub(crate) hdd_open: bool,
    pub(crate) network_open: bool,
    pub(crate) printer_open: bool,
    pub(crate) printer_text_view: bool,
    pub(crate) network_text_view: bool,
    pub(crate) network_settings_open: bool,
    pub(crate) network_mode_draft: crate::backend::NetworkMode,
    pub(crate) network_host_input: String,
    pub(crate) network_port_input: String,
    pub(crate) network_settings_error: Option<String>,
    pub(crate) hdd_file_exists: bool,
    pub(crate) hdd_generation: u64,
    pub(crate) hdd_show_image_contents: bool,
    pub(crate) hdd_image: ImagePreview,
    pub(crate) floppy_show_image_contents: bool,
    pub(crate) floppy_image: ImagePreview,
}

pub(crate) struct ShellState {
    pub(crate) status: String,
    pub(crate) status_kind: StatusKind,
    pub(crate) startup_frames_seen: u8,
    pub(crate) main_window_size: Size,
    pub(crate) open_menu: Option<MenuId>,
    pub(crate) top_menu_focus: Option<TopMenuFocus>,
    pub(crate) top_menu_indicator: TopMenuIndicator,
    pub(crate) about_dialog_open: bool,
    pub(crate) error_notice: Option<String>,
    pub(crate) error_notice_dismiss_at: Option<Instant>,
    pub(crate) main_window_id: Option<iced::window::Id>,
    pub(crate) window_maximized: bool,
    pub(crate) changelog_dialog: Option<ChangelogDialog>,
    pub(crate) help_dialog: Option<HelpDialog>,
}

pub(crate) struct RequestState {
    pub(crate) file_worker: Option<crate::runtime::file_work::FileWorker>,
    pub(crate) pending_requests: PendingRequests,
    pub(crate) tasks: Vec<Task<Message>>,
}
