use iced::{Subscription, event, keyboard, mouse, time};

use super::handlers::{ctrl_shortcut, plain_shortcut, tick_interval};
use super::messages::Message;
use super::state::DesktopApp;

impl DesktopApp {
    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            time::every(tick_interval(self.running, self.speed_tier)).map(|_| Message::Tick),
            iced::window::open_events().map(Message::WindowOpened),
            event::listen_with(|event, status, _window| match (event, status) {
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
                // Ctrl-modified shortcuts run unconditionally so
                // Ctrl+S saves even when a text_input has focus.
                (
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key,
                        physical_key,
                        modifiers,
                        ..
                    }),
                    _,
                ) if modifiers.command() => ctrl_shortcut(&key, physical_key, modifiers),
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
                (iced::Event::Mouse(mouse::Event::CursorMoved { position }), _) => {
                    Some(Message::CursorMoved(position))
                }
                // Captured presses must reach us — the focus
                // reconciler walks the tree from outside to clear
                // stale focus that `text_input::update` missed
                // across stacked panels.
                (iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)), _) => {
                    Some(Message::MousePressed)
                }
                (iced::Event::Window(iced::window::Event::CloseRequested), _) => {
                    Some(Message::WindowCloseRequested)
                }
                (iced::Event::Window(iced::window::Event::Resized(size)), _) => {
                    Some(Message::WindowResized(size.width))
                }
                _ => None,
            }),
        ];

        if self.startup_frames_seen < 2 {
            subscriptions.push(iced::window::frames().map(|_| Message::FrameRendered));
        }

        Subscription::batch(subscriptions)
    }
}
