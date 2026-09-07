use std::mem::{Discriminant, discriminant};

use iced::{Event, Task, keyboard, mouse, window};

use super::{DesktopApp, Message, ToolWindowKind, focus::cycle_index};
use crate::backend::DeviceStatus;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DeviceFocus {
    selected: Option<Discriminant<Message>>,
    pub(crate) keyboard: bool,
}

impl DeviceFocus {
    pub(crate) fn matches(self, action: &Message) -> bool {
        self.selected == Some(action_key(action))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DeviceToolbar {
    pub(crate) kind: ToolWindowKind,
    pub(crate) state: super::windows::ToolWindowState,
}

fn action_key(action: &Message) -> Discriminant<Message> {
    match action {
        Message::AttachToolWindow(kind) => discriminant(&Message::DetachToolWindow(*kind)),
        _ => discriminant(action),
    }
}

pub(crate) fn device_navigation_event(event: &Event) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };
    match key {
        keyboard::Key::Named(keyboard::key::Named::Tab)
            if modifiers.is_empty() || *modifiers == keyboard::Modifiers::SHIFT =>
        {
            super::menu_keyboard::navigation_key_message(key, *modifiers)
        }
        keyboard::Key::Named(keyboard::key::Named::Enter | keyboard::key::Named::Space)
            if modifiers.is_empty() =>
        {
            Some(Message::EnterPressed)
        }
        _ => None,
    }
}

impl DesktopApp {
    pub(crate) fn device_toolbar(&self, kind: ToolWindowKind) -> DeviceToolbar {
        DeviceToolbar {
            kind,
            state: *self.tool_window(kind),
        }
    }

    pub(super) fn handle_device_keyboard_event(
        &mut self,
        event: &Event,
        window: window::Id,
    ) -> Option<Task<Message>> {
        let kind = self.device_keyboard_owner(window)?;
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            self.tool_window_mut(kind).focus = DeviceFocus::default();
            return Some(Task::none());
        }
        let navigation = device_navigation_event(event)?;
        let actions = self.device_actions(kind);
        let focus = &mut self.tool_window_mut(kind).focus;
        let current = actions.iter().position(|action| focus.matches(action));
        Some(match navigation {
            Message::FocusCycle { backward } => {
                let index = cycle_index(current, actions.len(), backward);
                focus.selected = Some(action_key(&actions[index]));
                focus.keyboard = true;
                Task::none()
            }
            Message::EnterPressed => current.map_or_else(Task::none, |index| {
                self.press_device_button(kind, actions[index].clone())
            }),
            _ => unreachable!(),
        })
    }

    pub(super) fn press_device_button(
        &mut self,
        kind: ToolWindowKind,
        action: Message,
    ) -> Task<Message> {
        let opening_hex =
            matches!(action, Message::ToggleMonitorHexPopup) && !self.monitor_hex_popup;
        self.tool_window_mut(kind).focus = DeviceFocus {
            selected: (!opening_hex).then(|| action_key(&action)),
            keyboard: false,
        };
        self.update(action)
    }

    pub(crate) fn device_keyboard_owner(&self, window: window::Id) -> Option<ToolWindowKind> {
        let devices = [
            (ToolWindowKind::Monitor, self.monitor_open),
            (ToolWindowKind::Hdd, self.hdd_open),
            (ToolWindowKind::Floppy, self.floppy_open),
            (ToolWindowKind::Network, self.network_open),
            (ToolWindowKind::Printer, self.printer_open),
        ];
        let kind = if self.main_window_id == Some(window) {
            if self.pending_action.is_some()
                || self.export_modal_open
                || self.import_modal_open
                || self.subprogram_dialog.is_some()
                || self.settings_dialog.is_some()
                || self.about_dialog_open
                || self.changelog_dialog.is_some()
                || self.help_dialog.is_some()
                || self.printer_setup_dialog.is_some()
            {
                return None;
            }
            devices
                .into_iter()
                .find(|(kind, open)| *open && !self.tool_window(*kind).detached)?
                .0
        } else {
            devices
                .into_iter()
                .find(|(kind, open)| {
                    let state = self.tool_window(*kind);
                    *open && state.detached && state.id == Some(window)
                })?
                .0
        };
        if (kind == ToolWindowKind::Network && self.network_settings_open)
            || (kind == ToolWindowKind::Printer && self.printer_setup_dialog.is_some())
        {
            return None;
        }
        Some(kind)
    }

    fn device_actions(&self, kind: ToolWindowKind) -> Vec<Message> {
        use Message::*;
        if kind == ToolWindowKind::Monitor && self.monitor_hex_popup {
            return vec![CycleMonitorHexFilter, ToggleMonitorHexPopup];
        }
        let detached = self.tool_window(kind).detached;
        let mut actions = vec![if detached {
            AttachToolWindow(kind)
        } else {
            DetachToolWindow(kind)
        }];
        if detached {
            actions.push(ToggleToolWindowAlwaysOnTop(kind));
        }
        actions.extend_from_slice(match kind {
            ToolWindowKind::Monitor => &[
                ToggleMonitorSplit,
                ToggleMonitorHexPopup,
                ClearMonitorBuffer,
                SaveMonitorImage,
                CloseMonitor,
            ],
            ToolWindowKind::Floppy => &[
                OpenFloppyImage,
                SaveFloppyBuffer,
                DetachFloppyImage,
                ToggleFloppyImageContents,
                ToggleFloppyDebugBuffer,
                ClearFloppyBuffer,
                CloseFloppy,
            ],
            ToolWindowKind::Hdd => &[
                ChooseHddDirectory,
                ToggleHddImageContents,
                ToggleHddDebugBuffer,
                ClearHddBuffer,
                DeleteHddFile,
                CreateHddFile,
                CloseHdd,
            ],
            ToolWindowKind::Network => &[
                ToggleNetworkBufferView,
                OpenNetworkSettings,
                ClearNetworkBuffers,
                CloseNetwork,
            ],
            ToolWindowKind::Printer => &[
                TogglePrinterBufferView,
                ConfigurePrinterSession,
                PrintPrinterNative,
                ClearPrinterBuffer,
                ClosePrinter,
            ],
        });
        actions.retain(|action| match action {
            DeleteHddFile => self.hdd_file_exists,
            CreateHddFile => !self.hdd_file_exists,
            PrintPrinterNative => self.snapshot.devices.printer.status != DeviceStatus::Busy,
            _ => true,
        });
        actions
    }
}

#[cfg(test)]
mod tests;
