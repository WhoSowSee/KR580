mod controls;
#[cfg(test)]
mod controls_tests;
mod styles;

use iced::widget::{Space, column, container, mouse_area, opaque, row, stack};
use iced::{Element, Length};

use controls::{SourceGroupState, footer, source_group, target_dropdown_overlay};
use styles::{modal_backdrop_style, modal_dialog_style};

use crate::app::{ImportFileFormat, Message};
use crate::i18n::Lang;

const DIALOG_WIDTH: f32 = 500.0;
const COMPACT_CONTENT_HEIGHT: f32 = 126.0;
const TARGET_CONTENT_HEIGHT: f32 = 164.0;

pub(super) struct ImportModalViewState<'a> {
    pub(super) file_display: &'a str,
    pub(super) format: Option<ImportFileFormat>,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) target_dropdown_open: bool,
    pub(super) target_highlight: Option<usize>,
    pub(super) target_scroll_reveal: bool,
    pub(super) error: Option<&'a str>,
    pub(super) lang: Lang,
}

pub(super) fn import_modal_overlay<'a>(state: ImportModalViewState<'a>) -> Element<'a, Message> {
    let ImportModalViewState {
        file_display,
        format,
        target_input,
        target_options,
        target_dropdown_open,
        target_highlight,
        target_scroll_reveal,
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
            file_display,
            format,
            target_input,
            target_options,
            error,
            lang,
        }),
        footer(lang),
    ]
    .spacing(14)
    .width(Length::Fixed(DIALOG_WIDTH));

    let content_height = if format.is_some() || error.is_some() {
        TARGET_CONTENT_HEIGHT
    } else {
        COMPACT_CONTENT_HEIGHT
    };

    let body_content: Element<'_, Message> =
        if target_dropdown_open && format.is_some() && !target_options.is_empty() {
            let close_layer = mouse_area(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(content_height)),
            )
            .on_press(Message::ImportTargetDropdownToggled);
            stack![
                body_content,
                close_layer,
                target_dropdown_overlay(target_options, target_highlight, target_scroll_reveal),
            ]
            .width(Length::Fixed(DIALOG_WIDTH))
            .height(Length::Fixed(content_height))
            .into()
        } else {
            container(body_content)
                .height(Length::Fixed(content_height))
                .into()
        };

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
