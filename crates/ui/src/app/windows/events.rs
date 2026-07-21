use iced::{Task, window};

use super::{DesktopApp, Message, TOOL_WINDOWS};
use crate::app::PendingAction;
use crate::platform;

impl DesktopApp {
    pub(super) fn window_opened(&mut self, id: window::Id) -> Task<Message> {
        if self.printer_properties_window_id == Some(id) {
            return self.finish_detached_printer_properties_window_open(id);
        }
        if self.printer_setup_window_id == Some(id) {
            return self.finish_detached_printer_setup_window_open(id);
        }
        if let Some(kind) = self.tool_window_kind(id) {
            let detached = {
                let state = self.tool_window_mut(kind);
                state.ready = true;
                state.detached
            };
            let prepare = window::run(id, platform::set_rounded_corners).discard();
            return if detached {
                prepare.chain(self.show_tool_window(kind, id))
            } else {
                prepare
            };
        }
        if self.main_window_id != Some(id) {
            return Task::none();
        }
        Task::batch([
            window::run(id, |window| platform::cloak_window(window, true)).discard(),
            window::run(id, platform::set_rounded_corners).discard(),
            window::set_mode(id, window::Mode::Windowed),
            window::is_maximized(id).map(Message::WindowMaximizedChanged),
        ])
    }

    pub(super) fn window_closed(&mut self, id: window::Id) -> Task<Message> {
        if self.detached_printer_properties_window_closed(id) {
            return self
                .printer_setup_window_id
                .map_or_else(Task::none, window::gain_focus);
        }
        if self.detached_printer_setup_window_closed(id) {
            return self.close_detached_printer_setup_window();
        }
        if let Some(kind) = self.tool_window_kind(id) {
            *self.tool_window_mut(kind) = Default::default();
            self.reset_tool_window_presentation(kind);
            return Task::none();
        }
        if self.main_window_id != Some(id) {
            return Task::none();
        }
        self.main_window_id = None;
        let close_setup = self.cancel_detached_printer_setup();
        let close_windows = TOOL_WINDOWS.into_iter().filter_map(|kind| {
            self.reset_tool_window_presentation(kind);
            self.tool_window_mut(kind).id.take().map(window::close)
        });
        Task::batch(close_windows)
            .chain(close_setup)
            .chain(iced::exit())
    }

    pub(super) fn window_close_requested(&mut self, id: window::Id) -> Task<Message> {
        if self.printer_properties_window_id == Some(id) {
            return Task::done(Message::ClosePrinterProperties);
        }
        if self.printer_setup_window_id == Some(id) {
            if self.printer_properties_window_id.is_some() {
                return self.request_printer_properties_attention();
            }
            return self.cancel_detached_printer_setup();
        }
        if let Some(kind) = self.tool_window_kind(id) {
            return self.close_tool_window(kind);
        }
        if self.main_window_id != Some(id) {
            return Task::none();
        }
        if self.dirty {
            self.open_discard_modal(PendingAction::CloseWindow);
            Task::none()
        } else {
            window::close(id)
        }
    }

    pub(super) fn frame_rendered(&mut self) -> Task<Message> {
        if self.startup_frames_seen < u8::MAX {
            self.startup_frames_seen = self.startup_frames_seen.saturating_add(1);
        }
        if self.startup_frames_seen != 2 {
            return Task::none();
        }
        self.main_window_id.map_or_else(Task::none, |id| {
            window::run(id, |window| platform::cloak_window(window, false)).discard()
        })
    }
}
