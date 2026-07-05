use iced::widget::{Space, button, container, row, svg};
use iced::{Background, Border, Color, Element, Length, alignment};

use super::super::consts::toggle_segment_width;
use super::super::setting_row::setting_row;
use super::super::speed::segmented_button_width;
use crate::app::{ContentFocus, Message, SettingsDialog, SettingsSection};
use crate::i18n::{Key, Lang};
use crate::view::icons;
use crate::view::theme::{tokyo_border, tokyo_muted, tokyo_surface, tokyo_text, ui_text};

pub(super) fn follow_pc_toggle_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();

    let kb_focused = kb_focus == Some(ContentFocus::FollowPc);

    let segments = row![
        segmented_button_width(
            lang.t(Key::SettingsToggleOn),
            dialog.draft_follow_pc,
            kb_focused,
            Message::SettingsDraftFollowPcSet(true),
            toggle_segment_width(lang),
        ),
        segmented_button_width(
            lang.t(Key::SettingsToggleOff),
            !dialog.draft_follow_pc,
            false,
            Message::SettingsDraftFollowPcSet(false),
            toggle_segment_width(lang),
        ),
    ]
    .spacing(6);

    setting_row(
        lang.t(Key::SettingsFollowPcLabel),
        lang.t(Key::SettingsFollowPcHint),
        segments.into(),
    )
}

pub(super) fn memory_operand_highlighting_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();

    let kb_focused = kb_focus == Some(ContentFocus::MemoryOperandHighlighting);

    let segments = row![
        segmented_button_width(
            lang.t(Key::SettingsToggleOn),
            dialog.draft_memory_operand_highlighting,
            kb_focused,
            Message::SettingsDraftMemoryOperandHighlightingSet(true),
            toggle_segment_width(lang),
        ),
        segmented_button_width(
            lang.t(Key::SettingsToggleOff),
            !dialog.draft_memory_operand_highlighting,
            false,
            Message::SettingsDraftMemoryOperandHighlightingSet(false),
            toggle_segment_width(lang),
        ),
    ]
    .spacing(6);

    setting_row(
        lang.t(Key::SettingsMemoryOperandHighlightingLabel),
        lang.t(Key::SettingsMemoryOperandHighlightingHint),
        segments.into(),
    )
}

pub(super) fn hdd_directory_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let kb_focused = kb_focus == Some(ContentFocus::HddDirectory);

    let raw_path = dialog
        .draft_hdd_directory
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_owned())
        });
    let display = truncate_path(&raw_path, 36);

    let browse_btn = settings_browse_button(
        lang.t(Key::SettingsHddDirectoryBrowse),
        Message::SettingsHddDirectoryBrowse,
        kb_focused,
    );

    let control = row![
        Space::new().width(Length::Fill),
        ui_text(display, 13, tokyo_muted()),
        Space::new().width(Length::Fixed(8.0)),
        browse_btn,
    ]
    .align_y(alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsHddDirectoryLabel),
        lang.t(Key::SettingsHddDirectoryHint),
        control.into(),
    )
}

pub(super) fn file_association_row<'a>(
    dialog: &'a SettingsDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let kb_focused = kb_focus == Some(ContentFocus::FileAssociation);

    let registered = k580_ui::file_assoc::is_registered();
    let label = if registered {
        Key::SettingsFileAssociationRemove
    } else {
        Key::SettingsFileAssociationAdd
    };
    let message = if registered {
        Message::SettingsFileAssociationUnregister
    } else {
        Message::SettingsFileAssociationRegister
    };

    let btn = settings_browse_button(lang.t(label), message, kb_focused);
    let control = row![Space::new().width(Length::Fill), btn].align_y(alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsFileAssociationLabel),
        lang.t(Key::SettingsFileAssociationHint),
        control.into(),
    )
}

pub(super) fn floppy_image_row<'a>(dialog: &'a SettingsDialog, lang: Lang) -> Element<'a, Message> {
    let kb_focus = (dialog.section == SettingsSection::Content)
        .then_some(dialog.content_focus)
        .flatten();
    let kb_focused = kb_focus == Some(ContentFocus::FloppyImage);

    // The floppy row carries both a browse and a clear button, so the
    // path text has a smaller budget than the single-button HDD row.
    let path_display = dialog
        .draft_floppy_image_path
        .as_ref()
        .map(|p| truncate_path(&p.display().to_string(), 24))
        .unwrap_or_default();

    let browse_btn = settings_browse_button(
        lang.t(Key::SettingsFloppyImageBrowse),
        Message::SettingsFloppyImageBrowse,
        kb_focused,
    );

    let mut control = row![Space::new().width(Length::Fill)];
    if dialog.draft_floppy_image_path.is_some() {
        control = control.push(ui_text(path_display, 13, tokyo_muted()));
        control = control.push(Space::new().width(Length::Fixed(8.0)));
    }
    control = control.push(browse_btn);

    if dialog.draft_floppy_image_path.is_some() {
        let clear_btn = button(
            container(
                svg(icons::brush_cleaning())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(move |_theme, _status| svg::Style {
                        color: Some(tokyo_text()),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::SettingsFloppyImageClear)
        .padding(0)
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(28.0))
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Pressed => tokyo_border(),
                button::Status::Hovered => tokyo_surface(),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: tokyo_text(),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: tokyo_border(),
                },
                ..button::Style::default()
            }
        });
        control = control.push(Space::new().width(Length::Fixed(8.0)));
        control = control.push(clear_btn);
    }

    control = control.align_y(alignment::Vertical::Center);

    setting_row(
        lang.t(Key::SettingsFloppyImageLabel),
        lang.t(Key::SettingsFloppyImageHint),
        control.into(),
    )
}

fn settings_browse_button<'a>(
    label: &'static str,
    message: Message,
    kb_focused: bool,
) -> Element<'a, Message> {
    button(container(ui_text(label, 12, tokyo_text())).padding([2, 8]))
        .on_press(message)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Pressed => tokyo_border(),
                button::Status::Hovered => tokyo_surface(),
                _ if kb_focused => tokyo_surface(),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: tokyo_text(),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: tokyo_border(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

pub(super) fn truncate_path(path: &str, max: usize) -> String {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() <= max {
        return path.to_owned();
    }
    let sep = if path.contains('\\') { '\\' } else { '/' };
    let segments: Vec<&str> = path.split(sep).collect();
    if segments.len() < 3 {
        let head: String = chars.iter().take(max / 2).collect();
        let tail: String = chars.iter().skip(chars.len() - max / 2).collect();
        return format!("{head}…{tail}");
    }
    let first = segments[0];
    let last = segments[segments.len() - 1];
    let mut budget = max.saturating_sub(first.chars().count() + last.chars().count() + 3);
    let mut middle = String::new();
    for seg in segments[1..segments.len() - 1].iter().rev() {
        let cost = seg.chars().count() + 1;
        if cost > budget {
            break;
        }
        middle = format!("{sep}{seg}{middle}");
        budget -= cost;
    }
    if middle.is_empty() {
        format!("{first}{sep}…{sep}{last}")
    } else {
        format!("{first}{sep}…{middle}{sep}{last}")
    }
}
