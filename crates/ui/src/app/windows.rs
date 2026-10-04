mod events;
mod state;
pub(crate) use state::{NativeWindow, ToolWindowState};

use std::path::PathBuf;

use iced::{Size, Task, window};

use super::{DesktopApp, Message, ToolWindowKind};
use crate::i18n::Key;
use crate::platform;

const ICON_PNG: &[u8] = include_bytes!("../../assets/icons/icon-64.png");
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
            ToolWindowKind::Monitor => &self.panels.monitor_window,
            ToolWindowKind::Floppy => &self.panels.floppy_window,
            ToolWindowKind::Hdd => &self.panels.hdd_window,
            ToolWindowKind::Network => &self.panels.network_window,
            ToolWindowKind::Printer => &self.panels.printer_window,
        }
    }

    pub(crate) fn tool_window_mut(&mut self, kind: ToolWindowKind) -> &mut ToolWindowState {
        match kind {
            ToolWindowKind::Monitor => &mut self.panels.monitor_window,
            ToolWindowKind::Floppy => &mut self.panels.floppy_window,
            ToolWindowKind::Hdd => &mut self.panels.hdd_window,
            ToolWindowKind::Network => &mut self.panels.network_window,
            ToolWindowKind::Printer => &mut self.panels.printer_window,
        }
    }

    pub(crate) fn dialog_parent(&self, kind: Option<ToolWindowKind>) -> Option<window::Id> {
        kind.and_then(|kind| {
            let window = self.tool_window(kind);
            window.id().filter(|_| window.detached() && window.ready())
        })
        .or(self.shell.main_window_id)
    }

    pub(crate) fn boot(initial: Option<PathBuf>) -> (Self, Task<Message>) {
        let (mut app, startup) = Self::with_initial_path(initial);
        let (id, open) = window::open(main_window_settings());
        app.shell.main_window_id = Some(id);
        (app, Task::batch([startup, open.map(Message::WindowOpened)]))
    }

    pub(crate) fn title(&self, window: window::Id) -> String {
        self.tool_window_kind(window)
            .map(|kind| self.preferences.lang.t(tool_window_title(kind)).to_owned())
            .unwrap_or_else(|| "KR580 Emulator".to_owned())
    }

    pub(crate) fn dispatch_window_message(&mut self, message: &Message) -> Option<Task<Message>> {
        let task = match message {
            Message::WindowOpened(id) => self.window_opened(*id),
            Message::WindowClosed(id) => self.window_closed(*id),
            Message::WindowResized { id, size } => {
                if self.shell.main_window_id == Some(*id) {
                    self.shell.main_window_size = *size;
                }
                Task::none()
            }
            Message::FrameRendered => self.frame_rendered(),
            Message::WindowDragStart => self.drag_main_window(),
            Message::DetachedWindowDragStart(id) => window::drag(*id),
            Message::WindowMinimize => self
                .shell
                .main_window_id
                .map_or_else(Task::none, |id| window::minimize(id, true)),
            Message::WindowToggleMaximize => self.toggle_main_window_maximized(),
            Message::WindowClose => self
                .shell
                .main_window_id
                .map_or_else(Task::none, window::close),
            Message::WindowMaximizedChanged(maximized) => {
                self.shell.window_maximized = *maximized;
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
        if let Some(native) = self.tool_window(kind).native() {
            self.tool_window_mut(kind).detach(native);
            if self.tool_window(kind).ready() {
                return self.show_tool_window(kind, native.id());
            }
            return Task::none();
        }
        let (id, open) = window::open(tool_window_settings(kind, self.shell.main_window_size));
        self.tool_window_mut(kind).detach(NativeWindow::Opening(id));
        open.map(Message::WindowOpened)
    }

    fn attach_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        self.set_tool_window_open(kind, true);
        self.tool_window_mut(kind).attach();
        self.hide_or_close_tool_window(kind)
    }

    fn show_tool_window(&self, kind: ToolWindowKind, id: window::Id) -> Task<Message> {
        window::resize(id, tool_window_size(kind, self.shell.main_window_size))
            .chain(window::set_mode(id, window::Mode::Windowed))
            .chain(window::gain_focus(id))
    }

    fn hide_or_close_tool_window(&mut self, kind: ToolWindowKind) -> Task<Message> {
        let Some(id) = self.tool_window(kind).id() else {
            return Task::none();
        };
        if platform::SUPPORTS_HIDDEN_WINDOW_REUSE {
            window::set_level(id, window::Level::Normal)
                .chain(window::set_mode(id, window::Mode::Hidden))
        } else {
            let state = self.tool_window_mut(kind);
            state.take_id();
            window::close(id)
        }
    }

    fn toggle_tool_window_always_on_top(&mut self, kind: ToolWindowKind) -> Task<Message> {
        let Some((id, pinned)) = self.tool_window_mut(kind).toggle_pin() else {
            return Task::none();
        };
        window::set_level(
            id,
            if pinned {
                window::Level::AlwaysOnTop
            } else {
                window::Level::Normal
            },
        )
    }

    fn drag_main_window(&mut self) -> Task<Message> {
        if self.close_titlebar_popup_before_drag() {
            return Task::none();
        }
        self.shell
            .main_window_id
            .map_or_else(Task::none, iced::window::drag)
    }

    fn toggle_main_window_maximized(&mut self) -> Task<Message> {
        let Some(id) = self.shell.main_window_id else {
            return Task::none();
        };
        self.shell.window_maximized = !self.shell.window_maximized;
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
        state.focus = super::DeviceFocus::default();
        state.attach();
        if kind == ToolWindowKind::Monitor {
            self.panels.monitor_hex_popup = false;
        }
        if kind == ToolWindowKind::Network {
            self.panels.network_settings_open = false;
            self.panels.network_settings_error = None;
        }
    }

    fn set_tool_window_open(&mut self, kind: ToolWindowKind, open: bool) {
        match kind {
            ToolWindowKind::Monitor => self.panels.monitor_open = open,
            ToolWindowKind::Floppy => self.panels.floppy_open = open,
            ToolWindowKind::Hdd => self.panels.hdd_open = open,
            ToolWindowKind::Network => self.panels.network_open = open,
            ToolWindowKind::Printer => self.panels.printer_open = open,
        }
    }

    fn device_open(&self, kind: ToolWindowKind) -> bool {
        match kind {
            ToolWindowKind::Monitor => self.panels.monitor_open,
            ToolWindowKind::Floppy => self.panels.floppy_open,
            ToolWindowKind::Hdd => self.panels.hdd_open,
            ToolWindowKind::Network => self.panels.network_open,
            ToolWindowKind::Printer => self.panels.printer_open,
        }
    }

    /// Closes the open attached device panel; modals and device panels share one overlay slot.
    pub(crate) fn close_open_device_panel(&mut self) {
        for kind in TOOL_WINDOWS {
            if self.device_open(kind) && !self.tool_window(kind).detached() {
                self.reset_tool_window_presentation(kind);
            }
        }
    }

    fn tool_window_kind(&self, id: window::Id) -> Option<ToolWindowKind> {
        TOOL_WINDOWS
            .into_iter()
            .find(|kind| self.tool_window(*kind).id() == Some(id))
    }
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1180.0, 720.0),
        position: window::Position::Centered,
        min_size: Some(Size::new(1180.0, 720.0)),
        icon: window_icon(),
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
        icon: window_icon(),
        resizable: false,
        decorations: false,
        visible: false,
        exit_on_close_request: false,
        ..window::Settings::default()
    }
}

fn window_icon() -> Option<window::Icon> {
    let icon = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png)
        .ok()?
        .into_rgba8();
    let (width, height) = icon.dimensions();
    window::icon::from_rgba(icon.into_raw(), width, height).ok()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detached_tool_windows_are_not_user_resizable() {
        for kind in TOOL_WINDOWS {
            let settings = tool_window_settings(kind, Size::new(1180.0, 720.0));

            assert!(!settings.resizable, "{kind:?}");
        }
    }
}
