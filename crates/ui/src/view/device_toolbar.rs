use iced::widget::{Button, button};

use super::theme::{tokyo_blue, tokyo_device_accent};
use crate::app::{DeviceToolbar, Message};

pub(super) fn toolbar_button<'a>(
    button: Button<'a, Message>,
    action: Option<Message>,
    toolbar: Option<DeviceToolbar>,
    style: impl Fn(button::Status) -> button::Style + 'a,
) -> Button<'a, Message> {
    let focused = toolbar.is_some_and(|toolbar| {
        action
            .as_ref()
            .is_some_and(|action| toolbar.state.focus.matches(action))
    });
    let keyboard = toolbar.is_some_and(|toolbar| toolbar.state.focus.keyboard);
    button
        .on_press_maybe(action.map(|action| match toolbar {
            Some(toolbar) => Message::DeviceButtonPressed(toolbar.kind, Box::new(action)),
            None => action,
        }))
        .style(move |_theme, status| {
            let mut appearance = style(status);
            if focused {
                if keyboard {
                    appearance.border.color = tokyo_device_accent(tokyo_blue());
                    appearance.border.width = 2.0;
                } else {
                    appearance.background =
                        super::modal::modal_button_style(status, true, false).background;
                }
            }
            appearance
        })
}
