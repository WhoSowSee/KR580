use super::super::icons;
use super::super::theme::{tokyo_muted, tokyo_red, tokyo_text, ui_text};
use super::super::widgets::{modal_footer_button_focused, shorten_middle};
use super::styles::{
    drop_zone_style, dropdown_option_style, dropdown_panel_style, field_button_style_focused,
    footer_button_style, target_anchor_style_focused,
};
use crate::app::{ImportFileFormat, ImportModalFocus, Message};
use crate::i18n::{Key, Lang};
use iced::widget::{Space, button, column, container, opaque, row, scrollable, svg, text};
use iced::{Element, Length, Padding, alignment};

const FIELD_WIDTH: f32 = 420.0;
const LABEL_WIDTH: f32 = 72.0;
const ROW_HEIGHT: f32 = 34.0;
const EMPTY_DROP_ZONE_HEIGHT: f32 = 192.0;
const SELECTED_DROP_ZONE_HEIGHT: f32 = 144.0;
const SOURCE_TO_TARGET_GAP: f32 = 16.0;
const DROPDOWN_ANCHOR_GAP: f32 = 8.0;
const DROPDOWN_PANEL_PADDING: f32 = 4.0;
const DROPDOWN_TOP: f32 =
    SELECTED_DROP_ZONE_HEIGHT + SOURCE_TO_TARGET_GAP + ROW_HEIGHT + DROPDOWN_ANCHOR_GAP;
const DROPDOWN_LEFT: f32 = 80.0;
const DROPDOWN_OPTION_HEIGHT: f32 = 20.0;
const DROPDOWN_MAX_LIST_HEIGHT: f32 = 40.0;
const DROPDOWN_OPTION_CHARS: usize = 39;
pub(super) const TARGET_DROPDOWN_OVERLAY_HEIGHT: f32 =
    DROPDOWN_TOP + DROPDOWN_MAX_LIST_HEIGHT + DROPDOWN_PANEL_PADDING * 2.0 + 1.0;

pub(super) struct SourceGroupState<'a> {
    pub(super) focus: ImportModalFocus,
    pub(super) keyboard_focus_visible: bool,
    pub(super) file_drag_hovered: bool,
    pub(super) file_display: &'a str,
    pub(super) format: Option<ImportFileFormat>,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) error: Option<&'a str>,
    pub(super) lang: Lang,
}

pub(super) fn source_group<'a>(state: SourceGroupState<'a>) -> Element<'a, Message> {
    let SourceGroupState {
        focus,
        keyboard_focus_visible,
        file_drag_hovered,
        file_display,
        format,
        target_input,
        target_options,
        error,
        lang,
    } = state;
    let drop_zone_height = if format.is_none() && error.is_none() {
        EMPTY_DROP_ZONE_HEIGHT
    } else {
        SELECTED_DROP_ZONE_HEIGHT
    };

    let mut content = column![drop_zone(
        file_display,
        file_drag_hovered,
        focus == ImportModalFocus::Browse && keyboard_focus_visible,
        drop_zone_height,
        lang,
    )]
    .spacing(SOURCE_TO_TARGET_GAP);

    if let Some(format) = format {
        if target_options.is_empty() && error.is_none() {
            content = content.push(no_targets_row(lang));
        } else if !target_options.is_empty() {
            content = content.push(target_row(
                format,
                target_input,
                focus == ImportModalFocus::Target && keyboard_focus_visible,
                lang,
            ));
        }
    }
    if let Some(error) = error {
        content = content.push(error_row(error));
    }

    content.into()
}

pub(super) fn footer(
    focus: ImportModalFocus,
    keyboard_focus_visible: bool,
    can_import: bool,
    lang: Lang,
) -> Element<'static, Message> {
    let confirm: Element<'static, Message> = if can_import {
        modal_footer_button_focused(
            lang.t(Key::FileImport),
            Message::ConfirmImport,
            footer_button_style,
            keyboard_focus_visible && focus == ImportModalFocus::Confirm,
        )
    } else {
        button(
            container(ui_text(lang.t(Key::FileImport), 14, tokyo_muted()))
                .padding([7, 22])
                .align_x(alignment::Horizontal::Center),
        )
        .padding(0)
        .style(|_theme, status| footer_button_style(status))
        .into()
    };

    row![
        Space::new().width(Length::Fill),
        modal_footer_button_focused(
            lang.t(Key::DiscardCancel),
            Message::CancelImport,
            footer_button_style,
            keyboard_focus_visible && focus == ImportModalFocus::Cancel,
        ),
        confirm,
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn drop_zone<'a>(
    file_display: &'a str,
    hovered: bool,
    focused: bool,
    height: f32,
    lang: Lang,
) -> Element<'a, Message> {
    let glyph = svg(icons::file_down())
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(32.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });
    let mut controls = column![glyph]
        .spacing(if file_display.is_empty() { 4 } else { 8 })
        .align_x(alignment::Horizontal::Center);

    if file_display.is_empty() {
        controls = controls
            .push(ui_text(lang.t(Key::ImportDropPrompt), 14, tokyo_text()))
            .push(ui_text(lang.t(Key::ImportDropOr), 11, tokyo_muted()));
    } else {
        controls = controls.push(
            ui_text(shorten_middle(file_display, 48), 13, tokyo_text())
                .wrapping(text::Wrapping::None),
        );
    }

    controls = controls.push(browse_button(focused, lang));
    let content = column![
        controls,
        ui_text(lang.t(Key::ImportSupportedFormats), 11, tokyo_muted(),),
    ]
    .spacing(8)
    .align_x(alignment::Horizontal::Center);

    container(content)
        .padding([6, 16])
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |theme| drop_zone_style(theme, hovered, focused))
        .into()
}

fn browse_button(focused: bool, lang: Lang) -> Element<'static, Message> {
    let glyph = svg(icons::folder_open())
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_text()),
        });
    let content = row![
        glyph,
        ui_text(lang.t(Key::ImportBrowseTooltip), 12, tokyo_text()),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);

    button(container(content).padding([6, 14]))
        .on_press(Message::ImportFileBrowse)
        .padding(0)
        .style(move |_theme, status| field_button_style_focused(status, focused))
        .into()
}

fn target_row<'a>(
    format: ImportFileFormat,
    target_input: &'a str,
    focused: bool,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        row_label(lang.t(format.target_label_key())),
        target_anchor(target_input, focused),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(ROW_HEIGHT))
    .into()
}

fn no_targets_row(lang: Lang) -> Element<'static, Message> {
    container(ui_text(lang.t(Key::ImportNoTargets), 12, tokyo_muted()))
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn error_row(error: &str) -> Element<'_, Message> {
    container(ui_text(error, 12, tokyo_red()))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
}

fn target_anchor<'a>(value: &'a str, focused: bool) -> Element<'a, Message> {
    let chevron = svg(icons::chevron_down())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme, _status| svg::Style {
            color: Some(tokyo_muted()),
        });
    let content = row![
        ui_text(
            shorten_middle(value, DROPDOWN_OPTION_CHARS),
            13,
            tokyo_text(),
        )
        .wrapping(text::Wrapping::None),
        Space::new().width(Length::Fill),
        chevron,
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);

    button(
        container(content)
            .padding([6, 10])
            .width(Length::Fixed(FIELD_WIDTH))
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(alignment::Vertical::Center)
            .clip(true),
    )
    .on_press(Message::ImportTargetDropdownToggled)
    .padding(0)
    .style(move |_theme, status| target_anchor_style_focused(status, focused))
    .into()
}

pub(super) fn target_dropdown_overlay(
    options: &[String],
    highlighted: Option<usize>,
) -> Element<'static, Message> {
    column![
        Space::new().height(Length::Fixed(DROPDOWN_TOP)),
        row![
            Space::new().width(Length::Fixed(DROPDOWN_LEFT)),
            opaque(dropdown(options, highlighted)),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn dropdown(options: &[String], highlighted: Option<usize>) -> Element<'static, Message> {
    let mut list = column![].spacing(0);
    for (index, option) in options.iter().enumerate() {
        list = list.push(dropdown_option(option.clone(), highlighted == Some(index)));
    }
    let list_height = (options.len() as f32 * DROPDOWN_OPTION_HEIGHT).min(DROPDOWN_MAX_LIST_HEIGHT);
    let list = scrollable(list)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::hidden(),
        ))
        .height(Length::Fixed(list_height));

    container(list)
        .padding(DROPDOWN_PANEL_PADDING)
        .width(Length::Fixed(FIELD_WIDTH))
        .style(dropdown_panel_style)
        .into()
}

fn dropdown_option(label_text: String, highlighted: bool) -> Element<'static, Message> {
    let label = ui_text(
        shorten_middle(&label_text, DROPDOWN_OPTION_CHARS),
        13,
        tokyo_text(),
    )
    .line_height(1.0)
    .wrapping(text::Wrapping::None);
    button(
        container(label)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 5.0,
                left: 10.0,
            })
            .width(Length::Fill)
            .height(Length::Fixed(DROPDOWN_OPTION_HEIGHT))
            .align_y(alignment::Vertical::Center)
            .clip(true),
    )
    .on_press(Message::ImportTargetSelected(label_text))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| dropdown_option_style(status, highlighted))
    .into()
}

fn row_label(value: &'static str) -> Element<'static, Message> {
    container(ui_text(value, 13, tokyo_text()))
        .width(Length::Fixed(LABEL_WIDTH))
        .height(Length::Fixed(ROW_HEIGHT))
        .align_y(alignment::Vertical::Center)
        .into()
}
