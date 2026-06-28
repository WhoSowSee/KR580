use iced::{Subscription, Task, event, keyboard, mouse, time};

use super::handlers::tick_interval;
use super::messages::Message;
use super::shortcuts::{binding_from_event, shortcut_message};
use super::state::DesktopApp;
use crate::persistence::ShortcutSettings;

impl DesktopApp {
    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            time::every(tick_interval(self.running, self.speed_tier)).map(|_| Message::Tick),
            iced::window::close_events().map(Message::WindowClosed),
            event::listen_with(|event, status, window| {
                Some(Message::RuntimeEvent {
                    event,
                    status,
                    window,
                })
            }),
        ];

        if self.startup_frames_seen < 2 || self.settings_dialog.is_some() {
            subscriptions.push(iced::window::frames().map(|_| Message::FrameRendered));
        }

        Subscription::batch(subscriptions)
    }

    pub(crate) fn handle_runtime_event(
        &self,
        event: iced::Event,
        status: event::Status,
        window: iced::window::Id,
    ) -> Task<Message> {
        runtime_event_message(self, event, status, window)
            .map(Task::done)
            .unwrap_or_else(Task::none)
    }
}

fn runtime_event_message(
    app: &DesktopApp,
    event: iced::Event,
    status: event::Status,
    window: iced::window::Id,
) -> Option<Message> {
    let recording_shortcut = app
        .settings_dialog
        .as_ref()
        .and_then(|dialog| dialog.recording_shortcut)
        .is_some();
    match (event, status) {
        (iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)), _) => {
            Some(Message::ModifiersChanged(modifiers))
        }
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }),
            _,
        ) if recording_shortcut => Some(Message::SettingsShortcutCaptureCancelled),
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                physical_key,
                modifiers,
                ..
            }),
            _,
        ) if recording_shortcut => {
            binding_from_event(physical_key, modifiers).map(Message::SettingsShortcutCaptured)
        }
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }),
            _,
        ) => Some(Message::EscPressed),
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }),
            status,
        ) if modifiers.command() => command_shortcut_message(
            &app.shortcut_settings,
            &key,
            physical_key,
            modifiers,
            status,
        ),
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                physical_key,
                modifiers,
                ..
            }),
            _,
        ) if modifiers.alt() => shortcut_message(&app.shortcut_settings, physical_key, modifiers),
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
                modifiers,
                ..
            }),
            iced::event::Status::Ignored,
        ) => Some(Message::FocusCycle {
            backward: modifiers.shift(),
        }),
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                physical_key,
                modifiers,
                ..
            }),
            iced::event::Status::Ignored,
        ) => match key {
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::ArrowKey(1)),
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::ArrowKey(-1)),
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                Some(Message::HorizontalArrowKey(-1))
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                Some(Message::HorizontalArrowKey(1))
            }
            keyboard::Key::Named(keyboard::key::Named::PageUp) => {
                Some(Message::MemoryAddressPageUp)
            }
            keyboard::Key::Named(keyboard::key::Named::PageDown) => {
                Some(Message::MemoryAddressPageDown)
            }
            keyboard::Key::Named(keyboard::key::Named::F1) => Some(Message::OpenHelp),
            keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Message::EnterPressed),
            _ => shortcut_message(&app.shortcut_settings, physical_key, modifiers),
        },
        (
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }),
            iced::event::Status::Captured,
        ) => captured_register_arrow(&key, modifiers),
        (iced::Event::Mouse(mouse::Event::CursorMoved { position }), _) => {
            Some(Message::CursorMoved(position))
        }
        (
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            iced::event::Status::Ignored,
        ) => Some(Message::MousePressedIgnored),
        (iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), _) => {
            Some(Message::MousePressed)
        }
        (iced::Event::Window(iced::window::Event::CloseRequested), _) => {
            Some(Message::WindowCloseRequested(window))
        }
        (iced::Event::Window(iced::window::Event::Resized(size)), _) => {
            Some(Message::WindowResized { id: window, size })
        }
        _ => None,
    }
}

fn captured_register_arrow(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if modifiers.command() || modifiers.alt() || modifiers.shift() {
        return None;
    }
    let direction = match key {
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => super::RegisterMove::Up,
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => super::RegisterMove::Down,
        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => super::RegisterMove::Left,
        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => super::RegisterMove::Right,
        _ => return None,
    };
    Some(Message::RegisterArrowKey(direction))
}

fn command_shortcut_message(
    settings: &ShortcutSettings,
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
    status: event::Status,
) -> Option<Message> {
    if let Some(direction) = super::register_inline::ctrl_arrow_move(key, modifiers) {
        return Some(Message::RegisterArrowKey(direction));
    }
    if let keyboard::Key::Named(keyboard::key::Named::Tab) = key {
        return Some(Message::SettingsSectionCycle {
            backward: modifiers.shift(),
        });
    }
    if matches!(status, event::Status::Captured)
        && (is_text_select_all_shortcut(key, physical_key, modifiers)
            || is_text_paste_shortcut(key, physical_key, modifiers))
    {
        return None;
    }
    let configured = shortcut_message(settings, physical_key, modifiers);
    if configured.is_some() {
        return configured;
    }
    if matches!(status, event::Status::Ignored)
        && is_text_paste_shortcut(key, physical_key, modifiers)
    {
        return Some(Message::PasteMemoryBytesRequested);
    }
    None
}

fn is_text_select_all_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> bool {
    text_command_shortcut(key, physical_key, modifiers, 'a')
}

fn is_text_paste_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> bool {
    text_command_shortcut(key, physical_key, modifiers, 'v')
}

fn text_command_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
    expected: char,
) -> bool {
    if modifiers.shift() || modifiers.alt() {
        return false;
    }
    key.to_latin(physical_key)
        .is_some_and(|latin| latin.eq_ignore_ascii_case(&expected))
}

#[cfg(test)]
mod tests {
    use super::{captured_register_arrow, command_shortcut_message};
    use crate::app::{Message, RegisterMove};
    use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings};
    use iced::keyboard;
    use iced::keyboard::key::{Code, Physical};
    use iced::{event, keyboard::Modifiers};
    use std::mem::discriminant;

    fn char_key(value: &str) -> keyboard::Key {
        keyboard::Key::Character(value.into())
    }

    fn physical(code: Code) -> Physical {
        Physical::Code(code)
    }

    fn assert_message(actual: Option<Message>, expected: Message) {
        let actual = actual.expect("shortcut should resolve");
        assert_eq!(discriminant(&actual), discriminant(&expected));
    }

    fn default_settings() -> ShortcutSettings {
        ShortcutSettings::default()
    }

    #[test]
    fn captured_plain_arrows_are_forwarded_to_register_navigation() {
        for (key, expected) in [
            (keyboard::key::Named::ArrowUp, RegisterMove::Up),
            (keyboard::key::Named::ArrowDown, RegisterMove::Down),
            (keyboard::key::Named::ArrowLeft, RegisterMove::Left),
            (keyboard::key::Named::ArrowRight, RegisterMove::Right),
        ] {
            let message =
                captured_register_arrow(&keyboard::Key::Named(key), keyboard::Modifiers::default());
            assert!(
                matches!(message, Some(Message::RegisterArrowKey(actual)) if actual == expected)
            );
        }
    }

    #[test]
    fn modified_captured_arrows_keep_text_input_behavior() {
        let key = keyboard::Key::Named(keyboard::key::Named::ArrowRight);
        assert!(captured_register_arrow(&key, keyboard::Modifiers::SHIFT).is_none());
        assert!(captured_register_arrow(&key, keyboard::Modifiers::CTRL).is_none());
        assert!(captured_register_arrow(&key, keyboard::Modifiers::ALT).is_none());
    }

    #[test]
    fn captured_ctrl_a_keeps_text_input_select_all() {
        for (typed, code) in [("a", Code::KeyA), ("ф", Code::KeyA)] {
            assert!(
                command_shortcut_message(
                    &default_settings(),
                    &char_key(typed),
                    physical(code),
                    Modifiers::COMMAND,
                    event::Status::Captured,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn captured_ctrl_v_keeps_text_input_paste() {
        for (typed, code) in [("v", Code::KeyV), ("м", Code::KeyV)] {
            assert!(
                command_shortcut_message(
                    &default_settings(),
                    &char_key(typed),
                    physical(code),
                    Modifiers::COMMAND,
                    event::Status::Captured,
                )
                .is_none()
            );
        }
    }

    #[test]
    fn ignored_ctrl_v_requests_memory_paste() {
        assert_message(
            command_shortcut_message(
                &default_settings(),
                &char_key("м"),
                physical(Code::KeyV),
                Modifiers::COMMAND,
                event::Status::Ignored,
            ),
            Message::PasteMemoryBytesRequested,
        );
    }

    #[test]
    fn ignored_ctrl_a_still_opens_network_adapter() {
        assert_message(
            command_shortcut_message(
                &default_settings(),
                &char_key("ф"),
                physical(Code::KeyA),
                Modifiers::COMMAND,
                event::Status::Ignored,
            ),
            Message::OpenNetwork,
        );
    }

    #[test]
    fn captured_ctrl_s_still_saves_snapshot() {
        assert_message(
            command_shortcut_message(
                &default_settings(),
                &char_key("ы"),
                physical(Code::KeyS),
                Modifiers::COMMAND,
                event::Status::Captured,
            ),
            Message::SaveSnapshot,
        );
    }

    #[test]
    fn ignored_ctrl_v_uses_custom_shortcut_when_assigned() {
        let mut settings = ShortcutSettings::default();
        settings.assign(
            ShortcutAction::OpenMonitor,
            ShortcutBinding::new(true, false, false, ShortcutKey::V),
        );

        assert_message(
            command_shortcut_message(
                &settings,
                &char_key("м"),
                physical(Code::KeyV),
                Modifiers::COMMAND,
                event::Status::Ignored,
            ),
            Message::OpenMonitor,
        );
    }
}
