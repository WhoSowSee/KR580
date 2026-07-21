mod events;

use std::path::PathBuf;

use iced::{Size, Task, window};

use super::{DesktopApp, Message, ToolWindowKind};
use crate::i18n::Key;
use crate::platform;

const ICON_PNG: &[u8] = include_bytes!("../../assets/icons/icon-64.png");
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ToolWindowState {
    pub(crate) id: Option<iced::window::Id>,
    pub(crate) ready: bool,
    pub(crate) detached: bool,
    pub(crate) always_on_top: bool,
}

const TOOL_WINDOWS: [ToolWindowKind; 5] = [
    ToolWindowKind::Monitor,
    ToolWindowKind::Floppy,
    ToolWindowKind::Hdd,
    ToolWindowKind::Network,
    ToolWindowKind::Printer,
];

impl DesktopApp {
    pub(crate) fn tool_window(&self, kind: ToolWindowKind) -> &ToolWindowState {
        match kind {
            ToolWindowKind::Monitor => &self.monitor_window,
            ToolWindowKind::Floppy => &self.floppy_window,
            ToolWindowKind::Hdd => &self.hdd_window,
            ToolWindowKind::Network => &self.network_window,
            ToolWindowKind::Printer => &self.printer_window,
        }
    }

    pub(crate) fn tool_window_mut(&mut self, kind: ToolWindowKind) -> &mut ToolWindowState {
        match kind {
            ToolWindowKind::Monitor => &mut self.monitor_window,
            ToolWindowKind::Floppy => &mut self.floppy_window,
            ToolWindowKind::Hdd => &mut self.hdd_window,
            ToolWindowKind::Network => &mut self.network_window,
            ToolWindowKind::Printer => &mut self.printer_window,
        }
    }

    pub(crate) fn boot(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let (mut app, startup) = Self::with_initial_path(initial);
        let (id, open) = window::open(main_window_settings());
        app.main_window_id = Some(id);
        (app, Task::batch([startup, open.map(Message::WindowOpened)]))
    }

    pub(crate) fn title(&self, window: window::Id) -> String {
        self.tool_window_kind(window)
            .map(|kind| self.lang.t(tool_window_title(kind)).to_owned())
            .unwrap_or_else(|| "KR580 Emulator".to_owned())
    }

    pub(crate) fn dispatch_window_message(&mut self, message: &Message) -> Option<Task<Message>> {
        let task = match message {
            Message::WindowOpened(id) => self.window_opened(*id),
            Message::WindowClosed(id) => self.window_closed(*id),
            Message::WindowResized { id, size } => {
                if self.main_window_id == Some(*id) {
                    self.main_window_size = *size;
                }
                Task::none()
            }
            Message::FrameRendered => self.frame_rendered(),
            Message::WindowDragStart => self.drag_main_window(),
            Message::ToolWindowDragStart(kind) => self.drag_tool_window(*kind),
            Message::WindowMinimize => self
                .main_window_id
                .map_or_else(Task::none, |id| window::minimize(id, true)),
            Message::WindowToggleMaximize => self.toggle_main_window_maximized(),
            Message::WindowClose => self.main_window_id.map_or_else(Task::none, window::close),
            Message::WindowMaximizedChanged(maximized) => {
                self.window_maximized = *maximized;
                Task::none()
            }
            Message::WindowCloseRequested(id) => self.window_close_requested(*id),
            Message::DetachToolWindow(kind) => self.detach_tool_window(*kind),
            Message::AttachToolWindow(kind) => self.attach_tool_window(*kind),
            Message::ToggleToolWindowAlwaysOnTop(kind) => {
                self.toggle_tool_window_always_on_top(*kind)
            }
            _ => return None,
        };
        Some(task)
    }

    pub(crate) fn close_monitor(&mut self) -> Task<Message> {
        self.close_tool_window(ToolWindowKind::Monitor)
    }

    pub(crate) fn close_floppy(&mut self) -> Task<Message> {
        self.close_tool_window(ToolWindowKind::Floppy)
    }

    pub(crate) fn close_hdd(&mut self) -> Task<Message> {
        self.close_tool_window(ToolWindowKind::Hdd)
    }

    pub(crate) fn close_network(&mut self) -> Task<Message> {
        self.close_tool_window(ToolWindowKind::Network)
    }

    pub(crate) fn close_printer(&mut self) -> Task<Message> {
        self.close_tool_window(ToolWindowKind::Printer)
    }

    fn detach_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        self.set_tool_window_open(kind, true);
        let state = self.tool_window_mut(kind);
        state.detached = true;
        if let Some(id) = state.id {
            return if state.ready {
                self.show_tool_window(kind, id)
            } else {
                Task::none()
            };
        }
        self.open_tool_window(kind)
    }

    fn attach_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        self.set_tool_window_open(kind, true);
        let state = self.tool_window_mut(kind);
        state.detached = false;
        state.always_on_top = false;
        self.hide_or_close_tool_window(kind)
    }

    fn open_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        if self.tool_window(kind).id.is_some() {
            return Task::none();
        }
        let (id, open) = window::open(tool_window_settings(kind, self.main_window_size));
        let state = self.tool_window_mut(kind);
        state.id = Some(id);
        state.ready = false;
        open.map(Message::WindowOpened)
    }

    fn show_tool_window(&self, kind: ToolWindowKind, id: window::Id) -> Task<Message> {
        window::resize(id, tool_window_size(kind, self.main_window_size))
            .chain(window::set_mode(id, window::Mode::Windowed))
            .chain(window::gain_focus(id))
    }

    fn hide_or_close_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        let Some(id) = self.tool_window(kind).id else {
            return Task::none();
        };
        if platform::SUPPORTS_HIDDEN_WINDOW_REUSE {
            window::set_level(id, window::Level::Normal)
                .chain(window::set_mode(id, window::Mode::Hidden))
        } else {
            let state = self.tool_window_mut(kind);
            state.ready = false;
            state.id = None;
            window::close(id)
        }
    }

    fn toggle_tool_window_always_on_top(&mut self, kind: ToolWindowKind) -> Task<Message> {
        let state = self.tool_window_mut(kind);
        let Some(id) = state.id else {
            return Task::none();
        };
        state.always_on_top = !state.always_on_top;
        let level = if state.always_on_top {
            window::Level::AlwaysOnTop
        } else {
            window::Level::Normal
        };
        window::set_level(id, level)
    }

    fn drag_main_window(&mut self) -> Task<Message> {
        if self.close_titlebar_popup_before_drag() {
            return Task::none();
        }
        self.main_window_id
            .map_or_else(Task::none, iced::window::drag)
    }

    fn drag_tool_window(&self, kind: ToolWindowKind) -> Task<Message> {
        self.tool_window(kind)
            .id
            .map_or_else(Task::none, window::drag)
    }

    fn toggle_main_window_maximized(&mut self) -> Task<Message> {
        let Some(id) = self.main_window_id else {
            return Task::none();
        };
        self.window_maximized = !self.window_maximized;
        Task::batch([
            iced::window::toggle_maximize(id),
            iced::window::is_maximized(id).map(Message::WindowMaximizedChanged),
        ])
    }

    fn close_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        let close_setup = if kind == ToolWindowKind::Printer {
            self.cancel_detached_printer_setup()
        } else {
            Task::none()
        };
        self.reset_tool_window_presentation(kind);
        Task::batch([close_setup, self.hide_or_close_tool_window(kind)])
    }

    fn reset_tool_window_presentation(&mut self, kind: ToolWindowKind) {
        self.set_tool_window_open(kind, false);
        let state = self.tool_window_mut(kind);
        state.detached = false;
        state.always_on_top = false;
        if kind == ToolWindowKind::Monitor {
            self.monitor_hex_popup = false;
        }
        if kind == ToolWindowKind::Network {
            self.network_settings_open = false;
            self.network_settings_error = None;
        }
    }

    fn set_tool_window_open(&mut self, kind: ToolWindowKind, open: bool) {
        match kind {
            ToolWindowKind::Monitor => self.monitor_open = open,
            ToolWindowKind::Floppy => self.floppy_open = open,
            ToolWindowKind::Hdd => self.hdd_open = open,
            ToolWindowKind::Network => self.network_open = open,
            ToolWindowKind::Printer => self.printer_open = open,
        }
    }

    fn device_open(&self, kind: ToolWindowKind) -> bool {
        match kind {
            ToolWindowKind::Monitor => self.monitor_open,
            ToolWindowKind::Floppy => self.floppy_open,
            ToolWindowKind::Hdd => self.hdd_open,
            ToolWindowKind::Network => self.network_open,
            ToolWindowKind::Printer => self.printer_open,
        }
    }

    /// Closes the open attached device panel; modals and device panels share one overlay slot.
    pub(crate) fn close_open_device_panel(&mut self) {
        for kind in TOOL_WINDOWS {
            if self.device_open(kind) && !self.tool_window(kind).detached {
                self.reset_tool_window_presentation(kind);
            }
        }
    }

    fn tool_window_kind(&self, id: window::Id) -> Option<ToolWindowKind> {
        TOOL_WINDOWS
            .into_iter()
            .find(|kind| self.tool_window(*kind).id == Some(id))
    }
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1180.0, 720.0),
        position: window::Position::Centered,
        min_size: Some(Size::new(1180.0, 720.0)),
        icon: window::icon::from_file_data(ICON_PNG, None).ok(),
        decorations: false,
        visible: false,
        exit_on_close_request: false,
        ..window::Settings::default()
    }
}

fn tool_window_settings(kind: ToolWindowKind, main_window_size: Size) -> window::Settings {
    let size = tool_window_size(kind, main_window_size);
    window::Settings {
        size,
        position: window::Position::Centered,
        min_size: Some(tool_window_min_size(kind)),
        icon: window::icon::from_file_data(ICON_PNG, None).ok(),
        decorations: false,
        visible: false,
        exit_on_close_request: false,
        ..window::Settings::default()
    }
}

fn tool_window_size(kind: ToolWindowKind, main_window_size: Size) -> Size {
    match kind {
        ToolWindowKind::Monitor => detached_monitor_size(main_window_size),
        ToolWindowKind::Floppy
        | ToolWindowKind::Hdd
        | ToolWindowKind::Network
        | ToolWindowKind::Printer => detached_storage_size(),
    }
}

fn tool_window_min_size(kind: ToolWindowKind) -> Size {
    match kind {
        ToolWindowKind::Monitor => Size::new(720.0, 480.0),
        ToolWindowKind::Floppy
        | ToolWindowKind::Hdd
        | ToolWindowKind::Network
        | ToolWindowKind::Printer => detached_storage_size(),
    }
}

fn tool_window_title(kind: ToolWindowKind) -> Key {
    match kind {
        ToolWindowKind::Monitor => Key::HnMonitor,
        ToolWindowKind::Floppy => Key::HnFloppy,
        ToolWindowKind::Hdd => Key::HnHdd,
        ToolWindowKind::Network => Key::HnNetwork,
        ToolWindowKind::Printer => Key::HnPrinter,
    }
}

pub(super) fn detached_monitor_size(main_window_size: Size) -> Size {
    const MODAL_INSET: f32 = 120.0;
    Size::new(
        (main_window_size.width - MODAL_INSET).max(720.0),
        (main_window_size.height - MODAL_INSET).max(480.0),
    )
}

pub(crate) fn detached_storage_size() -> Size {
    Size::new(760.0, 340.0)
}
