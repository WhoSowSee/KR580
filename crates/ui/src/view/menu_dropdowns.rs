use iced::widget::{Space, button, column, container, row, svg};
use iced::{Element, Length, alignment};

use super::icons;
use super::styles::{menu_button_disabled_style, menu_button_style, opcode_dropdown_style};
use super::theme::{tokyo_border, tokyo_muted, tokyo_text, ui_text};
use super::tooltips::shortcut_hint;
use crate::app::{MenuId, Message, TopMenuIndicator, top_menu_action};
use crate::i18n::{Key, Lang};
use crate::persistence::ShortcutSettings;

/// Width of the floating File dropdown.
pub(super) const FILE_DROPDOWN_WIDTH: f32 = 290.0;

/// Width of the MP-System dropdown. Tuned for the longest label plus
/// the longest shortcut hint.
pub(super) const MP_DROPDOWN_WIDTH: f32 = 270.0;

/// Width of the Help dropdown. Tuned for the longest Russian label.
pub(super) const HELP_DROPDOWN_WIDTH: f32 = 260.0;

/// Width of the View dropdown. Tuned for the longest Russian device label plus shortcut.
pub(super) const VIEW_DROPDOWN_WIDTH: f32 = 360.0;

/// Edge length of the icon square that prefixes every dropdown row.
pub(super) const MENU_ICON_SIZE: f32 = 16.0;

pub(super) fn file_dropdown(
    lang: Lang,
    shortcuts: &ShortcutSettings,
    focused_item: Option<(usize, TopMenuIndicator)>,
) -> Element<'static, Message> {
    let item = |index, label, icon| {
        menu_item(
            label,
            shortcuts,
            icon,
            top_menu_action(MenuId::File, index).expect("valid File menu index"),
            true,
            item_focus(focused_item, index),
        )
    };
    let items: Vec<Element<'static, Message>> = vec![
        item(0, lang.t(Key::FileNew), icons::file()),
        item(1, lang.t(Key::FileOpen), icons::folder_open()),
        item(2, lang.t(Key::FileSave), icons::save()),
        item(3, lang.t(Key::FileSaveAs), icons::save_as()),
        item(4, lang.t(Key::FileImport), icons::file_down()),
        item(5, lang.t(Key::FileExport), icons::file_up()),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(FILE_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

pub(super) fn mp_dropdown(
    halted: bool,
    lang: Lang,
    shortcuts: &ShortcutSettings,
    focused_item: Option<(usize, TopMenuIndicator)>,
) -> Element<'static, Message> {
    let item = |index, label, icon, enabled| {
        menu_item(
            label,
            shortcuts,
            icon,
            top_menu_action(MenuId::Mp, index).expect("valid MP-System menu index"),
            enabled,
            item_focus(focused_item, index),
        )
    };
    let items: Vec<Element<'static, Message>> = vec![
        item(0, lang.t(Key::MpRunProgram), icons::play(), true),
        item(
            1,
            lang.t(Key::MpRunInstruction),
            icons::step_forward(),
            true,
        ),
        item(2, lang.t(Key::MpRunTact), icons::redo_dot(), true),
        menu_separator(),
        item(3, lang.t(Key::MpResetRam), icons::reset_ram(), true),
        item(4, lang.t(Key::MpResetCpu), icons::reset_registers(), true),
        item(5, lang.t(Key::MpClearHalt), icons::clear_halt(), halted),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(MP_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

pub(super) fn help_dropdown(
    lang: Lang,
    shortcuts: &ShortcutSettings,
    focused_item: Option<(usize, TopMenuIndicator)>,
) -> Element<'static, Message> {
    let item = |index, label, icon| {
        menu_item(
            label,
            shortcuts,
            icon,
            top_menu_action(MenuId::Help, index).expect("valid Help menu index"),
            true,
            item_focus(focused_item, index),
        )
    };
    let items: Vec<Element<'static, Message>> = vec![
        item(0, lang.t(Key::HelpShowDocs), icons::book_marked()),
        menu_separator(),
        item(1, lang.t(Key::HelpAbout), icons::info()),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(HELP_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

pub(super) fn view_dropdown(
    stack_view: bool,
    lang: Lang,
    shortcuts: &ShortcutSettings,
    focused_item: Option<(usize, TopMenuIndicator)>,
) -> Element<'static, Message> {
    let item = |index, label, icon| {
        menu_item(
            label,
            shortcuts,
            icon,
            top_menu_action(MenuId::View, index).expect("valid View menu index"),
            true,
            item_focus(focused_item, index),
        )
    };
    let items: Vec<Element<'static, Message>> = vec![
        item(0, lang.t(Key::DeviceMonitor), icons::device_monitor()),
        item(1, lang.t(Key::DeviceFloppy), icons::device_floppy()),
        item(2, lang.t(Key::DeviceHdd), icons::device_hdd()),
        item(3, lang.t(Key::DeviceNetwork), icons::device_network()),
        item(4, lang.t(Key::DevicePrinter), icons::device_printer()),
        menu_separator(),
        item(5, stack_view_label(stack_view, lang), icons::stack()),
    ];

    container(column(items).spacing(0))
        .padding(4)
        .width(Length::Fixed(VIEW_DROPDOWN_WIDTH))
        .style(opcode_dropdown_style)
        .into()
}

fn stack_view_label(stack_view: bool, lang: Lang) -> &'static str {
    lang.stack_view_area_label(stack_view)
}

fn item_focus(focused_item: Option<(usize, TopMenuIndicator)>, index: usize) -> TopMenuIndicator {
    focused_item
        .filter(|(focused_index, _)| *focused_index == index)
        .map_or(TopMenuIndicator::Hidden, |(_, indicator)| indicator)
}

fn menu_item(
    label: &'static str,
    shortcuts: &ShortcutSettings,
    icon: svg::Handle,
    action: Message,
    enabled: bool,
    indicator: TopMenuIndicator,
) -> Element<'static, Message> {
    let shortcut = shortcut_text(shortcuts, &action);
    let glyph_color = if enabled { tokyo_text() } else { tokyo_muted() };
    let label_color = if enabled { tokyo_text() } else { tokyo_muted() };

    let glyph = svg(icon)
        .width(Length::Fixed(MENU_ICON_SIZE))
        .height(Length::Fixed(MENU_ICON_SIZE))
        .style(move |_theme, _status| svg::Style {
            color: Some(glyph_color),
        });

    let label: Element<'static, Message> = ui_text(label, 13, label_color).into();

    let body = container(
        row![
            glyph,
            label,
            Space::new().width(Length::Fill),
            ui_text(shortcut, 11, tokyo_muted()),
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
            .style(move |_theme, status| menu_button_style(status, indicator));
    } else {
        btn = btn.style(move |_theme, status| menu_button_disabled_style(status));
    }
    btn.into()
}

fn shortcut_text(shortcuts: &ShortcutSettings, message: &Message) -> String {
    shortcut_hint(shortcuts, message).unwrap_or_default()
}

fn menu_separator() -> Element<'static, Message> {
    container(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.35,
                    ..tokyo_border()
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
    use super::{shortcut_text, stack_view_label};
    use crate::app::Message;
    use crate::i18n::Lang;
    use crate::persistence::{ShortcutAction, ShortcutBinding, ShortcutKey, ShortcutSettings};

    #[test]
    fn stack_view_menu_label_tracks_mode() {
        assert_eq!(
            stack_view_label(false, Lang::Ru),
            "Показать стековую область памяти"
        );
        assert_eq!(
            stack_view_label(true, Lang::Ru),
            "Скрыть стековую область памяти"
        );
    }

    #[test]
    fn menu_shortcut_text_tracks_custom_settings() {
        let mut settings = ShortcutSettings::default();
        settings.assign(
            ShortcutAction::OpenMonitor,
            ShortcutBinding::new(true, true, true, ShortcutKey::M),
        );

        assert_eq!(
            shortcut_text(&settings, &Message::OpenMonitor),
            "Ctrl+Shift+Alt+M"
        );
    }
}
