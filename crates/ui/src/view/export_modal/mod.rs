mod controls;
mod groups;
mod local_icons;
mod styles;
mod target;

use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length, alignment};

use groups::{MemoryGroupState, flags_group, memory_group, register_group};
use styles::{footer_button_style, modal_backdrop_style, modal_dialog_style, tab_button_style};

use super::theme::{TOKYO_TEXT, ui_text};
use crate::app::{
    ExportFlagSelection, ExportMemoryColumns, ExportRegisterSelection, ExportTab, Message,
};
use crate::i18n::{Key, Lang};

const DIALOG_WIDTH: f32 = 660.0;
const GROUP_HEIGHT: f32 = 258.0;
const FLAGS_GROUP_HEIGHT: f32 = 72.0;
const TAB_HEIGHT: f32 = 34.0;

pub(super) struct ExportModalViewState<'a> {
    pub(super) tab: ExportTab,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) target_dropdown_open: bool,
    pub(super) target_highlight: Option<usize>,
    pub(super) memory_start: &'a str,
    pub(super) memory_end: &'a str,
    pub(super) columns: ExportMemoryColumns,
    pub(super) registers: ExportRegisterSelection,
    pub(super) flags: ExportFlagSelection,
    pub(super) lang: Lang,
}

pub(super) fn export_modal_overlay<'a>(state: ExportModalViewState<'a>) -> Element<'a, Message> {
    let ExportModalViewState {
        tab,
        target_input,
        target_options,
        target_dropdown_open,
        target_highlight,
        memory_start,
        memory_end,
        columns,
        registers,
        flags,
        lang,
    } = state;

    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelExport);

    let body = container(
        column![
            tabs(tab, lang),
            row![
                memory_group(MemoryGroupState {
                    tab,
                    target_input,
                    target_options,
                    target_dropdown_open,
                    target_highlight,
                    memory_start,
                    memory_end,
                    columns,
                    lang,
                }),
                register_group(registers, lang),
            ]
            .spacing(12)
            .height(Length::Fixed(GROUP_HEIGHT)),
            container(flags_group(flags, lang))
                .height(Length::Fixed(FLAGS_GROUP_HEIGHT))
                .width(Length::Fill),
            footer(lang),
        ]
        .spacing(12)
        .width(Length::Fixed(DIALOG_WIDTH)),
    )
    .padding([18, 20])
    .style(modal_dialog_style);

    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(body),
            Space::new().width(Length::Fill),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![opaque(backdrop), centred]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn tabs(tab: ExportTab, lang: Lang) -> Element<'static, Message> {
    row![
        tab_button(
            lang.t(Key::ExportFormatXlsx),
            ExportTab::Xlsx,
            tab == ExportTab::Xlsx,
        ),
        tab_button(
            lang.t(Key::ExportFormatText),
            ExportTab::Text,
            tab == ExportTab::Text,
        ),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

fn tab_button(label: &'static str, target: ExportTab, active: bool) -> Element<'static, Message> {
    button(
        container(ui_text(label, 15, TOKYO_TEXT))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ExportTabSelected(target))
    .padding(0)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(TAB_HEIGHT))
    .style(move |_theme, status| tab_button_style(status, active))
    .into()
}

fn footer(lang: Lang) -> Element<'static, Message> {
    row![
        Space::new().width(Length::Fill),
        footer_button(lang.t(Key::DiscardCancel), Message::CancelExport,),
        footer_button(lang.t(Key::FileExport), Message::ConfirmExport),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn footer_button(label_text: &'static str, message: Message) -> Element<'static, Message> {
    button(
        container(ui_text(label_text, 14, TOKYO_TEXT))
            .padding([7, 22])
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(message)
    .padding(0)
    .style(move |_theme, status| footer_button_style(status))
    .into()
}

#[cfg(test)]
mod tests {
    use super::super::theme::{TOKYO_BORDER, TOKYO_SURFACE};
    use super::styles::{checkbox_style, flag_checkbox_style, tab_button_style};
    use iced::Background;
    use iced::widget::button;

    #[test]
    fn active_tab_uses_fill_without_accent_border() {
        let style = tab_button_style(button::Status::Active, true);

        assert_eq!(style.background, Some(Background::Color(TOKYO_SURFACE)));
        assert_eq!(style.border.color, TOKYO_BORDER);
    }

    #[test]
    fn checked_box_has_background_fill() {
        let style = checkbox_style(true);

        assert!(matches!(style.background, Some(Background::Color(_))));
    }

    #[test]
    fn unchecked_box_keeps_empty_background() {
        let style = checkbox_style(false);

        assert_eq!(style.background, None);
    }

    #[test]
    fn flag_checkbox_uses_rounder_border_than_regular_checkbox() {
        let regular = checkbox_style(false);
        let flag = flag_checkbox_style(false);

        assert_ne!(flag.border.radius, regular.border.radius);
    }
}
