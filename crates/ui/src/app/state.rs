use crate::backend::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use iced::{Point, Size, Task, keyboard};
use k580_core::RegisterName;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::help::HelpDialog;
use super::hex_stream_filter::HexStreamFilter;
use super::messages::{ExportTab, MenuId, Message, RegisterInlineTarget, SpeedTier};
use super::modal::DiscardModalButton;
use super::printer::PrinterSetupDialog;
use super::settings_modal::SettingsDialog;
use super::status::StatusKind;
use super::undo::UndoStack;
use super::windows::ToolWindowState;
use super::{
    ExportFlagSelection, ExportMemoryColumns, ExportModalFocus, ExportRegisterSelection,
    ExportTargetSettings, ImportFileFormat, ImportModalFocus,
};
use crate::i18n::{Key, Lang};
use crate::persistence::{ColorScheme, PrinterDialogMode, ShortcutSettings};
use crate::settings_storage::{lang_from_language, load_settings, speed_tier_from_preset};

#[derive(Clone, Debug)]
pub(crate) enum PendingAction {
    OpenSnapshot,
    NewFile,
    Import,
    CloseWindow,
    DeleteHdd,
}

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) status: String,
    pub(crate) status_kind: StatusKind,
    pub(crate) selected_register: RegisterName,
    pub(crate) register_name_input: String,
    pub(crate) register_value_input: String,
    pub(crate) active_register_target: Option<RegisterInlineTarget>,
    pub(crate) inline_register_target: Option<RegisterInlineTarget>,
    pub(crate) hovered_register_target: Option<RegisterInlineTarget>,
    pub(crate) memory_scroll_first_row: u16,
    pub(crate) memory_scroll_offset: f32,
    pub(crate) memory_viewport_height: f32,
    pub(crate) memory_scroll_visible_ticks: u8,
    pub(crate) opcode_scroll_visible_ticks: u8,
    pub(crate) monitor_hex_scroll_visible_ticks: u8,
    pub(crate) memory_address_input: String,
    pub(crate) memory_value_input: String,
    pub(crate) memory_inline_value_input: String,
    pub(crate) opcode_dropdown_address: Option<u16>,
    pub(crate) opcode_search_input: String,
    pub(crate) opcode_highlight_index: usize,
    /// Stored separately because every successful match overwrites
    /// `memory_address_input` with the matched 4-digit address.
    pub(crate) memory_search_pattern: Option<String>,
    pub(crate) memory_operand_return_address: Option<u16>,
    pub(crate) memory_operand_return_scroll_offset: Option<f32>,
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
    /// Cosmetic focus marker – iced 0.14 has no on_focus / on_blur.
    pub(crate) focused_input: Option<&'static str>,
    pub(crate) replacement_input: Option<&'static str>,
    pub(crate) replacement_placeholder: String,
    pub(crate) replacement_original_value: String,
    /// Cached for `MousePressed` – `ButtonPressed` carries identity
    /// only, not coordinates.
    pub(crate) latest_cursor_position: Point,
    /// iced drops local click history when the first click swaps in a text input.
    pub(crate) previous_left_click: Option<iced::advanced::mouse::Click>,
    pub(crate) mouse_press_generation: u64,
    pub(crate) replacement_reconcile_guard: Option<(u64, &'static str)>,
    pub(crate) running: bool,
    /// One-shot signal that the next `Tick` must run `follow_pc_during_run`
    /// even though `running` is already false (high-speed bursts where
    /// auto-pause clears `running` before Tick reads it).
    pub(crate) pending_follow_pc: bool,
    pub(crate) inline_register_just_entered: bool,
    /// Set on `TactAdvanced { instruction_boundary: true }`.
    pub(crate) last_tact_was_boundary: bool,
    pub(crate) startup_frames_seen: u8,
    pub(crate) main_window_size: Size,
    pub(crate) open_menu: Option<MenuId>,
    pub(crate) about_dialog_open: bool,
    pub(crate) current_snapshot_path: Option<PathBuf>,
    pub(crate) speed_tier: SpeedTier,
    pub(crate) halt_notice: Option<String>,
    pub(crate) halt_notice_dismiss_at: Option<Instant>,
    /// Disables every execution-side button until reset. Outlives the
    /// halt notice's 8-second fade – the contract is "until reset",
    /// not "until the message disappears".
    pub(crate) run_blocked_after_halt: bool,
    pub(crate) error_notice: Option<String>,
    pub(crate) error_notice_dismiss_at: Option<Instant>,

    pub(crate) main_window_id: Option<iced::window::Id>,
    pub(crate) monitor_window: ToolWindowState,
    pub(crate) floppy_window: ToolWindowState,
    pub(crate) hdd_window: ToolWindowState,
    pub(crate) network_window: ToolWindowState,
    pub(crate) printer_window: ToolWindowState,
    pub(crate) window_maximized: bool,
    pub(crate) follow_pc: bool,
    pub(crate) memory_operand_highlighting: bool,
    pub(crate) menu_categories_visible: bool,
    pub(crate) undo_stack: UndoStack,
    pub(crate) dirty: bool,
    pub(crate) saved_cpu: k580_core::Cpu8080State,
    pub(crate) discard_modal_focus: DiscardModalButton,
    pub(crate) pending_action: Option<PendingAction>,
    pub(crate) export_modal_open: bool,
    pub(crate) export_tab: ExportTab,
    pub(crate) export_modal_focus: ExportModalFocus,
    pub(crate) export_xlsx_page_input: String,
    pub(crate) export_text_section_input: String,
    pub(crate) export_xlsx_pages: Vec<String>,
    pub(crate) export_text_sections: Vec<String>,
    pub(crate) export_xlsx_page_settings: Vec<ExportTargetSettings>,
    pub(crate) export_text_section_settings: Vec<ExportTargetSettings>,
    pub(crate) export_target_dropdown_open: bool,
    pub(crate) export_target_highlight: Option<usize>,
    pub(crate) export_memory_start_input: String,
    pub(crate) export_memory_end_input: String,
    pub(crate) export_memory_columns: ExportMemoryColumns,
    pub(crate) export_registers: ExportRegisterSelection,
    pub(crate) export_flags: ExportFlagSelection,
    pub(crate) import_modal_open: bool,
    pub(crate) import_modal_focus: ImportModalFocus,
    pub(crate) import_file_path: Option<PathBuf>,
    pub(crate) import_file_display: String,
    pub(crate) import_file_format: Option<ImportFileFormat>,
    pub(crate) import_target_options: Vec<String>,
    pub(crate) import_target_input: String,
    pub(crate) import_target_dropdown_open: bool,
    pub(crate) import_target_highlight: Option<usize>,
    pub(crate) import_target_scroll_visible_ticks: u8,
    pub(crate) import_error: Option<String>,
    pub(crate) lang: Lang,
    pub(crate) default_speed: SpeedTier,
    pub(crate) color_scheme: ColorScheme,
    pub(crate) shortcut_settings: ShortcutSettings,
    pub(crate) settings_dialog: Option<SettingsDialog>,
    pub(crate) printer_default_settings: Option<k580_ui::devices::printer::PrinterSettings>,
    pub(crate) printer_dialog_mode: PrinterDialogMode,
    pub(crate) printer_session_settings: Option<k580_ui::devices::printer::PrinterSettings>,
    pub(crate) printer_setup_dialog: Option<PrinterSetupDialog>,
    pub(crate) printer_setup_pending: bool,
    /// Bumped whenever the OS file-association state changes so the
    /// settings overlay re-renders even when the dialog struct itself
    /// is unchanged.
    pub(crate) file_association_toggle_revision: u64,
    /// Last known OS file-association state, used by `handle_tick` to
    /// detect external changes.
    pub(crate) file_association_last_registered: bool,
    pub(crate) monitor_open: bool,
    pub(crate) monitor_split: bool,
    pub(crate) monitor_hex_popup: bool,
    pub(crate) floppy_open: bool,
    pub(crate) hdd_open: bool,
    pub(crate) network_open: bool,
    pub(crate) printer_open: bool,
    pub(crate) printer_text_view: bool,
    pub(crate) network_text_view: bool,
    pub(crate) stack_view: bool,
    pub(crate) stack_view_saved_address: Option<u16>,
    pub(crate) stack_view_saved_scroll_offset: f32,
    pub(crate) network_settings_open: bool,
    pub(crate) network_mode_draft: crate::backend::NetworkMode,
    pub(crate) network_host_input: String,
    pub(crate) network_port_input: String,
    pub(crate) network_settings_error: Option<String>,
    pub(crate) hdd_file_exists: bool,
    pub(crate) hdd_show_image_contents: bool,
    pub(crate) hdd_image_contents: Vec<u8>,
    pub(crate) hdd_image_error: Option<String>,
    pub(crate) floppy_show_image_contents: bool,
    pub(crate) floppy_image_contents: Vec<u8>,
    pub(crate) floppy_image_error: Option<String>,
    pub(crate) help_dialog: Option<HelpDialog>,
    pub(crate) monitor_hex_filter: HexStreamFilter,
}

impl DesktopApp {
    pub(crate) fn with_initial_path(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let handle = spawn_emulator();
        let startup_task = match initial {
            Some(path) => Task::done(Message::LoadSnapshotFromPath(path)),
            None => Task::none(),
        };
        let settings = load_settings();
        let lang = lang_from_language(settings.general.language);
        let _ = handle.send(crate::backend::AppCommand::AttachHddFile(
            crate::runtime::storage_files::hdd_default_path(),
        ));
        if let Some(ref path) = settings.general.floppy_image_path
            && path.is_file()
        {
            let _ = handle.send(crate::backend::AppCommand::AttachFloppyImage(path.clone()));
        }
        let network_mode = crate::backend::NetworkMode::Client;
        let network_host = settings.network.host.clone();
        let network_port = settings.network.port;
        let _ = handle.send(crate::backend::AppCommand::ConfigureNetwork {
            mode: network_mode,
            host: network_host.clone(),
            port: network_port,
        });
        let default_speed = speed_tier_from_preset(settings.general.default_speed);
        let printer_default_settings = settings.general.printer_settings.clone();
        let color_scheme = settings.ui.theme;
        let follow_pc = settings.general.follow_pc;
        let memory_operand_highlighting = settings.general.memory_operand_highlighting;
        let initial_status_kind = StatusKind::Ready;
        let initial_status = initial_status_kind
            .render(lang)
            .unwrap_or_else(|| lang.t(Key::StatusReady).to_owned());
        let mut app = Self {
            handle,
            snapshot: initial_snapshot(),
            status: initial_status,
            status_kind: initial_status_kind,
            selected_register: RegisterName::A,
            register_name_input: String::new(),
            register_value_input: String::new(),
            active_register_target: None,
            inline_register_target: None,
            hovered_register_target: None,
            memory_scroll_first_row: 0,
            memory_scroll_offset: 0.0,
            memory_viewport_height: 0.0,
            memory_scroll_visible_ticks: 0,
            opcode_scroll_visible_ticks: 0,
            monitor_hex_scroll_visible_ticks: 0,
            memory_address_input: String::new(),
            memory_value_input: String::new(),
            memory_inline_value_input: String::new(),
            opcode_dropdown_address: None,
            opcode_search_input: String::new(),
            opcode_highlight_index: 0,
            memory_search_pattern: None,
            memory_operand_return_address: None,
            memory_operand_return_scroll_offset: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            focused_input: None,
            replacement_input: None,
            replacement_placeholder: String::new(),
            replacement_original_value: String::new(),
            latest_cursor_position: Point::ORIGIN,
            previous_left_click: None,
            mouse_press_generation: 0,
            replacement_reconcile_guard: None,
            running: false,
            pending_follow_pc: false,
            inline_register_just_entered: false,
            last_tact_was_boundary: false,
            startup_frames_seen: 0,
            main_window_size: Size::new(1180.0, 720.0),
            open_menu: None,
            about_dialog_open: false,
            current_snapshot_path: None,
            speed_tier: default_speed,
            halt_notice: None,
            halt_notice_dismiss_at: None,
            run_blocked_after_halt: false,
            error_notice: None,
            error_notice_dismiss_at: None,
            main_window_id: None,
            monitor_window: ToolWindowState::default(),
            floppy_window: ToolWindowState::default(),
            hdd_window: ToolWindowState::default(),
            network_window: ToolWindowState::default(),
            printer_window: ToolWindowState::default(),
            window_maximized: false,
            menu_categories_visible: true,
            follow_pc,
            memory_operand_highlighting,
            undo_stack: UndoStack::default(),
            dirty: false,
            saved_cpu: k580_core::Cpu8080State::default(),
            discard_modal_focus: DiscardModalButton::Cancel,
            pending_action: None,
            export_modal_open: false,
            export_tab: ExportTab::Xlsx,
            export_modal_focus: ExportModalFocus::TabXlsx,
            export_xlsx_page_input: lang.t(Key::ExportPageDefault).to_owned(),
            export_text_section_input: lang.t(Key::ExportSectionDefault).to_owned(),
            export_xlsx_pages: vec![lang.t(Key::ExportPageDefault).to_owned()],
            export_text_sections: vec![lang.t(Key::ExportSectionDefault).to_owned()],
            export_xlsx_page_settings: vec![ExportTargetSettings::default()],
            export_text_section_settings: vec![ExportTargetSettings::default()],
            export_target_dropdown_open: false,
            export_target_highlight: None,
            export_memory_start_input: "0000".to_owned(),
            export_memory_end_input: "FFFF".to_owned(),
            export_memory_columns: ExportMemoryColumns::default(),
            export_registers: ExportRegisterSelection::default(),
            export_flags: ExportFlagSelection::default(),
            import_modal_open: false,
            import_modal_focus: ImportModalFocus::Browse,
            import_file_path: None,
            import_file_display: String::new(),
            import_file_format: None,
            import_target_options: Vec::new(),
            import_target_input: String::new(),
            import_target_dropdown_open: false,
            import_target_highlight: None,
            import_target_scroll_visible_ticks: 0,
            import_error: None,
            lang,
            default_speed,
            color_scheme,
            shortcut_settings: settings.shortcuts.clone(),
            settings_dialog: None,
            printer_default_settings,
            printer_dialog_mode: settings.general.printer_dialog_mode,
            printer_session_settings: None,
            printer_setup_dialog: None,
            printer_setup_pending: false,
            file_association_toggle_revision: 0,
            file_association_last_registered: k580_ui::file_assoc::is_registered(),
            help_dialog: None,
            monitor_open: false,
            monitor_split: false,
            monitor_hex_popup: false,
            floppy_open: false,
            hdd_open: false,
            network_open: false,
            printer_open: false,
            printer_text_view: false,
            network_text_view: false,
            stack_view: false,
            stack_view_saved_address: None,
            stack_view_saved_scroll_offset: 0.0,
            network_settings_open: false,
            network_mode_draft: network_mode,
            network_host_input: network_host,
            network_port_input: network_port.to_string(),
            network_settings_error: None,
            hdd_file_exists: true,
            hdd_show_image_contents: false,
            hdd_image_contents: Vec::new(),
            hdd_image_error: None,
            floppy_show_image_contents: false,
            floppy_image_contents: Vec::new(),
            floppy_image_error: None,
            monitor_hex_filter: HexStreamFilter::default(),
        };
        app.apply_speed_tier(default_speed);

        // Let the startup commands settle before the first frame so
        // that synchronous dispatchers (e.g. import) do not race with
        // pending StateChanged events from AttachHddFile / AttachFloppyImage.
        let settle_deadline = Instant::now() + Duration::from_millis(100);
        loop {
            let remaining = settle_deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            let events = app.handle.drain_until_state_change(remaining);
            if events.is_empty() {
                break;
            }
            for event in events {
                app.consume_event(event);
            }
        }

        (app, startup_task)
    }
}
