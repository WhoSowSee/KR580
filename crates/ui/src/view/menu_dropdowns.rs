use iced::widget::{Space, button, column, container, row, svg};
use iced::{Element, Length, alignment};

use super::icons;
use super::styles::{menu_button_disabled_style, menu_button_style, opcode_dropdown_style};
use super::theme::{TOKYO_BORDER, TOKYO_MUTED, TOKYO_TEXT, ui_text};
use crate::app::Message;
use crate::i18n::{Key, Lang};

/// Width of the floating File dropdown. Picked wide enough that the
/// legacy-format note and shortcut fit beside the base action label.
pub(super) const FILE_DROPDOWN_WIDTH: f32 = 290.0;

/// Width of the MP-System dropdown. Tuned for the longest label plus
/// the longest shortcut hint.
pub(super) const MP_DROPDOWN_WIDTH: f32 = 270.0;

/// Width of the Help dropdown. Tuned for the longest Russian label.
pub(super) const HELP_DROPDOWN_WIDTH: f32 = 220.0;

/// Edge length of the icon square that prefixes every dropdown row.
pub(super) const MENU_ICON_SIZE: f32 = 16.0;

pub(super) fn file_dropdown(lang: Lang) -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = vec![
        menu_item(
            lang.t(Key::FileNew),
            "Ctrl+N",
            icons::file(),
            Message::NewFile,
            true,
        ),
        menu_item(
            lang.t(Key::FileOpen),
            "Ctrl+O",
            icons::folder_open(),
            Message::OpenSnapshot,
            true,
        ),
        menu_item(
            lang.t(Key::FileSave),
            "Ctrl+S",
            icons::save(),
            Message::SaveSnapshot,
            true,
        ),
        menu_item(
            lang.t(Key::FileSaveAs),
            "Ctrl+Shift+S",
            icons::save_as(),
            Message::SaveSnapshotAs,
            true,
        ),
        menu_item(
            lang.t(Key::FileImport),
            "Ctrl+I",
            icons::file_down(),
            Message::Import,
            true,
        ),
        menu_item(
            lang.t(Key::FileExport),
            "Ctrl+E",
            icons::file_up(),
            Message::Export,
            true,
        ),
        menu_separator(),
        legacy_menu_item(
            lang.t(Key::FileOpen),
            lang.t(Key::LegacyFormatNote),
            "Ctrl+Alt+O",
            icons::folder_open(),
            Message::OpenLegacySnapshot,
        ),
        legacy_menu_item(
            lang.t(Key::FileSave),
            lang.t(Key::LegacyFormatNote),
            "Ctrl+Alt+S",
            icons::save(),
            Message::SaveLegacySnapshot,
        ),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(FILE_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

pub(super) fn mp_dropdown(halted: bool, lang: Lang) -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = vec![
        menu_item(
            lang.t(Key::MpRunProgram),
            "Ctrl+R",
            icons::play(),
            Message::ToggleRun,
            true,
        ),
        menu_item(
            lang.t(Key::MpRunInstruction),
            "Ctrl+T",
            icons::step_forward(),
            Message::StepInstruction,
            true,
        ),
        menu_item(
            lang.t(Key::MpRunTact),
            "Ctrl+Y",
            icons::redo_dot(),
            Message::StepTact,
            true,
        ),
        menu_separator(),
        menu_item(
            lang.t(Key::MpResetRam),
            "Ctrl+Shift+R",
            icons::reset_ram(),
            Message::ResetRam,
            true,
        ),
        menu_item(
            lang.t(Key::MpResetCpu),
            "Ctrl+Shift+G",
            icons::reset_registers(),
            Message::ResetCpu,
            true,
        ),
        menu_item(
            lang.t(Key::MpClearHalt),
            "Ctrl+Shift+H",
            icons::clear_halt(),
            Message::ClearHalt,
            halted,
        ),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(MP_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

pub(super) fn help_dropdown(lang: Lang) -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> = vec![
        menu_item(
            lang.t(Key::HelpShowDocs),
            "Ctrl+H",
            icons::book_marked(),
            Message::ShowHelpComingSoon,
            true,
        ),
        menu_separator(),
        menu_item(
            lang.t(Key::HelpAbout),
            "",
            icons::info(),
            Message::OpenAbout,
            true,
        ),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(HELP_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

fn legacy_menu_item(
    label: &'static str,
    note: &'static str,
    shortcut: &'static str,
    icon: svg::Handle,
    action: Message,
) -> Element<'static, Message> {
    menu_item_with_note(label, Some(note), shortcut, icon, action, true)
}

fn menu_item(
    label: &'static str,
    shortcut: &'static str,
    icon: svg::Handle,
    action: Message,
    enabled: bool,
) -> Element<'static, Message> {
    menu_item_with_note(label, None, shortcut, icon, action, enabled)
}

fn menu_item_with_note(
    label: &'static str,
    note: Option<&'static str>,
    shortcut: &'static str,
    icon: svg::Handle,
    action: Message,
    enabled: bool,
) -> Element<'static, Message> {
    let glyph_color = if enabled { TOKYO_TEXT } else { TOKYO_MUTED };
    let label_color = if enabled { TOKYO_TEXT } else { TOKYO_MUTED };

    let glyph = svg(icon)
        .width(Length::Fixed(MENU_ICON_SIZE))
        .height(Length::Fixed(MENU_ICON_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let label: Element<'static, Message> = match note {
        Some(note) => row![
            ui_text(label, 13, label_color),
            ui_text(note, 11, TOKYO_MUTED),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center)
        .into(),
        None => ui_text(label, 13, label_color).into(),
    };

    let body = container(
        row![
            glyph,
            label,
            Space::new().width(Length::Fill),
            ui_text(shortcut, 11, TOKYO_MUTED),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .align_y(alignment::Vertical::Center);

    let mut btn = button(body).padding(0).width(Length::Fill);
    if enabled {
        let pair = vec![Message::MenuClosed, action];
        btn = btn
            .on_press(Message::MenuBatch(pair))
            .style(move |_theme, status| menu_button_style(status));
    } else {
        btn = btn.style(move |_theme, status| menu_button_disabled_style(status));
    }
    btn.into()
}

fn menu_separator() -> Element<'static, Message> {
    container(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.35,
                    ..TOKYO_BORDER
                })),
                ..iced::widget::container::Style::default()
            }),
    )
    .padding([4, 8])
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use crate::i18n::{Key, Lang};

    #[test]
    fn legacy_format_note_translates_per_language() {
        assert_eq!(Lang::Ru.t(Key::LegacyFormatNote), "старый формат");
        assert_eq!(Lang::En.t(Key::LegacyFormatNote), "legacy format");
    }
}
