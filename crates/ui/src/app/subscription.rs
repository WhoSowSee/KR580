use iced::{Subscription, event, keyboard, mouse, time};

use super::handlers::{ctrl_shortcut, plain_shortcut, tick_interval};
use super::messages::Message;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            time::every(tick_interval(self.running, self.speed_tier)).map(|_| Message::Tick),
            iced::window::close_events().map(Message::WindowClosed),
            event::listen_with(|event, status, window| match (event, status) {
                (iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)), _) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Escape),
                        ..
                    }),
                    _,
                ) => Some(Message::EscPressed),
                // App shortcuts win over focused text widgets except Ctrl+A, which keeps native Select All.
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        physical_key,
                        modifiers,
                        ..
                    }),
                    status,
                ) if modifiers.command() => {
                    command_shortcut_message(&key, physical_key, modifiers, status)
                }
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
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        Some(Message::ArrowKey(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        Some(Message::ArrowKey(-1))
                    }
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
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        Some(Message::EnterPressed)
                    }
                    _ => plain_shortcut(&key, physical_key, modifiers),
                },
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }),
                    iced::event::Status::Captured,
                ) => captured_register_arrow(&key, modifiers),
                (iced::Event::Mouse(mouse::Event::CursorMoved { position }), _) => {
                    Some(Message::CursorMoved(position))
                }
                // Captured presses must reach us – the focus
                // reconciler walks the tree from outside to clear
                // stale focus that `text_input::update` missed
                // across stacked panels.
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
            }),
        ];

        if self.startup_frames_seen < 2 || self.settings_dialog.is_some() {
            subscriptions.push(iced::window::frames().map(|_| Message::FrameRendered));
        }

        Subscription::batch(subscriptions)
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
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
    status: event::Status,
) -> Option<Message> {
    if matches!(status, event::Status::Captured)
        && is_text_select_all_shortcut(key, physical_key, modifiers)
    {
        return None;
    }
    ctrl_shortcut(key, physical_key, modifiers)
}

fn is_text_select_all_shortcut(
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> bool {
    if modifiers.shift() || modifiers.alt() {
        return false;
    }
    key.to_latin(physical_key)
        .is_some_and(|latin| latin.eq_ignore_ascii_case(&'a'))
}

#[cfg(test)]
mod tests {
    use super::{captured_register_arrow, command_shortcut_message};
    use crate::app::{Message, RegisterMove};
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
    fn ignored_ctrl_a_still_opens_network_adapter() {
        assert_message(
            command_shortcut_message(
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
                &char_key("ы"),
                physical(Code::KeyS),
                Modifiers::COMMAND,
                event::Status::Captured,
            ),
            Message::SaveSnapshot,
        );
    }
}
