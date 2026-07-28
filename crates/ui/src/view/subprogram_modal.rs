use iced::widget::{Space, column, container, mouse_area, opaque, row, stack, text};
use iced::{Element, Length, alignment};

use super::styles::{modal_backdrop_style, panel_style as modal_dialog_style};
use super::theme::{tokyo_muted, tokyo_red, tokyo_text, ui_text};
use super::widgets::{modal_footer_button_focused, shorten_middle, text_input_shell};
use crate::app::{Message, SubprogramDialogFocus, SubprogramDialogMode};
use crate::i18n::{Key, Lang};

pub(super) struct SubprogramModalViewState<'a> {
    pub(super) mode: SubprogramDialogMode,
    pub(super) focus: SubprogramDialogFocus,
    pub(super) keyboard_focus_visible: bool,
    pub(super) path: &'a std::path::Path,
    pub(super) start: &'a str,
    pub(super) end: &'a str,
    pub(super) error: Option<&'a str>,
    pub(super) lang: Lang,
}

pub(super) fn subprogram_modal_overlay<'a>(
    state: SubprogramModalViewState<'a>,
) -> Element<'a, Message> {
    let title = match state.mode {
        SubprogramDialogMode::Open => state.lang.t(Key::SubprogramTitleOpen),
        SubprogramDialogMode::Save => state.lang.t(Key::SubprogramTitleSave),
    };
    let confirm_label = match state.mode {
        SubprogramDialogMode::Open => state.lang.t(Key::FileOpen),
        SubprogramDialogMode::Save => state.lang.t(Key::FileSave),
    };
    let start_focused = state.keyboard_focus_visible && state.focus == SubprogramDialogFocus::Start;
    let end_focused = state.keyboard_focus_visible && state.focus == SubprogramDialogFocus::End;
    let cancel_focused =
        state.keyboard_focus_visible && state.focus == SubprogramDialogFocus::Cancel;
    let confirm_focused =
        state.keyboard_focus_visible && state.focus == SubprogramDialogFocus::Confirm;

    let mut fields = column![
        path_row(state.path, state.lang),
        field_row(
            state.lang.t(Key::SubprogramStartAddress),
            state.start,
            Message::SubprogramStartChanged,
            start_focused,
        ),
    ]
    .spacing(10);
    if matches!(state.mode, SubprogramDialogMode::Save) {
        fields = fields.push(field_row(
            state.lang.t(Key::SubprogramEndAddress),
            state.end,
            Message::SubprogramEndChanged,
            end_focused,
        ));
    }
    if let Some(error) = state.error {
        fields = fields.push(text(error).size(12).color(tokyo_red()));
    }

    let footer = row![
        Space::new().width(Length::Fill),
        modal_footer_button_focused(
            state.lang.t(Key::DiscardCancel),
            Message::CancelSubprogram,
            super::styles::modal_field_button_style,
            cancel_focused,
        ),
        modal_footer_button_focused(
            confirm_label,
            Message::ConfirmSubprogram,
            super::styles::modal_field_button_style,
            confirm_focused,
        ),
    ]
    .spacing(10);

    let dialog = container(
        column![
            ui_text(title, 16, tokyo_text()),
            ui_text(state.lang.t(Key::SubprogramAddressHint), 12, tokyo_muted(),),
            fields,
            footer,
        ]
        .spacing(12)
        .width(Length::Fixed(500.0)),
    )
    .padding(18)
    .style(modal_dialog_style);

    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelSubprogram);
    let centred = column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            opaque(dialog),
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

fn path_row(path: &std::path::Path, lang: Lang) -> Element<'static, Message> {
    row![
        container(ui_text(lang.t(Key::ImportFileLabel), 13, tokyo_text()))
            .width(Length::Fixed(132.0))
            .align_y(alignment::Vertical::Center),
        container(ui_text(
            shorten_middle(&path.display().to_string(), 48),
            13,
            tokyo_text(),
        ))
        .width(Length::Fill)
        .padding([6, 8])
        .style(super::styles::inset_style),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn field_row<'a>(
    label: &'static str,
    value: &'a str,
    on_input: fn(String) -> Message,
    focused: bool,
) -> Element<'a, Message> {
    let input = text_input_shell("", value, on_input, Length::Fill);
    row![
        container(ui_text(label, 13, tokyo_text()))
            .width(Length::Fixed(132.0))
            .align_y(alignment::Vertical::Center),
        container(input).width(Length::Fill).style(move |_theme| {
            let mut style = super::styles::inset_style(_theme);
            if focused {
                style.border.color = tokyo_text();
            }
            style
        }),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into()
}
