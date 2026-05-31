use iced::{Point, Task, Theme, keyboard};
use k580_app::{AppSnapshot, EmulatorHandle, Snapshot580Flavour, initial_snapshot, spawn_emulator};
use k580_core::RegisterName;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::messages::{MenuId, Message, RegisterInlineTarget, SpeedTier};
use super::modal::DiscardModalButton;
use super::settings_modal::SettingsDialog;
use super::status::StatusKind;
use super::undo::UndoStack;
use crate::i18n::{Key, Lang};
use crate::settings_storage::{lang_from_language, load_settings, speed_tier_from_preset};

#[derive(Clone, Debug)]
pub(crate) enum PendingAction {
    OpenSnapshot,
    NewFile,
    Import,
    OpenLegacySnapshot,
    CloseWindow,
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
    /// Stored separately because every successful match overwrites
    /// `memory_address_input` with the matched 4-digit address.
    pub(crate) memory_search_pattern: Option<String>,
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
    /// Cosmetic focus marker — iced 0.14 has no on_focus / on_blur.
    pub(crate) focused_input: Option<&'static str>,
    /// Cached for `MousePressed` — `ButtonPressed` carries identity
    /// only, not coordinates.
    pub(crate) latest_cursor_position: Point,
    pub(crate) running: bool,
    /// One-shot signal that the next `Tick` must run `follow_pc_during_run`
    /// even though `running` is already false (high-speed bursts where
    /// auto-pause clears `running` before Tick reads it).
    pub(crate) pending_follow_pc: bool,
    pub(crate) inline_register_just_entered: bool,
    /// Set on `TactAdvanced { instruction_boundary: true }`.
    pub(crate) last_tact_was_boundary: bool,
    pub(crate) startup_frames_seen: u8,
    pub(crate) open_menu: Option<MenuId>,
    pub(crate) current_snapshot_path: Option<PathBuf>,
    /// Separate from `current_snapshot_path` so Ctrl+S (v1) and Ctrl+Alt+S
    /// (legacy) each remember their own path.
    pub(crate) current_legacy_snapshot_path: Option<PathBuf>,
    /// Scratch slot the runtime uses to communicate the result of an
    /// auto-detect `LoadAnySnapshot` dispatch back to its caller.
    pub(crate) pending_snapshot_flavour: Option<Snapshot580Flavour>,
    pub(crate) speed_tier: SpeedTier,
    pub(crate) halt_notice: Option<String>,
    pub(crate) halt_notice_dismiss_at: Option<Instant>,
    /// Disables every execution-side button until reset. Outlives the
    /// halt notice's 8-second fade — the contract is "until reset",
    /// not "until the message disappears".
    pub(crate) run_blocked_after_halt: bool,
    pub(crate) error_notice: Option<String>,
    pub(crate) error_notice_dismiss_at: Option<Instant>,
    pub(crate) info_notice: Option<String>,
    pub(crate) info_notice_dismiss_at: Option<Instant>,
    pub(crate) window_id: Option<iced::window::Id>,
    pub(crate) window_maximized: bool,
    pub(crate) menu_categories_visible: bool,
    pub(crate) undo_stack: UndoStack,
    pub(crate) dirty: bool,
    pub(crate) discard_modal_focus: DiscardModalButton,
    pub(crate) pending_action: Option<PendingAction>,
    pub(crate) lang: Lang,
    pub(crate) default_speed: SpeedTier,
    pub(crate) settings_dialog: Option<SettingsDialog>,
    pub(crate) monitor_open: bool,
    pub(crate) monitor_split: bool,
    pub(crate) monitor_hex_popup: bool,
    pub(crate) monitor_hex_filter: HexStreamFilter,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum HexStreamFilter {
    #[default]
    All,
    Graphics,
    Text,
}

impl HexStreamFilter {
    pub(crate) fn next(self) -> Self {
        match self {
            HexStreamFilter::All => HexStreamFilter::Graphics,
            HexStreamFilter::Graphics => HexStreamFilter::Text,
            HexStreamFilter::Text => HexStreamFilter::All,
        }
    }
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
        let default_speed = speed_tier_from_preset(settings.general.default_speed);
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
            register_name_input: "A".to_owned(),
            register_value_input: "00".to_owned(),
            active_register_target: None,
            inline_register_target: None,
            hovered_register_target: None,
            memory_scroll_first_row: 0,
            memory_scroll_offset: 0.0,
            memory_viewport_height: 0.0,
            memory_scroll_visible_ticks: 0,
            opcode_scroll_visible_ticks: 0,
            monitor_hex_scroll_visible_ticks: 0,
            memory_address_input: "0000".to_owned(),
            memory_value_input: "00".to_owned(),
            memory_inline_value_input: "00".to_owned(),
            opcode_dropdown_address: None,
            opcode_search_input: String::new(),
            memory_search_pattern: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            focused_input: None,
            latest_cursor_position: Point::ORIGIN,
            running: false,
            pending_follow_pc: false,
            inline_register_just_entered: false,
            last_tact_was_boundary: false,
            startup_frames_seen: 0,
            open_menu: None,
            current_snapshot_path: None,
            current_legacy_snapshot_path: None,
            pending_snapshot_flavour: None,
            speed_tier: default_speed,
            halt_notice: None,
            halt_notice_dismiss_at: None,
            run_blocked_after_halt: false,
            error_notice: None,
            error_notice_dismiss_at: None,
            info_notice: None,
            info_notice_dismiss_at: None,
            window_id: None,
            window_maximized: false,
            menu_categories_visible: true,
            undo_stack: UndoStack::default(),
            dirty: false,
            discard_modal_focus: DiscardModalButton::Cancel,
            pending_action: None,
            lang,
            default_speed,
            settings_dialog: None,
            monitor_open: false,
            monitor_split: false,
            monitor_hex_popup: false,
            monitor_hex_filter: HexStreamFilter::default(),
        };
        app.apply_speed_tier(default_speed);
        (app, startup_task)
    }

    pub(crate) fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub(crate) fn set_status(&mut self, kind: StatusKind) {
        if let Some(rendered) = kind.render(self.lang) {
            self.status = rendered;
            self.status_kind = kind;
        }
    }

    pub(crate) fn set_status_custom(&mut self, text: String) {
        self.status = text;
        self.status_kind = StatusKind::Custom;
    }

    pub(crate) fn refresh_localized_status(&mut self) {
        if let Some(rendered) = self.status_kind.render(self.lang) {
            self.status = rendered;
        }
    }

    pub(crate) fn clear_error_notice(&mut self) {
        self.error_notice = None;
        self.error_notice_dismiss_at = None;
    }

    pub(crate) fn clear_halt_notice(&mut self) {
        self.halt_notice = None;
        self.halt_notice_dismiss_at = None;
    }

    pub(crate) fn clear_info_notice(&mut self) {
        self.info_notice = None;
        self.info_notice_dismiss_at = None;
    }

    pub(crate) fn raise_info_notice(&mut self, text: String) {
        self.info_notice = Some(text);
        self.info_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(5));
    }

    /// Single chokepoint for halt-block sites — both the notice and
    /// the run-block latch are armed here so callers can't forget
    /// one half.
    pub(crate) fn raise_halt_notice(&mut self) {
        self.halt_notice = Some(self.lang.t(crate::i18n::Key::HaltNotice).to_owned());
        self.halt_notice_dismiss_at = Some(Instant::now() + Duration::from_secs(8));
        self.run_blocked_after_halt = true;
    }

    pub(crate) fn run_new_file(&mut self) {
        self.dispatch(k580_app::AppCommand::ResetRam);
        self.dispatch(k580_app::AppCommand::ResetCpu);
        self.running = false;
        self.current_snapshot_path = None;
        self.current_legacy_snapshot_path = None;
        self.undo_stack.clear();
        self.dirty = false;
        self.speed_tier = self.default_speed;
        self.set_status(StatusKind::NewFile);
    }

    pub(crate) fn apply_speed_tier(&mut self, tier: SpeedTier) {
        self.speed_tier = tier;
        let hz = super::tier_hz(tier);
        let interval = Duration::from_micros(1_000_000 / u64::from(hz.max(1)));
        self.dispatch(k580_app::AppCommand::SetStepInterval(interval));
        let mode = match tier {
            SpeedTier::Max => k580_app::RunMode::Burst {
                slice: Duration::from_millis(16),
            },
            _ => k580_app::RunMode::Paced,
        };
        self.dispatch(k580_app::AppCommand::SetRunMode(mode));
    }
}
