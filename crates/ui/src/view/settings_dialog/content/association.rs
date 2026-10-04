use iced::widget::{Space, row};
use iced::{Element, Length, alignment};

use super::super::setting_row::setting_row;
use super::rows::settings_browse_button;
use crate::app::{ContentFocus, Message, SettingsDialog};
use crate::i18n::{Key, Lang};

pub(super) fn file_association_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
    pending: bool,
) -> Element<'a, Message> {
    let focused = dialog.content_focus_is_visible(ContentFocus::FileAssociation);

    #[cfg(target_os = "windows")]
    let (label, message) = if dialog.file_association_registered {
        (
            Key::SettingsFileAssociationRemove,
            Message::SettingsFileAssociationUnregister,
        )
    } else {
        (
            Key::SettingsFileAssociationAdd,
            Message::SettingsFileAssociationRegister,
        )
    };
    #[cfg(not(target_os = "windows"))]
    let (label, message) = (
        Key::SettingsFileAssociationAdd,
        Message::SettingsFileAssociationRegister,
    );

    let (label, message) = if pending {
        (Key::SettingsFileAssociationWorking, None)
    } else {
        (label, Some(message))
    };
    let button = settings_browse_button(lang.t(label), message, focused);
    let control =
        row![Space::new().width(Length::Fill), button].align_y(alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsFileAssociationLabel),
        lang.t(Key::SettingsFileAssociationHint),
        control.into(),
    )
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use iced::advanced::{Layout, Shell, clipboard, layout, renderer::Headless, widget};
    use iced::{Event, Point, Size, mouse};

    #[test]
    fn cached_status_and_pending_state_update_the_existing_button() {
        let mut dialog = SettingsDialog::new(crate::app::settings_modal::SettingsInitialState {
            lang: Lang::En,
            speed: crate::app::messages::SpeedTier::High,
            follow_pc: false,
            memory_operand_highlighting: true,
            floppy_image_path: None,
            hdd_directory: None,
            network: crate::persistence::NetworkSettings::default(),
            ..Default::default()
        });
        let renderer = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(iced::Renderer::new(
                iced::Font::DEFAULT,
                13.0.into(),
                Some("tiny-skia"),
            ))
            .unwrap();
        let mut tree = widget::Tree::empty();
        for (registered, pending) in [(false, false), (false, true), (true, false)] {
            dialog.file_association_registered = registered;
            let mut root = file_association_row(&dialog, Lang::En, pending);
            tree.diff(root.as_widget());
            let node = root.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(600.0, 100.0)),
            );
            let layout = Layout::new(&node);
            let mut messages = Vec::new();
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                root.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    layout,
                    mouse::Cursor::Available(Point::new(layout.bounds().width - 10.0, 10.0)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &layout.bounds(),
                );
            }
            match (pending, registered) {
                (true, _) => assert!(messages.is_empty()),
                (false, true) => assert!(matches!(
                    messages.as_slice(),
                    [Message::SettingsFileAssociationUnregister]
                )),
                (false, false) => assert!(matches!(
                    messages.as_slice(),
                    [Message::SettingsFileAssociationRegister]
                )),
            }
        }
    }
}
