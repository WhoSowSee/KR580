mod controls;
mod styles;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use controls::{SourceGroupState, footer, source_group};
use styles::{modal_backdrop_style, modal_dialog_style};

use crate::app::{ImportFileFormat, ImportModalFocus, Message};
use crate::i18n::Lang;

const DIALOG_WIDTH: f32 = 500.0;
const BODY_HEIGHT: f32 = 244.0;

pub(super) struct ImportModalViewState<'a> {
    pub(super) focus: ImportModalFocus,
    pub(super) keyboard_focus_visible: bool,
    pub(super) file_drag_hovered: bool,
    pub(super) file_display: &'a str,
    pub(super) format: Option<ImportFileFormat>,
    pub(super) loading: bool,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) target_dropdown_open: bool,
    pub(super) target_highlight: Option<usize>,
    pub(super) error: Option<&'a str>,
    pub(super) lang: Lang,
}

pub(super) fn import_modal_overlay<'a>(state: ImportModalViewState<'a>) -> Element<'a, Message> {
    let ImportModalViewState {
        focus,
        keyboard_focus_visible,
        file_drag_hovered,
        file_display,
        format,
        loading,
        target_input,
        target_options,
        target_dropdown_open,
        target_highlight,
        error,
        lang,
    } = state;

    let backdrop = mouse_area(
        container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(modal_backdrop_style),
    )
    .on_press(Message::CancelImport);

    let body_content = column![
        source_group(SourceGroupState {
            focus,
            keyboard_focus_visible,
            file_drag_hovered,
            file_display,
            format,
            target_input,
            target_options,
            target_dropdown_open,
            target_highlight,
            error,
            lang,
        }),
        Space::new().height(Length::Fill),
        footer(
            focus,
            keyboard_focus_visible,
            format.is_some() && error.is_none() && !loading,
            lang,
        ),
    ]
    .spacing(8)
    .width(Length::Fixed(DIALOG_WIDTH))
    .height(Length::Fixed(BODY_HEIGHT));
    let body = container(body_content)
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
