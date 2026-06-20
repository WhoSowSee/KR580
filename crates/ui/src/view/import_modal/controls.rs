use super::super::icons;
use super::super::styles::scrollable_style;
use super::super::theme::{TOKYO_MUTED, TOKYO_RED, TOKYO_TEXT, ui_text};
use super::super::widgets::{modal_footer_button, modal_icon_button};
use super::styles::{
    badge_style, dropdown_option_style, dropdown_panel_style, field_button_style,
    footer_button_style, group_label_style, group_panel_style,
};
use crate::app::{ImportFileFormat, Message};
use crate::i18n::{Key, Lang};
use iced::widget::{Space, button, column, container, opaque, row, scrollable, stack, svg};
use iced::{Element, Length, Padding, alignment};
const FIELD_WIDTH: f32 = 352.0;
const XLSX_BADGE_WIDTH: f32 = 58.0;
const TEXT_BADGE_WIDTH: f32 = 112.0;
const LABEL_WIDTH: f32 = 74.0;
const ROW_HEIGHT: f32 = 34.0;
const ICON_SIZE: f32 = 34.0;
const DROPDOWN_TOP: f32 = 108.0;
const DROPDOWN_LEFT: f32 = 94.0;
const DROPDOWN_OPTION_HEIGHT: f32 = 24.0;
const DROPDOWN_MAX_LIST_HEIGHT: f32 = 48.0;
pub(super) struct SourceGroupState<'a> {
    pub(super) file_display: &'a str,
    pub(super) format: Option<ImportFileFormat>,
    pub(super) target_input: &'a str,
    pub(super) target_options: &'a [String],
    pub(super) error: Option<&'a str>,
    pub(super) lang: Lang,
}
pub(super) fn source_group<'a>(state: SourceGroupState<'a>) -> Element<'a, Message> {
    let SourceGroupState {
        file_display,
        format,
        target_input,
        target_options,
        error,
        lang,
    } = state;
    let compact = format.is_none() && error.is_none();
    let mut content = column![file_row(file_display, format, lang)].spacing(10);
    if let Some(format) = format {
        if target_options.is_empty() {
            content = content.push(no_targets_row(lang));
        } else {
            content = content.push(target_row(format, target_input, lang));
        }
    }
    if let Some(error) = error {
        content = content.push(error_row(error));
    }
    group_box(
        lang.t(Key::ImportSourceGroup),
        content,
        Length::Fixed(if compact { 78.0 } else { 116.0 }),
    )
}
pub(super) fn footer(lang: Lang) -> Element<'static, Message> {
    row![
        Space::new().width(Length::Fill),
        modal_footer_button(
            lang.t(Key::DiscardCancel),
            Message::CancelImport,
            footer_button_style,
        ),
        modal_footer_button(
            lang.t(Key::FileImport),
            Message::ConfirmImport,
            footer_button_style,
        ),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn file_row<'a>(
    file_display: &'a str,
    format: Option<ImportFileFormat>,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        row_label(lang.t(Key::ImportFileLabel)),
        file_anchor(file_display, format, lang),
        modal_icon_button(
            icons::file_down(),
            Message::ImportFileBrowse,
            lang.t(Key::ImportBrowseTooltip),
            ICON_SIZE,
        ),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(ROW_HEIGHT))
    .into()
}

fn target_row<'a>(
    format: ImportFileFormat,
    target_input: &'a str,
    lang: Lang,
) -> Element<'a, Message> {
    row![
        row_label(lang.t(format.target_label_key())),
        target_anchor(target_input),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(ROW_HEIGHT))
    .into()
}

fn no_targets_row(lang: Lang) -> Element<'static, Message> {
    row![
        Space::new().width(Length::Fixed(LABEL_WIDTH)),
        ui_text(lang.t(Key::ImportNoTargets), 12, TOKYO_MUTED),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(ROW_HEIGHT))
    .into()
}

fn error_row(error: &str) -> Element<'_, Message> {
    row![
        Space::new().width(Length::Fixed(LABEL_WIDTH)),
        ui_text(error, 12, TOKYO_RED).width(Length::Fixed(FIELD_WIDTH)),
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn file_anchor<'a>(
    file_display: &'a str,
    format: Option<ImportFileFormat>,
    lang: Lang,
) -> Element<'a, Message> {
    let label = if file_display.is_empty() {
        lang.t(Key::ImportNoFile).to_owned()
    } else {
        shorten_middle(
            file_display,
            match format {
                Some(ImportFileFormat::Text) => 21,
                Some(ImportFileFormat::Xlsx) => 28,
                None => 38,
            },
        )
    };
    let color = if file_display.is_empty() {
        TOKYO_MUTED
    } else {
        TOKYO_TEXT
    };
    let file_label = ui_text(label, 13, color)
        .width(Length::Fill)
        .wrapping(iced::widget::text::Wrapping::None);
    let mut row = row![file_label]
        .spacing(8)
        .align_y(alignment::Vertical::Center);
    if let Some(format) = format {
        row = row.push(format_badge(lang.t(format.label_key()), format));
    }

    button(
        container(row)
            .padding([6, 10])
            .width(Length::Fixed(FIELD_WIDTH))
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ImportFileBrowse)
    .padding(0)
    .style(move |_theme, status| field_button_style(status))
    .into()
}

fn target_anchor<'a>(value: &'a str) -> Element<'a, Message> {
    let chevron = svg(icons::chevron_down())
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme, _status| svg::Style {
            color: Some(TOKYO_MUTED),
        });

    let row = row![
        ui_text(value.to_owned(), 13, TOKYO_TEXT),
        Space::new().width(Length::Fill),
        chevron,
    ]
    .spacing(8)
    .align_y(alignment::Vertical::Center);

    button(
        container(row)
            .padding([6, 10])
            .width(Length::Fixed(FIELD_WIDTH))
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ImportTargetDropdownToggled)
    .padding(0)
    .style(move |_theme, status| field_button_style(status))
    .into()
}

pub(super) fn target_dropdown_overlay(
    options: &[String],
    highlighted: Option<usize>,
    scroll_reveal: bool,
) -> Element<'static, Message> {
    column![
        Space::new().height(Length::Fixed(DROPDOWN_TOP)),
        row![
            Space::new().width(Length::Fixed(DROPDOWN_LEFT)),
            opaque(dropdown(options, highlighted, scroll_reveal)),
        ]
        .width(Length::Fill),
        Space::new().height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn dropdown(
    options: &[String],
    highlighted: Option<usize>,
    scroll_reveal: bool,
) -> Element<'static, Message> {
    let mut list = column![].spacing(0);
    for (index, option) in options.iter().enumerate() {
        list = list.push(dropdown_option(option.clone(), highlighted == Some(index)));
    }
    let list_height = (options.len() as f32 * DROPDOWN_OPTION_HEIGHT).min(DROPDOWN_MAX_LIST_HEIGHT);

    let overflow = options.len() as f32 * DROPDOWN_OPTION_HEIGHT > DROPDOWN_MAX_LIST_HEIGHT;
    let list = scrollable(list)
        .height(Length::Fixed(list_height))
        .on_scroll(|_| Message::ImportTargetScrolled)
        .style(move |theme, status| scrollable_style(scroll_reveal && overflow, theme, status));

    container(list)
        .padding(4)
        .width(Length::Fixed(FIELD_WIDTH))
        .style(dropdown_panel_style)
        .into()
}

fn dropdown_option(label: String, highlighted: bool) -> Element<'static, Message> {
    let message_value = label.clone();
    button(
        container(ui_text(label, 13, TOKYO_TEXT))
            .padding([0, 10])
            .width(Length::Fill)
            .height(Length::Fixed(DROPDOWN_OPTION_HEIGHT))
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ImportTargetSelected(message_value))
    .padding(0)
    .width(Length::Fill)
    .style(move |_theme, status| dropdown_option_style(status, highlighted))
    .into()
}

fn group_box<'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
    height: Length,
) -> Element<'a, Message> {
    let panel: Element<'a, Message> = container(content)
        .padding(Padding {
            top: 20.0,
            right: 12.0,
            bottom: 12.0,
            left: 12.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(group_panel_style)
        .into();
    let label = row![
        Space::new().width(Length::Fill),
        container(ui_text(title, 14, TOKYO_TEXT))
            .padding([0, 6])
            .style(group_label_style),
        Space::new().width(Length::Fill),
    ];

    stack![
        column![Space::new().height(Length::Fixed(9.0)), panel]
            .height(Length::Fill)
            .width(Length::Fill),
        label,
    ]
    .width(Length::Fill)
    .height(height)
    .into()
}

fn row_label(value: &'static str) -> Element<'static, Message> {
    container(ui_text(value, 13, TOKYO_TEXT))
        .width(Length::Fixed(LABEL_WIDTH))
        .height(Length::Fixed(ROW_HEIGHT))
        .align_y(alignment::Vertical::Center)
        .into()
}

fn format_badge(label: &'static str, format: ImportFileFormat) -> Element<'static, Message> {
    let width = match format {
        ImportFileFormat::Xlsx => XLSX_BADGE_WIDTH,
        ImportFileFormat::Text => TEXT_BADGE_WIDTH,
    };
    container(ui_text(label, 11, TOKYO_MUTED).wrapping(iced::widget::text::Wrapping::None))
        .padding([3, 8])
        .width(Length::Fixed(width))
        .align_x(alignment::Horizontal::Center)
        .style(badge_style)
        .into()
}

pub(super) fn shorten_middle(value: &str, budget: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= budget {
        return value.to_owned();
    }
    let remaining = budget.saturating_sub(1);
    let head_len = remaining / 2;
    let tail_len = remaining - head_len;
    let head: String = chars.iter().take(head_len).collect();
    let tail: String = chars.iter().skip(chars.len() - tail_len).collect();
    format!("{head}…{tail}")
}
