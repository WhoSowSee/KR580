use iced::{Point, Size, Task, window};
use std::time::Instant;

use super::{DesktopApp, PrinterSetupTarget};
use crate::app::Message;

const DETACHED_PRINTER_SETUP_SIZE: Size = Size::new(720.0, 500.0);
const DETACHED_PRINTER_PROPERTIES_SIZE: Size = Size::new(1040.0, 680.0);

impl DesktopApp {
    pub(super) fn prepare_detached_printer_dialog(&self) -> Task<Message> {
        if !self.printer_setup_uses_detached_window() {
            return Task::none();
        }
        self.printer_window.id.map_or_else(Task::none, |id| {
            window::position(id).map(Message::PrinterSetupWindowPositionLoaded)
        })
    }

    pub(super) fn open_detached_printer_setup_window(
        &mut self,
        owner_position: Option<Point>,
    ) -> Task<Message> {
        if !self.printer_setup_uses_detached_window() {
            return Task::none();
        }
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            dialog.owner_position = owner_position;
        }
        if let Some(id) = self.printer_setup_window_id {
            return window::gain_focus(id);
        }
        let position = owner_position.map_or(window::Position::Centered, |position| {
            window::Position::Specific(detached_printer_setup_position(position))
        });
        let (id, open) = window::open(printer_dialog_window_settings(
            DETACHED_PRINTER_SETUP_SIZE,
            position,
            self.printer_window.always_on_top,
        ));
        self.printer_setup_window_id = Some(id);
        open.map(Message::WindowOpened)
    }

    pub(super) fn open_detached_printer_properties_window(&mut self) -> Task<Message> {
        if !self.printer_setup_uses_detached_window()
            || self
                .printer_setup_dialog
                .as_ref()
                .is_none_or(|dialog| dialog.properties.is_none())
        {
            return Task::none();
        }
        if let Some(id) = self.printer_properties_window_id {
            return window::gain_focus(id);
        }
        let owner_position = self
            .printer_setup_dialog
            .as_ref()
            .and_then(|dialog| dialog.owner_position);
        let position = owner_position.map_or(window::Position::Centered, |position| {
            window::Position::Specific(detached_printer_properties_position(position))
        });
        let (id, open) = window::open(printer_dialog_window_settings(
            DETACHED_PRINTER_PROPERTIES_SIZE,
            position,
            self.printer_window.always_on_top,
        ));
        self.printer_properties_window_id = Some(id);
        open.map(Message::WindowOpened)
    }

    pub(crate) fn finish_detached_printer_setup_window_open(
        &mut self,
        id: window::Id,
    ) -> Task<Message> {
        if self.printer_setup_window_id != Some(id) {
            return Task::none();
        }
        if let Some(dialog) = self.printer_setup_dialog.as_mut() {
            dialog.owner_ready = true;
        }
        show_printer_dialog_window(id)
    }

    pub(crate) fn finish_detached_printer_properties_window_open(
        &mut self,
        id: window::Id,
    ) -> Task<Message> {
        if self.printer_properties_window_id != Some(id) {
            return Task::none();
        }
        let Some(dialog) = self.printer_setup_dialog.as_mut() else {
            self.printer_properties_window_id = None;
            return window::close(id);
        };
        dialog.properties_surface_ready = true;
        show_printer_dialog_window(id)
    }

    pub(super) fn close_detached_printer_properties_window(&mut self) -> Task<Message> {
        let Some(id) = self.printer_properties_window_id.take() else {
            return Task::none();
        };
        let close = window::close(id);
        match self.printer_setup_window_id {
            Some(setup_id) => close.chain(window::gain_focus(setup_id)),
            None => close,
        }
    }

    pub(crate) fn request_printer_properties_attention(&mut self) -> Task<Message> {
        if let Some(properties) = self
            .printer_setup_dialog
            .as_mut()
            .and_then(|dialog| dialog.properties.as_mut())
        {
            properties.restart_attention(Instant::now());
        }
        self.printer_properties_window_id
            .map_or_else(Task::none, window::gain_focus)
    }

    pub(crate) fn close_detached_printer_setup_window(&mut self) -> Task<Message> {
        let close_properties = self
            .printer_properties_window_id
            .take()
            .map_or_else(Task::none, window::close);
        let close_setup = self
            .printer_setup_window_id
            .take()
            .map_or_else(Task::none, window::close);
        Task::batch([close_properties, close_setup])
    }

    pub(crate) fn cancel_detached_printer_setup(&mut self) -> Task<Message> {
        let close = self.close_detached_printer_setup_window();
        if self
            .printer_setup_dialog
            .as_ref()
            .is_some_and(|dialog| dialog.target == PrinterSetupTarget::Session)
        {
            self.close_printer_setup_dialog();
        }
        close
    }

    pub(crate) fn detached_printer_setup_window_closed(&mut self, id: window::Id) -> bool {
        if self.printer_setup_window_id != Some(id) {
            return false;
        }
        self.printer_setup_window_id = None;
        self.close_printer_setup_dialog();
        true
    }

    pub(crate) fn detached_printer_properties_window_closed(&mut self, id: window::Id) -> bool {
        if self.printer_properties_window_id != Some(id) {
            return false;
        }
        self.printer_properties_window_id = None;
        self.close_printer_properties();
        true
    }
}

fn printer_dialog_window_settings(
    size: Size,
    position: window::Position,
    always_on_top: bool,
) -> window::Settings {
    window::Settings {
        size,
        position,
        min_size: Some(size),
        max_size: Some(size),
        resizable: false,
        decorations: false,
        visible: false,
        level: if always_on_top {
            window::Level::AlwaysOnTop
        } else {
            window::Level::Normal
        },
        exit_on_close_request: false,
        ..window::Settings::default()
    }
}

fn show_printer_dialog_window(id: window::Id) -> Task<Message> {
    window::run(id, crate::platform::set_rounded_corners)
        .discard()
        .chain(window::set_mode(id, window::Mode::Windowed))
        .chain(window::gain_focus(id))
}

pub(super) fn detached_printer_setup_position(owner_position: Point) -> Point {
    detached_printer_dialog_position(owner_position, DETACHED_PRINTER_SETUP_SIZE)
}

pub(super) fn detached_printer_properties_position(owner_position: Point) -> Point {
    detached_printer_dialog_position(owner_position, DETACHED_PRINTER_PROPERTIES_SIZE)
}

fn detached_printer_dialog_position(owner_position: Point, dialog_size: Size) -> Point {
    let owner_size = super::super::windows::detached_storage_size();
    Point::new(
        owner_position.x + (owner_size.width - dialog_size.width) / 2.0,
        owner_position.y + (owner_size.height - dialog_size.height) / 2.0,
    )
}
