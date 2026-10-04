mod components;
mod memory;
pub(crate) use memory::{MemoryView, MemoryViewport, OperandReturn};
mod dropdown;
pub(crate) use dropdown::DropdownState;

use components::{
    DevicePanels, DocumentState, ExecutionState, ExportDialogState, ImportDialogState,
    InteractionState, MemoryEditorState, PreferencesState, PrinterSetupState, RegisterEditorState,
    RequestState, ShellState,
};

use super::hex_stream_filter::HexStreamFilter;
use super::messages::{ExportTab, Message, TopMenuIndicator};
use super::modal::DiscardModalButton;
use super::status::StatusKind;
use super::undo::UndoStack;
use super::windows::ToolWindowState;
use super::{
    ExportFlagSelection, ExportMemoryColumns, ExportModalFocus, ExportRegisterSelection,
    ExportTarget, ImportModalFocus,
};
use crate::backend::{AppSnapshot, EmulatorHandle, initial_snapshot, spawn_emulator};
use crate::i18n::Key;
use crate::settings_storage::{lang_from_language, load_settings, speed_tier_from_preset};
use iced::{Point, Size, Task, keyboard};
use k580_core::RegisterName;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) struct DesktopApp {
    pub(crate) handle: EmulatorHandle,
    pub(crate) snapshot: AppSnapshot,
    pub(crate) document: DocumentState,
    pub(crate) memory: MemoryEditorState,
    pub(crate) register: RegisterEditorState,
    pub(crate) interaction: InteractionState,
    pub(crate) execution: ExecutionState,
    pub(crate) export: ExportDialogState,
    pub(crate) import: ImportDialogState,
    pub(crate) preferences: PreferencesState,
    pub(crate) printer_setup: PrinterSetupState,
    pub(crate) panels: DevicePanels,
    pub(crate) shell: ShellState,
    pub(crate) requests: RequestState,
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
            crate::runtime::storage_files::hdd_default_path(&settings),
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
            document: DocumentState {
                current_snapshot_path: None,
                current_subprogram_range: None,
                subprogram_dialog: None,
                undo_stack: UndoStack::default(),
                dirty: false,
                saved_cpu: k580_core::Cpu8080State::default(),
                discard_modal_focus: DiscardModalButton::Cancel,
                discard_modal_keyboard_focus_visible: false,
                pending_action: None,
                file_drag_hovered: false,
                file_drag_cursor_position: None,
                edit_epoch: 0,
            },
            memory: MemoryEditorState {
                memory_scroll_first_row: 0,
                memory_scroll_offset: 0.0,
                memory_viewport_height: 0.0,
                memory_scroll_visible_ticks: 0,
                opcode_scroll_visible_ticks: 0,
                opcode_scroll_offset: 0.0,
                memory_address_input: String::new(),
                memory_value_input: String::new(),
                memory_inline_value_input: String::new(),
                opcode_dropdown_address: None,
                opcode_search_input: String::new(),
                opcode_highlight_index: 0,
                memory_search_pattern: None,
                operand_return: None,
                view: MemoryView::Ram,
            },
            register: RegisterEditorState {
                selected_register: RegisterName::A,
                register_name_input: String::new(),
                register_value_input: String::new(),
                active_register_target: None,
                inline_register_target: None,
                hovered_register_target: None,
                inline_register_just_entered: false,
            },
            interaction: InteractionState {
                keyboard_modifiers: keyboard::Modifiers::default(),
                focused_input: None,
                replacement_input: None,
                replacement_placeholder: String::new(),
                replacement_original_value: String::new(),
                latest_cursor_position: Point::ORIGIN,
                previous_left_click: None,
                mouse_press_generation: 0,
                replacement_reconcile_guard: None,
            },
            execution: ExecutionState {
                running: false,
                pending_follow_pc: false,
                speed_tier: default_speed,
                halt_notice: None,
                halt_notice_dismiss_at: None,
                run_blocked_after_halt: false,
            },
            export: ExportDialogState {
                export_modal_open: false,
                export_tab: ExportTab::Xlsx,
                export_modal_focus: ExportModalFocus::TabXlsx,
                export_modal_keyboard_focus_visible: false,
                export_xlsx_page_input: lang.t(Key::ExportPageDefault).to_owned(),
                export_text_section_input: lang.t(Key::ExportSectionDefault).to_owned(),
                export_xlsx_pages: vec![ExportTarget::named(
                    lang.t(Key::ExportPageDefault).to_owned(),
                )],
                export_text_sections: vec![ExportTarget::named(
                    lang.t(Key::ExportSectionDefault).to_owned(),
                )],
                export_target_dropdown: DropdownState::Closed,
                export_memory_start_input: "0000".to_owned(),
                export_memory_end_input: "FFFF".to_owned(),
                export_memory_columns: ExportMemoryColumns::default(),
                export_registers: ExportRegisterSelection::default(),
                export_flags: ExportFlagSelection::default(),
            },
            import: ImportDialogState {
                generation: 0,
                loading: false,
                import_modal_open: false,
                import_modal_focus: ImportModalFocus::Browse,
                import_modal_keyboard_focus_visible: false,
                import_file_drag_hovered: false,
                import_file_path: None,
                import_file_display: String::new(),
                import_file_format: None,
                import_target_options: Vec::new(),
                import_target_input: String::new(),
                import_target_dropdown: DropdownState::Closed,
                import_error: None,
            },
            preferences: PreferencesState {
                stored: settings.clone(),
                dialog_generation: 0,
                directory_generation: 0,
                lang,
                default_speed,
                color_scheme,
                shortcut_settings: settings.shortcuts.clone(),
                settings_dialog: None,
                settings_notice: None,
                file_association_pending: false,
                follow_pc,
                memory_operand_highlighting,
                show_file_name: settings.general.show_file_name,
                menu_categories_visible: true,
            },
            printer_setup: PrinterSetupState {
                printer_default_settings,
                printer_dialog_mode: settings.general.printer_dialog_mode,
                printer_session_settings: None,
                printer_setup_dialog: None,
                printer_setup_window_id: None,
                printer_properties_window_id: None,
                printer_setup_pending: false,
            },
            panels: DevicePanels {
                monitor_window: ToolWindowState::default(),
                floppy_window: ToolWindowState::default(),
                hdd_window: ToolWindowState::default(),
                network_window: ToolWindowState::default(),
                printer_window: ToolWindowState::default(),
                monitor_open: false,
                monitor_split: settings.general.monitor_split,
                monitor_hex_popup: false,
                monitor_hex_scroll_visible_ticks: 0,
                monitor_hex_scroll_offset: 0.0,
                monitor_hex_filter: HexStreamFilter::default(),
                floppy_open: false,
                hdd_open: false,
                network_open: false,
                printer_open: false,
                printer_text_view: false,
                network_text_view: false,
                network_settings_open: false,
                network_mode_draft: network_mode,
                network_host_input: network_host,
                network_port_input: network_port.to_string(),
                network_settings_error: None,
                hdd_file_exists: true,
                hdd_generation: 0,
                hdd_show_image_contents: false,
                hdd_image: Default::default(),
                floppy_show_image_contents: false,
                floppy_image: Default::default(),
            },
            shell: ShellState {
                status: initial_status,
                status_kind: initial_status_kind,
                startup_frames_seen: 0,
                main_window_size: Size::new(1180.0, 720.0),
                open_menu: None,
                top_menu_focus: None,
                top_menu_indicator: TopMenuIndicator::Hidden,
                about_dialog_open: false,
                error_notice: None,
                error_notice_dismiss_at: None,
                main_window_id: None,
                window_maximized: false,
                changelog_dialog: None,
                help_dialog: None,
            },
            requests: RequestState {
                file_worker: None,
                pending_requests: HashMap::new(),
                tasks: Vec::new(),
            },
        };
        app.apply_speed_tier(default_speed);
        app.dispatch_request(crate::backend::AppCommand::RequestSnapshot);

        (app, startup_task)
    }
}
