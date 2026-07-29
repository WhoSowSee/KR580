mod controls;
mod groups;
mod local_icons;
mod styles;
mod target;

use iced::widget::{Space, button, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length, alignment};

use groups::{MemoryGroupState, flags_group, memory_group, register_group};
use styles::{footer_button_style, modal_backdrop_style, modal_dialog_style, tab_button_style};

use super::theme::{tokyo_text, ui_text};
use super::widgets::modal_footer_button_focused;
use crate::app::{
    ExportFlagSelection, ExportMemoryColumns, ExportModalFocus, ExportRegisterSelection, ExportTab,
    Message,
};
use crate::i18n::{Key, Lang};

const DIALOG_WIDTH: f32 = 660.0;
const GROUP_HEIGHT: f32 = 258.0;
const FLAGS_GROUP_HEIGHT: f32 = 72.0;
const TAB_HEIGHT: f32 = 34.0;

pub(super) struct ExportModalViewState<'a> {
    pub(super) tab: ExportTab,
    pub(super) focus: ExportModalFocus,
    pub(super) keyboard_focus_visible: bool,
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
        focus,
        keyboard_focus_visible,
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
            tabs(tab, focus, keyboard_focus_visible, lang),
            row![
                memory_group(MemoryGroupState {
                    tab,
                    focus,
                    keyboard_focus_visible,
                    target_input,
                    target_options,
                    target_dropdown_open,
                    target_highlight,
                    memory_start,
                    memory_end,
                    columns,
                    lang,
                }),
                register_group(registers, focus, keyboard_focus_visible, lang),
            ]
            .spacing(12)
            .height(Length::Fixed(GROUP_HEIGHT)),
            container(flags_group(flags, focus, keyboard_focus_visible, lang))
                .height(Length::Fixed(FLAGS_GROUP_HEIGHT))
                .width(Length::Fill),
            footer(focus, keyboard_focus_visible, lang),
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

fn tabs(
    tab: ExportTab,
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
    lang: Lang,
) -> Element<'static, Message> {
    row![
        tab_button(
            lang.t(Key::ExportFormatXlsx),
            ExportTab::Xlsx,
            tab == ExportTab::Xlsx,
            keyboard_focus_visible && focus == ExportModalFocus::TabXlsx,
        ),
        tab_button(
            lang.t(Key::ExportFormatText),
            ExportTab::Text,
            tab == ExportTab::Text,
            keyboard_focus_visible && focus == ExportModalFocus::TabText,
        ),
    ]
    .spacing(4)
    .width(Length::Fill)
    .into()
}

fn tab_button(
    label: &'static str,
    target: ExportTab,
    active: bool,
    focused: bool,
) -> Element<'static, Message> {
    button(
        container(ui_text(label, 15, tokyo_text()))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ExportTabSelected(target))
    .padding(0)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(TAB_HEIGHT))
    .style(move |_theme, status| tab_button_style(status, active, focused))
    .into()
}

fn footer(
    focus: ExportModalFocus,
    keyboard_focus_visible: bool,
    lang: Lang,
) -> Element<'static, Message> {
    row![
        Space::new().width(Length::Fill),
        modal_footer_button_focused(
            lang.t(Key::DiscardCancel),
            Message::CancelExport,
            footer_button_style,
            keyboard_focus_visible && focus == ExportModalFocus::Cancel,
        ),
        modal_footer_button_focused(
            lang.t(Key::FileExport),
            Message::ConfirmExport,
            footer_button_style,
            keyboard_focus_visible && focus == ExportModalFocus::Confirm,
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::super::theme::{tokyo_border, tokyo_surface, tokyo_text};
    use super::GROUP_HEIGHT;
    use super::styles::{
        checkbox_style, checklist_button_style, flag_checkbox_style, tab_button_style,
        target_icon_focus_color,
    };
    use super::target::dropdown_list_height;
    use iced::Background;
    use iced::widget::button;

    #[test]
    fn short_target_list_is_not_capped() {
        assert_eq!(dropdown_list_height(1), 28.0);
        assert_eq!(dropdown_list_height(6), 168.0);
    }

    #[test]
    fn long_target_list_is_capped_at_six_rows_so_it_scrolls() {
        assert_eq!(dropdown_list_height(7), 168.0);
        assert_eq!(dropdown_list_height(64), 168.0);
    }

    #[test]
    fn capped_target_list_stays_inside_the_memory_group() {
        const PANEL_PADDING: f32 = 4.0 + 4.0;
        const DROPDOWN_OFFSET: f32 = 35.0;
        const GROUP_CONTENT_HEIGHT: f32 = GROUP_HEIGHT - 9.0 - 18.0 - 12.0;

        let bottom = DROPDOWN_OFFSET + dropdown_list_height(64) + PANEL_PADDING;

        assert!(bottom <= GROUP_CONTENT_HEIGHT);
    }

    #[test]
    fn active_tab_uses_fill_without_accent_border() {
        let style = tab_button_style(button::Status::Active, true, false);

        assert_eq!(style.background, Some(Background::Color(tokyo_surface())));
        assert_eq!(style.border.color, tokyo_border());
    }

    #[test]
    fn keyboard_focused_tab_uses_text_border_without_changing_fill() {
        let style = tab_button_style(button::Status::Active, true, true);

        assert_eq!(style.background, Some(Background::Color(tokyo_surface())));
        assert_eq!(style.border.color, tokyo_text());
    }

    #[test]
    fn keyboard_focused_checkbox_uses_text_border() {
        let style = checklist_button_style(button::Status::Active, true);

        assert_eq!(style.border.color, tokyo_text());
        assert_eq!(style.border.width, 1.0);
    }

    #[test]
    fn keyboard_focused_target_icon_uses_text_border() {
        assert_eq!(target_icon_focus_color(), tokyo_text());
    }

    #[test]
    fn checked_box_background_is_transparent() {
        let style = checkbox_style(true);

        assert_eq!(style.background, None);
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
